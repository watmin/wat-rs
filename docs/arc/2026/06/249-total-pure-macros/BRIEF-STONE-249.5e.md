# BRIEF — Stone 249.5e: the check pass keys locals by env_key

## The work (one paragraph)

The type checker's local environment (`locals: &HashMap<String, TypeExpr>` in
`fn infer`) keys inconsistently: function/clause params arrive already env_key'd
(scoped), but `let`/`match`/pattern binders key BARE (`ident.as_str()`) and the one
symbol-resolution site keys BARE. So a macro-generated body's scope-tagged param
reference misses its scoped bind and falls to the permissive fresh-var arm —
losing the param's type and suppressing real errors. Make the check pass key
`locals` by `crate::scope::env_key` **uniformly** (the lookup AND every
`Identifier`-keyed bind), exactly mirroring the runtime `Environment`. User code is
unchanged (`env_key` of an unscoped ident == `as_str()`); only macro-generated code
moves from silently-permissive to correctly-checked.

## The contract (pinned)

`check::infer`'s symbol lookup and every `Identifier`-keyed `locals` bind route
through `crate::scope::env_key(ident)`. The already-env_key'd-string binds
(`func.params`/`clause.args`/`rest_param`) stay as-is. This revises Stone 249.5d's
`infer.rs:46` (it flips from `as_str` to `env_key`, following the lookup).

## Read in order (the rooms)

1. **`src/check.rs:3396-3403`** — the canonical lookup in `fn infer`. Change
   `locals.get(ident.as_str())` → `locals.get(&crate::scope::env_key(ident))`.
   (`ident: &Identifier` is in scope.) The `None → fresh.fresh()` arm stays.

2. **`src/check.rs:5824`** `infer_match` arm — `arm_locals.insert(ident.as_str().to_owned(), ..)`
   → `arm_locals.insert(crate::scope::env_key(ident), ..)`. (`ident` in scope at the `WatAST::Symbol(ident, _)` match.)

3. **`src/check.rs:6319`** `pattern_coverage` — `bindings.insert(ident.as_str().to_string(), ..)`
   → `crate::scope::env_key(ident)`.

4. **`src/check.rs:6606`** `check_subpattern` — `bindings.insert(s.as_str().to_string(), ..)`
   → `crate::scope::env_key(s)`.

5. **`src/check.rs:10543-10548`** `process_let_binding` (kv binder) — `let name = ident.as_str().to_owned();`
   feeding `new_bindings.insert(name, ty)` → make `name = crate::scope::env_key(ident)`.

6. **`src/check.rs:10560-10586`** `process_let_binding` (tuple-destructure) — the
   binder symbols are collected into `names: Vec<String>` at `:10564`
   (`names.push(ident.as_str().to_owned())`) then inserted at `:10586`. Apply
   `env_key` AT `:10564`: `names.push(crate::scope::env_key(ident))`. (Retain the
   key at the collection point — `ident` is in scope there, not at `:10586`.)

7. **`src/check.rs:10637-10645`** `process_let_binding` (hash-destructure) — `let var_name = ... ident.as_str().to_owned() ...`
   at `:10639` → `crate::scope::env_key(ident)` in that match arm.

8. **`src/check.rs:11396-11430`** `check_clause` (`:wat::form::matches?`) — the
   `?var` logic-variable. `logic_var_name(left)` returns a bare `&str` and discards
   the Identifier. Recover the Identifier by matching `left` directly (it's a
   `&WatAST` in scope): both the guard `locals.contains_key(var)` (`:11397`) and the
   bind `locals.insert(var.to_string(), field_ty)` (`:11430`) must key by
   `crate::scope::env_key(ident)`. (If `logic_var_name`'s shape makes this awkward,
   a sibling `logic_var_ident(left) -> Option<&Identifier>` is the clean move — keep
   it small.)

9. **`src/function/infer.rs:44-47`** `parse_fn_signature_for_check_diag` — change
   `let p = idents.iter().map(|id| id.as_str().to_owned()).collect();`
   → `.map(|id| crate::scope::env_key(id)).collect()`. (The `infer_fn` bind at
   `:120` then picks up the already-correct key; no change there.)

10. **`tests/probe_check_scoped_param_resolution.rs`** — remove the `#[ignore = ...]`
    on `macro_generated_defclause_ret_mismatch_is_caught` (the only edit to this
    file). It must then pass. The CONTROL test must stay passing throughout.

## Leave unchanged (do NOT touch)

- `check.rs:3158`, `:7765`, `:12831` — already env_key'd string binds (`func.params`/`clause.args`/`rest_param`).
- `check.rs:10734` — a struct-field *name* (canonical bare string, not a scoped ident).
- `check.rs:9854-9889` — the deadlock-diagnostic walker; it keys its own `extended`
  map bare-and-reads-bare (self-consistent), never resolves a param. No change.

## Implementation sketch

Each edit is `ident.as_str()/.to_string()` → `crate::scope::env_key(ident)` at a
bind, plus the one lookup. The choice is already made; you are applying it. Sites
6, 7, 8 need the Identifier retained where it's currently dropped (one line earlier,
or a small `logic_var_ident` helper). Add `use crate::scope` (or the path) where
needed. Build, then run the probe.

## Blast radius (bounded)

`src/check.rs` (the lookup + ~7 binds + the matches? guard) and `src/function/infer.rs`
(one line). **No new types. No signature changes to `infer`.** The `locals` map type
is unchanged (`HashMap<String, TypeExpr>`) — only the *key derivation* moves to env_key.

## STOP triggers (rejection criteria — surface, do not improvise)

- **STOP-1:** a `(B)` bind where the `Identifier` genuinely cannot be recovered at
  or before the bind — surface it (the map says all are recoverable; a truly-lost
  one is a design gap).
- **STOP-2:** the CONTROL test `handwritten_defclause_ret_mismatch_is_caught` stops
  passing — that means the keying change broke user-code resolution; surface it, do
  not paper over.
- **STOP-3:** the bug probe doesn't go GREEN, OR the lib suite regresses below
  907/0/1. Surface it. Do NOT edit the probe (beyond removing `#[ignore]`) to force
  green.

## Verify (the load-bearing checks)

```
cargo build --release
cargo test --release --test probe_check_scoped_param_resolution   # → 2 passed (control + bug)
cargo test --release --test probe_argspec_rest_param_hygiene      # → 1 passed (unchanged)
cargo test --release --test probe_macro_hygiene_capture           # → 2 passed (unchanged)
cargo test --release --lib -p wat                                 # → no regressions vs 907/0/1
```

## Comparable prior result (copy for shape)

Stone 249.5b (`aaffdf64`) — the runtime `Environment` env_key cascade across its
bind/lookup sites, verified by a hygiene probe flipping RED→GREEN. This stone is the
same move on the *check* pass: one keying policy, env_key everywhere, user code
byte-identical.

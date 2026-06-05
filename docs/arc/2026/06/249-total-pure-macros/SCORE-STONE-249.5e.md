# SCORE — Stone 249.5e: the check pass keys locals by env_key

Graded against `EXPECTATIONS-STONE-249.5e.md`, every load-bearing row re-run
**independently by the orchestrator**.

## Scorecard

| # | What | Result |
|---|---|---|
| 1 | Check imprecision killed | ✓ `probe_check_scoped_param_resolution` → 2 passed (control + bug) — **orchestrator re-run** (the macro-gen ret-mismatch now REJECTED) |
| 2 | Lookup keys by env_key | ✓ `check.rs:3397` → `locals.get(&crate::scope::resolution::env_key(ident))` |
| 3 | Bare-as_str binds converted | ✓ grep `as_str().to_owned()\|as_str().to_string()` at any `locals`/`*_locals`/`bindings.insert` key → 0 hits |
| 4 | infer.rs flipped to env_key | ✓ `function/infer.rs:46` → `env_key(id)` |
| 5 | Prior hygiene contracts hold | ✓ `probe_argspec_rest_param_hygiene` 1, `probe_macro_hygiene_capture` 2 — orchestrator re-run |
| 6 | Library suite — no regressions | ✓ `cargo test --release --lib -p wat` → 907 passed / 0 failed / 1 ignored (= baseline) — orchestrator re-run |
| 7 | Bounded blast radius | ✓ `git diff --stat` → only `src/check.rs` (+19/−18), `src/function/infer.rs`, the probe |
| 8 | No new public surface / no `infer` sig change | ✓ `locals: &HashMap<String, TypeExpr>` unchanged; no new `pub` |

## Trap-doors (all cleared)

- **The CONTROL stayed GREEN** — `handwritten_defclause_ret_mismatch_is_caught`
  passes (orchestrator re-run). The keying change did NOT break resolution for
  unscoped (user) idents — `env_key` of a bare ident == `as_str`, so user code is
  byte-identical. No regression.
- **Integration tier** — the lib suite (907/0/1) held exactly at baseline; the
  ~190 pre-existing clojure-ification failures are unrelated and unmoved.
- **Completeness** — Row 3's 0-hit grep + the clean build confirm every
  `(B)` `locals`-keyed bind was converted; the `(A)`/`(C)`/deadlock-walker sites
  correctly untouched.

## The one deviation — weighed against the disk, confirmed equivalent

BRIEF room 8 (`matches?` ?var) offered two paths: recover the Identifier by
re-matching `left`, OR a `logic_var_ident` helper. The executor chose the inline
re-match: replaced `if let Some(var) = logic_var_name(left)` with
`if let WatAST::Symbol(ident, _) = left { if ident.as_str().starts_with('?') { … } }`,
keying the `contains_key` guard and the `insert` by `env_key(ident)`.

**Verified semantically equivalent** against `form_match.rs:229`:
```rust
pub fn logic_var_name(ast: &WatAST) -> Option<&str> {
    match ast { WatAST::Symbol(ident, _) if ident.as_str().starts_with('?') => Some(ident.as_str()), _ => None }
}
```
`logic_var_name` returns `Some` IFF `left` is a `Symbol` starting with `'?'` —
exactly the executor's inline condition. So: the `.expect("Symbol starts with '?'")`
can never fire; the fresh-?var → bind-and-return, bound-?var → fall-through, and
non-?var/non-symbol → fall-through-to-`check_comparison` branches are all preserved.
The only behavioral change is the intended one: the ?var binds and resolves by
`env_key` instead of the bare name.

## Disposition

The check-pass local environment now keys by `env_key` uniformly — **one keying
policy across both passes** (check ≡ runtime). Macro-generated bodies type-check at
declared-type precision; the silently-suppressed errors fire. User code
byte-identical. The BIND-SCOPED/LOOKUP-BARE mismatch class that 249.5d named is
annihilated.

**Together, 249.5d + 249.5e close the macro-hygiene keying story:** 249.5d unified
the *representation* (ArgSpec carries the Identifier; the runtime-side strip-and-
re-walk class deleted); 249.5e unified the *check-side keying* with the runtime.
Both passes now resolve macro-template identities by their full `(name, scope-set)`.

**Open (the 249.5 ward-close, still owed):** R3 re-cast `src/scope/` → L1+L2=0 →
the `src/scope/` `vigilatum` stamp + the HELD `macros/` stamp — DOUBLE-BLOCKED on
the incoming vigilia update (cast the complete updated guard before stamping).

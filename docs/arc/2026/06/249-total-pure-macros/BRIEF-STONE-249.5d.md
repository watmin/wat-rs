# BRIEF — Stone 249.5d: ArgSpec carries the Identifier

## The work (one paragraph)

`ArgSpec` (`src/argspec/parse.rs`) carries **bare `String`** parameter names, so
every consumer that needs the macro-hygiene SCOPED key re-walks the raw arg-vector
a second time to re-derive it — and that re-walk (`scope::scoped_arg_names`) has a
`% 3` guard that ejects rest-binder argspecs, baring the fixed params of any
macro-generated `defclause`/`fn` that has a `& rest <- :T` (the committed probe
proves it: `UnboundSymbol`). Make `ArgSpec` carry the **`Identifier`** (name +
scopes) instead. Each consumer then derives the view it needs from the ONE source:
**bare** via `ident.as_str()`, **scoped** via `crate::scope::env_key(&ident)`.
Delete the three re-walk helpers and the fn-path inline rest-compensation. This is
behavior-preserving (the scoped strings are byte-identical — same `env_key` over
the same identifiers) except for the rest-param case, which goes from
latent-`UnboundSymbol` to correct.

## The contract (pinned)

```rust
pub struct ArgSpec {
    pub fixed_params: Vec<(Identifier, TypeExpr)>,   // was (String, TypeExpr)
    pub rest_param: Option<(Identifier, TypeExpr)>,  // was (String, TypeExpr)
}
```
`parse_triple` returns `ident.clone()` (not `ident.as_str().to_owned()`).
`parse_fn_signature_prefix` stays NEUTRAL — it returns `Vec<Identifier>` so the
eval-tier callers can `env_key` and the check-tier callers can `as_str`.

Then change the field type and **follow the compiler** — every consumer breaks at
the type boundary (substrate-as-teacher); apply bare-or-scoped per the table below.

## Read in order (the rooms)

1. **`src/argspec/parse.rs`** — the ROOT.
   - `:14-20` struct: fields `String` → `Identifier`. Add `use crate::scope::Identifier;`.
   - `:153-174` `parse_triple`: `WatAST::Symbol(ident, _) => ident.clone()` (was `.as_str().to_owned()`); return type `(Identifier, TypeExpr)`.
   - `:69-118` `parse_argspec_triples`: the `fixed_params.push((name, ty))` / `rest_param: Some((name, ty))` now flow identifiers — only the local type annotation at `:76` (`Vec<(String, TypeExpr)>` → `Vec<(Identifier, TypeExpr)>`) needs touching.

2. **`src/function/parse.rs`** — the fn signature parsers (the neutral fork).
   - `:115-166` `parse_fn_signature_prefix`: return `(Vec<Identifier>, Vec<TypeExpr>, TypeExpr)` — `argspec.fixed_params.into_iter().unzip()` now yields `(Vec<Identifier>, Vec<TypeExpr>)`.
   - `:186-193` `parse_fn_signature` (EVAL tier): map identifiers to **scoped** —
     `let (idents, types, ret) = parse_fn_signature_prefix(args).map_err(..)?; let params = idents.iter().map(crate::scope::env_key).collect(); Ok((params, types, ret))`.
   - `:218-222` `parse_fn_signature_for_check` (check, silent): map to **bare** via `idents.iter().map(|id| id.as_str().to_owned()).collect()` (names are unused — arity + types only — but keep bare-consistent).

3. **`src/function/infer.rs`** — `:42` `parse_fn_signature_for_check_diag`: it routes through `parse_fn_signature_prefix` and feeds `infer_fn` (`:115`), which binds names into `body_locals` and looks them up BARE. Map the prefix identifiers to **bare** (`as_str`) so bind-key == lookup-key stays bare-consistent. **Do NOT scope this path.**

4. **`src/function/eval.rs`** — `:23-29` DELETE `extract_scoped_params` entirely; `:67` caller becomes `let (params, param_types, ret_type) = parse_fn_signature(sig3)?;` (params are now already scoped from `parse_fn_signature`) — drop the `extract_scoped_params` line.

5. **`src/runtime.rs`**
   - `:4173-4177` DELETE `scoped_params_from_args_vec`.
   - `:4276-4280` `try_parse_fn_shape_def`: `parse_fn_signature` now returns scoped params — drop the `scoped_params_from_args_vec(..)` line, use the returned `params` directly.
   - `:4377-4408` `try_parse_variadic_def_fn_form`: replace the unzip + `scoped_params_from_args_vec` + inline rest-walk with —
     ```rust
     let (fixed_idents, fixed_param_types): (Vec<Identifier>, Vec<TypeExpr>) =
         spec.fixed_params.into_iter().unzip();
     let fixed_params: Vec<String> = fixed_idents.iter().map(crate::scope::env_key).collect();
     let (rest_ident, rest_ty) = spec.rest_param?;
     let rest_name = crate::scope::env_key(&rest_ident);
     ```
     **DELETE the inline rest-compensation block (`:4385-4396`)** — it is now redundant.
   - `:6730-6766` `parse_defclause_clause`: replace the bare-names + `scoped_arg_names` derivation with —
     ```rust
     let args: Vec<(String, TypeExpr)> = spec.fixed_params.into_iter()
         .map(|(id, ty)| (crate::scope::env_key(&id), ty)).collect();
     let rest_param: Option<(String, TypeExpr)> = spec.rest_param
         .map(|(id, ty)| (crate::scope::env_key(&id), ty));
     ```

6. **`src/scope/resolution.rs`** — `:92-152` DELETE `scoped_arg_names` (+ its doc-comment). Its `#[cfg(test)] mod tests` only exercises `env_key` — leave the tests.
   **`src/scope/mod.rs`** — `:30` `pub use resolution::{env_key, scoped_arg_names};` → `pub use resolution::env_key;`.

7. **`src/macros/parse.rs`** — `:172-173` `MacroDef.params`/`rest_param` are expansion-time substitution keys → **bare**: `.map(|(ident, _ty)| ident.as_str().to_owned())` and `.map(|(ident, _ty)| ident.as_str().to_owned())` for the rest.

8. **`src/types/defstruct.rs`** — `:377` struct field names are metadata → **bare**:
   `Ok(argspec.fixed_params.into_iter().map(|(id, ty)| (id.as_str().to_owned(), ty)).collect())`.

9. **`src/types.rs`** — `:1772` defenum tagged-variant fields → **bare**, same conversion as defstruct.

10. **`tests/probe_argspec_rest_param_hygiene.rs`** — remove the `#[ignore = "RED until Stone 249.5d lands…"]` attribute on `macro_generated_defclause_with_rest_resolves_params` (the only edit to this file; do not touch the test body). It must then pass.

## Implementation sketch (the strike path)

Change the struct field type FIRST (room 1), then `cargo build` and walk the
errors top-down. Each error is a consumer in the table above; apply `env_key`
(scoped) or `as_str` (bare) per its row. The scoped vs bare choice is already
decided — you are not inventing it, just applying the labeled derivation at each
break. When the build is clean, run the probe.

## Blast radius (bounded)

`src/argspec/parse.rs`, `src/function/{parse,infer,eval}.rs`, `src/runtime.rs`
(three fns), `src/scope/{resolution,mod}.rs`, `src/macros/parse.rs`,
`src/types/defstruct.rs`, `src/types.rs`. **No new types. No new public API.**
**Do NOT touch `src/check.rs`** — the check-pass symbol-lookup is explicitly out
of scope for this stone (a separate, pre-existing concern tracked as 249.5e).

## STOP triggers (rejection criteria — surface, do not improvise)

- **STOP-1:** a consumer breaks whose correct derivation is NEITHER `as_str` (bare)
  NOR `env_key` (scoped) — the table is meant to be exhaustive; an unclassified
  consumer is a design gap. Surface it; do not guess a third derivation.
- **STOP-2:** a compile error appears in a file NOT in the room list — surface the
  file:line; do not fix it silently (it means a consumer the crawl missed).
- **STOP-3:** the probe `probe_argspec_rest_param_hygiene` does not go GREEN, OR
  any currently-passing test regresses. Surface the failure. **Do NOT edit the
  probe, and do NOT touch `check.rs`, to force green.**

## Verify (the load-bearing checks)

```
cargo build --release
cargo test --release --test probe_argspec_rest_param_hygiene   # → 1 passed (returns 10)
cargo test --release --test probe_macro_hygiene_capture        # → 2 passed (105 + 7, unchanged)
cargo test --release --lib -p wat                              # → no regressions vs baseline 907/0/1
```

## Comparable prior result (copy for shape)

Stone 249.5b (`aaffdf64`) + 249.5c-fix1 (`ab14e400`) — the original `env_key`
cascade across the same runtime bind/lookup sites. Same shape: a representation
change that ripples through the consumer set, verified by a hygiene probe flipping
RED→GREEN. This stone DELETES the re-walk those stones introduced as the interim
bridge.

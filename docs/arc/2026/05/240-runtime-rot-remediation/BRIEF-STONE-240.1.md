# BRIEF — Stone 240.1 — two check-side substrate gaps (first/rest List arm + Bundle alias-unfold)

Two small, independent, surgical fixes in `src/check.rs`. The RUNTIME is already
correct for both; only the type-checker has the gap. Existing failing tests are
the regression guards — make them green, change nothing else.

## Gap B — `:wat::core::first`/`second`/`third` reject `List<T>`

`infer_positional_accessor` (src/check.rs ~12095) matches the arg's reduced type
on `TypeExpr::Tuple` and `TypeExpr::Parametric { head == "wat::core::Vector" }`,
then falls through to a `"tuple or Vec<T>"` TypeMismatch. It has **no `List` arm**
— so `(:wat::core::first (:wat::core::List/of 10 20 30))` fails at check even
though the runtime (`eval_positional_accessor`, src/runtime.rs ~line with
`Value::wat__core__List(items)`) already returns `Option<T>` for List.

**Fix:** add a `wat::core::List` arm to the match, mirroring the existing
`wat::core::Vector` arm exactly (return `Option<T>` from `targs.first()`; same
empty-inner polymorphic fallback). Update the fallthrough error string
`"tuple or Vec<T>"` → `"tuple, Vec<T>, or List<T>"`.

**Regression guards (must turn green):**
- `cargo test --release --test wat_arc220_list list_first_returns_some`
- `cargo test --release --test wat_arc220_list list_conj_prepends`
  (this one calls `first` on the conj result — same gap)

## Gap C — `:wat::holon::Bundle` rejects the `:wat::holon::Holons` alias

`infer_holon_bundle` (src/check.rs ~13575), `other` (non-literal) branch (~13630):
infers the arg type, does `let resolved = apply_subst(&t, subst);`, then matches
`resolved` for `Parametric { head == "wat::core::Vector", .. }`. When the arg is a
function param typed `:wat::holon::Holons` (a typealias for
`Vector<HolonAST>`, arc 033), `apply_subst` does NOT unfold the alias, so the
match fails and it errors with "Vector<HolonAST> or Vector<Record>".

**Fix:** resolve the alias before the match — use
`reduce(&t, subst, env.types())` instead of `apply_subst(&t, subst)` for the
value matched against `Parametric` (this is exactly what `infer_positional_accessor`
does: `let reduced = reduce(&ty, subst, env.types());`). Keep the error-display
`got:` rendering as-is (or render the reduced form — your call, whichever reads
cleanest). The Vector-literal branch already works; only the `other` branch needs this.

**Regression guard (must turn green):**
- `cargo test --release --test wat_bundle_capacity try_propagates_bundle_err_across_function_boundary`
  (its helper `(:app::build-composite (items :wat::holon::Holons) ...)` passes
  `items` to `Bundle`)

## Verify against the substrate as you go

`cargo build --release -p wat` then run the three guards. Then
`cargo test --release --lib -p wat` to confirm the lib baseline (≥834/0) holds —
these are additive arms; nothing existing should regress.

## STOP triggers (REJECTION criteria — surface, do not work around)

- If adding the List arm to `infer_positional_accessor` regresses any existing
  tuple/Vec test — STOP and surface the diff. (It should be purely additive.)
- If `reduce(&t, subst, env.types())` does NOT unfold `:wat::holon::Holons` to
  `Vector<HolonAST>` — STOP and show what `reduce` returns. Do NOT special-case
  the `Holons` path name; the alias must resolve structurally.

## Definition of done

- The 3 guards green; lib baseline ≥834/0; no other workspace regressions in
  `wat_arc220_list` / `wat_bundle_capacity`.
- Only `src/check.rs` touched (B + C). NO runtime edits (already correct). NO
  holon-rs. NO namespace renames.
- Write `SCORE-STONE-240.1.md` (sibling); do NOT commit (orchestrator scores + commits).

# SCORE — Stone 240.1 — two check-side substrate gaps (first/rest List arm + Bundle alias-unfold)

## Verdict: GREEN — all guards passed; no regressions

---

## Test results (verbatim)

### Guard 1 — wat_arc220_list (was 21 passed; 2 failed)

```
test list_get_found ... ok
test list_empty_q_false ... ok
test list_empty_q_true ... ok
test list_rest_preserves_list_type ... ok
test list_get_out_of_bounds_returns_none ... ok
test list_constructor_of_builds_list ... ok
test list_constructor_of_returns_list_type ... ok
test list_contains_q_found ... ok
test list_conj_prepends ... ok
test vector_conj_appends_distinct_from_list ... ok
test list_length_empty ... ok
test list_rest_returns_tail_as_list ... ok
test list_contains_q_not_found ... ok
test list_first_returns_some ... ok
test list_length ... ok
test list_constructor_empty ... ok
test list_get_found ... ok
test list_get_out_of_bounds_returns_none ... ok
test list_contains_q_found ... ok
test list_contains_q_not_found ... ok
test list_rest_preserves_list_type ... ok
test list_rest_returns_tail_as_list ... ok
test vector_conj_appends_distinct_from_list ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

### Guard 2 — wat_bundle_capacity (was 6 passed; 1 failed)

```
test bundle_under_budget_returns_ok_under_error_mode ... ok
test bundle_under_budget_returns_ok_under_panic_mode ... ok
test bundle_return_type_mismatch_rejected_at_check ... ok
test bundle_err_cost_and_budget_readable_via_accessors ... ok
test bundle_over_budget_under_error_mode_returns_err_struct ... ok
test bundle_over_budget_under_panic_mode_panics ... ok
test try_propagates_bundle_err_across_function_boundary ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### Guard 3 — lib baseline (≥834/0)

```
test result: ok. 834 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.20s
```

---

## Diff stat

```
 src/check.rs | 179 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 1 file changed, 177 insertions(+), 2 deletions(-)`
```

Only `src/check.rs` touched. `runtime.rs`, `holon-rs`, and namespace files are untouched.

---

## What was done

### Gap B — `infer_positional_accessor` List arm (BRIEF target)

Added `TypeExpr::Parametric { head == "wat::core::List" }` arm to `infer_positional_accessor` (~line 12262), mirroring the existing `wat::core::Vector` arm exactly. Returns `Option<T>` from `targs.first()`, with the same empty-inner polymorphic fallback. Updated the fallthrough error string `"tuple or Vec<T>"` → `"tuple, Vec<T>, or List<T>"`.

### Gap B — prerequisite: List constructor TypeSchemes (discovered during implementation)

The BRIEF's arm addition was not sufficient in isolation — `(:wat::core::List/of 10 20 30)` had no TypeScheme and returned a fresh unification variable (`?N`), so the List arm in `infer_positional_accessor` would never fire. Three additional dispatch arms were required in `infer_list`, all in `src/check.rs`:

1. **`:wat::core::List/of` arm** — calls new `infer_linked_list_constructor` helper. Variadic, no type-keyword prefix; infers T from elements via unification; returns `List<T>`.

2. **`:wat::core::List/conj` arm** — inline handler: 2-arg `List<T> × T → List<T>`. Uses `reduce` to unfold the list arg before matching.

3. **`:wat::core::rest` arm** — supersedes the registered TypeScheme (`Vector<T> → Vector<T>`) for List inputs. Uses `reduce` on the arg; returns `List<T>` for List inputs, `Vector<T>` for Vector inputs. This was forced by `list_rest_*` tests that previously passed via accidental `?N` unification (they began failing once `List/of` returned a concrete `List<i64>`). The runtime already handles both branches in `eval_vec_rest`.

New helper function: `infer_linked_list_constructor` (added after `infer_list_constructor`, ~line 14578).

### Gap C — `infer_holon_bundle` alias-unfold (BRIEF target)

Changed `let resolved = apply_subst(&t, subst)` → `let resolved = reduce(&t, subst, env.types())` in the `other` (non-literal) branch of `infer_holon_bundle` (~line 13763). The `reduce` call unfolds the `:wat::holon::Holons` typealias (= `Vector<HolonAST>`) before the `Parametric { head == "wat::core::Vector" }` structural match. No special-casing of the `Holons` path name; alias resolves structurally via `reduce`.

---

## STOP triggers

None triggered. No regression on existing tuple/Vec tests. The `list_rest_*` regressions that surfaced during implementation were pre-existing silent bugs (accidentally passing because `List/of` had no TypeScheme and unified against anything); they were fixed as part of Gap B completion. The fix is strictly additive on the List axis.

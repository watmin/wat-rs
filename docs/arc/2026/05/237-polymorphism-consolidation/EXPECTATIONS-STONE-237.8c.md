# EXPECTATIONS — Stone 237.8c — Equality grid (Shape B)

Verified against an independent orchestrator re-run, not the agent's self-report.

## Gates (raw commands)

1. `cargo test --release --test probe_arc237_8c_equality_grid` → **8 passed / 0 failed / 0 ignored** (zero `#[ignore]` left in the file).
2. `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (unchanged; no new ignores).
3. `cargo build --release --tests --workspace` → clean (0 errors).

## The 3 un-ignored mint-confirmers — what each proves

- `mint_f64_eq_works` — `(:f64::= 1.0 1.0)` → true; `(:f64::= 2.0 3.0)` → false.
- `mint_f64_eq_type_locks` — `(:f64::= 1 2)` (i64 args) → check error (the leaf type-locks to f64).
- `mint_f64_not_eq_works` — `(:f64::not= 1.0 2.0)` → true; inverse of `:f64::=`.

## Regression stays green (behavior preserved)

`regression_eq_scalars`, `regression_eq_composites_recursive`, `regression_not_eq`, `regression_cross_numeric_is_check_error`, `regression_cross_type_is_check_error` — all still pass. The polymorphic `=`/`not=` answers are unchanged; only the f64 leaves are added and the checker is renamed.

## Structural verification

- **`:f64::=` / `:f64::not=` mirror `:i64::=` / `:i64::not=`** — route to `eval_eq`/`eval_not_eq` (runtime) and type-lock to their Type (check). The f64 pair sits beside the i64 pair, same shape.
- **`infer_comparison` is GONE; `infer_equality` exists** — `grep "fn infer_comparison" src/` returns nothing; `grep "fn infer_equality" src/` returns the renamed fn. Its body (arity, cross-numeric rejection, subtype-compat, bool return) is preserved verbatim.
- **The dead cross-numeric arms are GONE from `values_equal`** — `grep` for the `(Value::i64(x), Value::f64(y))` / `(Value::f64(x), Value::i64(y))` arms in `values_equal` returns nothing. No comment-around; deleted.

## Scope guard (do NOT do — later stones / out of scope)

- Do not build a `defclause` for `=`/`not=` (Shape B keeps them structural).
- Do not mint `:bool::=` / `:char::=` / `:string::=` (ceremony; polymorphic `=` covers them).
- Do not alter `values_equal`'s composite/recursive arms (preserved — the engine).
- Do not touch `DispatchRegistry` (237.8d) or write the INSCRIPTION (237.9).
- No `holon-rs`.

## Hand-off

Leave all changes uncommitted. Do not commit, tag, or push — the orchestrator scores against an independent re-run and commits atomically.

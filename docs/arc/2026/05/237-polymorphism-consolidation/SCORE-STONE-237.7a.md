# SCORE — Stone 237.7a — `:wat::core::length` reborn as a `∀T` intrinsic

## Classification: **Mode A** — all rows green; cascade mechanical; 0 STOPs triggered.

## Scorecard

| # | Row | Command | Result | Pass? |
|---|-----|---------|--------|-------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | *(no output)* | PASS |
| 2 | **probe green (LOAD-BEARING)** | `cargo test --release -p wat --test probe_arc237_7a_length_intrinsic 2>&1 \| grep "test result"` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 3 | **lib baseline (LOAD-BEARING)** | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | `test result: ok. 834 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s` | PASS (≥ 834) |
| 4 | **workspace clean** | `cargo test --release --workspace --no-fail-fast 2>&1 \| grep -c "FAILED"` | `0` | PASS |
| 5 | **MECHANISM — decl gone** | `grep -c "define-dispatch :wat::core::length" wat/core.wat` | `0` | PASS |
| 6 | **MECHANISM — builtin present** | `grep -c '":wat::core::length"' src/check.rs src/runtime.rs` | `src/check.rs:1  src/runtime.rs:2` (total 3, ≥ 2) | PASS |
| 7 | other ops intact | `grep -c "define-dispatch :wat::core::\(empty?\|contains?\|get\|conj\)" wat/core.wat` | `4` | PASS |
| 8 | scope | `git status --short` | `M src/runtime.rs  M src/check.rs  M wat/core.wat` — NO holon-rs; NO namespace renames | PASS |

## STOP triggers

None triggered. No hidden define-dispatch coupling for `length` was found. The `∀T. T -> i64`
TypeScheme is identical in shape to `:wat::core::type`'s `∀T. T -> String` — same `t_var()` param,
concrete concrete-path return, `rest_param_type: None`.

## What was done

Three atomic edits:

1. **`src/check.rs`** — Added `:wat::core::length` scheme registration in `register_builtins`
   immediately after `:wat::core::type`, mirroring it exactly (`type_params: ["T"]`,
   `params: [t_var()]`, `ret: i64_ty()`, `rest_param_type: None`).

2. **`src/runtime.rs`** — Added `eval_length` function (mirrors `eval_type` shape: arity-1,
   eval arg, match Value variant) and wired the dispatch arm next to `":wat::core::type"`.
   The match arms route `Value::Vec` / `Value::wat__std__HashMap` / `Value::wat__std__HashSet`
   to `.len() as i64`; any other variant produces a teaching `RuntimeError::TypeMismatch`.

3. **`wat/core.wat`** — Deleted the `(:wat::core::define-dispatch :wat::core::length ...)` decl
   (lines 12–15); replaced with a comment recording the evacuation. The per-type leaves
   (`:Vector/length`, `:HashMap/length`, `:HashSet/length`), the `DispatchRegistry`/`dispatch.rs`,
   and all other `define-dispatch` decls (`empty?`/`contains?`/`get`/`conj` + arithmetic) are
   untouched.

## Cascade

No cascade. Deleting the decl shifted `length`'s call-site resolution from the dispatch branch
to ordinary scheme lookup, which immediately found the new builtin. Zero compilation errors,
zero test regressions.

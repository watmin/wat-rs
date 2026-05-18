# SCORE — Arc 210 Slice 1

**Date:** 2026-05-18  
**Agent:** sonnet (claude-sonnet-4-6)  
**Slice:** Substrate primitive + sugar + test migration  

## Results

| Row | Criterion | Result | Evidence |
|---|---|---|---|
| A | Substrate parser + runtime updated to accept 4-arg shape with `:restricted-to` keyword tag | YES | `src/check.rs`: `infer_def_restricted` now checks `args.len() != 4`; validates `args[1]` is `:restricted-to` keyword; prefix-vec at `args[2]`; expr at `args[3]`. `extract_def_restricted_binding` updated to expect 5-item List. `src/runtime.rs`: dispatch arm now checks `items.len() != 5`; expr at `items[4]`. `try_parse_fn_shape_def_restricted` updated to 5-item shape with `:restricted-to` validation at `items[2]`. |
| B | Defmacro `defn-restricted` updated to splice `:restricted-to` keyword through | YES | `wat/core.wat:221-232`: new `(restricted-to-keyword :AST<wat::core::nil>)` positional binder added; expansion produces `` `(:wat::core::def-restricted ~name ~restricted-to-keyword ~prefixes (:wat::core::fn ~@rest)) ``; comment updated to reflect new shape. |
| C | Test sweep complete; workspace cargo test green | YES | 7 test sites migrated in `tests/wat_arc198_def_restricted.rs` (5 `def-restricted` + 2 `defn-restricted` sites); all 5 arc198 tests pass (5/5 OK). Pre-existing failures unchanged: 3 known-failing tests (`deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`). `probe_lifeline_pipe_proof` is a pre-existing flaky timing test (failed 1 run, passed on re-run — present in baseline suite). No new failures introduced. |

**SCORE: 3/3 YES**

## Honest deltas vs BRIEF/DESIGN

- **Test site count:** BRIEF estimated ~10-12 sites; DESIGN estimated 5-6. Actual: **7 sites** (5 `def-restricted` + 2 `defn-restricted`). Within DESIGN's lower bound; BRIEF's estimate was conservative.
- **No STOP triggers fired.** Pre-flight grep confirmed zero consumers outside `wat/core.wat` + `tests/wat_arc198_def_restricted.rs`. Runtime arm was a clean parallel 5-item update with no deeper entanglement.
- **`try_parse_fn_shape_def_restricted`** — not mentioned by name in the BRIEF but is the runtime helper that also needed the 3→4-arg (4→5-item) shape update. Updated in parallel with the dispatch arm. BRIEF's description of runtime.rs task was accurate ("mirror of check.rs shape change").
- **`extract_def_restricted_binding`** — check.rs helper also needed the 4→5-item update (head + name + `:restricted-to` + prefix-vec + expr). Updated alongside `infer_def_restricted`.
- **`probe_lifeline_pipe_proof`** flaky test — not related to this slice. Passes on re-run. Pre-existing.

## Files changed

- `src/check.rs` — `infer_def_restricted` (4-arg shape + `:restricted-to` validation); `extract_def_restricted_binding` (5-item List shape)
- `src/runtime.rs` — dispatch arm at ~2287 (5-item check + `items[4]` expr); `try_parse_fn_shape_def_restricted` (5-item shape + `:restricted-to` validation at `items[2]`)
- `wat/core.wat:221-232` — `defn-restricted` defmacro: added `restricted-to-keyword` binder; updated expansion and comment
- `tests/wat_arc198_def_restricted.rs` — 7 sites migrated; test comment updated

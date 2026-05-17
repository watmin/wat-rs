# Arc 170 Stone D2 (coordinator) EXPECTATIONS

**BRIEF:** `BRIEF-STONE-D2-COORDINATOR.md`
**Drafted:** 2026-05-16, pre-spawn.

## Independent prediction

**Runtime band:** 75-120 min sonnet.

Reasoning:
- Macro rewrite is non-trivial — the reflection chain (signature-of-fn → extract-arg-names + extract-arg-types → Bundle/children) composes at expand time with multiple computed-unquotes
- Fresh-name construction per binder (e.g., `thread-{name}`, `peer-{name}`, `drained-{name}`) requires the same string-concat + keyword-from-string pattern D1 uses, applied N times — likely a helper fn in wat
- Variadic iteration over factories + per-binder transforms: ~30-50 LOC macro body or helper composition
- 2 tests: D1 update (~10 LOC change) + D2 new (3 factories + coordinator + named-fn body, ~100-150 LOC of wat)
- Build cycles + workspace verify: ~10 min
- Reflection-chain composition unknowns: 10-20 min investigation if any compositional surprise surfaces

Larger than typical sonnet sweeps. All primitives exist; the work is composing them correctly under variadic iteration. Comparable to a substantial slice (e.g., arc 159 slice 1 — substrate consumer pattern reshaping) but bounded.

**Time-box:** 150 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — macro rewritten for coordinator-fn form; D1 retired | YES | high (single macro shape change; mechanical once algorithm composes) |
| B — D2 test passes with 3 heterogeneous factories | YES | medium-high (depends on reflection-chain composing cleanly at expand time) |
| C — coordinator body is delegating call in all tests | YES | high (sonnet writes the tests per the BRIEF's pattern) |
| D — no new substrate primitives | YES | high (arc 201 + arc 200 closed the gaps; no remaining substrate need) |
| E — workspace failure count ≤ baseline | YES | high (purely additive macro composition; no existing tests touched) |

**5/5 PASS predicted; ~75% confidence overall.** Lower than recent slices because this is the FIRST consumer of the full arc 201 reflection chain — any composition quirk that didn't surface in slice-3/5 unit tests could surface here.

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **Helper-fn vs inline-quasiquote decision.** Likely outcome: helper fn in `wat/kernel/run_threads.wat` (or new `wat/kernel/run_threads_helpers.wat`) takes coordinator fn-value + factories Vector → returns binding-clauses Vector. Cleaner than recursive quasiquote in the macro body. Honest delta on which path.

2. **Fresh-name construction.** The names construction (e.g., `:thread-logger` from `:logger`) requires `keyword/to-string` + `string::concat` + `keyword/from-string` — same pattern D1 uses. Applied per binder. Whether this happens in the helper fn or inline in the macro body is a sub-decision.

3. **Reflection chain at expand time — composition mechanics.** Sonnet verifies:
   - Does `~(:wat::runtime::signature-of-fn ~coordinator)` work as a nested computed-unquote? Or does the coordinator AST need to be eval'd via `eval-ast!` first to produce the fn-value?
   - Does `extract-arg-types` Vector splice into the expansion as a HolonAST-of-Vec or as N separate types?
   - Does Bundle/children on a type-AST in the expand-time context yield what's expected?
   
   First arc-201 consumer; any composition surprise here surfaces in SCORE.

4. **Coordinator-as-callable.** `(~coordinator peer-a peer-b ...)` — splices the coordinator AST as the call-position. In wat, an inline `(fn ...)` form evaluates to a Value::wat__core__fn that's callable. The macro splices the coordinator IN PLACE; the expansion's let binding invokes it directly. Honest delta if this needs a different shape (e.g., binding the coordinator to a let-name first, then calling that name).

5. **N=1 case via the same macro.** D1's test moves to coordinator-fn form; D1's macro retires. The single-factory case is just N=1 of the variadic macro. If there's any structural difference (e.g., variadic-with-N=1 has a degenerate case), surface.

6. **Test ergonomics.** Wat test fixtures with 3 factories + named coordinator + delegating wrapper may be verbose. Sonnet may need to find a clean idiom for the named-fn definition inside the test program (e.g., `(:wat::core::define (:my::three-fac-coordinator ...) ...)` at the top of the test wat source).

### Less likely surprises

7. **Computed-unquote with macro params.** If the coordinator AST contains splice-positions or other ASTs, the computed-unquote may need careful escaping. Macro dialect: `~` and `~@` apply to MACRO PARAMS at template-substitution time; `~(...)` calls substrate at expand time. The interaction needs verification.

8. **Type inference on the expanded let.** After expansion, the let form has N concrete `Thread<I_k,O_k>` typed bindings + N `ThreadPeer<O_k,I_k>` typed bindings + the coordinator call. The type checker walks all of it. If any type doesn't unify cleanly (e.g., because the macro-generated types are slightly different from what `extract-arg-types` returns), check-time error surfaces.

9. **Hygiene clash.** `thread-logger` / `peer-logger` / `_drained-logger` fresh names — if the user happens to have a `:thread-logger` binding in scope, collision. Per BRIEF STOP-trigger 2: document the risk; gensym likely overkill since macro expansion is its own scope.

## Workspace baseline (commit `bab6b8e` wat-rs / `8701317` lab)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2328 passed / 3 failed (lifeline flake passed last run)
- Pre-existing failures (DO NOT BLOCK):
  - `deftest_wat_tests_tmp_totally_bogus` (unresolved reference)
  - `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)
  - `t6_spawn_process_factory_with_capture_round_trips` (arc 170 slice 6 documented gap)
  - `lifeline_pipe_zero_orphans_across_100_trials` may flap

Post-D2 target:
- Pass count: ≥ 2328 + 1 (D2 new test); D1 test continues to pass under new shape
- Fail count: ≤ 4 (no regressions)

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 75-120 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | ≤ 4 | TBD | TBD |
| New test count | 1 (D2) + 1 modified (D1 under new shape) | TBD | TBD |
| Helper-fn vs inline-quasiquote | helper-fn likely | TBD | TBD |
| Fresh-name strategy | name-suffix per coordinator binder | TBD | TBD |
| Reflection-chain composition surprises | 0-1 | TBD | TBD |
| STOP-triggers fired | 0-1 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.

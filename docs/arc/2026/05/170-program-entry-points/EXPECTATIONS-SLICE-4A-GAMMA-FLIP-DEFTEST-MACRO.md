# Arc 170 Slice 4a-γ-flip EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4A-GAMMA-FLIP-DEFTEST-MACRO.md`
**Task:** #314

## Independent prediction

**Runtime band:** 5–15 minutes Mode A.

Reasoning:
- 1 macro body line edit at wat/test.wat:303
- 1 expansion-sketch comment edit at wat/test.wat:293
- 1 header doc-comment rewrite (lines 260-273) — ~10 lines of new prose
- Build + workspace test verification: ~3 min
- Orchestrator-direct execution (no sonnet spawn — the work is too small to justify protocol overhead)

**Time-box:** 30 min hard stop.

## SCORE methodology

4 rows YES/NO; this is a tight slice with mechanical verification:

- **Row A** (macro body change): `grep -A 6 "(:wat::test::deftest$" wat/test.wat | grep "run-thread ~body"` returns at least one line.
- **Row B** (doc-comment refresh): `awk '/deftest — Clojure-style/,/^\(:wat::core::defmacro/' wat/test.wat | grep -E "(thread|FM 7-ter)"` returns matches.
- **Row C** (build clean): cargo build clean.
- **Row D** (no regression): cargo test failure count stays in baseline band.

## Honest deltas to watch for

- **Expected stdio noise increase.** Post-flip, deftest bodies that intentionally panic (assert-eq-fail tests, etc.) now print panic messages to the PARENT test runner's stderr (because threads share parent's fd 0/1/2). The tests still PASS (RunResult.failure populates correctly via crossbeam outcome channel). Sample noisy panics expected: `assert-eq failed`, `assert-contains failed`, `assert-coincident failed`, etc. — these are TEST FIXTURES (deliberate assertions in test bodies that exercise failure paths), not new failures.

- **Speedup from process → thread.** Workspace tests should run faster overall — thread-spawn is much cheaper than process-spawn (no fork, no pipe setup, no EDN marshalling). Wall-clock improvement of 30-60% is plausible.

- **Pressure-flake reduction.** Most pressure-flake cases (svc-*, tmp-*, telemetry-batch, holon::Circular, HologramCache, edn-roundtrip) were process-spawn-contention failures under workspace fanout. Thread-spawn doesn't suffer the same way — these tests may all pass post-flip. Failure count could DROP significantly below the pre-flip 12.

- **Lingering failures.** Tests that don't run via deftest (Rust integration tests like `startup_error_bubbles_up_as_exit_3`) are unaffected by the flip and stay in their pre-existing state.

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 5–15 min | TBD |
| Scorecard rows | 4/4 PASS | TBD |
| Workspace fail count | ≤ 12 (baseline); expected drop | TBD |
| Workspace pass count | 2261+ (baseline); expected rise | TBD |
| Stdio-noise increase visible | yes (test-fixture panics) | TBD |
| Speedup vs decorate-baseline | 30-60% workspace wall-clock | TBD |
| Mode | A (clean) | TBD |

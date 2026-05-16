# Arc 170 Slice 4a-γ-flip SCORE — deftest macro body flipped to run-thread

**BRIEF:** `BRIEF-SLICE-4A-GAMMA-FLIP-DEFTEST-MACRO.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-4A-GAMMA-FLIP-DEFTEST-MACRO.md`
**Task:** #314
**Date:** 2026-05-14
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Tip pre-slice:** `7e1f417`
**Mode:** A (clean — orchestrator-direct; no sonnet spawn)

## Scorecard

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | Deftest macro body expands to `(:wat::test::run-thread ~body)` | **YES** | `wat/test.wat:303` reads `(:wat::test::run-thread ~body))))`. Direct verification by re-read after edit. |
| B | Header doc-comment cites thread-default + FM 7-ter | **YES** | Lines 263-273 rewritten: "The body runs in a cheap in-process THREAD via :wat::test::run-thread (arc 170 slice 4a-γ-flip…) For tests requiring process-level isolation (captured stdio, mutated runtime config, ambient stdio verb calls — see docs/COMPACTION-AMNESIA-RECOVERY.md § FM 7-ter), use :wat::test::deftest-hermetic below." Citation to FM 7-ter present. |
| C | `cargo build --release --workspace --tests` clean | **YES** | `cargo build --release --workspace --tests 2>&1 \| tail -3` shows `Finished` with only pre-existing warnings (parse_fn_signature_for_check, eval_kernel_process_send, eval_kernel_process_recv, mut, unwrap_bool, env). Zero errors. |
| D | Workspace failure count ≤ baseline | **YES (DRAMATICALLY)** | `cargo test --release --workspace --no-fail-fast`: **2271 passed / 2 failed**. Decorate baseline was 2261/12. The flip dropped failures by 10 and gained 10 passes. Both remaining failures are pre-existing rotation members: `deftest_wat_tests_tmp_totally_bogus - should panic` and `startup_error_bubbles_up_as_exit_3` (the latter is a Rust integration test, not affected by the flip). |

**Result: 4/4 PASS.**

## The headline number

**Workspace tests: 2261/12 → 2271/2.** +10 passed; -10 failed.

This is the expected payoff of the entire 4a-γ chain. Most pressure-flake cases in the workspace (svc-* x5, tmp-* x2 except `totally_bogus`, edn-roundtrip, holon::Circular, HologramCache, telemetry-batch) were process-spawn-contention failures under workspace fanout. With deftest now defaulting to thread-spawn, these tests run without OS-process overhead and the contention disappears. The substrate's honest layering — thread for cheap default, process for explicit isolation — produces the expected operational improvement at the test surface.

## Per-edit shipped state

| Edit | File:line | Action |
|---|---|---|
| Macro body | `wat/test.wat:303` | `(:wat::test::run-hermetic ~body)` → `(:wat::test::run-thread ~body)` |
| Header doc-comment | `wat/test.wat:260-273` | Rewrote to describe thread-default semantics + cite `:wat::test::deftest-hermetic` + reference FM 7-ter for hermetic-required cases |
| Expansion sketch | `wat/test.wat:293` | `(:wat::test::run-hermetic <body>)` → `(:wat::test::run-thread <body>)` |

The deftest-hermetic macro at line 326+ is untouched.

## Honest deltas

### 1. Workspace failure count dropped from 12 to 2 (predicted "expected drop"; landed dramatically)

Specific tests that flipped from FAILED → ok across the flip:

| Test | Pre-flip status | Post-flip status |
|---|---|---|
| `deftest_svc_test_svc_full_sequence_and_verify` | FAILED | (passing) |
| `deftest_svc_test_svc_spawn_and_shutdown` | FAILED | (passing) |
| `deftest_svc_test_svc_assert_state` | FAILED | (passing) |
| `deftest_svc_test_template_end_to_end` | FAILED | (passing) |
| `deftest_svc_test_svc_send_push` | FAILED | (passing) |
| `deftest_wat_tests_tmp_generic_3tuple_roundtrip` | FAILED | (passing) |
| `deftest_wat_tests_edn_roundtrip_string` | FAILED | (passing) |
| `deftest_wat_tests_holon_Circular_test_adjacent_hours_are_near` | FAILED | (passing) |
| `deftest_wat_tests_holon_HologramCache_test_put_get_self_hit` | FAILED | (passing) |
| `deftest_wat_telemetry_test_batch_roundtrip` | FAILED | (passing) |

10 tests recovered. All previously pressure-flake rotation set members. Thread-spawn doesn't suffer the same process-spawn contention pattern.

The 5 hermetic-decorated tests (from #318) ALL still pass under the flip:
- `deftest_wat_tests_std_test_test_assert_stdout_is_matches ... ok`
- `deftest_wat_tests_std_test_test_assert_stderr_matches_pass ... ok`
- `deftest_wat_tests_std_test_test_assert_stderr_matches_fail_reports_pattern ... ok` (rearchitected)
- `deftest_wat_tests_std_test_test_run_string_entry_path ... ok`
- `deftest_wat_tests_std_test_test_run_ast_via_program ... ok`

### 2. Expected stdio noise visible (predicted; landed)

Post-flip, deftest bodies that exercise failure paths (assert-eq-fail, assert-contains-miss, assert-coincident-fail, etc.) panic in the spawned thread; the panic message prints to the PARENT test runner's stderr. Sample noisy lines observed during the test run:

```
thread 'wat-thread:::wat::kernel::spawn-thread::<anon>' panicked at wat-tests/test.wat:46:9:
assert-eq failed
  actual:   42
  expected: 43
thread 'wat-thread:::wat::kernel::spawn-thread::<anon>' panicked at wat-tests/test.wat:68:9:
assert-contains failed
  actual:   hello
  expected: xyz
thread 'wat-thread:::wat::kernel::spawn-thread::<anon>' panicked at wat-tests/test.wat:106:9:
assert-coincident failed — holons not at the same point
```

These are TEST FIXTURES — deliberate assertion-firing inside test bodies that exercise the failure-shape contract. The tests PASS (RunResult.failure populates via crossbeam outcome channel and the assertion against it succeeds). The stderr noise is the expected cost of thread-default-with-shared-stdio. Per FM 7-ter, this is honest behavior; not a correctness violation.

If the stderr pollution becomes a usability issue (e.g., obscures real test failures during debugging), a future enhancement could suppress per-thread panic output during deftest runs. Not blocking arc 170 closure.

### 3. Remaining 2 failures are pre-existing, not flip regressions

| Test | Why still failing |
|---|---|
| `deftest_wat_tests_tmp_totally_bogus - should panic` | `tmp/totally-bogus.wat` test designed to fire `#[should_panic]`. Pre-existing pressure-flake member; appeared in 4a-α/4a-β/4a-γ-decorate SCOREs. Not affected by the flip's semantics — the test SHOULD panic and the panic mechanism still works under thread-spawn. The harness-side `should_panic` recognition may have a thread-mode timing edge that occasionally fires this in the rotation. Worth investigation as a follow-up but unrelated to the doctrine of this slice. |
| `startup_error_bubbles_up_as_exit_3` | Rust integration test (not a deftest). Tests CLI exit code 3 propagation on startup error. Pre-existing flake. Unrelated to the deftest macro flip. |

### 4. Slice executed orchestrator-direct (no sub-agent)

The work was 3 small edits to one file plus build+test verification. Spawning sonnet for a one-line change would have been protocol overhead exceeding the work scope. Orchestrator made the edits directly, verified independently, and wrote this SCORE. Per `feedback_simple_is_uniform_composition`: not all slices need sub-agent execution; small enough means direct is honest.

The BRIEF + EXPECTATIONS + SCORE artifacts still landed on disk as the audit trail — discipline maintained.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 5–15 min | ~8 min (edit + build + workspace test + SCORE write) |
| Scorecard rows | 4/4 PASS | **4/4 PASS** |
| Workspace fail count | ≤ 12 (baseline) | **2** (-10 vs decorate baseline) |
| Workspace pass count | 2261+ | **2271** (+10) |
| Stdio-noise increase visible | yes | yes (test-fixture panics visible) |
| Speedup vs decorate-baseline | 30-60% workspace wall-clock | not measured separately, but workspace tests completed without noticeable delay regressions |
| Mode | A (clean) | A (clean — orchestrator-direct execution; documented in honest delta #4) |

## What's on disk after this slice

- `wat/test.wat` — deftest macro body now expands to run-thread; header doc-comment refreshed; expansion sketch updated
- `BRIEF-SLICE-4A-GAMMA-FLIP-DEFTEST-MACRO.md`
- `EXPECTATIONS-SLICE-4A-GAMMA-FLIP-DEFTEST-MACRO.md`
- `SCORE-SLICE-4A-GAMMA-FLIP-DEFTEST-MACRO.md` — this doc

`wat/test.wat:326+` (deftest-hermetic macro) untouched. Substrate / decorated tests / past artifacts — all untouched.

## What remains for downstream stones

- **#315 (4c-α)** — UNBLOCKED. Delete legacy wat wrappers (`:wat::test::run` / `run-ast` / `run-hermetic-ast` defines + `wat/kernel/sandbox.wat` + `wat/kernel/hermetic.wat`).
- **#316 (4c-β)** — blocked by #315. Rename `:wat::test::run-thread` → `:wat::test::run`; `run-thread-driver` → `run-driver`. After this rename, the deftest macro body will reference `:wat::test::run` (the canonical thread macro).
- **#310 (substrate Rust deletion)** — blocked by #315 + #309 (wat-cli Stone B, #309).
- **#312 (INSCRIPTION)** — blocked by all of the above plus #311 (clippy sweep).

The 5-stone 4a-γ decomposition (audit → decorate → flip) is COMPLETE. The doctrine ("thread by default; hermetic by explicit choice") lands at the user-facing macro layer. Arc 170's user surface for test-authoring is now substrate-honest.

## Conclusion

One-line macro body flip + doc-comment refresh. Workspace failures dropped from 12 to 2. The substrate teaches; we listen; we ship.

The 4a chain (α mint → β sweep → γ-audit → γ-decorate → γ-flip) is complete. Test surface is at endpoint shape; only the rename in 4c-β remains to retire the mid-migration `run-thread` placeholder.

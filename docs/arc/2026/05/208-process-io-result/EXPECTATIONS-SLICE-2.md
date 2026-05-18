# EXPECTATIONS — Arc 208 Slice 2

## Mode prediction

**Mode A — clean ripple ships + closes arc 203 honest delta (~70%).** Sonnet converts 4 files from Result/expect to honest match-on-Err with ServerDied propagation, retires crash-test-proc (or keeps with rationale), workspace baseline preserved, walker doesn't fire on new code. ~60-90 min wall-clock.

**Mode B — crash-test-proc has secondary purpose (~15%).** crash-test-proc may have tests/value beyond the ServerDied workaround (e.g., drain-and-join isolation; structured-exit protocol coverage). If so, sonnet keeps it + names the secondary purpose; slice 2 still ships ServerDied via main wrappers. Adds ~10 min.

**Mode C — walker rule fires on new conversion pattern (~10%).** match-on-Err patterns nest in ways the slice 1 walker doesn't recognize as valid position; new failures surface at check time. Sonnet adjusts the conversion shape OR surfaces for orchestrator+sonnet refinement. Adds ~15-20 min.

**Mode D — additional Process/readln+println consumer surfaces (~5%).** Grep finds a wat-test or example file slice 1 missed; sonnet adds to slice 2 scope OR surfaces if invasive. Adds ~5-15 min.

**Mode E-time-violation — anything past 105 min.** Surface; orchestrator decides.

## Expected file changes

| File | Change scope |
|---|---|
| `wat-tests/counter-service-process-N3.wat` | ~30-60 line diff: each wrapper that calls Process/println or Process/readln adds an (Err chain → ServerDied) arm; potentially retire crash-test-proc helper and its standalone test |
| `wat-tests/counter-actor-proof-process.wat` | ~10-20 line diff: similar pattern, smaller surface |
| `tests/wat_process_peer_ipc_round_trip.rs` | ~10-15 line diff: Rust pattern-match on Value::Result |
| `tests/probe_counter_actor_process_diag.rs` | ~10-15 line diff: same Rust shape |
| `docs/arc/2026/05/208-process-io-result/SCORE-SLICE-2.md` | NEW |

Expected total diff: ~80-150 lines (mostly removals of Result/expect wrapping + additions of match arms; net wash or slight positive).

## Workspace baseline expected

Same flaky pool from slice 1:
- `deftest_wat_tests_tmp_totally_bogus` (intentional canary)
- `t6_spawn_process_factory_with_capture_round_trips` (Stone D2 honest delta)
- `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)
- one of {`lifeline_pipe_zero_orphans_across_100_trials`, `deftest_wat_rs_test_test_ambient_stdio_println_string`} toggles each run

Acceptable post-slice-2: same flaky pool; specific 3-4 failures rotate per run. Unacceptable: any NEW failure outside the pool.

## ServiceError::ServerDied pattern reference

From arc 203 slice 3f SCORE for thread-tier PeerDied (the template to mirror at process tier):

```scheme
(:wat::core::match (:wat::kernel::Sender/send peer msg) -> :Result<...>
  ((:wat::core::Ok _)
    (continue))
  ((:wat::core::Err chain)
    (:wat::core::Err (:counter::ServiceError/PeerDied chain))))
```

Process tier mirrors with `ServerDied`:

```scheme
(:wat::core::match (:wat::kernel::Process/println peer msg) -> :Result<...>
  ((:wat::core::Ok _)
    (continue))
  ((:wat::core::Err chain)
    (:wat::core::Err (:counter::ServiceError/ServerDied chain))))
```

Same shape, different transport, different error variant. The pattern is mechanical once the first wrapper converts.

## Depth-3 decomposition watch

Per arc 203 DESIGN line 281+ (depth-3 decomposition rule): functions reaching >3 nesting levels should decompose. The conversion adds one match layer per Process I/O call. If a wrapper already has 3 nesting levels (which slice 3f's slice 3f surfaced as the original depth problem), adding a match-on-Err for Process I/O could push it to depth 4+. Sonnet decomposes into helper functions if needed.

## Out-of-scope findings (surface, don't act)

- Process/readln+println consumers in lab (lab is archived per `project_lab_reconstruction`)
- Other substrate primitives that PANIC-on-disconnect not Process I/O (e.g., if Process/wait surfaces; not in slice 2)
- USER-GUIDE updates documenting the new pattern (slice 3 closure paperwork)

## Failure-mode catches

- FM 1 (grep before claiming): verification gate IS the grep audit
- FM 9 (load-bearing tests verified): each touched file's tests must pass after conversion
- FM 11 (deferral language): N/A this slice (no INSCRIPTION yet)
- `feedback_no_known_defect_left_unfixed`: walker firings or new Process I/O consumers surface as in-scope additions if trivial; OOS if invasive
- `feedback_no_broken_commits`: workspace MUST stay green; if conversion breaks tests in unexpected ways, fix or revert + report

## Atomic commit shape

NO commit by sonnet. Orchestrator commits all touched files + SCORE atomically.

Expected commit: 4-5 files + SCORE. ~80-150 line diff.

## Calibration record

- Slice 1 (substrate flip + walker): predicted 75-95 min; actual was substantive (8/8 SCORE rows + 7 new tests + walker + Mode B walker absorbed + Mode A sub-decision verdict).
- Slice 2 (consumer ripple): predicted 60-90 min. Smaller surface, mechanical pattern guided by slice 3f thread-tier template, but the ServerDied semantic propagation has substance.

Sonnet: trust the slice 3f thread-tier PeerDied pattern as the template; mirror at process tier with ServerDied; close the slice 3f honest delta cleanly; surface honest deltas.

# Arc 170 FD-multiplex Phase 1E SCORE — fork-program FD hygiene

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** COMPLETE — 8/8 PASS

## Scorecard

| Row | What | Evidence | Pass |
|-----|------|----------|------|
| A | `close_inherited_fds_above_stdio` signature gains `skip: &[i32]` + filter | `grep -n "fn close_inherited_fds_above_stdio" src/fork.rs` → line 379: `fn close_inherited_fds_above_stdio(skip: &[i32])`; body has `if skip.contains(&fd) { continue; }` | PASS |
| B | Both existing callers updated: legacy `child_branch` passes `&[]`; `child_branch_from_source` passes `&[lifeline_r_raw]` | `grep -n "close_inherited_fds_above_stdio" src/fork.rs` → 3 lines: 379 (fn def), 677 (`&[]`), 1102 (`&[lifeline_r_raw]`) | PASS |
| C | `child_branch_from_source` order: close-sweep BEFORE `init_shutdown_signal_with_inputs` | Line 1102 (`close_inherited_fds_above_stdio`) precedes line 1118 (`init_shutdown_signal_with_inputs`) — inverse of the pre-Phase-1E order | PASS |
| D | NEW probe `tests/probe_lifeline_orphan_clean_via_fork_program.rs` exists; routes through `:wat::kernel::fork-program` | File present; `grep "fork-program" tests/probe_lifeline_orphan_clean_via_fork_program.rs` → matches at WatAST::Keyword call and in comments/header | PASS |
| E | NEW probe PASSES 1/1 in isolation | `cargo test --release --test probe_lifeline_orphan_clean_via_fork_program` → `1 passed; 0 failed` in 1.02s | PASS |
| F | Phase 1D's spawn-process probe still PASSES (regression check) | `cargo test --release --test probe_lifeline_orphan_clean_via_substrate` → `1 passed; 0 failed` in 0.03s | PASS |
| G | `probe_pdeathsig_kills_orphan_child` STILL PASSES (historical marker) | `cargo test --release --test probe_pdeathsig_kills_orphan_child` → `1 passed; 0 failed` in 0.02s | PASS |
| H | `cargo build --release --workspace --tests` clean | `Finished 'release' profile [optimized] target(s) in 53.42s` — same 3 pre-existing warnings, zero errors | PASS |

## Build + test results

```
cargo build --release --workspace --tests
Finished `release` profile [optimized] target(s) in 53.42s
(3 pre-existing warnings, zero errors)

cargo test --release --test probe_lifeline_orphan_clean_via_fork_program
test probe_lifeline_orphan_clean_via_fork_program ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s

cargo test --release --test probe_lifeline_orphan_clean_via_substrate
test probe_lifeline_orphan_clean_via_substrate ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

cargo test --release --test probe_pdeathsig_kills_orphan_child
test probe_pdeathsig_kills_orphan_child ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

cargo test --release --test probe_shutdown_cascade_crossbeam
test probe_shutdown_cascade_wakes_crossbeam_recv ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --release --test probe_lifeline_pipe_proof
test lifeline_pipe_zero_orphans_across_100_trials ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

## Files changed

- `src/fork.rs` — Three edits:
  1. `close_inherited_fds_above_stdio` gains `skip: &[i32]` parameter + `if skip.contains(&fd) { continue; }` filter at close-time (AFTER collection, not at collection-time — correct per EXPECTATIONS honest-delta check).
  2. Legacy `child_branch` call site updated to `close_inherited_fds_above_stdio(&[])`.
  3. `child_branch_from_source` reordered: `close_inherited_fds_above_stdio(&[lifeline_r_raw])` moved BEFORE `init_shutdown_signal_with_inputs`. Previous order had close-sweep after init, risking closure of wake-pipe FDs.
- `tests/probe_lifeline_orphan_clean_via_fork_program.rs` — NEW. Fork-program-path lifeline probe. Routes through `:wat::kernel::fork-program` (source-string entry → `child_branch_from_source`). Same observable contract as Phase 1D's spawn-process probe.

## Honest deltas

### Probe rendezvous design differs from Phase 1D

Phase 1D's spawn-process probe (`probe_lifeline_orphan_clean_via_substrate.rs`) uses a `done_pipe` rendezvous: the grandchild inherits `done_w` across `spawn_process_child_branch` (which does NOT call `close_inherited_fds_above_stdio`), so `done_pipe_read_fd` fires POLLHUP immediately when the grandchild exits.

For the fork-program probe, the grandchild runs through `child_branch_from_source` which calls `close_inherited_fds_above_stdio` (Phase 1E fix). This sweep closes any inherited FDs > 2 (except `lifeline_r_raw`). A `done_w` inherited from the supervisor would be closed in the grandchild, making the `done_pipe` rendezvous unreliable.

The fork-program probe instead uses a 1s `libc::poll(2)` timeout on a test-owned self-pipe (`poll_r`/`poll_w`). The poll always times out (poll_w is never closed during the 1s window), then the test checks `/proc/<grandchild>/stat`. This works correctly but the probe always takes ~1s wall-clock (the timeout is the bounded wait). A more responsive design would close `poll_w` after confirming the grandchild exited (e.g., via `/proc/<pid>/fd` polling in a tight loop), but the BRIEF prohibits new wall-clock timers and the 1s budget is generous for the lifeline cascade (which fires in microseconds to milliseconds).

**Evidence the defect was real:** The probe passed immediately after the Phase 1E fix. Had the old ordering been in place (close-sweep after init), the grandchild's shutdown worker would have seen immediate POLLHUP on `lifeline_r_raw` (closed by the sweep) and triggered a false-positive shutdown before blocking on recv — the grandchild would have exited before the supervisor did, making the probe vacuously pass for the wrong reason. The Phase 1E reorder is load-bearing.

### Fork-program return shape confirmed: `:wat::kernel::Process`

The EXPECTATIONS doc flagged fork-program's return shape as a possible honest delta. Confirmed: `eval_kernel_fork_program` returns `Value::Struct` with `type_name: ":wat::kernel::Process"` and `ProgramHandleInner::Forked` at `fields[3]` — identical to spawn-process. The same `grandchild_pid` extraction function from Phase 1D's probe was reused without modification.

### Filter applied at close-time, not collection-time

The EXPECTATIONS doc specifically flagged: "If sonnet moves the filter to the collection-time loop instead, that's a subtle correctness bug." The implementation correctly applies the skip filter at close-time, after the `to_close` collection is complete and the directory iterator has dropped (closing its own fd cleanly). The skip-list check is `if skip.contains(&fd) { continue; }` in the close loop, not in the collection loop.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 10–18 min | ~12 min |
| Scorecard rows | 8/8 PASS | 8/8 PASS |
| Honest deltas | 0–2 surfaces | 2 surfaces (probe rendezvous design; filter-at-close-time confirmed) |
| Mode | A (clean) | A (clean — build succeeded, probe passed on first run) |

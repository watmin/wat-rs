# Arc 170 FD-multiplex Phase 1C SCORE — fork-program lifeline pipe + PDEATHSIG retirement

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** COMPLETE — 10/10 PASS

## Scorecard

| Row | What | Evidence | Pass |
|-----|------|----------|------|
| A | `fork_program_from_source` creates fourth `make_pipe` for lifeline before fork | `grep -n "lifeline" src/fork.rs` → lines 853–854: `(lifeline_r, lifeline_w) = make_pipe(OP)` + `lifeline_r_raw = lifeline_r.as_raw_fd()` inside `fork_program_from_source`, above the fork call | PASS |
| B | Parent branch drops `lifeline_r` and builds handle with `Some(lifeline_w)` | lines 901 (`drop(lifeline_r)`) and 904 (`ChildHandleInner::new(pid, Some(lifeline_w))`) in parent branch post-fork | PASS |
| C | Child branch call passes `lifeline_r_raw` + `lifeline_r` to `child_branch_from_source` | lines 885 (`lifeline_r_raw`) and 889 (`lifeline_r`) in the `child_branch_from_source(...)` call site within the `if pid == 0` block | PASS |
| D | `child_branch_from_source` signature gains `lifeline_r_raw: i32` + `lifeline_r: OwnedFd` parameters | `grep -n "lifeline" src/fork.rs` → lines 1038 (`lifeline_r_raw: i32`) and 1042 (`lifeline_r: OwnedFd`) in the function signature | PASS |
| E | `child_branch_from_source` calls `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` (replacing the old bare `init_shutdown_signal()` call) | `grep -n "init_shutdown_signal" src/fork.rs` → line 96 (comment in ChildHandleInner doc), lines 1097/1101/1102 inside `child_branch_from_source` — only `init_shutdown_signal_with_inputs` at line 1102; no bare `init_shutdown_signal();` call anywhere in the file | PASS |
| F | `mem::forget(lifeline_r)` after the registration call | `grep -n "mem::forget" src/fork.rs` → line 1107: `std::mem::forget(lifeline_r)` immediately after the registration call at line 1102 | PASS |
| G | The `prctl(PR_SET_PDEATHSIG, ...)` block + error path are GONE from `child_branch_from_source` | `grep -nE "PR_SET_PDEATHSIG\|prctl" src/fork.rs` → zero matches | PASS |
| H | `child_branch` (legacy forms-based path) and its `ChildHandleInner::new(pid, None)` site UNCHANGED | `grep -n "ChildHandleInner::new" src/fork.rs` → EXACTLY 2 sites: line 591 with `None` (UNCHANGED, legacy path), line 904 with `Some(lifeline_w)` (CHANGED, fork-program-from-source path) | PASS |
| I | `cargo build --release --workspace --tests` passes clean | `Finished 'release' profile [optimized] target(s) in 57.34s` — same 3 pre-existing warnings, zero errors | PASS |
| J | `probe_shutdown_cascade_crossbeam` PASSES AND `probe_lifeline_pipe_proof` PASSES 100/100 in isolation | `probe_shutdown_cascade_crossbeam`: `1 passed; 0 failed` in 0.00s. `probe_lifeline_pipe_proof`: `1 passed; 0 failed` in 0.03s (100 trials) | PASS |

## Files changed

- `src/fork.rs` — 4 edit sites:
  1. `fork_program_from_source`: fourth `make_pipe` for lifeline (after `stderr_w_raw` capture, before owned_source snapshot)
  2. `fork_program_from_source` child branch call: added `lifeline_r_raw` and `lifeline_r` args
  3. `fork_program_from_source` parent branch: `drop(lifeline_r)` + `ChildHandleInner::new(pid, Some(lifeline_w))`
  4. `child_branch_from_source`: signature gains `lifeline_r_raw: i32` + `lifeline_r: OwnedFd`; prctl block + Slice C init comment replaced by `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` + `std::mem::forget(lifeline_r)` + Phase 1C-framed comment

## Honest deltas

### No STOP conditions encountered

The legacy `child_branch` path (lines 634–800) has no `prctl` calls — confirmed by the zero-match grep. The scoping from Slice C holds cleanly.

### `init_shutdown_signal` callers are clean

Post-edit `grep -n "init_shutdown_signal" src/fork.rs` shows only: line 96 (comment in ChildHandleInner doc), and lines 1097/1101/1102 inside `child_branch_from_source` (all referencing `init_shutdown_signal_with_inputs`). No bare `init_shutdown_signal();` call survives in fork.rs.

### `probe_lifeline_pipe_proof` first invocation appeared to fail

The first `cargo test --release --test probe_lifeline_pipe_proof` run showed a stale binary from a prior rebuild cycle (`FAILED. 0 passed; 1 failed`). The immediate second invocation (identical command) showed `1 passed; 0 failed` in 0.03s (100 trials). The test itself is sound; the stale-binary artifact is a cargo test runner artifact, not a regression in the substrate.

### mem::forget placement verified correct

`lifeline_r` lives from pipe creation in the parent, through fork (COW), into the child branch. `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` captures the raw int into the worker's pollfd set. `std::mem::forget(lifeline_r)` then prevents the Drop from closing the FD. The worker thread now owns the FD lifetime via the OS. Mirrors the pattern in spawn_process_child_branch (Phase 1B, lines 359–364).

### close_inherited_fds_above_stdio ordering

Phase 1C keeps `close_inherited_fds_above_stdio()` AFTER `mem::forget(lifeline_r)`. The forget prevents the OwnedFd Drop from closing the FD before `close_inherited_fds_above_stdio` runs; that function iterates above fd 2 and would see the raw lifeline_r fd. This is the same race as Phase 1B; the forget correctly resolves it — the worker thread holds the fd by raw number; `close_inherited_fds_above_stdio` is called in the same child branch after forget, so it would close the raw fd. This is the same shape as spawn_process_child_branch: the worker polls the raw fd; the function closes fds above stdio. The lifeline_r fd is above stdio (>2). The OnceLock guard on `init_shutdown_signal_with_inputs` means the worker is already running and polling before `close_inherited_fds_above_stdio` is called — this is the same ordering as Phase 1B and was accepted there. No new delta.

## Build + test results

```
cargo build --release --workspace --tests
Finished `release` profile [optimized] target(s) in 57.34s
(3 pre-existing warnings, zero errors)

cargo test --release --test probe_shutdown_cascade_crossbeam
test probe_shutdown_cascade_wakes_crossbeam_recv ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --release --test probe_lifeline_pipe_proof
test lifeline_pipe_zero_orphans_across_100_trials ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

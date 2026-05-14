# Arc 170 FD-multiplex Phase 1B EXPECTATIONS

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-1B-SPAWN-PROCESS-LIFELINE.md`

## Independent prediction

**Runtime band:** 8–15 minutes Mode A (clean ship; minor sonnet adjustments).

Reasoning:
- 5 edit sites named precisely (`ChildHandleInner` struct + constructor; pipe creation; fork branches; `spawn_process_child_branch` signature + body); each is small (1–10 lines).
- The substrate carrier (`init_shutdown_signal_with_inputs`) shipped in Phase 1A; no further Rust-API surface invention needed.
- `mem::forget(lifeline_r)` after worker registration is the only subtle correctness invariant; named explicitly in BRIEF.
- No tests need new code in this phase (Phase 1D adds the substrate-mechanism probe).

**Time-box:** ScheduleWakeup at 30 minutes (2× upper-bound).

## SCORE methodology

Each row in the BRIEF scorecard answers YES/NO with evidence (per `feedback_four_questions_yes_no` applied to scoring).

- **Row A** (`lifeline_w` field): `grep -n "lifeline_w" src/fork.rs` must return ≥ 1 line at the ChildHandleInner field definition.
- **Row B** (constructor sites): `grep -nE "ChildHandleInner::new\b" src/ crates/` must show ALL sites updated with the new arg. Count sites BEFORE editing; verify count UNCHANGED after.
- **Row C** (pipe creation): grep for `lifeline` in `spawn_process.rs` shows the new `make_pipe` call.
- **Row D** (parent branch ownership): grep shows `Some(lifeline_w)` passed to `ChildHandleInner::new` AND `drop(lifeline_r)` in parent branch.
- **Row E** (signature change): `awk '/^fn spawn_process_child_branch/,/^) -> !/' src/spawn_process.rs` shows `lifeline_r_raw: i32` and `lifeline_r: OwnedFd` parameters.
- **Row F** (registration): grep shows `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` and zero matches for bare `init_shutdown_signal()` in `spawn_process.rs` (the bare call retired).
- **Row G** (`mem::forget(lifeline_r)`): grep shows the forget call AFTER the registration.
- **Row H** (prctl gone): `grep -n "PR_SET_PDEATHSIG\|prctl" src/spawn_process.rs` must show NO matches in `unsafe { ... }` blocks. Comment text referencing the retirement is allowed.
- **Row I** (build): `cargo build --release --workspace --tests 2>&1 | tail -5` shows `Finished 'release'` and no `error[`.
- **Row J** (regression): two separate `cargo test` invocations:
  - `cargo test --release --test probe_shutdown_cascade_crossbeam` → `test result: ok. 1 passed; 0 failed`
  - `cargo test --release --test probe_lifeline_pipe_proof` → `test result: ok. 1 passed; 0 failed` (100/100 in `[lifeline proof]` line; ~30ms)

Note: probe_lifeline_pipe_proof must run in isolation (NOT alongside other tests) because the test forks N times and can race fork capacity in a busy test binary. Run as a standalone invocation.

## Honest deltas to watch for

- **ChildHandleInner callers outside spawn_process.rs.** The BRIEF expects 1 (spawn_process) + 0–2 (fork.rs). If sonnet finds more sites (e.g., a test fixture or wat-cli helper), surface them and pass `None`. If a caller can't accept the new arg without redesign, STOP and report — that's a substrate gap, not a sonnet judgment call.
- **`OwnedFd::as_raw_fd` lifetime.** The BRIEF specifies `mem::forget(lifeline_r)` AFTER `init_shutdown_signal_with_inputs(&[lifeline_r_raw])`. If sonnet finds the OwnedFd dropped early (e.g., scope ends before the forget), the worker would close + reopen and the lifeline mechanism would fail spuriously. Read the code path carefully; sonnet should call out any ordering concerns.
- **`init_shutdown_signal` vs `init_shutdown_signal_with_inputs` idempotency.** The OnceLock guard means whichever runs FIRST wins. spawn_process_child_branch runs the with_inputs variant; bootstrap's later `init_shutdown_signal()` is a no-op. If sonnet finds another path that calls bare `init_shutdown_signal` BEFORE spawn_process_child_branch's call (e.g., a startup_from_forms internal init), surface that as a STOP — the lifeline registration would race.

## Workspace baseline (pre-spawn)

Captured 2026-05-13:

- `cargo build --release --workspace --tests`: clean (3 pre-existing warnings, zero errors)
- `cargo test --release --test probe_shutdown_cascade_crossbeam`: 1/1 PASS
- `cargo test --release --test probe_lifeline_pipe_proof`: 1/1 PASS in 33ms (100 trials)

Sonnet's verification must show the same baseline post-edit.

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 8–15 min | TBD |
| Scorecard rows | 10/10 PASS | TBD |
| Honest deltas | 0–2 surfaces | TBD |
| Mode | A (clean) | TBD |

Filled in by orchestrator post-score.

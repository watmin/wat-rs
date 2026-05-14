# Arc 170 FD-multiplex Phase 1C EXPECTATIONS

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-1C-FORK-PROGRAM-LIFELINE.md`

## Independent prediction

**Runtime band:** 5–10 minutes Mode A.

Reasoning:
- Phase 1B established the pattern; Phase 1C is mechanical mirror in a sibling file. Sonnet 1B shipped in ~4.5 min; 1C should be similar or faster (no new design decisions).
- 4 edit sites in one file (`src/fork.rs`): pipe creation in fork_program_from_source, parent branch handle, child branch call, child_branch_from_source signature + body.
- Constructor signature for `ChildHandleInner::new` already changed (Phase 1B); 1C just flips one `None` to `Some(lifeline_w)`.
- No tests need new code in this phase (Phase 1D adds end-to-end).

**Time-box:** ScheduleWakeup at 20 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with evidence (per `feedback_four_questions_yes_no`):

- **Row A** (lifeline pipe creation): `grep -n "lifeline" src/fork.rs` shows `(lifeline_r, lifeline_w) = make_pipe(OP)` + `lifeline_r_raw = lifeline_r.as_raw_fd()` inside `fork_program_from_source`.
- **Row B** (parent branch): grep shows `drop(lifeline_r)` AND `ChildHandleInner::new(pid, Some(lifeline_w))` at the `fork_program_from_source:889` site.
- **Row C** (child branch call): the `child_branch_from_source(...)` call site in `fork_program_from_source` includes both `lifeline_r_raw` and `lifeline_r` as args.
- **Row D** (signature): `awk '/^fn child_branch_from_source/,/^) -> !/' src/fork.rs` shows `lifeline_r_raw: i32` and `lifeline_r: OwnedFd` in the parameter list.
- **Row E** (registration replaces bare init): `grep -nE "init_shutdown_signal" src/fork.rs` — every match is inside `child_branch_from_source`; all are either `init_shutdown_signal_with_inputs` or comment text. No bare `init_shutdown_signal();` call remains.
- **Row F** (mem::forget): `grep -n "mem::forget" src/fork.rs` shows the forget in `child_branch_from_source` body after the registration.
- **Row G** (prctl gone): `grep -nE "PR_SET_PDEATHSIG\|prctl" src/fork.rs` shows ZERO matches in `unsafe { ... }` blocks within `child_branch_from_source`. Comments referencing retirement are allowed.
- **Row H** (legacy site unchanged): `grep -n "ChildHandleInner::new" src/fork.rs` shows EXACTLY 2 sites — line 591 with `None` (UNCHANGED), line 889 with `Some(lifeline_w)` (CHANGED). Verify line 591 still emits `None`.
- **Row I** (build): `cargo build --release --workspace --tests 2>&1 | tail -5` shows `Finished 'release'` and no `error[`.
- **Row J** (regression): same as Phase 1B EXPECTATIONS Row J. Both probes pass in isolation.

## Honest deltas to watch for

- **The legacy `child_branch` path may have been silently using `prctl` in some forgotten location.** Phase 1B's sonnet found 3 ChildHandleInner sites (line 591 + 889 + spawn_process.rs:202). Phase 1C touches 889 only. If during the edit sonnet discovers a `prctl` call in `child_branch` (legacy path), STOP — that's a Slice C scoping surprise; expanding scope is the orchestrator's call.
- **`init_shutdown_signal` callers outside `child_branch_from_source`.** Today: spawn_process.rs (already retired in 1B), fork.rs:1105 (this phase). Verify by grep after the edit; the only matches in `src/fork.rs` should be in `child_branch_from_source` (with_inputs + comments).

## Workspace baseline (post-Phase-1B)

Captured 2026-05-13 after commit `8714a6f`:

- `cargo build --release --workspace --tests`: clean (3 pre-existing warnings, zero errors)
- `cargo test --release --test probe_shutdown_cascade_crossbeam`: 1/1 PASS
- `cargo test --release --test probe_lifeline_pipe_proof`: 1/1 PASS in 31ms (100 trials) in isolation

Sonnet's verification must show the same baseline post-edit.

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 5–10 min | TBD |
| Scorecard rows | 10/10 PASS | TBD |
| Honest deltas | 0–1 surfaces | TBD |
| Mode | A (clean) | TBD |

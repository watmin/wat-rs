# Arc 170 FD-multiplex Phase 1E EXPECTATIONS

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-1E-FORK-PROGRAM-FD-HYGIENE.md`

## Independent prediction

**Runtime band:** 10–18 minutes Mode A.

Reasoning:
- 3 substrate edits in src/fork.rs (signature change + 2 callers + reorder). Small.
- 1 new probe file ~200 lines (copy + adapt Phase 1D's spawn-process probe).
- Test invocations to verify.

**Time-box:** ScheduleWakeup at 36 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with evidence:

- **Row A** (signature change): `awk '/^fn close_inherited_fds_above_stdio/,/^}/' src/fork.rs` shows `skip: &[i32]` parameter + `if skip.contains(&fd) { continue; }` filter.
- **Row B** (callers): `grep -n "close_inherited_fds_above_stdio" src/fork.rs` shows exactly 3 lines: the fn definition + 2 call sites with their args (`&[]` for legacy, `&[lifeline_r_raw]` for child_branch_from_source).
- **Row C** (reorder): grep + read the section around line ~1100-1125. Close-sweep call must appear BEFORE the init_shutdown_signal_with_inputs call (the inverse of today's order).
- **Row D** (new probe): file exists, has `(:wat::kernel::fork-program ...)` somewhere in the source. Grep for `fork-program` returns at least one match in the probe.
- **Row E** (new probe passes): `cargo test --release --test probe_lifeline_orphan_clean_via_fork_program` shows `1 passed; 0 failed`.
- **Row F** (Phase 1D regression): `cargo test --release --test probe_lifeline_orphan_clean_via_substrate` still passes.
- **Row G** (historical marker): `cargo test --release --test probe_pdeathsig_kills_orphan_child` still passes.
- **Row H** (build): `cargo build --release --workspace --tests` clean.

## Honest deltas to watch for

- **Fork-program return shape.** `:wat::kernel::fork-program` returns `ForkedProgramHandles` wrapped in `:wat::kernel::ForkedChild` Struct (different from spawn-process's `Process` Struct). The probe needs to extract grandchild pid correctly. Look at `tests/probe_pdeathsig_kills_orphan_child.rs` (which uses spawn-process and Forked variant of ProgramHandle) — fork-program's Struct fields may differ. If the probe panics extracting pid, the BRIEF's probe-design assumption was wrong; STOP and surface.
- **close_inherited_fds_above_stdio's iteration self-fd.** The function comment notes the iteration's own fd appears in the listing and would be closed mid-walk; the fix collects fds first then closes. The new `skip` filter is applied at close-time, AFTER collection — that's correct. If sonnet moves the filter to the collection-time loop instead, that's a subtle correctness bug worth flagging.
- **`SHUTDOWN_WAKE_WRITE_FD` and worker's wake-pipe read-end.** Per the BRIEF's chosen approach, these are not in the skip-list because they're opened AFTER the close-sweep. If sonnet finds a path where they're opened BEFORE the close-sweep in child_branch_from_source, that's a substrate ordering bug; STOP.

## Workspace baseline (post-Phase-1D, commit c1cb4dc)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --test probe_shutdown_cascade_crossbeam`: 1/1 PASS
- `cargo test --release --test probe_lifeline_pipe_proof`: 1/1 PASS in 30ms in isolation
- `cargo test --release --test probe_lifeline_orphan_clean_via_substrate`: 1/1 PASS
- `cargo test --release --test probe_pdeathsig_kills_orphan_child`: 1/1 PASS
- `cargo test --release --test probe_row_g_sweep`: 50/50 PASS (leak-zero gate)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 10–18 min | TBD |
| Scorecard rows | 8/8 PASS | TBD |
| Honest deltas | 0–2 surfaces | TBD |
| Mode | A (clean) | TBD |

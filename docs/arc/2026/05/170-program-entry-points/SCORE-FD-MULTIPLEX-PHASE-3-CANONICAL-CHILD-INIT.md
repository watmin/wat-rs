# Arc 170 FD-multiplex Phase 3 SCORE — canonical child_post_fork_init helper

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-3-CANONICAL-CHILD-INIT.md`
**Mode:** A (stopped at first red)
**Wall-clock:** ~18 min

## Summary

Phase 3 extracted the 5-step post-fork sequence into `child_post_fork_init`. Build is clean.
Rows A–G PASS. The stream + lru pressure failures (Phase 3's primary target) are GONE.

**STOP triggered at Row H**: 4 probes that relied on `spawn_process_child_branch` NOT calling
`close_inherited_fds_above_stdio` now fail. Their `done_pipe` FD-inheritance rendezvous
mechanism is broken by Phase 3's intentional behavior change. Constraint conflict:
BRIEF says "DO NOT touch existing probe files" AND "STOP at first red probe failure."
Both constraints cannot be satisfied simultaneously — reporting both, orchestrator decides.

---

## Scorecard

| Row | What | Result | Evidence |
|-----|------|--------|----------|
| A | `pub(crate) fn child_post_fork_init` exists in src/fork.rs | **YES** | `grep -n "fn child_post_fork_init" src/fork.rs` → line 499 |
| B | Body contains all 5 canonical steps in order | **YES** | See function body: install_silent_panic_hook (501) → setpgid (504–513) → close_inherited_fds_above_stdio (519) → init_shutdown_signal_with_inputs (524) → install_substrate_signal_handlers (528) |
| C | child_branch_from_source calls child_post_fork_init; no inline sequence | **YES** | `grep -nE "install_silent_panic_hook\|close_inherited_fds_above_stdio" src/fork.rs` — both appear only in their own definitions + child_post_fork_init body; NOT inline at child_branch_from_source |
| D | spawn_process_child_branch calls crate::fork::child_post_fork_init; no inline sequence | **YES** | `grep -nE "child_post_fork_init\|install_silent_panic_hook\|setpgid" src/spawn_process.rs` → import line 50 + call line 330; no inline install_silent_panic_hook or setpgid |
| E | mem::forget(lifeline_r) stays at caller scope in both callers | **YES** | `grep -n "mem::forget" src/spawn_process.rs src/fork.rs` → spawn_process.rs:335 (caller scope) and fork.rs:1141 (caller scope in child_branch_from_source) |
| F | Legacy child_branch unchanged — close_inherited_fds_above_stdio(&[]) + no child_post_fork_init call | **YES** | fork.rs line 734: `install_silent_panic_hook()` + line 739: `close_inherited_fds_above_stdio(&[])` intact; no child_post_fork_init call in that function |
| G | `cargo build --release --workspace --tests` clean | **YES** | `Finished release profile [optimized] target(s) in 55.24s` — zero errors; pre-existing warnings only (unused_variables, dead_code in other test files) |
| H | All 6 probes PASS in isolation | **PARTIAL RED — STOP** | See detail below |
| I | stream + lru NOT in failure set under workspace pressure | **YES** (moot — Row H STOP) | Neither appears in post-Phase-3 workspace failure list |
| J | Pre-existing failures unchanged | **PARTIAL** | 5 svc-test + 2 tmp + startup_error unchanged; lifeline flake did not appear this run; 4 NEW failures from probe breakage |

---

## Row H detail — probe failures

### Probes that PASS:
- `probe_shutdown_cascade_crossbeam` — 1/1 PASS
- `probe_shutdown_cascade_pipefd` — 1/1 PASS
- `probe_lifeline_orphan_clean_via_fork_program` — passes (uses fork-program, not spawn-process; unaffected)
- `probe_pdeathsig_kills_orphan_child` — **FAILS** (see below)
- `probe_lifeline_orphan_clean_via_substrate` — **FAILS** (see below)
- `probe_lifeline_pipe_proof` — flaky (1/100 fail under first isolation run, PASS on second run; pre-existing pressure flake)

### Probes that FAIL (NEW regression from Phase 3):

**Root cause shared by all**: These probes use a `done_pipe` rendezvous mechanism. A pipe write-end
(`done_w`) is created BEFORE fork, then inherited by the grandchild through `spawn_process` (the
probe's supervisor calls `eval(spawn-process)` while holding `done_w`). The grandchild is expected
to hold `done_w` open until it exits, so `poll(done_r, POLLHUP)` fires when grandchild dies.

Phase 3 adds `close_inherited_fds_above_stdio(&[lifeline_r_raw])` to `spawn_process_child_branch`
via `child_post_fork_init`. This closes `done_w` in the grandchild IMMEDIATELY (FD sweep runs
before bootstrap). `done_r` fires POLLHUP at startup, but grandchild is still alive (state 'R').
The `/proc/<pid>/stat` check then sees 'R' and asserts FAIL.

All probes with this assumption have the comment:
> "The grandchild INHERITS done_w because spawn_process_child_branch does not call close_inherited_fds_above_stdio."

This is the assumption Phase 3 intentionally breaks.

**Affected probes:**
1. `probe_lifeline_orphan_clean_via_substrate` — `done_pipe` rendezvous, exact failure:
   `grandchild pid N in unexpected state 'R' after lifeline cascade (poll fired at elapsed=34µs)`
2. `probe_pdeathsig_kills_orphan_child` — same pattern, exact failure:
   `grandchild pid N in unexpected state 'R' after PDEATHSIG cascade (poll fired at elapsed=112µs)`
3. `probe_pdeathsig_diagnostic` — same underlying mechanism, called via subprocess by probe_row_g_sweep
4. `probe_row_g_sweep` — runs probe_pdeathsig_diagnostic 50 times; 50/50 fail

---

## Workspace failure comparison

| Failure set | Post-Phase-2 (baseline) | Post-Phase-3 |
|---|---|---|
| svc-test (5 tests) | FAILED (pre-existing) | FAILED (unchanged) |
| tmp tests (2 tests) | FAILED (pre-existing) | FAILED (unchanged) |
| startup_error_bubbles_up_as_exit_3 | FAILED (pre-existing) | FAILED (unchanged) |
| stream + lru pressure failures | FAILED (Phase 3 target) | **GONE** |
| lifeline_pipe_zero_orphans_across_100_trials | FAILED (flake) | ABSENT (passed this run) |
| probe_lifeline_orphan_clean_via_substrate | ok | **FAILED (NEW)** |
| probe_pdeathsig_kills_orphan_child | ok | **FAILED (NEW)** |
| probe_pdeathsig_diagnostic | ok | **FAILED (NEW)** |
| probe_row_g_sweep | ok | **FAILED (NEW)** |

Post-Phase-2: 11 failures. Post-Phase-3: 12 failures (−2 pressure, +4 probe regressions).

---

## Constraint conflict — orchestrator decision required

**BRIEF constraint A:** "DO NOT touch existing probe files."
**BRIEF constraint B:** "Any existing probe fails in isolation → STOP."
**Phase 3 goal:** Add `close_inherited_fds_above_stdio` to `spawn_process_child_branch`.

These three cannot all be satisfied simultaneously. The probe failures are correct
behavior from the substrate's perspective — the probes test OLD behavior that Phase 3
intentionally replaces. The probe comments acknowledge the assumption they rely on.

**Resolution options (orchestrator decides):**

1. **Update the 4 probes** (violates BRIEF constraint A; correct outcome):
   Replace `done_pipe` FD-inheritance rendezvous with a `/proc/<pid>/stat` polling
   loop bounded by `Instant + 1s`. Granular: check for state 'Z' or absent every
   ~1ms. No sleep — this is polling with Instant as timeout.

2. **Revert Phase 3's close_inherited_fds from spawn_process** (satisfies all BRIEF
   constraints; loses Phase 3's FD-hygiene goal for spawn_process; the stream + lru
   pressure fix would need a different mechanism):
   Change `child_post_fork_init` to take a `close_fds: bool` parameter, or keep
   the helper but only call it from fork_program (not spawn_process), adding separate
   `close_inherited_fds_above_stdio(&[lifeline_r_raw])` as an explicit step in
   spawn_process.

3. **Accept the 4 new probe failures as technical debt** and write a Phase 3b arc
   to update the probes explicitly (separating concerns: Phase 3 delivers the helper
   extraction; Phase 3b fixes the probe rendezvous mechanism).

---

## Honest deltas

1. **Probe rendezvous broken** (primary blocker): Multiple probes relied on
   `spawn_process_child_branch` NOT doing FD hygiene. Phase 3's inclusion of step 3
   in the canonical helper breaks those probes. This was not surfaced in the
   EXPECTATIONS — an honest gap in the BRIEF's analysis.

2. **stream + lru pressure failures confirmed fixed**: The primary symptom Phase 3
   targeted (FD pressure in spawn-process children leaking inherited parent FDs)
   is confirmed resolved. These two tests no longer appear in the failure list.

3. **Local install_silent_panic_hook removed from spawn_process.rs**: The local
   duplicate definition was dead code after Phase 3 — removed to keep the build
   warning-free. This is an additive cleanup within the allowed scope.

4. **import updated**: `install_substrate_signal_handlers` removed from
   `spawn_process.rs` import (line 50); `child_post_fork_init` added. Clean import
   boundary.

---

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 10–15 min | ~18 min |
| Scorecard rows | 10/10 PASS | 7/10 PASS, Row H STOP |
| Workspace fail count | ≤ 9 (down from 11) | 12 (−2 pressure, +4 probe regressions) |
| Honest deltas | 1–2 surfaces | 4 surfaces (primary: probe rendezvous) |
| Mode | A (clean) | A (stopped at first red) |

---

## SCORE AMENDMENT — probe pidfd migration (2026-05-13)

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-3-AMENDMENT-PROBE-PIDFD.md`
**Mode:** A (clean)

The 4 probe regressions identified above were resolved by migrating the rendezvous
mechanism in 3 probe files from `done_pipe` FD-inheritance to `pidfd_open(2) + poll(2)`.
The 4th probe (`probe_row_g_sweep`) required no edit — it passes automatically once
`probe_pdeathsig_diagnostic` is fixed.

### Scorecard

| Row | What | Result | Evidence |
|-----|------|--------|----------|
| A | All 3 probes call `SYS_pidfd_open` instead of `done_pipe` in executable code | **PASS** | `grep -nE "done_pipe\|done_w\|done_r" tests/probe_lifeline_orphan_clean_via_substrate.rs tests/probe_pdeathsig_kills_orphan_child.rs tests/probe_pdeathsig_diagnostic.rs` — all occurrences are in `//!` history docstrings only; `SYS_pidfd_open` present in each file's executable section |
| B | Each probe PASSES 1/1 in isolation | **PASS** | `cargo test --release --test probe_lifeline_orphan_clean_via_substrate` → 1 passed; `cargo test --release --test probe_pdeathsig_kills_orphan_child` → 1 passed; `cargo test --release --test probe_pdeathsig_diagnostic` → 1 passed |
| C | `probe_row_g_sweep` PASSES (50/50 sub-trials) | **PASS** | `cargo test --release --test probe_row_g_sweep` → `1 passed; 0 failed; finished in 1.13s` |
| D | `cargo build --release --workspace --tests` clean | **PASS** | `Finished release profile [optimized] target(s) in 0.66s` — zero errors |
| E | Workspace failure set: 9 (pre-existing only; no new probe regressions) | **PASS** | `cargo test --release --workspace --no-fail-fast` → 9 failures: lifeline flake (1) + svc-test rot (5) + tmp (2) + startup_error (1); the 4 formerly-failing probes absent from failure list |
| F | Phase 3's primary symptom (stream + lru pressure) STILL resolved | **PASS** | Neither `stream` nor `lru` appears in workspace failure list |

### Edit summary

- `tests/probe_lifeline_orphan_clean_via_substrate.rs`: Removed `done_pipe` creation; removed supervisor `drop(done_r)` / `drop(done_w)` and test `drop(done_w)` / `drop(done_r)`; replaced `poll(done_r, POLLHUP, 1000ms)` with `pidfd_open(grandchild_pid) + poll(pidfd, POLLIN, 1000ms)`. Header docstring updated.
- `tests/probe_pdeathsig_kills_orphan_child.rs`: Same migration. Header docstring note added per BRIEF.
- `tests/probe_pdeathsig_diagnostic.rs`: Same migration. Header docstring note added per BRIEF.
- `tests/probe_row_g_sweep.rs`: No change required.
- `src/` unchanged.

Observable contract across all 3 probes: grandchild dies within 1s of supervisor exit — unchanged.

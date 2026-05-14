# Arc 170 FD-multiplex Phase 1E BRIEF — fork-program FD hygiene + lifeline probe

**Phase:** 1E of DESIGN-FD-MULTIPLEX-SHUTDOWN.md.
**Predecessor:** Phase 1D (`c1cb4dc`) — spawn-process lifeline verified empirically; symmetric defect surfaced in fork-program path.
**Goal:** Fix the latent defect in `child_branch_from_source` where `close_inherited_fds_above_stdio()` closes the lifeline FD (and the substrate's wake-pipe FDs) after Phase 1C wired them in. Add a fork-program lifeline probe symmetric to Phase 1D's spawn-process probe.

## The defect (from Phase 1D's honest delta)

`src/fork.rs::child_branch_from_source` order today (post-Phase-1C):

```
... dup2 stdio + setpgid ...
init_shutdown_signal_with_inputs(&[lifeline_r_raw])  ← opens wake-pipe + spawns worker; worker polls lifeline_r_raw + wake-pipe-read
mem::forget(lifeline_r)
install_substrate_signal_handlers
close_inherited_fds_above_stdio                       ← CLOSES every fd > 2 including lifeline_r_raw, SHUTDOWN_WAKE_WRITE_FD, worker's wake-pipe read-end
```

Closing those FDs causes:
- Worker's `poll(2)` over `lifeline_r_raw` → instant POLLHUP (FD closed) → false-positive shutdown cascade on every fork-program spawn
- Signal handler's `libc::write(SHUTDOWN_WAKE_WRITE_FD, ...)` → fails silently (EBADF)
- Worker's read-end-of-wake-pipe → also POLLHUP → cascade

Latent today because no test exercises a fork-program child reaching its blocking-recv state and observing parent-death cascade. The new probe in this BRIEF surfaces it.

## Edits

### 1. `src/fork.rs::close_inherited_fds_above_stdio` — gains skip-list

Today (line ~379):

```rust
fn close_inherited_fds_above_stdio() {
    let mut to_close: Vec<i32> = Vec::new();
    // ... iterate /proc/self/fd, collect fds > 2 ...
    for fd in to_close {
        unsafe { libc::close(fd); }
    }
}
```

Becomes:

```rust
fn close_inherited_fds_above_stdio(skip: &[i32]) {
    let mut to_close: Vec<i32> = Vec::new();
    // ... iterate /proc/self/fd, collect fds > 2 ...
    for fd in to_close {
        if skip.contains(&fd) {
            continue;
        }
        unsafe { libc::close(fd); }
    }
}
```

### 2. `src/fork.rs::child_branch` (legacy path, line ~634) — pass empty skip-list

Update the call at line ~672:

```rust
close_inherited_fds_above_stdio(&[]);
```

The legacy path has no substrate-owned FDs to protect (no lifeline, no init_shutdown_signal call), so empty skip-list is correct.

### 3. `src/fork.rs::child_branch_from_source` — REORDER

Move `close_inherited_fds_above_stdio` to BEFORE `init_shutdown_signal_with_inputs`. The current order has the substrate-opened wake-pipe FDs at risk; the proposed order opens substrate FDs AFTER the inherited-fd sweep so they're never in the sweep set.

New order (post-edit):

```
... dup2 stdio + install_silent_panic_hook + setpgid ...

// MOVED — close inherited fds BEFORE opening any substrate-owned FDs.
// Skip lifeline_r_raw because it's the parent's lifeline-write-end pair
// we INTEND to keep open across this call. All other inherited fds > 2
// are leaked from parent and should be closed.
close_inherited_fds_above_stdio(&[lifeline_r_raw]);

// NOW open substrate FDs — the wake-pipe and worker are fresh and not
// at risk of the close-sweep.
crate::runtime::init_shutdown_signal_with_inputs(&[lifeline_r_raw]);
std::mem::forget(lifeline_r);

install_substrate_signal_handlers();
```

This eliminates the close-sweep-nukes-substrate-FDs defect by ORDER, not by exception. The substrate's wake-pipe FDs are opened AFTER the sweep, so they can never be in `to_close`. The lifeline_r_raw is the only inherited FD we need to protect, and the skip-list parameter handles that explicitly.

### 4. NEW probe: `tests/probe_lifeline_orphan_clean_via_fork_program.rs`

Symmetric to Phase 1D's `tests/probe_lifeline_orphan_clean_via_substrate.rs` but routes through `:wat::kernel::fork-program` instead of `:wat::kernel::spawn-process`. Same observable contract: supervisor forks (raw libc::fork) → supervisor evals fork-program → grandchild blocks on recv → supervisor `_exit(0)` → grandchild dies within 1s via lifeline EOF.

Copy structure from `tests/probe_lifeline_orphan_clean_via_substrate.rs`; adjust the wat-level call to use `(:wat::kernel::fork-program SOURCE :wat::core::None)` with a SOURCE string containing the blocking-child wat program.

Test name: `probe_lifeline_orphan_clean_via_fork_program`.

Note: `fork-program` returns a different shape than `spawn-process` — it returns `ForkedProgramHandles` via `:wat::kernel::ForkedChild` Struct. The probe extracts grandchild pid via the appropriate ProgramHandle accessor; reference `tests/probe_pdeathsig_kills_orphan_child.rs` if needed (which also uses Forked variant).

## Scorecard (8 rows)

| Row | What | Evidence |
|-----|------|----------|
| A | `close_inherited_fds_above_stdio` signature gains `skip: &[i32]` | `awk '/^fn close_inherited_fds_above_stdio/,/^}/' src/fork.rs` shows the new signature + filter |
| B | Both existing callers updated: legacy `child_branch` passes `&[]`; `child_branch_from_source` passes `&[lifeline_r_raw]` | `grep -n "close_inherited_fds_above_stdio" src/fork.rs` shows the two callers with their args |
| C | `child_branch_from_source` order: close-sweep BEFORE `init_shutdown_signal_with_inputs` | grep shows the new order; close-sweep call precedes init call |
| D | NEW probe `tests/probe_lifeline_orphan_clean_via_fork_program.rs` exists; routes through `:wat::kernel::fork-program` | `ls tests/probe_lifeline_orphan_clean_via_fork_program.rs` + grep for `fork-program` |
| E | NEW probe PASSES 1/1 in isolation | `cargo test --release --test probe_lifeline_orphan_clean_via_fork_program` shows `1 passed; 0 failed` |
| F | Phase 1D's spawn-process probe still PASSES (regression check) | `cargo test --release --test probe_lifeline_orphan_clean_via_substrate` shows `1 passed; 0 failed` |
| G | `probe_pdeathsig_kills_orphan_child` STILL PASSES (historical marker) | `cargo test --release --test probe_pdeathsig_kills_orphan_child` shows `1 passed; 0 failed` |
| H | `cargo build --release --workspace --tests` clean | build output |

## Constraints

- NO Mutex / RwLock / CondVar.
- NO new wall-clock timers; use libc::poll(2).
- DO NOT modify `tests/probe_pdeathsig_kills_orphan_child.rs` (historical artifact).
- DO NOT modify Phase 1D's probe (probe_lifeline_orphan_clean_via_substrate.rs).
- DO NOT add `close_inherited_fds_above_stdio` to `spawn_process_child_branch` — that's a separate hygiene concern, out of scope here.
- Per `feedback_inscription_immutable`: do NOT edit Slice C's INSCRIPTION / SCORE doc / BRIEF.

## STOP-at-first-red

- `cargo build` fails after edits → STOP, report.
- New probe fails after edits → STOP. Use /proc snapshot of any stuck procs to root-cause. Likely either: (a) the close-sweep still includes a substrate FD we missed, or (b) the wat-level fork-program return shape differs from the probe's extraction code.
- Phase 1D's spawn-process probe regresses → STOP. The skip-list change should be backwards-compatible; if not, something subtle is wrong.

## On completion

Write `SCORE-FD-MULTIPLEX-PHASE-1E-FORK-PROGRAM-FD-HYGIENE.md`. 8 rows. Note any honest deltas (e.g., if the close-sweep needs additional skip entries we missed; if the fork-program return shape required adjustments).

Do NOT commit. Orchestrator commits atomically after independent verification.

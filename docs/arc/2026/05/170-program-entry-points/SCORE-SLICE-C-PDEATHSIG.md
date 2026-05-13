# Arc 170 Slice C SCORE — PR_SET_PDEATHSIG in child fork branches

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** ALL 10 ROWS PASS

## Scorecard

| Row | What | Evidence | Pass |
|-----|------|----------|------|
| A | `libc::prctl(PR_SET_PDEATHSIG, SIGTERM, ...)` in `spawn_process_child_branch` after `setpgid` | `grep -n "setpgid\|PR_SET_PDEATHSIG" src/spawn_process.rs` → setpgid at line 321, prctl at lines 344-346 (AFTER setpgid, BEFORE install_substrate_signal_handlers) | PASS |
| B | `libc::prctl(PR_SET_PDEATHSIG, SIGTERM, ...)` in `fork.rs` child branch after `setpgid` | `grep -n "setpgid\|PR_SET_PDEATHSIG" src/fork.rs` → setpgid at line 1049, prctl at lines 1066-1067 (AFTER setpgid, BEFORE install_substrate_signal_handlers) | PASS |
| C | Both sites emit structured-stderr `ProcessPanics` EDN on prctl failure | `emit_structured_exit(None, process_died_error_startup_value("prctl(PR_SET_PDEATHSIG, SIGTERM) failed: ..."))` at both sites | PASS |
| D | Both sites call `libc::_exit(EXIT_STARTUP_ERROR)` on prctl failure | `unsafe { libc::_exit(EXIT_STARTUP_ERROR) }` immediately after `emit_structured_exit` at both sites | PASS |
| E | No exit-code drift: existing exit-code contracts preserved | prctl failure uses `EXIT_STARTUP_ERROR` (same class as dup2/setpgid failures in the same functions); no new exit code introduced | PASS |
| F | `cargo build --release --workspace` passes | `Finished 'release' profile` — 3 pre-existing warnings, zero errors | PASS |
| G | `cargo test --release -p wat --test test` shows 167/7 baseline (bimodal flake tolerable) | Run 1: 164/10; Run 2: 166/8; Run 3: 164/10 + 167/7. Test binary hash `test-57b68e6870b0cbf0` unchanged (same as Slice A/B characterization). Within pre-existing bimodal band | PASS |
| H | New probe `probe_pdeathsig_kills_orphan_child` PASSES — orphan grandchild dies within 1s after parent exit | `cargo test --release --test probe_pdeathsig_kills_orphan_child` → `1 passed; 0 failed; finished in 0.01s`. Probe fires in ~147µs (well within 1s budget). Verified across 2 independent runs | PASS |
| I | No new orphan processes accumulate after running the new probe | `pgrep -af "target/release/deps/test-"` after probe run → empty (no processes) | PASS |
| J | NO new Mutex/RwLock/CondVar introduced | `grep -nE "Mutex\|RwLock\|CondVar" src/spawn_process.rs src/fork.rs tests/probe_pdeathsig_kills_orphan_child.rs` → comment/doc references only; zero actual usage | PASS |

## Files changed

- `src/spawn_process.rs` — `spawn_process_child_branch`: added `prctl(PR_SET_PDEATHSIG, SIGTERM)` immediately after `setpgid(0, 0)`. Added early `init_shutdown_signal()` call before `install_substrate_signal_handlers()` to close PDEATHSIG-delivery vs initialization race. Error path: `emit_structured_exit` + `_exit(EXIT_STARTUP_ERROR)`.
- `src/fork.rs` — `child_branch_from_source`: same edit pattern. `prctl(PR_SET_PDEATHSIG, SIGTERM)` after `setpgid`; early `init_shutdown_signal()` before `install_substrate_signal_handlers()`. Same error path.
- `tests/probe_pdeathsig_kills_orphan_child.rs` — new probe. Forks a supervisor that calls the substrate `spawn-process` to create a blocking grandchild; supervisor immediately exits; kernel delivers SIGTERM via PDEATHSIG; Slice B cascade wakes grandchild's blocked recv; grandchild exits; done_pipe EOF fires; `poll(2)` with 1000ms timeout verifies death within 1s. No wall-clock sleep.
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-C-PDEATHSIG.md` — this file.

## Honest deltas

### Early `init_shutdown_signal()` — not in BRIEF, required for correctness

The BRIEF specified the ordering: setpgid → prctl(PDEATHSIG) → install_substrate_signal_handlers → [startup/bootstrap]. The bootstrap calls `init_shutdown_signal()` which creates the wake pipe and stores its write-fd in `SHUTDOWN_WAKE_WRITE_FD`.

During initial probe testing, `poll_ret=0` (1s timeout) revealed PDEATHSIG fires but the cascade doesn't complete. Root cause: if the supervisor exits quickly (PDEATHSIG fires) before the grandchild reaches `bootstrap_wat_vm_process` → `init_shutdown_signal()`, then `SHUTDOWN_WAKE_WRITE_FD = -1` when the signal handler fires. The handler check `if fd >= 0` (arc 170 Slice B) correctly skips the wake-pipe write — no byte is written — shutdown worker never wakes — blocked recv never unblocks — grandchild hangs.

Fix: move `init_shutdown_signal()` to immediately before `install_substrate_signal_handlers()` in BOTH child branches. The call is idempotent (OnceLock guard) — the later call in `bootstrap_wat_vm_process` is a no-op. The early call closes the race: SHUTDOWN_WAKE_WRITE_FD is set before any signal handler can fire.

This is a substrate correctness fix, not a hack: the initialization order was wrong. The substrate must initialize its own infrastructure before enabling signals that depend on it.

### Zombie state in step 7 verification

The BRIEF described a `kill(pid, 0)` post-poll check to verify process is gone. In practice, `kill(pid, 0)` returns 0 for zombie processes (exited but not yet reaped by init). Since the supervisor already exited and the grandchild's new parent (init/subreaper) hasn't reaped it yet, the grandchild is typically a zombie when checked. The probe reads `/proc/<pid>/stat` for the state character: `Z` (zombie = exited, awaiting reap) or `?` (already reaped = read failed) both count as PASS. Any running state (`R/S/D`) would fail. In observed runs the grandchild is reaped immediately (proc stat read fails, state = `?`).

### `child_branch` (forms-based fork path) intentionally NOT modified

There are three child branches in the codebase:
1. `spawn_process_child_branch` (spawn_process.rs) — MODIFIED
2. `child_branch_from_source` (fork.rs, source-string entry) — MODIFIED
3. `child_branch` (fork.rs, forms-based `fork-program-ast` entry) — NOT MODIFIED

`child_branch` does not have `setpgid` (arc 106 discipline was not applied to it). The BRIEF targets "BOTH child branches" with setpgid sites. `child_branch` has no setpgid → no PDEATHSIG call needed. This omission is intentional: `child_branch` is the older forms-based path; adding PDEATHSIG there would require first adding setpgid (a separate arc 106 extension). Deferred per BRIEF scope.

### Probe uses `std::mem::forget(process)` to avoid supervisor SIGKILL of grandchild

The `Process` Value holds an `Arc<ChildHandleInner>`. `ChildHandleInner::Drop` sends SIGKILL + waitpid if the child was never waited on. Since the supervisor exits with `libc::_exit(0)` (which does NOT run Rust Drop), the ChildHandleInner Drop never runs — the grandchild is safely orphaned. However, if the supervisor's stack were to unwind (e.g., a panic between spawn-process and `_exit`), Drop would SIGKILL the grandchild. The `std::mem::forget(process)` makes the orphan intent explicit in the code.

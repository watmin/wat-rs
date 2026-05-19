# Arc 213 stone β — SCORE: migrate run_in_fork to spawn_lifelined

## Summary

Internal refactor of `src/fork.rs::run_in_fork` (lines 149-171 post-migration).
Public signature unchanged: `F: FnOnce() + std::panic::UnwindSafe`.
All 11 caller sites compile and pass unchanged.
Zero modifications to any file outside `src/fork.rs`.

## File changes

- `src/fork.rs` — `run_in_fork` body only (lines 149-185 pre-migration → 149-171 post-migration; body shrank by ~14 lines)
  - Removed: `libc::fork()`, `libc::waitpid()`, manual `WIFEXITED`/`WEXITSTATUS` decoding, child branch + `catch_unwind` + `_exit(0/1)`
  - Added: `spawn_lifelined(|_lifeline_r| { body(); }).expect(...)` + `pidfd.wait_status().expect(...)` + `matches!(status, ExitStatus::Exited(0))` assert

## Verification

### Build

```
cargo build --release 2>&1 | tail -5
```
Result: `Finished release profile [optimized]` — clean; 5 pre-existing warnings, no new warnings.

### α smoke probe (sanity regression)

```
cargo test --release --test probe_pidfd_primitive
```
Result: **2/2 PASS** (`pidfd_observes_normal_exit`, `pidfd_observes_signal_exit`)

### Affected test binaries

| Test binary | Sites | Result |
|---|---|---|
| `wat_harness_deps` | 3 run_in_fork | **3/3 PASS** |
| `probe_shutdown_cascade_crossbeam` | 1 run_in_fork | **1/1 PASS** |
| `probe_shutdown_cascade_pipefd` | 1 run_in_fork | **1/1 PASS** |
| `wat-cli wat_cli` | 1 run_in_fork | **15/15 PASS** |
| `wat --lib` (5 signal tests) | 5 run_in_fork | **5/5 PASS** |

Total: **25/25 tests pass** across all 5 binaries.

## Honest-delta notes

### Panic message string change

Old: `"forked child exited with failure (status={:#x})"` (raw POSIX wstatus integer)
New: `"forked child exited with failure: {:?}"` (Debug-format `ExitStatus` enum, e.g. `Exited(1)` or `Signaled(9)`)

No caller in the 5 test binaries asserts on the exact panic string
(`#[should_panic(expected = "...")]` or substring match). Zero test
failures from this delta.

### Process group change

`spawn_lifelined` calls `setpgid(0, 0)` in child — child becomes its own
process-group leader. Old `libc::fork()` inherited the parent's process
group. None of the 11 caller sites send `kill(0, sig)` (whole-pgroup
signal); zero test failures from this delta.

### FD count change

Each `run_in_fork` call now opens 2 additional FDs (lifeline pipe pair) in
the parent during the child's lifetime; both are cleaned up on return.
None of the callers stress-test FD counts. Zero test failures from this
delta.

## Scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | `run_in_fork` body migrated to spawn_lifelined + Pidfd::wait_status | YES |
| 2 | Public signature unchanged (`F: FnOnce() + UnwindSafe`) | YES |
| 3 | `libc::fork()` call removed from run_in_fork | YES |
| 4 | `libc::waitpid()` call removed from run_in_fork | YES |
| 5 | Manual WIFEXITED/WEXITSTATUS decoding removed from run_in_fork | YES |
| 6 | cargo build --release clean (5 pre-existing warnings unchanged) | YES |
| 7 | Smoke probe `probe_pidfd_primitive` still passes 2/2 | YES |
| 8 | `wat_harness_deps` 3 run_in_fork sites still pass | YES |
| 9 | `probe_shutdown_cascade_crossbeam` still passes | YES |
| 10 | `probe_shutdown_cascade_pipefd` still passes | YES |
| 11 | `wat-cli wat_cli` test using run_in_fork still passes | YES |
| 12 | 5 runtime.rs lib tests using run_in_fork still pass | YES |
| 13 | Zero modifications to ANY caller site | YES |
| 14 | Zero changes outside src/fork.rs | YES |
| 15 | SCORE inscribes panic-message-string deltas + caller verification | YES |

All 15 criteria: YES.

## Mode classification

**Mode A** — migration shipped; cargo build clean; all 5 test binaries'
affected tests still pass (25/25); α probe still green (2/2); SCORE written.

Substrate fork-path inconsistency closed: every fork path in wat-rs now
routes through `spawn_lifelined`. The "every spawn has a lifeline"
guarantee is no longer a lie.

# Arc 170 FD-multiplex Phase 1B SCORE — spawn-process lifeline pipe + PDEATHSIG retirement

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** COMPLETE — 10/10 PASS

## Scorecard

| Row | What | Evidence | Pass |
|-----|------|----------|------|
| A | `ChildHandleInner` gains `lifeline_w: Option<OwnedFd>` field | `grep -n "lifeline_w" src/fork.rs` → line 214 (field), 218 (constructor param), 223 (struct literal) | PASS |
| B | `ChildHandleInner::new` signature updated; all callers pass `Some(lifeline_w)` (spawn_process) or `None` (others) | `grep -nE "ChildHandleInner::new\b" src/ crates/` → `spawn_process.rs:202` `Some(lifeline_w)`, `fork.rs:591` `None`, `fork.rs:889` `None` — all 3 sites updated | PASS |
| C | `eval_kernel_spawn_process` creates a fourth `make_pipe` for the lifeline before fork | `grep -n "lifeline" src/spawn_process.rs` → lines 157–163 show pipe creation + `lifeline_r_raw` capture, both above the fork call | PASS |
| D | Parent branch drops `lifeline_r`, builds handle with `Some(lifeline_w)` | lines 200 (`drop(lifeline_r)`) and 202 (`ChildHandleInner::new(pid, Some(lifeline_w))`) in parent branch post-fork | PASS |
| E | `spawn_process_child_branch` signature includes `lifeline_r_raw: i32` + `lifeline_r: OwnedFd` parameters | function signature at lines 287–297: `lifeline_r_raw: i32` (line 292) and `lifeline_r: OwnedFd` (line 296) | PASS |
| F | `spawn_process_child_branch` calls `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` (replacing the old bare `init_shutdown_signal()` call) | `grep -n "init_shutdown_signal_with_inputs\|init_shutdown_signal(" src/spawn_process.rs` → only line 359 (`init_shutdown_signal_with_inputs`); zero matches for bare `init_shutdown_signal(` | PASS |
| G | `mem::forget(lifeline_r)` after the registration call | line 364: `std::mem::forget(lifeline_r)` immediately after the `init_shutdown_signal_with_inputs` call at line 359 | PASS |
| H | The `prctl(PR_SET_PDEATHSIG, ...)` block + error path are GONE from `spawn_process.rs` | `grep -n "PR_SET_PDEATHSIG\|prctl" src/spawn_process.rs` → zero matches | PASS |
| I | `cargo build --release --workspace --tests` passes clean | `Finished 'release' profile [optimized]` — same 3 pre-existing warnings, zero errors | PASS |
| J | `probe_shutdown_cascade_crossbeam` PASSES AND `probe_lifeline_pipe_proof` PASSES 100/100 in isolation | `probe_shutdown_cascade_crossbeam`: `1 passed; 0 failed` in 0.00s. `probe_lifeline_pipe_proof`: `1 passed; 0 failed` in 0.03s (100 trials) | PASS |

## Files changed

- `src/fork.rs` — `ChildHandleInner` struct: added `lifeline_w: Option<std::os::fd::OwnedFd>` field with full doc comment; `new` constructor updated to accept `lifeline_w: Option<std::os::fd::OwnedFd>` parameter; two existing callers at lines 591 and 889 updated to pass `None` (Phase 1C fills them)
- `src/spawn_process.rs` — 6 edit sites:
  1. Fourth `make_pipe` call for lifeline (before fork)
  2. Child branch call updated with `lifeline_r_raw` + `lifeline_r` args
  3. Parent branch: `drop(lifeline_r)` + `ChildHandleInner::new(pid, Some(lifeline_w))`
  4. `spawn_process_child_branch` signature gains `lifeline_r_raw: i32` + `lifeline_r: OwnedFd`
  5. prctl block + Slice C early-init comment block replaced by `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` + `std::mem::forget(lifeline_r)` + updated comment
  6. Replaced the Slice C "early init" rationale comment with Phase 1B framing

## Honest deltas

### ChildHandleInner callers: 3 sites (BRIEF predicted 2)

The BRIEF's "expected sites" section listed `spawn_process.rs:188` and "any others (likely in fork.rs)" generically. Grep surfaced exactly 3 sites: `spawn_process.rs:202` + `fork.rs:591` + `fork.rs:889`. Both fork.rs callers are in `eval_kernel_fork_program` (the legacy child_branch path) and `eval_kernel_fork_program_ast` (the modern child_branch_from_source path). Both get `None` per the BRIEF's Phase 1C deferral. No STOP condition triggered; the count was expected by the BRIEF's "any others" language.

### `init_shutdown_signal_with_inputs` idempotency ordering verified

The EXPECTATIONS flagged a concern: if any code path calls bare `init_shutdown_signal()` BEFORE the child branch reaches `init_shutdown_signal_with_inputs(&[lifeline_r_raw])`, the lifeline FD would not be registered. Verification: in `spawn_process_child_branch`, the order is: dup2 → install_silent_panic_hook → setpgid → **`init_shutdown_signal_with_inputs`** → `mem::forget` → `install_substrate_signal_handlers` → `startup_from_forms` → `bootstrap_wat_vm_process`. The bootstrap's `init_shutdown_signal()` call is downstream; OnceLock guard makes it a no-op. No early path races the registration.

### `mem::forget` placement verified correct

The `lifeline_r` OwnedFd is alive from creation (parent) through the fork, into the child branch, past `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` (which captures the raw int into the worker's pollfd set), then `std::mem::forget(lifeline_r)` prevents the Drop from running. The worker thread now owns the FD lifetime via the OS. This mirrors the dup2 → drop(original) pattern for stdio; here it's register-raw-fd → forget-owned-fd.

### Drop order on ChildHandleInner is correct

Per the BRIEF: SIGKILL → waitpid → field drops → lifeline_w closes. Rust's Drop implementation runs explicit Drop body first (SIGKILL + waitpid in the `unsafe` block), then fields drop in declaration order. `lifeline_w` is the last field declared; it closes after `pid`, `reaped`, `cached_exit` (which are all Copy/no-drop types). Child is dead before `lifeline_w` closes — closing is harmless cleanup.

## Build + test results

```
cargo build --release --workspace --tests
Finished `release` profile [optimized] target(s) in 57.81s
(3 pre-existing warnings, zero errors)

cargo test --release --test probe_shutdown_cascade_crossbeam
test probe_shutdown_cascade_wakes_crossbeam_recv ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --release --test probe_lifeline_pipe_proof
test lifeline_pipe_zero_orphans_across_100_trials ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

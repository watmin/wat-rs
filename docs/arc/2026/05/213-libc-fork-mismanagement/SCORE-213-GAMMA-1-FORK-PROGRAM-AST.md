# Arc 213 stone γ-1 — SCORE: migrate eval_kernel_fork_program_ast to spawn_lifelined

## Summary

Site migrated. `eval_kernel_fork_program_ast` (src/fork.rs, fork call at line 600)
replaced with `spawn_lifelined`-based pattern. `child_branch` extended with
`lifeline_r_raw: i32` + `lifeline_r: OwnedFd` parameters; calls
`child_post_fork_init(lifeline_r_raw)` + `std::mem::forget(lifeline_r)` (mirroring
sister fn `child_branch_from_source` at lines 1111-1116). Lifeline gap closed:
`ChildHandleInner::new(pid, None)` → `ChildHandleInner::new(pid, Some(lifeline_w))`.

## File changes

Single file modified: `src/fork.rs` only.

### 1. Import addition (line 33)

`IntoRawFd` added to the `std::os::fd` import (required for `OwnedFd::into_raw_fd()`
calls in the fork body).

### 2. `LifelineWriter::into_owned_fd` method added (after `close` method, ~line 1504)

```rust
pub fn into_owned_fd(self) -> OwnedFd {
    self.fd
}
```

Additive — one-line consumption method. ChildHandleInner unchanged.

### 3. `eval_kernel_fork_program_ast` body (lines ~584-693, approximate post-migration)

Replaced the bare `libc::fork()` block with `spawn_lifelined`-based pattern:
- Stdio pipe OwnedFds converted to raw `i32` integers via `into_raw_fd()` before
  the closure (spawn_lifelined uses clone3 — both parent and child inherit all kernel
  fds; raw ints passed into closure avoid Rust's single-ownership constraint).
- `spawn_lifelined(move |lifeline_r_raw: i32| { ... })` closure reconstructs
  OwnedFds from raw ints in the child and calls `child_branch(...)` with the new
  lifeline parameters.
- Parent branch closes child-side fds manually (via `libc::close`) since
  `into_raw_fd()` disabled OwnedFd::Drop.
- Parent reconstructs parent-side OwnedFds from raw ints.
- `lifeline_writer.into_owned_fd()` extracts the OwnedFd for ChildHandleInner.
- `pidfd.pid()` retrieves the pid; `pidfd` dropped (δ migrates ChildHandleInner
  to hold Pidfd).
- `ChildHandleInner::new(pid, Some(lifeline_w))` — gap closed.

### 4. `child_branch` signature extension (line ~728)

Two new parameters added after `stderr_w_raw`:
- `lifeline_r_raw: i32` — raw fd for child_post_fork_init
- `lifeline_r: OwnedFd` — OwnedFd wrapper for mem::forget after registration

Existing `install_silent_panic_hook()` + `close_inherited_fds_above_stdio(&[])`
calls replaced by `child_post_fork_init(lifeline_r_raw)` + `std::mem::forget(lifeline_r)`.

## Verification: pre/post pass counts

| Test binary | Pre-γ-1 | Post-γ-1 |
|---|---|---|
| `arc112_slice2b_process_send_recv` | 1/1 PASS | 1/1 PASS |
| `wat_arc170_program_contracts` | 24/24 PASS | 24/24 PASS |
| `probe_run_hermetic_ast_stdout_capture` | 1/1 PASS | 1/1 PASS |
| `probe_run_hermetic_no_deadlock` | 2/2 PASS | 2/2 PASS |
| `wat-cli wat_cli` | 15/15 PASS | 15/15 PASS |
| `probe_pidfd_primitive` (α regression) | 2/2 PASS | 2/2 PASS |

**Total: 45/45 → 45/45. Zero regressions.**

## ChildHandleInner type-design choice

**Option (a) selected: `LifelineWriter::into_owned_fd(self) -> OwnedFd`.**

### Blast radius analysis

`ChildHandleInner::lifeline_w` field has type `Option<OwnedFd>`. Sites that pass
`Some(OwnedFd)` to `ChildHandleInner::new`:
- `src/fork.rs:946` — `fork_program_from_source` (sister fn, γ-2 territory)
- `src/spawn_process.rs:204` — passes `Some(lifeline_w)` directly as `OwnedFd`

2 blast sites (≤5 threshold for option (c)). However, option (c) would require
touching `spawn_process.rs` to change `lifeline_w: OwnedFd` to `lifeline_w: LifelineWriter`
— this violates the "ZERO other code edits" scope constraint. Option (a) is therefore
mandatory regardless of blast count: adding `into_owned_fd` to LifelineWriter
(inside `src/fork.rs`) is the only scope-compliant bridge.

### Correctness of option (a)

`ChildHandleInner::lifeline_w: Option<OwnedFd>` is the canonical type for the
write-end store. `LifelineWriter` wraps `OwnedFd` and implements `Drop` via the
`OwnedFd`'s own `Drop` — semantically identical once the `OwnedFd` is extracted.
`into_owned_fd` consumes the `LifelineWriter` (preventing double-close) and
transfers the OwnedFd to `ChildHandleInner`. Behavior: equivalent to storing a
`LifelineWriter` directly.

## Subtleties

### UnwindSafe and AssertUnwindSafe

`spawn_lifelined` requires `F: FnOnce(i32) + std::panic::UnwindSafe`.
`std::panic::AssertUnwindSafe<F>` implements `FnOnce()` (zero args) but NOT
`FnOnce(i32)` — wrapping with `AssertUnwindSafe` fails at the bound level.

Resolution: use the closure directly without `AssertUnwindSafe`. The closure
captures only `i32` raw fds (trivially `UnwindSafe`), `Vec<WatAST>`, and
`Option<Config>`. The latter two are pure data (no `&mut T` or interior
mutability via `Cell/RefCell/Mutex` references) and satisfy `UnwindSafe` by
auto-trait rules. Compiler accepted the bare `move |lifeline_r_raw: i32| { ... }`
closure without any wrapper.

### OwnedFd ownership across clone3

`into_raw_fd()` was required (not `as_raw_fd()`) to strip the OwnedFd RAII
wrappers before the closure. Rust's ownership rules prevent moving OwnedFds into
the closure AND using the same values in the parent afterward. Converting to raw
`i32` first delegates ownership to the engineer: child reconstructs OwnedFds from
raw ints; parent reconstructs parent-side OwnedFds and manually closes child-side
fds via `libc::close`. `IntoRawFd` trait import was added to the existing
`std::os::fd` import line.

### dup2 ordering relative to child_post_fork_init

dup2 runs BEFORE `child_post_fork_init`. This is correct:
- `child_post_fork_init` step 1 installs the silent panic hook — fd 2 must
  already be the subprocess stderr pipe (after dup2) for the hook to write to
  the correct destination.
- `child_post_fork_init` step 3 calls `close_inherited_fds_above_stdio(&[lifeline_r_raw])`.
  The OLD code called `close_inherited_fds_above_stdio(&[])` at this point.
  If the old call had remained and executed before `child_post_fork_init`,
  `lifeline_r_raw` (above fd 2) would have been closed — false-positive POLLHUP.
  The replacement of the old calls with `child_post_fork_init(lifeline_r_raw)` is
  not just additive; it corrects the skip-list to protect lifeline_r.

### setpgid double-call

`spawn_lifelined` calls `setpgid(0, 0)` in the child (step 1 of its child branch).
`child_post_fork_init` (step 2) also calls `setpgid(0, 0)`. The second call is
harmless — idempotent for a process already in its own pgrp. No test regression.

### Double catch_unwind

`spawn_lifelined` wraps `child_body` in `catch_unwind`. `child_branch` internally
`_exit`s via its own match arms — the closure never returns `Ok(())` to
`spawn_lifelined`'s catch. If `child_branch` panics (e.g., inside `startup_from_forms`
before the catch_unwind at line ~825), `spawn_lifelined`'s outer catch_unwind
catches it and `_exit(1)`. The behavior is equivalent to the old bare-fork path
where a pre-catch panic would propagate up the stack and eventually abort. No
test regression observed.

### pidfd dropped

`pidfd` (Pidfd struct wrapping the kernel pidfd + pid) is retrieved for its `pid()`
method and immediately dropped. The kernel pidfd closes without affecting the child
process (pidfd is a reference handle, not a lifecycle control). Stone δ migrates
`ChildHandleInner` to store a `Pidfd` instead of raw `pid_t`.

## Mode classification

**Mode A.** Site migrated; `child_branch` extended with `lifeline_r_raw` + `lifeline_r`
+ `child_post_fork_init` + `mem::forget`; lifeline gap closed
(`ChildHandleInner::new(pid, Some(lifeline_w))`); `cargo build --release` clean
(5 pre-existing warnings, zero errors); all 45 baselines preserved (45/45 → 45/45);
SCORE written.

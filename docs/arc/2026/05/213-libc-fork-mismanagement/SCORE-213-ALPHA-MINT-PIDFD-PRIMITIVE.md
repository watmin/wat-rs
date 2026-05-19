# Arc 213 stone α — SCORE: mint canonical Pidfd + spawn_lifelined

## Summary

Types + helper minted. Smoke probe verifies the primitive end-to-end. `cargo
build --release` is clean (5 pre-existing warnings, none from this stone after
one fix). Both smoke tests pass on first attempt.

## File changes

- **`src/fork.rs`** — additive (~280 LOC appended after the last existing
  function `child_branch_from_source`). Added:
  - `ExitStatus` enum (Exited / Signaled / Stopped)
  - `Pidfd` struct with `OwnedFd fd` + `pid_t pid`, `Send + Sync`
  - `Pidfd::poll_exit`, `wait_status`, `try_wait`, `send_signal`, `pid`
  - `Pidfd::waitid_inner` (private, shared by `wait_status` and future internal callers)
  - `extract_exit_status_from_siginfo` (module-private helper)
  - `LifelineWriter` struct with `OwnedFd fd`, `close()` method
  - `spawn_lifelined<F>` function
  - Local UAPI constants: `CLONE_PIDFD_FLAG`, `CLONE_CLEAR_SIGHAND_FLAG`,
    `P_PIDFD_CONST`, `SYS_PIDFD_SEND_SIGNAL`, `CloneArgs` struct
  - `use std::time::Duration` (new import for `poll_exit` signature)

- **`src/lib.rs`** — no change. The fork module was already `pub mod fork`,
  so `wat::fork::Pidfd` etc. are immediately accessible.

- **`tests/probe_pidfd_primitive.rs`** — NEW. 2 tests:
  - `pidfd_observes_normal_exit`
  - `pidfd_observes_signal_exit`

## Verification

```
cargo build --release 2>&1 | tail -3
  warning: `wat` (lib) generated 5 warnings
      Finished `release` profile [optimized] target(s) in 0.06s

cargo test --release --test probe_pidfd_primitive 2>&1 | tail -6
  running 2 tests
  test pidfd_observes_signal_exit ... ok
  test pidfd_observes_normal_exit ... ok
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Honest-delta notes (plumbing subtleties)

### 1. `libc::clone_args` — not relied on; defined locally

Rather than depend on `libc::clone_args` being present in libc 0.2.185, a
local `CloneArgs` repr(C) struct was defined using the exact UAPI field layout
(`__u64` for all 11 fields per `<linux/sched.h>`). This is safe and stable:
the kernel ABI is immutable; the struct was added in Linux 5.2 and the layout
is frozen.

### 2. `CLONE_PIDFD` / `CLONE_CLEAR_SIGHAND` — defined locally

Both constants are defined as raw `u64` values (`0x00001000` and
`0x100000000`). These are stable UAPI values from `<linux/sched.h>`.

### 3. `pidfd_send_signal` — `libc::syscall(SYS_PIDFD_SEND_SIGNAL, ...)` path

As predicted in EXPECTATIONS, `pidfd_send_signal` is not exposed as a direct
libc fn. The substrate uses `libc::syscall(424, ...)` on x86_64. The syscall
number is defined locally with a `compile_error!` guard for non-x86_64 arches
(Linux-only substrate per `feedback_no_windows`).

### 4. `waitid(P_PIDFD, ...)` — `P_PIDFD_CONST = 3` defined locally

`libc::P_PIDFD` may not be in libc 0.2.185 on all configurations. Defined
locally as `const P_PIDFD_CONST: libc::idtype_t = 3`. The value is stable
POSIX/Linux UAPI.

### 5. `OwnedFd` wrapping the kernel-returned raw pidfd

The kernel populates `pidfd_raw` (the int pointed to by `args.pidfd`) during
`clone3`. In the parent branch, `OwnedFd::from_raw_fd(pidfd_raw)` takes
ownership. The child branch runs `_exit` immediately without touching `pidfd`
(the child does not hold this fd — it was only populated in the parent's
address space).

### 6. `info.si_code` — safe field access

Initial draft used `unsafe { info.si_code }` but rustc flagged it as an
unnecessary unsafe block. Fixed to `info.si_code` (safe field). `si_status()`
accessor remains in unsafe because it is an accessor method that may dereference
a union field internally.

### 7. Struct size to `clone3`

`std::mem::size_of::<CloneArgs>()` = 88 bytes (11 u64 fields × 8 bytes). The
kernel validates this against its own UAPI struct size. If this stone were
ported to a kernel that added fields to `clone_args`, the size would still be
correct for the fields we initialize — the kernel validates that any
unrecognized tail bytes are zero, which they are (Rust's struct literal
initializes all fields explicitly to 0 where unused).

### 8. `cgroup: 0` — use current cgroup

BRIEF sketch used `u64::MAX` as a sentinel; the correct value for "use the
caller's current cgroup" is `0`. Set to `0`.

### 9. `exit_signal: libc::SIGCHLD as u64`

Required so the parent's `waitid` / `SIGCHLD` handler fires when the child
exits. Without this, the kernel would not deliver SIGCHLD and `waitid` would
still work (it observes the pidfd directly) but parent signal handlers would
not fire. Consistent with conventional fork behavior.

## Scorecard (against EXPECTATIONS)

| # | Criterion | Result |
|---|---|---|
| 1 | `Pidfd` type minted with Drop (via OwnedFd), Send, Sync, NO from_pid constructor | YES |
| 2 | `LifelineWriter` type minted with Drop (via OwnedFd) | YES |
| 3 | `spawn_lifelined` uses clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND | YES |
| 4 | Lifeline pipe created pre-fork; inherited atomically via clone3 | YES |
| 5 | setpgid(0, 0) in child post-fork | YES |
| 6 | `poll_exit` / `wait_status` / `try_wait` / `send_signal` all use kernel-direct syscalls (no /proc) | YES |
| 7 | Smoke probe: `pidfd_observes_normal_exit` passes (Exited(42)) | YES |
| 8 | Smoke probe: `pidfd_observes_signal_exit` passes (Signaled(SIGTERM)) | YES |
| 9 | cargo build --release clean | YES |
| 10 | Zero existing-code modifications (purely additive) | YES |
| 11 | SCORE inscribes libc/syscall plumbing subtleties | YES |

## Mode classification

**Mode A** — types + helper minted; 2 smoke tests pass; cargo build clean;
SCORE written. Primitive is ready for stone β (migrate `run_in_fork`),
stone γ (migrate existing fork sites), stone δ (migrate waitpid/kill callers),
stone ε (migrate /proc reads), stone ζ (enforce module privacy on libc::fork).

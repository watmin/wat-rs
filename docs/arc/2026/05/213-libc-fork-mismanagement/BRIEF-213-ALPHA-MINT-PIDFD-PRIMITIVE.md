# Arc 213 stone α — Mint canonical `Pidfd` type + `spawn_lifelined` helper

**Your ONE concern this spawn:** mint the canonical Linux 5.3+ process primitive that subsequent stones migrate consumers to. Add types + helper to `src/fork.rs`. Write a smoke probe with 2 test cases. Nothing else.

This is FOUNDATION minting (parallel to `WatAST::children()` at arc 212 β). No consumer migration. No existing code modification. The primitive needs to exist before β/γ/δ/ε can route through it.

---

## What to mint

### 1. `Pidfd` type (in `src/fork.rs`)

```rust
/// File descriptor representing a child process, returned atomically
/// at fork time via clone3 + CLONE_PIDFD. PID-reuse race eliminated —
/// the fd is bound to THIS specific child, not its (potentially reused)
/// PID.
///
/// Arc 213 — canonical process-handle primitive. Use this for every
/// child-process operation (exit observation, signaling, status query).
/// Legacy POSIX (waitpid by PID, kill by PID) is structurally weaker;
/// migration to Pidfd methods eliminates a class of race conditions.
pub struct Pidfd {
    fd: OwnedFd,
    pid: libc::pid_t,  // retained for diagnostic + cascade interop
}

impl Pidfd {
    /// poll(2) on the pidfd with POLLIN. Returns Ok(true) if process
    /// has exited, Ok(false) if timeout elapsed, Err on poll failure.
    /// Timeout None = block forever.
    pub fn poll_exit(&self, timeout: Option<Duration>) -> io::Result<bool> { ... }

    /// waitid(P_PIDFD, fd, WEXITED) — blocking wait that reaps the
    /// zombie atomically. Returns ExitStatus.
    pub fn wait_status(&self) -> io::Result<ExitStatus> { ... }

    /// waitid(P_PIDFD, fd, WEXITED | WNOHANG) — non-blocking poll.
    /// Returns Ok(Some(status)) if exited + reaped, Ok(None) if still
    /// running, Err on syscall failure.
    pub fn try_wait(&self) -> io::Result<Option<ExitStatus>> { ... }

    /// pidfd_send_signal(fd, sig, ...) — send signal to THIS specific
    /// process. PID-reuse-safe.
    pub fn send_signal(&self, sig: i32) -> io::Result<()> { ... }

    /// Retained for cascade interop (`killpg(pid, sig)`) + diagnostic.
    /// Do NOT use for kill(pid) — use send_signal instead.
    pub fn pid(&self) -> libc::pid_t { self.pid }
}

// NO `from_pid` constructor. The ONLY way to construct a Pidfd is via
// spawn_lifelined's return value. This is the typestate-equivalent for
// "non-stale handle" — the substrate refuses any path that constructs
// a Pidfd from a PID alone (which has a PID-reuse race window).
```

`Pidfd: Send + Sync` (fd is safely cross-thread). `Drop` closes the fd via `OwnedFd`.

### 2. `LifelineWriter` type (in `src/fork.rs`)

```rust
/// Write-end of the lifeline pipe held by the parent process. Never
/// written to — its sole purpose is to be CLOSED (explicitly or via
/// Drop) so the child's lifeline_r reads return EOF and the child can
/// shut down. Inherited atomically with clone3.
pub struct LifelineWriter {
    fd: OwnedFd,
}

impl LifelineWriter {
    /// Explicit close — equivalent to dropping. Useful when the parent
    /// wants to signal child shutdown WITHOUT waiting for the parent's
    /// own death.
    pub fn close(self) { drop(self.fd) }
}
```

`Drop` closes the fd via `OwnedFd`.

### 3. `spawn_lifelined` helper (in `src/fork.rs`)

```rust
/// Canonical fork-and-observe primitive. Uses Linux 5.3+ syscalls
/// (clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND) for atomic process
/// creation with race-free pidfd binding. Installs lifeline pipe
/// (parent holds write_end via returned LifelineWriter; child inherits
/// read_end via fork inheritance and observes parent death via
/// pipe-EOF).
///
/// Child setup:
/// 1. setpgid(0, 0) — child becomes its own process-group leader
/// 2. Drops the lifeline_w copy it inherited (parent-only handle)
/// 3. Runs `child_body` with `lifeline_r_raw: i32` as the inherited fd
/// 4. _exit(0) on Ok return, _exit(1) on panic
///
/// Parent receives:
/// - `Pidfd` — canonical process handle (atomic with fork)
/// - `LifelineWriter` — must be held until parent wants child cleanup
///
/// The `child_body` closure receives the lifeline_r fd; the child is
/// responsible for incorporating it into its event loop (poll/select)
/// for parent-death detection.
pub fn spawn_lifelined<F>(child_body: F) -> io::Result<(Pidfd, LifelineWriter)>
where
    F: FnOnce(i32) + std::panic::UnwindSafe,
{
    // 1. Create lifeline pipe (pre-fork)
    let (lifeline_r, lifeline_w) = make_pipe(":wat::fork::spawn_lifelined")?;
    let lifeline_r_raw = lifeline_r.as_raw_fd();

    // 2. Build clone_args
    let mut pidfd: libc::c_int = -1;
    let mut clone_args = libc::clone_args {
        flags: (libc::CLONE_PIDFD | libc::CLONE_CLEAR_SIGHAND) as u64,
        pidfd: &mut pidfd as *mut _ as u64,
        // ... other fields zero ...
    };

    // 3. clone3 — returns child pid + populates pidfd
    let pid = unsafe {
        libc::syscall(
            libc::SYS_clone3,
            &mut clone_args as *mut _,
            std::mem::size_of::<libc::clone_args>(),
        )
    } as libc::pid_t;

    if pid < 0 {
        return Err(io::Error::last_os_error());
    }

    if pid == 0 {
        // ── CHILD ──────────────────────────────────────────────────
        // setpgid(0, 0) — child becomes its own pgrp leader
        unsafe { libc::setpgid(0, 0); }
        // Drop the inherited lifeline_w (parent-only); child only
        // holds lifeline_r.
        drop(lifeline_w);
        // Run body in catch_unwind to surface panics as exit-1.
        let outcome = std::panic::catch_unwind(|| child_body(lifeline_r_raw));
        match outcome {
            Ok(()) => unsafe { libc::_exit(0) },
            Err(_) => unsafe { libc::_exit(1) },
        }
    }

    // ── PARENT ────────────────────────────────────────────────────
    // Drop parent's copy of lifeline_r (child-only).
    drop(lifeline_r);

    // Wrap the pidfd returned by kernel into OwnedFd.
    let pidfd_owned = unsafe { OwnedFd::from_raw_fd(pidfd) };

    Ok((
        Pidfd { fd: pidfd_owned, pid },
        LifelineWriter { fd: lifeline_w.into() },  // OwnedFd from the existing OwnedFd
    ))
}
```

### 4. `ExitStatus` enum (or use existing — check src/fork.rs first)

If the existing `src/fork.rs` already has an exit status type, reuse it. Otherwise mint:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Exited(i32),
    Signaled(i32),
    Stopped(i32),  // probably not load-bearing for arc 213
}
```

Use whatever shape fits with existing fork.rs conventions.

---

## What NOT to do

- **DO NOT** migrate `run_in_fork` to spawn_lifelined — that's stone β
- **DO NOT** migrate the 3 existing `libc::fork()` sites — that's stone γ
- **DO NOT** migrate `waitpid`/`kill` callers — that's stone δ
- **DO NOT** migrate probe /proc reads — that's stone ε
- **DO NOT** make `libc::fork` module-private yet — that's stone ζ
- **DO NOT** modify any existing fork.rs function (just add the new types + helper)

---

## The smoke probe (the wat-test proof gate)

Create new file: `tests/probe_pidfd_primitive.rs`

Two test cases (both use `spawn_lifelined`; demonstrate the primitive):

**Test 1 — normal exit:**
```rust
#[test]
fn pidfd_observes_normal_exit() {
    let (pidfd, _lifeline) = wat::fork::spawn_lifelined(|_lifeline_r| {
        // Child exits with code 42 immediately.
        unsafe { libc::_exit(42) };
    }).expect("spawn_lifelined succeeds");

    let status = pidfd.wait_status().expect("wait_status returns exit status");
    assert_eq!(status, wat::fork::ExitStatus::Exited(42));
}
```

**Test 2 — signal exit:**
```rust
#[test]
fn pidfd_observes_signal_exit() {
    let (pidfd, lifeline) = wat::fork::spawn_lifelined(|_lifeline_r| {
        // Child blocks forever (until signaled).
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }).expect("spawn_lifelined succeeds");

    // Send SIGTERM via the canonical Pidfd interface.
    pidfd.send_signal(libc::SIGTERM).expect("send_signal succeeds");

    let status = pidfd.wait_status().expect("wait_status returns signal status");
    assert_eq!(status, wat::fork::ExitStatus::Signaled(libc::SIGTERM));

    drop(lifeline);  // explicit drop for clarity
}
```

These tests prove the primitive works end-to-end: clone3 + CLONE_PIDFD creates the pidfd atomically; wait_status observes exit via waitid(P_PIDFD); send_signal signals via pidfd_send_signal.

---

## Verification protocol

1. Read existing `src/fork.rs` to understand conventions (error types, imports, existing helpers like `make_pipe`)
2. Add the new types + helper (Pidfd, LifelineWriter, spawn_lifelined, ExitStatus if needed)
3. Make types `pub` (they're the substrate's canonical primitives)
4. Re-export from `src/lib.rs` so `wat::fork::Pidfd` etc. are accessible (check existing fork re-exports)
5. Create the smoke probe `tests/probe_pidfd_primitive.rs`
6. Run cargo build:
   ```bash
   cargo build --release 2>&1 | tail -5
   ```
7. Run the smoke probe:
   ```bash
   cargo test --release --test probe_pidfd_primitive 2>&1 | tail -10
   ```
8. Write SCORE file at `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-ALPHA-MINT-PIDFD-PRIMITIVE.md`

---

## STOP triggers — VERBATIM

Non-negotiable.

1. **clone3 syscall fails or is unavailable.** STOP. Inscribe in SCORE the syscall error + your environment (`uname -a` if helpful). The Linux 5.3+ requirement may not be met (extremely unlikely on user's Linux 6 system but always possible in CI). Return.

2. **Either smoke test FAILS.** STOP. Inscribe which test + the diagnostic. Common failure causes: clone_args struct layout wrong; pidfd not properly wrapped in OwnedFd; setpgid before fork (should be 0/0 post-fork); panic in child_body not caught. Do not investigate beyond what cargo's diagnostic tells you.

3. **cargo build FAILS.** STOP. Inscribe the error. If the fix is obviously syntactic (missing import, wrong libc constant name), correct ONCE and retry. If still failing, STOP.

4. **You see a failing test OUTSIDE the smoke probe.** STOP. Workspace failure count is NOT your concern.

5. **You feel the urge to migrate run_in_fork OR existing fork sites while you're here.** STOP. ONE stone, FOUNDATION minting only. Migration is β/γ/δ/ε — separate stones.

6. **You feel the urge to make libc::fork private OR add module-privacy enforcement.** STOP. That's ζ. Not this stone.

7. **clone_args struct field names differ between libc versions.** Use whatever the current `libc` crate exposes. Check `Cargo.toml` if needed; do NOT bump the libc dependency.

---

## What the SCORE file contains

`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-ALPHA-MINT-PIDFD-PRIMITIVE.md`:

1. Header: `# Arc 213 stone α — SCORE: mint canonical Pidfd + spawn_lifelined`
2. Summary: types + helper minted; smoke probe verifies primitive end-to-end
3. File changes:
   - `src/fork.rs` — additive (Pidfd, LifelineWriter, spawn_lifelined, ExitStatus)
   - `src/lib.rs` — re-export new types if needed
   - `tests/probe_pidfd_primitive.rs` — NEW, 2 tests
4. Verification: 2 test results + cargo build clean
5. Notes: any subtleties encountered (e.g., clone_args layout; how OwnedFd wraps the kernel-returned pidfd; setpgid placement)
6. Mode classification

---

## Constraints

- Edit `src/fork.rs` (additive only — no modifications to existing fns)
- Edit `src/lib.rs` if needed for re-exports (single line; if at all)
- New file `tests/probe_pidfd_primitive.rs`
- Zero other code edits
- Zero git operations (orchestrator commits)
- Run only the smoke probe + cargo build

---

## Time prediction

30-60 min. Substrate primitive minting + libc syscall plumbing + smoke probe. Bigger than mechanical walker migrations but bounded: one new module's worth of types + one helper + one test file.

---

## Mode classification

- **Mode A:** types + helper minted; 2 smoke tests pass; cargo build clean; SCORE written
- **Mode B (acceptable):** clone3 syscall plumbing has an issue you can diagnose but not resolve in this stone (e.g., clone_args field layout differs from expected; libc crate missing CLONE_PIDFD constant); REVERT + inscribe + return
- **Mode C:** STOP rule broken (migrated existing code, made libc private, scope-crept into β/γ/δ/ε/ζ work)

The substrate teaches; you mint the canonical primitive; the rest of arc 213 builds on it.

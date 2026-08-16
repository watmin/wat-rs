//! Linux process-creation primitives (arc 213 stone α).
//!
//! `Pidfd` + `LifelineWriter` + `spawn_lifelined` + `spawn_lifelined_any` +
//! `ExitStatus` + `make_pipe`. These types are the substrate's canonical
//! process-handle primitive. Every subsequent arc 213 stone (β through ζ)
//! migrates consumers to these types.
//!
//! Plumbing notes:
//!
//! 1. `libc::clone_args` and `libc::CLONE_PIDFD` / `libc::CLONE_CLEAR_SIGHAND`
//!    availability is version-specific. Rather than assume, we define a local
//!    `CloneArgs` repr(C) struct and the constants as raw u64 values derived
//!    from <linux/sched.h>. This is safe: the kernel ABI is stable; the struct
//!    layout is defined by UAPI and will not change.
//!
//! 2. `pidfd_send_signal` is not exposed as a direct libc fn in most libc
//!    versions; we call via `libc::syscall(SYS_pidfd_send_signal, ...)`.
//!    The syscall number (424 on x86_64) is defined locally.
//!
//! 3. `waitid(P_PIDFD, ...)` uses `P_PIDFD_CONST = 3` (from <sys/wait.h>
//!    Linux 5.4+) defined locally in case libc does not expose it.
//!
//! 4. OwnedFd wrapping the kernel-returned raw pidfd: the kernel populates
//!    the pidfd field in the clone_args struct after clone3 returns. The
//!    raw fd is wrapped via `OwnedFd::from_raw_fd(pidfd_raw)` in the parent
//!    branch. The child branch runs _exit immediately; it never sees this fd.

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::time::Duration;

use crate::runtime::{RuntimeError, RuntimeErrorKind};

// ── Linux clone3 UAPI constants (from <linux/sched.h>) ────────────────────
// Defined locally so the substrate is not blocked on libc crate version.
// Values are stable kernel UAPI.
const CLONE_PIDFD_FLAG: u64 = 0x00001000; // CLONE_PIDFD — Linux 5.2+
const CLONE_CLEAR_SIGHAND_FLAG: u64 = 0x100000000; // CLONE_CLEAR_SIGHAND — Linux 5.3+

// ── clone_args UAPI struct (from <linux/sched.h>) ─────────────────────────
// repr(C): kernel requires each field to be 64-bit aligned. All fields are
// __u64 in the kernel header. We zero-initialize unused fields.
#[repr(C)]
pub(super) struct CloneArgs {
    pub(super) flags: u64,        // mask of CLONE_* flags
    pub(super) pidfd: u64,        // pointer to pidfd (if CLONE_PIDFD set)
    pub(super) child_tid: u64,    // unused (CLONE_CHILD_SETTID)
    pub(super) parent_tid: u64,   // unused (CLONE_PARENT_SETTID)
    pub(super) exit_signal: u64,  // signal to parent on child exit (SIGCHLD = 17)
    pub(super) stack: u64,        // start of stack (0 = inherit)
    pub(super) stack_size: u64,   // stack size (0 = inherit)
    pub(super) tls: u64,          // new TLS (CLONE_SETTLS)
    pub(super) set_tid: u64,      // pointer to pid array
    pub(super) set_tid_size: u64, // size of pid array
    pub(super) cgroup: u64,       // CLONE_INTO_CGROUP target (0 = current cgroup)
}

// ── P_PIDFD constant (waitid idtype_t = 3, Linux 5.4+) ───────────────────
// Defined locally: libc may expose this as libc::P_PIDFD but we use our own
// to be robust across libc versions.
const P_PIDFD_CONST: libc::idtype_t = 3;

// ── SYS_pidfd_send_signal syscall number ──────────────────────────────────
// Available as libc::SYS_pidfd_send_signal on x86_64 in modern libc.
// We define our own constant to be robust against older libc versions.
#[cfg(target_arch = "x86_64")]
const SYS_PIDFD_SEND_SIGNAL: libc::c_long = 424;
#[cfg(not(target_arch = "x86_64"))]
compile_error!("arc 213: SYS_pidfd_send_signal syscall number not defined for this arch — add it");

/// Exit status of a child process, observed via `Pidfd::wait_status`.
///
/// Arc 213 — canonical exit-status discriminated union. Used by all
/// `Pidfd` wait methods. Distinct from `exit_status_to_i64` (this file)
/// (which flattens to i64 with shell convention) — this type preserves
/// the full discriminant so callers can pattern-match on the actual cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// Process exited normally via `_exit(code)` or `return` from `main`.
    Exited(i32),
    /// Process was killed by a signal (WTERMSIG value).
    Signaled(i32),
    /// Process is stopped (WIFSTOPPED). Rare: only observed when waitid
    /// is called with WSTOPPED; spawn_lifelined does not request WSTOPPED.
    Stopped(i32),
}

/// File descriptor representing a child process, returned atomically
/// at fork time via clone3 + CLONE_PIDFD. PID-reuse race eliminated —
/// the fd is bound to THIS specific child, not its (potentially reused)
/// PID.
///
/// Arc 213 — canonical process-handle primitive. Use this for every
/// child-process operation (exit observation, signaling, status query).
/// Legacy POSIX (waitpid by PID, kill by PID) is structurally weaker;
/// migration to Pidfd methods eliminates a class of race conditions.
///
/// The ONLY way to construct a Pidfd is via `spawn_lifelined`'s return
/// value. There is no `from_pid` constructor — that would admit a
/// PID-reuse race window. This is the typestate-equivalent for
/// "non-stale handle": the substrate refuses any path that constructs
/// a Pidfd from a PID alone.
pub struct Pidfd {
    fd: OwnedFd,
    pid: libc::pid_t, // retained for diagnostic + cascade interop
}

impl std::fmt::Debug for Pidfd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pidfd")
            .field("fd", &self.fd.as_raw_fd())
            .field("pid", &self.pid)
            .finish()
    }
}

// SAFETY: OwnedFd is Send + Sync; libc::pid_t (i32) is Copy; both are
// safely transferable across threads. Pidfd is a thin wrapper; no
// interior mutability.
unsafe impl Send for Pidfd {}
unsafe impl Sync for Pidfd {}

impl Pidfd {
    /// `poll(2)` on the pidfd with `POLLIN`. Returns `Ok(true)` if the
    /// process has exited, `Ok(false)` if the timeout elapsed, `Err` on
    /// poll failure.
    ///
    /// `timeout` of `None` blocks forever. `Some(Duration::ZERO)` is a
    /// non-blocking check.
    // rune:purgare(safety-margin) — completes the pidfd primitive surface
    // (blocking wait_status / non-blocking try_wait / poll_exit); no current caller.
    pub fn poll_exit(&self, timeout: Option<Duration>) -> std::io::Result<bool> {
        let timeout_ms: libc::c_int = match timeout {
            None => -1,
            Some(d) => {
                let ms = d.as_millis();
                if ms > i32::MAX as u128 {
                    i32::MAX
                } else {
                    ms as libc::c_int
                }
            }
        };
        let mut pollfd = libc::pollfd {
            fd: self.fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, timeout_ms) };
        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(ret > 0 && (pollfd.revents & libc::POLLIN) != 0)
    }

    /// `waitid(P_PIDFD, fd, WEXITED)` — blocking wait that reaps the
    /// zombie atomically. Returns `ExitStatus`.
    ///
    /// This is the canonical blocking-wait method. After this returns,
    /// the process is reaped.
    pub fn wait_status(&self) -> std::io::Result<ExitStatus> {
        self.waitid_inner(0)
    }

    /// `waitid(P_PIDFD, fd, WEXITED | WNOHANG)` — non-blocking poll.
    /// Returns `Ok(Some(status))` if exited + reaped, `Ok(None)` if still
    /// running, `Err` on syscall failure.
    // rune:purgare(safety-margin) — completes the pidfd primitive surface
    // (blocking wait_status / non-blocking try_wait / poll_exit); no current caller.
    // rune:perspicere(read-once) — the nested Option<ExitStatus> is the minimal
    // type for a non-blocking wait; a caller that needs blocking uses wait_status().
    pub fn try_wait(&self) -> std::io::Result<Option<ExitStatus>> {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let ret = unsafe {
            libc::waitid(
                P_PIDFD_CONST,
                self.fd.as_raw_fd() as libc::id_t,
                &mut info as *mut _,
                libc::WEXITED | libc::WNOHANG,
            )
        };
        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // WNOHANG: if no child changed state, si_pid == 0.
        let si_pid = unsafe { info.si_pid() };
        if si_pid == 0 {
            return Ok(None);
        }
        Ok(Some(extract_exit_status_from_siginfo(&info)))
    }

    /// `pidfd_send_signal(fd, sig, ...)` — send signal to THIS specific
    /// process. PID-reuse-safe: the signal is delivered to the exact process
    /// represented by this fd, not any future process that reuses the PID.
    pub fn send_signal(&self, sig: i32) -> std::io::Result<()> {
        // pidfd_send_signal(pidfd, sig, info=NULL, flags=0).
        // NULL siginfo_t means "use default signal info".
        let ret = unsafe {
            libc::syscall(
                SYS_PIDFD_SEND_SIGNAL,
                self.fd.as_raw_fd() as libc::c_long,
                sig as libc::c_long,
                0i64, // NULL siginfo_t
                0i64, // flags = 0
            )
        };
        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    /// Retained for cascade interop (`killpg(pid, sig)`) + diagnostics.
    ///
    /// Do NOT use this PID for `kill(pid, sig)` — use `send_signal` instead.
    /// This PID may be reused after the child exits; `send_signal` is safe.
    pub fn pid(&self) -> libc::pid_t {
        self.pid
    }

    /// Internal: waitid using P_PIDFD. `extra_flags` is ORed with WEXITED.
    fn waitid_inner(&self, extra_flags: libc::c_int) -> std::io::Result<ExitStatus> {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let ret = unsafe {
            libc::waitid(
                P_PIDFD_CONST,
                self.fd.as_raw_fd() as libc::id_t,
                &mut info as *mut _,
                libc::WEXITED | extra_flags,
            )
        };
        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(extract_exit_status_from_siginfo(&info))
    }
}

/// Map an `ExitStatus` to the shell-convention `i64` exit code.
///
/// Shell convention: normal exit → the exit code (0–255); signal kill →
/// `128 + signal_number`. Supersedes the former `handle.rs::extract_exit_code`
/// (c_int-status decoder; removed in Stone 6.w δ-3); migration preserves the
/// exact same values callers observe through `ChildHandle::wait_or_cached_exit`.
pub(super) fn exit_status_to_i64(status: ExitStatus) -> i64 {
    match status {
        ExitStatus::Exited(code) => code as i64,
        ExitStatus::Signaled(sig) => 128 + sig as i64,
        ExitStatus::Stopped(_) => -1, // WIFSTOPPED — only with WUNTRACED; should not fire here
    }
}

/// Extract `ExitStatus` from a `siginfo_t` populated by `waitid`.
///
/// `si_code` discriminates: `CLD_EXITED` (1) → normal exit; `CLD_KILLED`
/// (2) / `CLD_DUMPED` (3) → signal termination; `CLD_STOPPED` (5) → stopped.
fn extract_exit_status_from_siginfo(info: &libc::siginfo_t) -> ExitStatus {
    // si_code values for CLD_* (from <sys/siginfo.h>):
    const CLD_EXITED: libc::c_int = 1;
    const CLD_KILLED: libc::c_int = 2;
    const CLD_DUMPED: libc::c_int = 3;
    const CLD_STOPPED: libc::c_int = 5;

    let si_code = info.si_code;
    match si_code {
        CLD_EXITED => {
            // si_status holds the exit code (WEXITSTATUS).
            let code = unsafe { info.si_status() };
            ExitStatus::Exited(code)
        }
        CLD_KILLED | CLD_DUMPED => {
            // si_status holds the signal number (WTERMSIG).
            let sig = unsafe { info.si_status() };
            ExitStatus::Signaled(sig)
        }
        CLD_STOPPED => {
            let sig = unsafe { info.si_status() };
            ExitStatus::Stopped(sig)
        }
        _ => {
            // Unexpected si_code. Treat as signal termination with si_code
            // as the "signal" — diagnostic-only; should not occur in practice.
            ExitStatus::Signaled(si_code)
        }
    }
}

/// Write-end of the lifeline pipe held by the parent process. Never
/// written to — its sole purpose is to be CLOSED (explicitly or via
/// `Drop`) so the child's lifeline_r reads return EOF and the child
/// can shut down. Inherited atomically with `clone3`.
pub struct LifelineWriter {
    pub(super) fd: OwnedFd,
}

impl LifelineWriter {
    /// Arc 213 γ-1 — extract the inner OwnedFd by consumption. Used by
    /// all fork/spawn callers (verbs.rs) to store the lifeline write-end in
    /// `ChildHandle::lifeline_w` (which holds `Option<OwnedFd>`).
    /// `ChildHandle::lifeline_w` is `Option<OwnedFd>` rather than
    /// `Option<LifelineWriter>` because callers construct it from pre-existing
    /// OwnedFd values (not always from `spawn_lifelined`) — keeping the field
    /// type minimal avoids forcing every construction path through LifelineWriter.
    pub fn into_owned_fd(self) -> OwnedFd {
        self.fd
    }
}

// ── SYS_close_range syscall number ───────────────────────────────────────────
// close_range(2) is Linux 5.9+. Used via raw libc::syscall — mirrors the
// SYS_PIDFD_SEND_SIGNAL raw-syscall pattern (Pidfd::send_signal in this file).
// compile_error! arch-guard for future ports; x86_64 = 436.
#[cfg(target_arch = "x86_64")]
#[cfg(not(target_arch = "x86_64"))]
compile_error!(
    "close_range: SYS_close_range syscall number not defined for this arch — add it"
);

/// Allocate a pipe pair; returns `(read_end, write_end)` as OwnedFds.
///
/// Public for arc 170 slice 1c — `tests/wat_arc170_channel_pipes.rs`
/// composes typed-channel pairs over fresh OS pipes. The function
/// is otherwise substrate-internal; user code reaches for typed
/// channels via the spawn primitives' Process/tx/rx (tier 2).
pub fn make_pipe(op: &str) -> Result<(OwnedFd, OwnedFd), RuntimeError> {
    let mut fds = [0i32; 2];
    // pipe2(O_CLOEXEC): atomic CLOEXEC at creation. In fork-without-exec the
    // flag doesn't fire (no exec clears it), so inheritance is unchanged.
    // Belt: any future exec path gets CLOEXEC for free; no racy fcntl needed.
    let ret = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        // arc 138: no span — make_pipe OS error; no WatAST context available
        return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("pipe2(2): {}", err)
        }));
    }
    let r = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let w = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    Ok((r, w))
}

/// Shared fork-and-observe kernel. Called by both `spawn_lifelined` and
/// `spawn_lifelined_any`; those public functions are the unwind-safe boundary.
///
/// `child_body` is `Box<dyn FnOnce(i32)>` — already type-erased at the call
/// site (the public fn wrapped the original closure). `AssertUnwindSafe` is
/// applied here once; the safety argument: the child process calls `libc::_exit`
/// on every code path, so no Rust panic-unwind machinery ever observes the
/// erased closure after catch_unwind is entered.
fn spawn_lifelined_inner(child_body: Box<dyn FnOnce(i32)>) -> std::io::Result<(Pidfd, LifelineWriter)> {
    // 1. Create lifeline pipe pre-fork. The child inherits both ends
    //    across clone3; parent drops lifeline_r; child drops lifeline_w.
    let (lifeline_r, lifeline_w) =
        make_pipe(":wat::process::spawn_lifelined").map_err(|e| {
            std::io::Error::other(format!("{}", e))
        })?;
    // Witness, not presence: write #wat.boot/Here BEFORE clone so the child
    // finds it waiting on fd 3. An inherited harness pipe has no such frame.
    crate::process::boot::write_boot_line(
        lifeline_w.as_raw_fd(),
        &crate::process::boot::BootFrame::Here.to_wire(),
    )
    .map_err(|e| std::io::Error::other(format!("{e}")))?;
    let lifeline_r_raw = lifeline_r.as_raw_fd();
    let lifeline_w_raw = lifeline_w.as_raw_fd();

    // 2. Build clone_args. Zero-initialize the struct (all unused fields
    //    must be zero per kernel documentation — extra fields added in
    //    future kernel versions must be zero or clone3 returns EINVAL).
    let mut pidfd_raw: libc::c_int = -1;
    let mut args = CloneArgs {
        flags: CLONE_PIDFD_FLAG | CLONE_CLEAR_SIGHAND_FLAG,
        pidfd: &mut pidfd_raw as *mut libc::c_int as u64,
        child_tid: 0,
        parent_tid: 0,
        exit_signal: libc::SIGCHLD as u64, // deliver SIGCHLD to parent on child exit
        stack: 0,
        stack_size: 0,
        tls: 0,
        set_tid: 0,
        set_tid_size: 0,
        cgroup: 0, // 0 = use current cgroup
    };

    // 3. Invoke clone3 via syscall. Returns child PID in parent, 0 in child.
    //    SAFETY: clone3 is a raw process-creation syscall. The args struct
    //    layout matches the kernel UAPI exactly (all u64, SIGCHLD exit_signal).
    //    We pass the struct size so the kernel knows the struct version.
    let pid = unsafe {
        libc::syscall(
            libc::SYS_clone3,
            &mut args as *mut CloneArgs,
            std::mem::size_of::<CloneArgs>(),
        )
    };

    if pid < 0 {
        // clone3 failed. Surface the OS error. Common causes:
        //   ENOSYS — kernel < 5.3 (should not occur on Linux 6.x)
        //   EINVAL — clone_args size mismatch or unsupported flag
        return Err(std::io::Error::last_os_error());
    }

    if pid == 0 {
        // ── CHILD BRANCH ──────────────────────────────────────────────
        // At this point we are in the child process. clone3 with
        // CLONE_CLEAR_SIGHAND cleared all inherited signal handlers —
        // child has clean signal disposition.

        // Step 1: become own process-group leader.
        unsafe { libc::setpgid(0, 0) };

        // Step 2: drop the inherited lifeline_w (parent-only handle).
        // SAFETY: lifeline_w_raw is a valid fd we created above; the child
        // inherited it across clone3 and must close its copy so the parent's
        // close of lifeline_w is the last reference (enabling EOF detection).
        unsafe { libc::close(lifeline_w_raw) };

        // Step 3: run child_body inside catch_unwind. child_body receives
        // the raw fd for the lifeline_r (read end of the lifeline pipe).
        // AssertUnwindSafe: the public callers guarantee the child calls
        // libc::_exit on every code path — no panic-unwinding can observe
        // aliased state after _exit terminates the process. The boxed
        // closure is erased at the public-fn boundary; unwind safety is
        // the public fn's contract, not this inner fn's.
        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| child_body(lifeline_r_raw)));

        // Step 4: _exit (not exit) to skip parent atexit handlers.
        match outcome {
            Ok(()) => unsafe { libc::_exit(0) },
            Err(_) => unsafe { libc::_exit(1) },
        }
    }

    // ── PARENT BRANCH ─────────────────────────────────────────────────
    // pid > 0: we are in the parent. clone3 has populated pidfd_raw.

    // Drop parent's copy of lifeline_r (child-only end).
    drop(lifeline_r);

    // Wrap the kernel-returned pidfd into OwnedFd. The kernel populated
    // pidfd_raw via the pointer we passed in args.pidfd during clone3.
    // SAFETY: pidfd_raw is a valid open file descriptor returned by the
    // kernel. OwnedFd::from_raw_fd takes ownership and will close it
    // on Drop.
    let pidfd_owned = unsafe { OwnedFd::from_raw_fd(pidfd_raw) };

    Ok((
        Pidfd {
            fd: pidfd_owned,
            pid: pid as libc::pid_t,
        },
        LifelineWriter { fd: lifeline_w },
    ))
}

/// Canonical fork-and-observe primitive. Uses Linux 5.3+ syscalls
/// (clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND) for atomic process
/// creation with race-free pidfd binding. Installs lifeline pipe
/// (parent holds write_end via returned `LifelineWriter`; child
/// inherits read_end via fork inheritance and observes parent death
/// via pipe-EOF).
// rune:perspicere(read-once) — (Pidfd, LifelineWriter) is the only honest
// return: the two are always born together at fork time and must always be
// held together (Pidfd alone ≠ lifeline-managed; LifelineWriter alone ≠ handle).
// A tuple is clearer than a named struct for a 2-field bundle with no invariants
// beyond "produced by spawn_lifelined".
///
/// Child setup:
/// 1. `setpgid(0, 0)` — child becomes its own process-group leader.
/// 2. Drops the lifeline_w copy it inherited (parent-only handle).
/// 3. Runs `child_body` with `lifeline_r_raw: i32` as the inherited fd.
/// 4. `_exit(0)` on `Ok` return, `_exit(1)` on panic.
///
/// Parent receives:
/// - `Pidfd` — canonical process handle (atomic with fork)
/// - `LifelineWriter` — must be held until parent wants child cleanup
///
/// The `child_body` closure receives the lifeline_r fd; the child is
/// responsible for incorporating it into its event loop
/// (poll/select) for parent-death detection.
///
/// # Errors
///
/// Returns `Err` if the `clone3` syscall fails or the lifeline pipe
/// cannot be created.
pub fn spawn_lifelined<F>(child_body: F) -> std::io::Result<(Pidfd, LifelineWriter)>
where
    F: FnOnce(i32) + std::panic::UnwindSafe + 'static,
{
    // Box the closure (erasing the type). spawn_lifelined_inner wraps it in
    // AssertUnwindSafe at the catch_unwind site — valid because the child
    // calls _exit on every code path (F: UnwindSafe is the public guarantee).
    spawn_lifelined_inner(Box::new(child_body))
}

/// Like [`spawn_lifelined`] but without the `UnwindSafe` bound on
/// `child_body`. Callers use this when the child closure captures
/// `!UnwindSafe` types (e.g. `Arc<Function>`, `comms::process::Receiver<T>`)
/// AND the child is guaranteed to call `libc::_exit` on every code path —
/// never returning to the Rust stack unwinding machinery.
///
/// The `UnwindSafe` contract (no aliased mutable refs across a panic
/// boundary) is satisfied here because `_exit` terminates the child
/// process before any panic-unwinding can observe aliased state.
/// `AssertUnwindSafe` is applied inside `spawn_lifelined_inner` so
/// both public functions share one fork body without duplicating it.
pub(crate) fn spawn_lifelined_any<F>(child_body: F) -> std::io::Result<(Pidfd, LifelineWriter)>
where
    F: FnOnce(i32) + 'static,
{
    // AssertUnwindSafe: the child calls _exit on every code path.
    spawn_lifelined_inner(Box::new(child_body))
}

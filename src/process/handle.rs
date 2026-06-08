//! Parent-side process handles.
//!
//! `ChildHandleInner` — the payload of `Value::wat__kernel__ChildHandle`.
//! `ForkedProgramHandles` — bundle returned by `fork_program_from_source`.

use std::os::fd::OwnedFd;
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};

use super::clone::Pidfd;

/// The payload of a `Value::wat__kernel__ChildHandle`. Holds the
/// child's pid plus a `reaped` flag set by `wait_or_cached`, plus a
/// `cached_exit` OnceLock that caches the exit code for idempotent
/// reads (arc-112 exit-status path; sub-fog 2c resolution). The
/// retired `:wat::kernel::wait-child` is gone (arc 112/214) —
/// `Drop` now owns the only unconditional SIGKILL+reap path.
///
/// `Drop` sends `SIGKILL` and blocks on `waitpid` if the caller
/// never called `wait_or_cached` — keeps zombies out of the process
/// table. Drop does not populate `cached_exit` because nobody can
/// read it (the Arc is going away).
#[derive(Debug)]
pub struct ChildHandleInner {
    /// Diagnostic + libc interop until δ-3 retires this field.
    /// Libc paths (wait_or_cached, Drop) use this directly via
    /// libc::waitpid / libc::kill. δ-2 migrates those paths to use
    /// self.pidfd methods; δ-3 removes this field.
    pub pid: libc::pid_t,
    pub reaped: AtomicBool,
    pub cached_exit: OnceLock<i64>,
    /// Arc 170 FD-multiplex — substrate-owned lifeline write-end.
    /// Parent holds this; never writes. When the parent process dies for
    /// any reason (clean exit / panic / SIGKILL / OOM-kill / segfault),
    /// the kernel closes all the parent's FDs as part of process
    /// teardown — including this one. The child's poll(2) over its
    /// lifeline read-end fires POLLHUP and the substrate shutdown
    /// cascade triggers.
    ///
    /// Wrapped in Option because tier-1 callers (Forked in-process by
    /// the legacy fork-program path before Phase 1C) may not yet plumb
    /// a lifeline. Once Phase 1C ships, fork-program-ast also wires
    /// one and this is always Some for forked children.
    pub lifeline_w: Option<OwnedFd>,
    /// δ-1: substrate-canonical pidfd for this forked child. Owned by
    /// ChildHandleInner; Drop fires when the last Arc<ChildHandleInner>
    /// drops (matches lifeline_w lifetime exactly). δ-2 routes wait/kill
    /// paths through this; δ-3 retires the libc fallback + pid field.
    pub pidfd: Pidfd,
}

impl ChildHandleInner {
    pub fn new(pidfd: Pidfd, lifeline_w: Option<OwnedFd>) -> Self {
        Self {
            pid: pidfd.pid(),
            reaped: AtomicBool::new(false),
            cached_exit: OnceLock::new(),
            lifeline_w,
            pidfd,
        }
    }

    /// Block on `waitpid` (idempotently) and return the exit code.
    /// Caches the first observation; subsequent calls return the
    /// cached value. Used by arc-112's unified ProgramHandle Forked
    /// variant + Process/join-result. The retired
    /// `:wat::kernel::wait-child` is gone (arc 112/214).
    pub fn wait_or_cached(&self) -> i64 {
        if let Some(&code) = self.cached_exit.get() {
            return code;
        }
        let mut status: libc::c_int = 0;
        let ret = unsafe { libc::waitpid(self.pid, &mut status, 0) };
        if ret < 0 {
            // waitpid failure (rare; ECHILD or EINTR). Surface as a
            // sentinel non-zero exit so the caller treats it as
            // catastrophic. The errno ride-along would land in arc 113.
            return -1;
        }
        let code = extract_exit_code(status);
        let _ = self.cached_exit.set(code);
        self.reaped.store(true, Ordering::SeqCst);
        code
    }
}

impl Drop for ChildHandleInner {
    fn drop(&mut self) {
        if self.reaped.load(Ordering::SeqCst) {
            return;
        }
        // Child was never reaped via wait_or_cached. Kill + reap.
        // SIGKILL is unignorable; waitpid with status pointer reaps
        // the zombie.
        unsafe {
            libc::kill(self.pid, libc::SIGKILL);
            let mut status: libc::c_int = 0;
            libc::waitpid(self.pid, &mut status, 0);
        }
    }
}

/// Extract an `:i64` exit code from the status word `waitpid(2)`
/// fills. Normal exit returns `WEXITSTATUS` (0–255). Signal
/// termination encodes as `128 + WTERMSIG`, matching the shell
/// convention — readable alongside normal codes in the same `:i64`
/// slot without a separate discriminator.
fn extract_exit_code(status: libc::c_int) -> i64 {
    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status) as i64
    } else if libc::WIFSIGNALED(status) {
        128 + libc::WTERMSIG(status) as i64
    } else {
        // WIFSTOPPED (only with WUNTRACED) — we don't pass
        // WUNTRACED to waitpid, so this branch shouldn't fire.
        -1
    }
}

/// Bundle of pipe ends + child handle returned by
/// `fork_program_from_source` for Rust callers (arc 104c's wat-cli).
/// The wat-level `eval_kernel_fork_program` wraps these into a
/// `:wat::kernel::Process` struct value.
pub struct ForkedProgramHandles {
    pub child_handle: Arc<ChildHandleInner>,
    pub stdin_w: OwnedFd,
    pub stdout_r: OwnedFd,
    pub stderr_r: OwnedFd,
}

//! Parent-side process handles.
//!
//! `ChildHandle` — the payload of `Value::wat__kernel__ChildHandle`.
//! `ForkedProgramHandles` — bundle returned by `fork_program_from_source`.

use std::os::fd::OwnedFd;
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};

use super::clone::{exit_status_to_i64, Pidfd};

/// The payload of a `Value::wat__kernel__ChildHandle`. Holds the
/// child's pidfd plus a `reaped` flag set by `wait_or_cached_exit`, plus a
/// `cached_exit` OnceLock that caches the exit code for idempotent
/// reads (arc-112 exit-status path; sub-fog 2c resolution). The
/// retired `:wat::kernel::wait-child` is gone (arc 112/214) —
/// `Drop` now owns the only unconditional SIGKILL+reap path.
///
/// `Drop` sends `SIGKILL` and blocks on `wait_status` if the caller
/// never called `wait_or_cached_exit` — keeps zombies out of the process
/// table. Drop does not populate `cached_exit` because nobody can
/// read it (the Arc is going away).
///
/// δ-2 (Stone 6.w): all wait/kill paths use `self.pidfd` methods —
/// `pidfd.wait_status()` + `pidfd.send_signal(SIGKILL)` — eliminating
/// the PID-reuse race present in the legacy `libc::waitpid(self.pid)`
/// / `libc::kill(self.pid)` paths. `reaped.compare_exchange` gates the
/// reap so exactly one caller (Drop vs concurrent `wait_or_cached_exit`)
/// performs the wait (arc 213 secare TOCTOU fix).
///
/// δ-3 (Stone 6.w): the `pid` field is retired; `pidfd.pid()` serves
/// the cascade / killpg interop callers (see `Pidfd::pid()` doc).
#[derive(Debug)]
pub struct ChildHandle {
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
    /// Wrapped in Option because tier-1 callers (fork-program-ast and
    /// fork-program-from-source) wire a lifeline for every child (arc
    /// 213 γ-1/γ-2). Always Some for forked children post-Phase-1C.
    pub lifeline_w: Option<OwnedFd>,
    /// δ-1: substrate-canonical pidfd for this forked child. Owned by
    /// ChildHandle; Drop fires when the last Arc<ChildHandle>
    /// drops (matches lifeline_w lifetime exactly). δ-2 routes all
    /// wait/kill through this; δ-3 retired the raw pid field.
    pub pidfd: Pidfd,
}

impl ChildHandle {
    pub fn new(pidfd: Pidfd, lifeline_w: Option<OwnedFd>) -> Self {
        Self {
            reaped: AtomicBool::new(false),
            cached_exit: OnceLock::new(),
            lifeline_w,
            pidfd,
        }
    }

    /// Block on `pidfd.wait_status()` (idempotently) and return the
    /// exit code using the shell convention (normal: code, signal:
    /// 128+sig). Caches the first observation; subsequent calls return
    /// the cached value. Used by arc-112's unified ProgramHandle Forked
    /// variant + Process/join-result. The retired
    /// `:wat::kernel::wait-child` is gone (arc 112/214).
    ///
    /// δ-2 migration: uses `self.pidfd.wait_status()` (waitid via pidfd —
    /// PID-reuse-safe, atomic reap) instead of `libc::waitpid(self.pid)`.
    /// `reaped.compare_exchange` gates the wait so exactly one caller
    /// (this fn vs Drop) performs the reap (secare TOCTOU fix).
    pub fn wait_or_cached_exit(&self) -> i64 {
        if let Some(&code) = self.cached_exit.get() {
            return code;
        }
        // compare_exchange(false→true): winner performs the wait;
        // loser returns the cached sentinel or spins on the OnceLock.
        // AcqRel on success: establishes happens-before with the winner's
        // subsequent wait_status write. Acquire on failure: observes the
        // winner's AcqRel store.
        if self.reaped.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            // Another caller (Drop or concurrent wait_or_cached_exit) won
            // the compare_exchange and will reap. Spin until cached_exit
            // is populated, then return it.
            loop {
                if let Some(&code) = self.cached_exit.get() {
                    return code;
                }
                std::hint::spin_loop();
            }
        }
        // We won the race — perform the wait.
        let code = match self.pidfd.wait_status() {
            Ok(status) => exit_status_to_i64(status),
            Err(_) => -1, // waitid failure (rare; ECHILD or EINTR). Sentinel.
        };
        let _ = self.cached_exit.set(code);
        code
    }
}

impl Drop for ChildHandle {
    fn drop(&mut self) {
        // compare_exchange(false→true): if the reap was already done by
        // wait_or_cached_exit, this is a no-op.  If we win, we kill+reap.
        if self.reaped.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return;
        }
        // Child was never reaped. Kill + reap via pidfd — PID-reuse-safe.
        // SIGKILL is unignorable; wait_status reaps the zombie.
        // Errors are ignored: best-effort teardown.
        let _ = self.pidfd.send_signal(libc::SIGKILL);
        let _ = self.pidfd.wait_status();
    }
}

/// Bundle of pipe ends + child handle returned by
/// `fork_program_from_source` for Rust callers (arc 104c's wat-cli).
/// The wat-level `eval_kernel_fork_program` wraps these into a
/// `:wat::kernel::Process` struct value.
pub struct ForkedProgramHandles {
    pub child_handle: Arc<ChildHandle>,
    pub stdin_w: OwnedFd,
    pub stdout_r: OwnedFd,
    pub stderr_r: OwnedFd,
}

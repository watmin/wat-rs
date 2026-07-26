//! OS signal handlers (arc 104d signal forwarding) + the child
//! process-group atomic they cascade through. Split out of
//! `distribution/mod.rs` (arc 170) — part of the run/exit path, kept
//! in its own file since it's a self-contained concern with real
//! async-signal-safety constraints (handlers may only do atomic
//! loads/stores and `killpg(2)`).
//!
//! Handlers do TWO things, in this order:
//!
//! 1. Flip the cli's local atomic flag (kernel_stop, sigusr1, etc.).
//!    These flags are inherited from pre-arc-104; they're harmless under
//!    the fork model because the cli isn't running user code, but they
//!    stay so test harnesses that spin up the cli's library API
//!    (wat::Harness::*) without going through fork still observe them.
//!
//! 2. Forward the SAME signal to the child PID via kill(2). The child
//!    has its own copy of every signal handler reset to SIG_DFL (per
//!    fork.rs::child_branch_from_source) and observes default behavior:
//!    SIGINT/SIGTERM/SIGHUP terminate; SIGUSR1/SIGUSR2 either terminate
//!    or are ignored unless the child installs its own handler. (A
//!    long-running wat program running in the child can install its
//!    own handlers via :wat::kernel::sigusr1?-style polling — same
//!    primitives, but they hook the child's flags, not the cli's.)
//!
//! The forward_signal helper reads CHILD_PGID; if -1 (no child yet),
//! no-op. If > 0, killpg(pgid, sig) — broadcasts to every process in
//! the child's process group (child + any wat-program-forked
//! grandchildren, recursively). Async-signal-safe: atomic load +
//! libc::killpg are both legal in handler context.
//!
//! Arc 106 swap: kill(pid, sig) → killpg(pgid, sig). The substrate's
//! `child_branch_from_source` calls `setpgid(0, 0)` so the child's
//! pgid == its pid, and inherited fork-program calls keep the same
//! pgid. One syscall reaches every descendant; the kernel's process-
//! group abstraction is the substrate's child-tracking mechanism.

use std::sync::atomic::{AtomicI32, Ordering};

use crate::runtime::{
    request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1, set_kernel_sigusr2,
};

// ─── Child process-group atomic for signal cascade (arc 104d → arc 106) ─
//
// Set after fork; read by signal handlers. -1 sentinel = no child yet
// (cli is still in argv parsing or pre-fork). Handlers check >= 0
// before calling killpg(2) to avoid signaling pgid 0 (the cli's own
// group, which would loop) or -1 (every process the cli can signal).
//
// Arc 106 generalizes arc 104d's CHILD_PID to a process-group ID. The
// substrate's `child_branch_from_source` calls `setpgid(0, 0)` so the
// child is its own group leader — pgid == child_pid. Subsequent
// `:wat::kernel::fork-program` calls in the wat program inherit the
// pgid by POSIX default; the kernel tracks group membership; the cli's
// `killpg(CHILD_PGID, sig)` cascades to every descendant in one
// syscall. No registry maintenance required.
pub(super) static CHILD_PGID: AtomicI32 = AtomicI32::new(-1);

extern "C" fn forward_signal(sig: libc::c_int) {
    let pgid = CHILD_PGID.load(Ordering::SeqCst);
    if pgid > 0 {
        unsafe {
            libc::killpg(pgid, sig);
        }
    }
}

extern "C" fn on_stop_signal(sig: libc::c_int) {
    request_kernel_stop();
    forward_signal(sig);
}

extern "C" fn on_sigusr1(sig: libc::c_int) {
    set_kernel_sigusr1();
    forward_signal(sig);
}

extern "C" fn on_sigusr2(sig: libc::c_int) {
    set_kernel_sigusr2();
    forward_signal(sig);
}

extern "C" fn on_sighup(sig: libc::c_int) {
    set_kernel_sighup();
    forward_signal(sig);
}

/// Install OS signal handlers BEFORE fork so they're inherited by
/// the child (which immediately resets to SIG_DFL — see fork.rs).
/// Arc 104d's signal-forwarding additions hook into these same
/// handler addresses.
pub(super) fn install_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGINT, on_stop_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_stop_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGUSR1, on_sigusr1 as *const () as libc::sighandler_t);
        libc::signal(libc::SIGUSR2, on_sigusr2 as *const () as libc::sighandler_t);
        libc::signal(libc::SIGHUP, on_sighup as *const () as libc::sighandler_t);
    }
}

//! Child-side envelope (post-clone3, pre-user code).
//!
//! Signal mask (the five are blocked; delivery is signalfd) and the
//! canonical post-fork initialization sequence.

// ─── Arc 106 — substrate-level signal handlers for fork children ─────
//
// Wat programs in spawned children observe SIGTERM / SIGINT /
// SIGUSR1/2 / SIGHUP through the same `(:wat::kernel::stopped?)` /
// `(:wat::kernel::sigusr1?)` polling contract. Delivery is signalfd;
// the five are blocked, not handled.
//
// HISTORICAL, and named so it is not read as live: the cli once had its
// own handlers that ALSO called `killpg(CHILD_PGID, sig)` to cascade to a
// forked child. Both halves of that are gone — arc 170 stopped the cli
// forking at all (`f56ad55b`), and the cascade was already fictional before
// that, since every spawned child calls `setpgid(0, 0)` and therefore sits
// in its OWN group, which a killpg on the cli's group never reaches.
//
// The substrate's handlers only flip flags. A spawned child gets its own
// handlers after its exec (`distribution::spawned_runtime`), on its own
// fresh statics.

/// Block the five substrate signals on this thread (inherited by threads
/// spawned afterward). Delivery is via `signalfd` in
/// `init_shutdown_signal_with_inputs` — there is no handler.
///
/// The name is historical. Callers that used to install `sigaction`
/// handlers now establish the blocked mask; the shutdown worker reads
/// `signalfd_siginfo` and measures. Kept so existing call sites do not
/// fork into a second installer.
pub fn install_substrate_signal_handlers() {
    crate::runtime::block_substrate_signals();
}


/// Arc 170 slice 1i — install a no-op Rust panic hook in fork child
/// branches so Rust's default "thread '...' panicked at" / "note: run
/// with RUST_BACKTRACE=1" lines never reach fd 2. The substrate's
/// `emit_structured_exit` is the SOLE source of stderr content per panic.
///
/// Must be called after dup2 (so fd 2 is the subprocess stderr pipe)
/// and before any Rust code that might panic. setpgid(2) and dup2(2)
/// are C syscalls — they do not panic in Rust — so the hook covers
/// everything that follows.
pub(crate) fn install_silent_panic_hook() {
    std::panic::set_hook(Box::new(|_info| {
        // Suppressed: substrate's catch_unwind + emit_structured_exit
        // handles panic propagation to stderr. Rust's default handler
        // must not leak plain text on fd 2 in wat-process children.
    }));
}



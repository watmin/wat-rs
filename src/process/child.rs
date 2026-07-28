//! Child-side envelope (post-clone3, pre-user code).
//!
//! Signal handlers and the canonical post-fork initialization
//! sequence (5-step: silent panic hook / setpgid / fd close-sweep /
//! shutdown-signal registration / signal-handler installation).

// ─── Arc 106 — substrate-level signal handlers for fork children ─────
//
// Wat programs in forked children must observe SIGTERM / SIGINT /
// SIGUSR1/2 / SIGHUP through the same `(:wat::kernel::stopped?)` /
// `(:wat::kernel::sigusr1?)` polling contract that worked when the
// program ran in the cli's process pre-arc-104. The handlers below
// flip the substrate's kernel flags; the wat program polls; the
// program returns cleanly when the flag is observed.
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

// Arc 170 "stopping is a protocol" Phase 3 — the handler measures, it does
// not transition, same shape as its three siblings below: one call. The
// wake-pipe write (needed so the shutdown worker's poll(2) wakes and can
// run the ask-then-await-Stopped protocol before severing anything — see
// runtime.rs's shutdown worker) used to live here inline, making this the
// one handler that did two things. It moved into `request_kernel_stop`
// itself (`src/runtime.rs`), which stays exactly as async-signal-safe as
// this call site was — `AtomicBool::store` and `libc::write` to an
// already-open pipe fd are both on the POSIX async-signal-safe list
// (signal-safety(7)); relocating them changed nothing about that.
extern "C" fn substrate_on_stop_signal(_sig: libc::c_int) {
    crate::runtime::request_kernel_stop();
}

extern "C" fn substrate_on_sigusr1(_sig: libc::c_int) {
    crate::runtime::set_kernel_sigusr1();
}

extern "C" fn substrate_on_sigusr2(_sig: libc::c_int) {
    crate::runtime::set_kernel_sigusr2();
}

extern "C" fn substrate_on_sighup(_sig: libc::c_int) {
    crate::runtime::set_kernel_sighup();
}

/// Install the substrate's wat signal handlers in the calling process.
///
/// Called by `distribution::spawned_runtime` on a freshly `execve`'d runtime,
/// and by the cli entry for its own process, to give each a working
/// `(:wat::kernel::stopped?)` / `(sigusr1?)` polling contract.
///
/// The handlers flip substrate-level static atomics (KERNEL_STOPPED,
/// KERNEL_SIGUSR1, …). Arc 170 step 4: those statics are no longer COW-copied
/// from a parent — a spawned runtime execs, so it starts with its own fresh
/// set and there is nothing inherited to flip. Each process owns its flags
/// because each process IS its own image, not because a copy was made.
///
/// Must be async-signal-safe. Each handler body is exactly one call —
/// `substrate_on_sigusr1`/`substrate_on_sigusr2`/`substrate_on_sighup` each
/// call a `set_kernel_*` that is one atomic store; `substrate_on_stop_signal`
/// calls `request_kernel_stop`, which is an atomic store plus an
/// async-signal-safe wake-pipe write (arc 170 "stopping is a protocol"
/// Phase 3) — still one call at this handler's own level, and the handler
/// itself performs no other work. The kernel MEASURES; userland owns the
/// transitions.
pub fn install_substrate_signal_handlers() {
    unsafe {
        libc::signal(
            libc::SIGINT,
            substrate_on_stop_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            substrate_on_stop_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGUSR1,
            substrate_on_sigusr1 as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGUSR2,
            substrate_on_sigusr2 as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGHUP,
            substrate_on_sighup as *const () as libc::sighandler_t,
        );
    }
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



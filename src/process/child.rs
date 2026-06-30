//! Child-side envelope (post-clone3, pre-user code).
//!
//! Signal handlers, run_in_fork, and the canonical post-fork initialization
//! sequence (5-step: silent panic hook / setpgid / fd close-sweep /
//! shutdown-signal registration / signal-handler installation).

use std::sync::atomic::Ordering;

use super::clone::{spawn_lifelined, ExitStatus, SYS_CLOSE_RANGE};
use super::verbs::{EXIT_STARTUP_ERROR, emit_structured_exit};

// ─── Arc 106 — substrate-level signal handlers for fork children ─────
//
// Wat programs in forked children must observe SIGTERM / SIGINT /
// SIGUSR1/2 / SIGHUP through the same `(:wat::kernel::stopped?)` /
// `(:wat::kernel::sigusr1?)` polling contract that worked when the
// program ran in the cli's process pre-arc-104. The handlers below
// flip the substrate's kernel flags; the wat program polls; the
// program returns cleanly when the flag is observed.
//
// Distinct from `crates/wat-cli/src/lib.rs`'s handlers: the cli's
// handlers ALSO call `killpg(CHILD_PGID, sig)` to cascade. The
// substrate's handlers only flip flags — fork children rely on the
// kernel's process-group delivery (cli broadcasts via killpg; the
// kernel delivers to every group member; each child's handler runs
// in its own process). No forwarding logic needed in substrate
// children.

extern "C" fn substrate_on_stop_signal(_sig: libc::c_int) {
    // Arc 106 — flip the kernel stop flag (existing, async-signal-safe:
    // AtomicBool::store uses a single atomic instruction).
    crate::runtime::request_kernel_stop();
    // Arc 170 Slice B — wake the shutdown worker via the wake pipe so
    // blocked crossbeam recvs are unblocked (via SHUTDOWN_RX Disconnected).
    // ONLY libc::write is called here — it is on the POSIX async-signal-safe
    // list per signal-safety(7). crossbeam::Sender::send is NOT async-signal-safe
    // and must NOT be called from a signal handler. The worker thread reads the
    // byte and calls trigger_shutdown() in normal (non-signal) context.
    let fd = crate::runtime::SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);
    if fd >= 0 {
        let byte: u8 = b'!';
        // Safety: libc::write is async-signal-safe per signal-safety(7).
        // `fd` is a valid write end of the wake pipe, set before the first
        // signal handler can fire (init_shutdown_signal() is called at
        // bootstrap, before any user code runs).
        unsafe { libc::write(fd, &byte as *const u8 as *const libc::c_void, 1) };
    }
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
/// Called by `child_branch_from_source` after fork to give the forked
/// child a working `(:wat::kernel::stopped?)` / `(sigusr1?)` / etc.
/// polling contract. The handlers reference substrate-level static
/// atomics (KERNEL_STOPPED, KERNEL_SIGUSR1, etc.) which are COW-copied
/// at fork; each process flips its own copy independently.
///
/// Must be async-signal-safe. The handlers do exactly one atomic
/// store; nothing else.
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

/// Rebirth all substrate globals with attendant threads/fds after fork.
///
/// Every substrate global with an ATTENDANT (worker thread, fd reader,
/// held lock) must rebirth here — the state forks, the attendant doesn't.
///
/// # Current inventory (Stone 214.6.4)
///
/// - **Shutdown infra**: `SHUTDOWN_RX_PTR`, `SHUTDOWN_TX_PTR`,
///   `SHUTDOWN_WAKE_WRITE_FD`, `SHUTDOWN_BROADCAST_READ_FD`, and the
///   `wat-shutdown-worker` thread. The pid-aware guard in
///   `init_shutdown_signal_with_inputs` detects the fork-child pid and
///   rebuilds automatically. The OLD inherited boxes LEAK BY DESIGN —
///   they are the parent's COW-copied state; freeing them would corrupt
///   the parent.
///
/// # Pre-gate region (async-signal-safe)
///
/// The region from `clone3` return to this call is constrained to
/// async-signal-safe operations only (the OS clears signal handlers via
/// `CLONE_CLEAR_SIGHAND`; new handlers are installed by
/// `install_substrate_signal_handlers` AFTER this call). `libc::close`
/// (called inside the guard for the inherited wake-fd) is
/// async-signal-safe per `signal-safety(7)`.
///
/// # The top rung
///
/// `fork+exec` into a fresh address space is the banked arc the
/// 214 INSCRIPTION cites — that path needs no rebirth (exec replaces
/// the address space entirely). This function targets fork-without-exec
/// children only.
///
/// Call this FIRST from every fork child before installing signal
/// handlers or doing any substrate work that depends on the infra.
pub(crate) fn rebirth_substrate_after_fork() {
    // The pid-aware guard in init_shutdown_signal detects that the
    // inherited SHUTDOWN_INIT_PID belongs to the parent (pid differs),
    // closes the inherited wake-fd, and rebuilds the channel + wake-pipe
    // + broadcast-pipe + worker thread fresh.
    crate::runtime::init_shutdown_signal();
}

/// Run `body` in a forked child process; parent waits + asserts the
/// child exited 0. Test utility for isolating per-process state
/// (OnceLock, static mut, signal handlers, install_dep_sources) when
/// multiple tests in one binary need fresh state.
///
/// The child runs `body` inside `catch_unwind`; panic → `libc::_exit(1)`
/// so the parent's assert fails with the panic visible in the child's
/// inherited stderr. Uses `_exit` (not `exit`) to skip atexit handlers
/// the parent's test harness registered — those would flush / close
/// resources the parent still owns.
///
/// Originally `runtime.rs::tests::in_signal_subprocess` for signal
/// tests (arc 012 side quest). Promoted here because any test that
/// touches process-global state can use the same pattern —
/// `tests/wat_harness_deps.rs`'s OnceLock race being the second
/// caller.
pub fn run_in_fork<F>(body: F)
where
    F: FnOnce() + std::panic::UnwindSafe + 'static,
{
    // Arc 213 β — use the canonical Pidfd + lifeline primitive instead of
    // bare libc::fork(). spawn_lifelined handles: clone3+CLONE_PIDFD,
    // setpgid(0,0), lifeline pipe, catch_unwind, and _exit(0/1).
    // run_in_fork's body doesn't poll lifeline_r — the lifeline value is
    // that if the parent dies before wait_status returns, the kernel closes
    // _lifeline (LifelineWriter) and the child's lifeline_r EOFs.
    let (pidfd, _lifeline) = spawn_lifelined(|_lifeline_r| {
        // Arc 214 Stone 6.4 — THE REBIRTH GATE: rebuild all attendant-bearing
        // substrate globals before any user code runs. The inherited shutdown
        // infra is a zombie: SHUTDOWN_INIT_PID holds the parent's pid; the
        // worker thread did not survive fork. The pid-aware guard detects this
        // and rebuilds fresh (channel + wake-pipe + broadcast-pipe + worker).
        rebirth_substrate_after_fork();
        body();
    })
    .expect("spawn_lifelined failed");

    // Parent — wait via canonical Pidfd interface + assert clean exit.
    let status = pidfd.wait_status().expect("wait_status failed");
    assert!(
        matches!(status, ExitStatus::Exited(0)),
        "forked child exited with failure: {:?}",
        status
    );
}

/// Close every inherited fd above stdio (fd > 2) in the fork child, skipping
/// every fd in `skip`.
///
/// Authoritative replacement for the old `/proc/self/fd` directory-walk oracle.
/// Uses `close_range(2)` via raw syscall — at most `skip.len() + 1` calls, no
/// filesystem traversal, no allocation. Safe in a fork(2) child because a fork
/// child is SINGLE-THREADED (fork duplicates only the calling thread), so the
/// process-global range operation cannot race with sibling threads' fd opens.
///
/// Stone 4.5-fix: the full skip-list is honored. Previously only `skip[0]` was
/// preserved (a silent single-fd limitation — dark-class per
/// `feedback_silent_swallow_is_dark_class`). The new algorithm:
///   1. Sort + dedup the skip list (O(n log n); n is tiny in practice — ≤ 5 fds).
///   2. Sweep [3, first_kept - 1], then each gap (kept[i]+1, kept[i+1]-1),
///      then [last_kept + 1, MAX].
///
/// When `skip` is empty one call covers [3, MAX].
///
/// Errors from `close_range` are ignored (consistent with the previous
/// best-effort close(2) loop). EBADF from an already-closed fd is harmless.
fn close_inherited_fds_above_stdio(skip: &[i32]) {
    // Inline helper: close_range(lo, hi, flags=0) via raw syscall (mirrors
    // SYS_PIDFD_SEND_SIGNAL at clone.rs Pidfd::send_signal). Ignore errors: best-effort.
    let sweep = |lo: libc::c_uint, hi: libc::c_uint| {
        if lo > hi {
            return; // empty range — no-op
        }
        unsafe {
            libc::syscall(SYS_CLOSE_RANGE, lo as libc::c_ulong, hi as libc::c_ulong, 0u32);
        }
    };

    if skip.is_empty() {
        // No fd to preserve — one call closes [3, MAX].
        sweep(3, libc::c_uint::MAX);
        return;
    }

    // Sort + dedup the skip list so the range sweep is deterministic.
    // n is tiny (≤ 5 fds in practice); stack allocation is fine.
    let mut kept: Vec<libc::c_uint> = skip.iter().map(|&fd| fd as libc::c_uint).collect();
    kept.sort_unstable();
    kept.dedup();

    // Sweep [3, first_kept - 1].
    sweep(3, kept[0].saturating_sub(1));

    // Sweep each gap between consecutive kept fds.
    for w in kept.windows(2) {
        sweep(w[0].saturating_add(1), w[1].saturating_sub(1));
    }

    // Sweep [last_kept + 1, MAX].
    sweep(kept[kept.len() - 1].saturating_add(1), libc::c_uint::MAX);
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
fn install_silent_panic_hook() {
    std::panic::set_hook(Box::new(|_info| {
        // Suppressed: substrate's catch_unwind + emit_structured_exit
        // handles panic propagation to stderr. Rust's default handler
        // must not leak plain text on fd 2 in wat-process children.
    }));
}

/// Arc 170 FD-multiplex Phase 3 — canonical post-fork initialization for
/// substrate-spawned wat-vm children, preserving extra fds across the
/// close-sweep.
///
/// Stone 4.5-fix: the single implementation. `child_post_fork_init` is now
/// a thin wrapper calling this with `extra_preserved = &[]`.
///
/// Both fork paths (`run_forked_child` and `spawn_process_peer`) call
/// `child_post_fork_init(l)` (the zero-extra-preserve variant) immediately
/// after their pipe-specific dup2 + drop work. `spawn_process_peer` calls
/// this directly with the comms endpoint fds in `extra_preserved`.
///
/// The 5-step canonical sequence:
///
/// 1. Install the silent panic hook (substrate's structured-stderr emit owns
///    panic propagation; Rust's default panic output is suppressed).
/// 2. Make the child its own process-group leader (arc 106 signal cascade
///    discipline). Structured-stderr + `_exit` on failure.
/// 3. Close inherited FDs above stdio (FD hygiene). The close-sweep skip-list
///    is `[lifeline_r_raw] ∪ extra_preserved` — all fds in that set survive;
///    everything else > 2 closes.
/// 4. THE REBIRTH GATE (Stone 214.6.4): Initialize the shutdown infra via
///    `init_shutdown_signal_with_inputs(&[lifeline_r_raw])`. The pid-aware
///    guard detects the fork child (inherited `SHUTDOWN_INIT_PID` holds the
///    parent's pid; `getpid()` returns the child's pid) and rebuilds the
///    entire shutdown infra (channel + wake-pipe + broadcast-pipe + worker)
///    with the lifeline FD registered. The old inherited boxes LEAK BY DESIGN.
///    Runs AFTER the close-sweep so the new wake-pipe FDs are safe from sweep.
///    Signal handlers are installed AFTER this step so they always see the
///    new wake-fd (ordering: new wake-fd stored before pid stored, pid stored
///    before handler install).
/// 5. Install substrate signal handlers (SIGTERM/SIGINT/SIGUSR1/2/SIGHUP)
///    wired through the wake-pipe to the shutdown cascade.
///
/// On any failure inside, emits structured ProcessPanics on fd 2 and
/// `_exit(EXIT_STARTUP_ERROR)`. Never returns to caller on failure; either
/// completes all 5 steps or terminates the child.
///
/// `mem::forget(lifeline_r)` stays in the CALLER's scope (transfer of
/// OwnedFd ownership to the substrate worker via the raw fd; the OwnedFd
/// value's drop must not run, but this function takes only the raw fd, so
/// the caller is the one with the OwnedFd in scope).
pub(crate) fn child_post_fork_init_preserving(lifeline_r_raw: i32, extra_preserved: &[i32]) {
    // Step 1 — suppress Rust's default panic output on fd 2.
    install_silent_panic_hook();

    // Step 2 — make this child its own process-group leader.
    if unsafe { libc::setpgid(0, 0) } < 0 {
        let err = std::io::Error::last_os_error();
        emit_structured_exit(
            None,
            crate::runtime::process_died_error_startup_value(&crate::to_edn::FlatMessage {
                tag: "StartupError",
                key: "message",
                message: &format!("setpgid(0, 0) failed: {}", err),
            }),
        );
        unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
    }

    // Step 3 — FD hygiene: close inherited fds BEFORE opening any
    // substrate-owned FDs. The skip-list is [lifeline_r_raw] ∪ extra_preserved
    // so all of those fds survive. All other inherited fds > 2 close here.
    let mut skip: Vec<i32> = Vec::with_capacity(1 + extra_preserved.len());
    skip.push(lifeline_r_raw);
    skip.extend_from_slice(extra_preserved);
    close_inherited_fds_above_stdio(&skip);

    // Step 4 — register the lifeline read-end with the shutdown worker.
    // Must run AFTER the close-sweep so wake-pipe FDs opened here are not
    // at risk of being closed by the sweep.
    crate::runtime::init_shutdown_signal_with_inputs(&[lifeline_r_raw]);

    // Step 5 — install signal handlers AFTER shutdown infrastructure is
    // ready so SIGTERM/SIGINT route through the existing wake-pipe path.
    install_substrate_signal_handlers();
}

/// Arc 170 FD-multiplex Phase 3 — canonical post-fork initialization, no
/// extra fd preservation. Thin wrapper around `child_post_fork_init_preserving`
/// with `extra_preserved = &[]`. All call sites that don't need comms endpoint
/// preservation use this form.
///
/// See `child_post_fork_init_preserving` for the full doc.
pub(crate) fn child_post_fork_init(lifeline_r_raw: i32) {
    child_post_fork_init_preserving(lifeline_r_raw, &[]);
}

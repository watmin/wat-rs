//! Runtime — AST walker for `define` / `fn` / `let` / `if` +
//! a small set of `:wat::core::*` built-in primitives + algebra-core
//! UpperCall construction.
//!
//! This is the first slice where a multi-form wat program runs
//! end-to-end. Not yet: kernel primitives (queue/spawn/select),
//! stdio handles, `:user/main`, or the measurements tier (cosine/dot).
//! Those live in later slices.
//!
//! # Value surface
//!
//! [`Value`] covers what a runtime expression can evaluate to:
//! `Bool`, `Int`, `Float`, `String`, `Keyword`, `Holon`, `Function`,
//! `Unit`, and `List` for the small set of list-shaped runtime values
//! (currently only used as return values from explicit `:wat::core::vec`
//! calls). No `Null`. No `Any`.
//!
//! # Environment model
//!
//! - [`Environment`] is a lexical-scope chain via `Arc`. Each `let` /
//!   function application creates a child env; lookups walk outward.
//! - [`SymbolTable`] holds keyword-path ↦ `Arc<Function>` entries
//!   registered by `:wat::core::defn`. Functions are looked up directly
//!   by their full path.
//!
//! # Functions
//!
//! `defn` (Clojure-aligned; Stone 241.11/241.16 retired the old Scheme-style `define`)
//! registers at call to [`register_defines`]; the body is
//! stored as an AST and evaluated on each invocation. `fn` at
//! evaluation time captures the enclosing [`Environment`] and produces
//! a `Value::Function` that can be passed, stored, and invoked.
//!
//! # Types
//!
//! The runtime treats type annotations as opaque — parse-level
//! validation rejects `:Any` and malformed type keywords, but no
//! runtime-level type enforcement happens here. The type checker
//! runs its own phase during the startup pipeline (see
//! [`crate::check`]); by the time `eval` runs, every expression
//! has already been type-verified.

use crate::ast::WatAST;
use crate::declare::parse::{
    is_declaration_form, is_declaration_head, is_type_arg_shaped,
    parse_type_slot,
};
use crate::declare::register::{meta_has_doc_axis_key, register_runtime_defs};
// Arc 109 Stone the-declare-home — test-only after the move: the lib target has no
// non-test caller left in this file, so an ungated import is `unused_imports` under -D warnings.
#[cfg(test)]
use crate::declare::register::register_defines;
use crate::declare::typevar::{angle_minted_name_reason, angle_type_head_in_name};
use crate::holon::*;
use crate::span::Span;
use holon::HolonAST;
use num_bigint::BigInt;
use num_rational::BigRational;
use std::os::fd::{AsRawFd, FromRawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use wat_macros::wat_intrinsic;
use wat_macros::wat_special_form_impl;

/// Kernel-owned stop flag read by `(:wat::kernel::stopped?)`.
///
/// The wat binary installs OS signal handlers for SIGINT and
/// SIGTERM; both set this flag to `true`. User programs poll via the
/// `:wat::kernel::stopped?` form to decide whether to continue their
/// main loops — whenever `true`, they drop their output senders
/// and return, which cascades clean shutdown through the channel
/// disconnects.
///
/// Lives under `:wat::kernel::` (not `:wat::config::`) because
/// config is user-set and frozen after startup; the stop flag
/// mutates at runtime under kernel control.
pub static KERNEL_STOPPED: AtomicBool = AtomicBool::new(false);

/// Set the kernel stop flag to `true` AND wake the shutdown worker.
/// Called by the wat CLI's SIGINT/SIGTERM signal handlers (and by
/// `compose.rs`'s external-crate equivalent). After `true` is set, any
/// user program polling `(:wat::kernel::stopped?)` will observe it and
/// can begin clean shutdown.
///
/// Arc 170 "stopping is a protocol" Phase 3 — the wake-pipe write used to
/// live in `substrate_on_stop_signal` (`src/process/child.rs`) alongside
/// this store, making that handler the one signal handler in the file
/// that did two things instead of one. Moved here so the handler itself
/// is a single call — matching `sigusr1`/`sigusr2`/`sighup`'s shape
/// (`set_kernel_sigusr1` et al., each one atomic store) — while the wake
/// behaviour itself is preserved exactly, just relocated. Still fully
/// async-signal-safe to call from a signal handler: an `AtomicBool::store`
/// and a `libc::write` to an already-open pipe fd are both on the POSIX
/// async-signal-safe list (signal-safety(7)); nothing added here changes
/// that. `SHUTDOWN_WAKE_WRITE_FD == -1` (shutdown infra not yet
/// initialized, e.g. a bare unit test calling this directly) is a safe
/// no-op — the guard below short-circuits before the write.
pub fn request_kernel_stop() {
    KERNEL_STOPPED.store(true, Ordering::SeqCst);
    let fd = SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);
    if fd >= 0 {
        let byte: u8 = b'!';
        // SAFETY: libc::write is async-signal-safe per signal-safety(7).
        // `fd` is either -1 (guarded above) or a valid write end of the
        // wake pipe, set before the first signal handler can fire
        // (init_shutdown_signal() runs at bootstrap, before any user code).
        unsafe { libc::write(fd, &byte as *const u8 as *const libc::c_void, 1) };
    }
}

/// Reset the kernel stop flag. Used only by test harnesses that
/// exercise the flag within a single process — the flag is a
/// process-lifetime static and test ordering can otherwise leak
/// state between tests.
#[cfg(test)]
pub fn reset_kernel_stop() {
    KERNEL_STOPPED.store(false, Ordering::SeqCst);
}

/// Non-terminal user-signal flags — SIGUSR1, SIGUSR2, SIGHUP. Per the
/// 2026-04-19 signal-model stance: the kernel MEASURES; userland owns
/// the transitions. OS signal handlers set these true; wat programs
/// poll via `(:wat::kernel::sigusr1?)` / `(sigusr2?)` / `(sighup?)`
/// and clear via the matching `reset-*!` primitive.
///
/// Unlike [`KERNEL_STOPPED`] (terminal, set-once), these flags are
/// designed to be flipped back to `false` from userland. The boolean
/// is coalesced — five SIGHUPs in a burst read as one "yes" on the
/// next poll. Callers that need counter semantics build that in
/// userland.
pub static KERNEL_SIGUSR1: AtomicBool = AtomicBool::new(false);
pub static KERNEL_SIGUSR2: AtomicBool = AtomicBool::new(false);
pub static KERNEL_SIGHUP: AtomicBool = AtomicBool::new(false);

/// Set the SIGUSR1 flag. Called by the OS signal handler.
pub fn set_kernel_sigusr1() {
    KERNEL_SIGUSR1.store(true, Ordering::SeqCst);
}

/// Set the SIGUSR2 flag. Called by the OS signal handler.
pub fn set_kernel_sigusr2() {
    KERNEL_SIGUSR2.store(true, Ordering::SeqCst);
}

/// Set the SIGHUP flag. Called by the OS signal handler.
pub fn set_kernel_sighup() {
    KERNEL_SIGHUP.store(true, Ordering::SeqCst);
}

/// Reset all user-signal flags. Test-only — production uses the per-flag
/// `reset-*!` wat primitives.
#[cfg(test)]
pub fn reset_user_signals() {
    KERNEL_SIGUSR1.store(false, Ordering::SeqCst);
    KERNEL_SIGUSR2.store(false, Ordering::SeqCst);
    KERNEL_SIGHUP.store(false, Ordering::SeqCst);
}

/// Process-wide argv ambient — populated once by wat-cli (or any
/// embedder) before `:user::main` runs; thereafter accessible from
/// any wat code via `(:wat::runtime::argv)`.
///
/// Per arc 170 REALIZATIONS pass 7 (ambient runtime) the four-arg
/// `:user::main` shape (stdin/stdout/stderr/argv) retires; argv moves
/// to an ambient runtime value. Pattern mirrors [`KERNEL_STOPPED`]
/// (set-once kernel-owned static) but uses `OnceLock<Arc<Vec<String>>>`
/// because the value is structured (a Vec of String, not a single
/// boolean). The Arc keeps clone-out cheap on each ambient read.
///
/// Set once via [`set_argv`]; subsequent `set_argv` calls panic via
/// `OnceLock::set`'s Err path (caller misuse — argv is set-once at
/// process start).
pub static ARGV: OnceLock<Arc<Vec<String>>> = OnceLock::new();

/// Set the process-wide argv ambient. Called by wat-cli (or any
/// embedder) BEFORE `invoke_user_main` runs; `(:wat::runtime::argv)`
/// reads thereafter.
///
/// Set-once: a second call returns the original value back via
/// `OnceLock::set`'s Err arm. We swallow that Err — the semantics are
/// "first set wins" and tests that re-invoke wat-cli inside one
/// process get the first invocation's argv. Production wat-cli runs
/// exactly once per process; the Err arm is a test-isolation
/// affordance, not user surface.
pub fn set_argv(argv: Vec<String>) {
    let _ = ARGV.set(Arc::new(argv));
}

/// Read the process-wide argv ambient. Returns an empty Vec if no
/// embedder set argv (in-process tests, library bridges that bypass
/// wat-cli). The ambient is "always available"; callers don't have
/// to gate on whether it was set.
pub fn argv() -> Arc<Vec<String>> {
    ARGV.get().cloned().unwrap_or_else(|| Arc::new(Vec::new()))
}

// Note: `OnceLock` has no public reset; tests that need to vary argv
// across cases run in separate processes (cargo test default) or
// cooperate by reading `argv()` and not assuming one fixed value
// across calls. The "first set wins" semantics of `set_argv` are
// the correct shape for production (wat-cli sets once at process
// start); test cooperation handles the rest.

// ── Arc 170 Slice A — process-wide shutdown signal infrastructure ──────────
//
// Four statics form the lock-free shutdown cascade.  All follow the
// ZERO-MUTEX doctrine (docs/ZERO-MUTEX.md): AtomicPtr + Box,
// NO Mutex / RwLock / OnceLock / CondVar.
//
// Stone 214.6.4 — FORK-AWARE redesign: SHUTDOWN_RX_PTR replaces
// SHUTDOWN_RX (OnceLock). AtomicPtr swap enables the pid-aware guard to
// rebuild the entire shutdown infra in a clone3 child (the inherited worker
// thread doesn't survive fork; the OnceLock guard was the lie that blocked
// rebirth). SHUTDOWN_INIT_PID records which process last initialized so the
// guard can detect "same-process no-op" vs "child needs rebirth".

// Type aliases for the shutdown channel endpoints (Stone 6.w perspicere L3):
// AtomicPtr<ShutdownRx> / AtomicPtr<ShutdownTx> reads more clearly than the
// 2-level nested form at every declaration site and in shutdown_rx()'s return.
type ShutdownRx = crossbeam_channel::Receiver<()>;
type ShutdownTx = crossbeam_channel::Sender<()>;

/// Heap-boxed Receiver for the process-wide shutdown signal channel.
/// AtomicPtr allows the fork-aware guard in `init_shutdown_signal_with_inputs`
/// to swap in a fresh Receiver after fork (the inherited Receiver is the
/// child's process-local copy; it leaks by design — comment in the guard).
///
/// null = uninitialized; non-null = initialized. Load with SeqCst; the
/// `shutdown_rx()` getter returns a `Option<&'static ShutdownRx>`.
///
/// Previously `SHUTDOWN_RX: OnceLock<Receiver<()>>` (Stone 214.6.4
/// replaced OnceLock with AtomicPtr to allow fork-aware rebirth).
// rune:sequi(ambient-context) — ZERO-MUTEX shutdown cascade channel; threading
// this through every recv signature would bloat every blocking call. Documented
// in ZERO-MUTEX.md as the declared ambient-context exception.
static SHUTDOWN_RX_PTR: std::sync::atomic::AtomicPtr<ShutdownRx> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// The pid of the process that last successfully initialized the shutdown
/// infra. 0 = never initialized. Used by `init_shutdown_signal_with_inputs`
/// to detect fork children: if `SHUTDOWN_RX_PTR` is non-null but
/// `SHUTDOWN_INIT_PID != getpid()`, we are in a clone3 child whose
/// inherited state is a zombie — rebuild (the inherited worker thread
/// does not exist).
static SHUTDOWN_INIT_PID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Get a shared reference to the process-wide shutdown Receiver, or
/// `None` if the shutdown infra has not been initialized yet (pre-init
/// or bootstrap fallback path).
///
/// SAFETY: The pointer was stored via `Box::into_raw` in
/// `init_shutdown_signal_with_inputs`; it is valid for the lifetime of
/// the pointer value (i.e., `'static`). The old box leaks by design on
/// fork-rebirth (see the guard comment) — the pointer is never freed
/// while any caller could still observe it.
pub(crate) fn shutdown_rx() -> Option<&'static ShutdownRx> {
    let ptr = SHUTDOWN_RX_PTR.load(Ordering::SeqCst);
    if ptr.is_null() {
        None
    } else {
        // SAFETY: ptr is non-null and was produced by Box::into_raw in
        // init_shutdown_signal_with_inputs. The lifetime is effectively
        // 'static — the box leaks on fork-rebirth; in the normal single-process
        // path the box is never freed (the shutdown infra lives forever).
        Some(unsafe { &*ptr })
    }
}

/// Heap-boxed Sender for the shutdown signal. AtomicPtr swap-to-null +
/// Box::from_raw drop is the ZERO-MUTEX way to atomically drop the
/// Sender (waking all SHUTDOWN_RX clones with Disconnected). Initialized
/// via [`init_shutdown_signal`]; consumed by [`trigger_shutdown`].
// rune:sequi(ambient-context) — ZERO-MUTEX shutdown cascade trigger; paired
// with SHUTDOWN_RX_PTR. trigger_shutdown() atomically drops the Sender.
static SHUTDOWN_TX_PTR: std::sync::atomic::AtomicPtr<ShutdownTx> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// Write-end of the wake pipe. The SIGTERM/SIGINT signal handler writes
/// a byte here (async-signal-safe per signal-safety(7)). The shutdown
/// worker thread reads from the corresponding read-end and calls
/// [`trigger_shutdown`] in normal context (where Sender drop is safe).
/// -1 means uninitialized; signal handler no-ops if so.
pub static SHUTDOWN_WAKE_WRITE_FD: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(-1);

/// Arc 170 Phase 2 — substrate-owned shutdown broadcast read-fd.
/// Worker holds the write-end; drops it after trigger_shutdown.
/// All PipeFd-backed Receiver recvs poll this fd; POLLHUP → Shutdown.
/// Value -1 until init_shutdown_signal_with_inputs runs; valid fd
/// after. RE-SET in fork children (init_shutdown_signal_with_inputs
/// rebuilds the fd for each child's private shutdown worker — the
/// inherited worker thread does not transfer across clone3).
// rune:sequi(ambient-context) — ZERO-MUTEX broadcast fd; every poll()-based
// recv in comms::process reads this fd. Threading it through every recv
// signature is the alternative the doctrine explicitly rejects.
pub static SHUTDOWN_BROADCAST_READ_FD: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(-1);

/// Initialize the shutdown signal infrastructure. Idempotent within a
/// process; FORK-AWARE across them — a clone3 child's first call rebuilds
/// (the inherited worker thread does not exist; the inherited state is a
/// zombie). The 2026-06-07 live diagnosis is the WHY (SCORE-STONE-6.3
/// § the live catch).
///
/// Creates:
///   1. A crossbeam unbounded channel pair (rx → SHUTDOWN_RX_PTR, tx → SHUTDOWN_TX_PTR)
///   2. A wake pipe (write-end → SHUTDOWN_WAKE_WRITE_FD, read-end → worker)
///   3. A worker thread that blocks on the wake pipe read; on wake,
///      calls trigger_shutdown
pub fn init_shutdown_signal() {
    init_shutdown_signal_with_inputs(&[])
}

/// Same as [`init_shutdown_signal`] but the spawned worker polls an
/// additional input-FD set alongside the wake pipe. Any FD becoming
/// ready (POLLIN | POLLHUP) → `trigger_shutdown`. The lifeline-pipe
/// pattern (per `DESIGN-FD-MULTIPLEX-SHUTDOWN.md`) registers the
/// child-side read-end here so parent-process death → kernel closes
/// parent's write-end → child's poll returns POLLHUP → shutdown cascade.
///
/// All extra input FDs must remain valid for the lifetime of the
/// process (the worker holds them in its poll set forever). The wake
/// pipe is owned by the substrate; extra FDs are caller-owned and
/// caller-managed (e.g., the bootstrap path keeps the lifeline read-end
/// alive via an OwnedFd held in `ProcessRuntime`).
///
/// Idempotent within a process: if `SHUTDOWN_RX_PTR` is non-null AND
/// `SHUTDOWN_INIT_PID == getpid()`, this is a no-op. Fork-aware: if the
/// ptr is non-null but the pid differs, we are in a clone3 child whose
/// inherited shutdown worker thread does not exist — the guard fires and
/// rebuilds the entire infra. The OLD heap boxes (Receiver + Sender) are
/// NOT freed — they are the child's inherited process-local copies and
/// must not be dropped (that would corrupt the parent's state via
/// copy-on-write). They LEAK BY DESIGN. The old inherited wake write-fd
/// is closed (it was the parent's fd; the new fd is stored BEFORE signal
/// handler re-installation in the child sequence).
pub fn init_shutdown_signal_with_inputs(extra_input_fds: &[i32]) {
    let current_pid = unsafe { libc::getpid() };
    // Guard: initialized AND same process → no-op.
    let rx_ptr = SHUTDOWN_RX_PTR.load(Ordering::SeqCst);
    if !rx_ptr.is_null() && SHUTDOWN_INIT_PID.load(Ordering::SeqCst) == current_pid {
        return; // already initialized in this process
    }

    // Either first call ever (rx_ptr is null) OR we are in a fork child
    // (rx_ptr non-null, pid differs). In the fork-child case:
    //   - rx_ptr and the old SHUTDOWN_TX_PTR point to the PARENT's
    //     heap-boxed values (COW copy). We must NOT free them (Box::from_raw
    //     would corrupt the parent on the next COW write). They LEAK.
    //   - The inherited wake-write fd is the PARENT's fd; close it now
    //     so the new fd takes its place before the signal handler fires.
    if !rx_ptr.is_null() {
        // Fork child — close the inherited wake write-fd.
        // The new fd is stored below BEFORE signal handler installation.
        //
        // CROSS-STEP NOTE (F4): child.rs::child_post_fork_init step 3
        // (close_range) already closed this fd. This guard fires on the
        // same raw int a second time → EBADF, which we discard. This is
        // intentional and benign: single-threaded child at this point means
        // no fd recycling can occur between step 3 and here. The guard
        // exists so that if a future caller invokes init_shutdown_signal_with_inputs
        // WITHOUT the close_range step, the fd is still closed safely.
        // Do NOT reorder step 3 and step 4 — that would create a real
        // recycled-fd double-close risk.
        let old_write_fd = SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);
        if old_write_fd >= 0 {
            unsafe { libc::close(old_write_fd) };
        }
        // Do NOT free rx_ptr or SHUTDOWN_TX_PTR — they are the parent's
        // COW-copied boxes; freeing would corrupt the parent. LEAK BY DESIGN.
    }

    let (tx, rx) = crossbeam_channel::unbounded::<()>();
    // Store the new Receiver via AtomicPtr.
    let rx_boxed = Box::into_raw(Box::new(rx));
    SHUTDOWN_RX_PTR.store(rx_boxed, Ordering::SeqCst);
    let tx_boxed = Box::into_raw(Box::new(tx));
    SHUTDOWN_TX_PTR.store(tx_boxed, Ordering::SeqCst);

    // Create wake pipe (async-signal-safe write-end; blocking read-end).
    // pipe2(O_CLOEXEC): atomic CLOEXEC — belt for any future exec path; in
    // fork-without-exec the flag doesn't fire, close_range handles hygiene.
    let mut fds = [0_i32; 2];
    let pipe_result = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if pipe_result != 0 {
        // pipe2(2) failed — substrate cannot safely operate. Structured
        // stderr diagnostic + exit. Should never happen in practice on Linux.
        // (Using write(2) directly avoids stdio locking in this early context.)
        let msg = b"substrate: pipe2(2) failed during shutdown init\n";
        unsafe { libc::write(2, msg.as_ptr() as *const _, msg.len()) };
        // _exit(2): fork-safe; skips atexit/stdio flush which would
        // corrupt shared parent state in a forked child context.
        unsafe { libc::_exit(1) };
    }
    let read_fd = fds[0];
    let write_fd = fds[1];
    SHUTDOWN_WAKE_WRITE_FD.store(write_fd, Ordering::SeqCst);

    // Phase 2 — broadcast pipe for tier-2 PipeFd recvs.
    // Worker holds the write-end; drops it after trigger_shutdown().
    // All PipeFd-backed Receiver recvs poll the read-end; POLLHUP → Shutdown.
    let mut broadcast_fds = [0_i32; 2];
    // pipe2(O_CLOEXEC): atomic CLOEXEC — belt for any future exec path.
    let broadcast_result = unsafe { libc::pipe2(broadcast_fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if broadcast_result != 0 {
        let msg = b"substrate: pipe2(2) failed during broadcast init\n";
        unsafe { libc::write(2, msg.as_ptr() as *const _, msg.len()) };
        // _exit(2): fork-safe; same rationale as pipe2 failure above.
        unsafe { libc::_exit(1) };
    }
    let broadcast_r_fd = broadcast_fds[0];
    // SAFETY: broadcast_fds[1] is a valid owned fd from pipe2(2); wrapping as
    // OwnedFd ensures the write-end is closed unconditionally on any worker
    // exit path (including panic), so POLLHUP propagates to all readers.
    let broadcast_w_fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(broadcast_fds[1]) };
    SHUTDOWN_BROADCAST_READ_FD.store(broadcast_r_fd, Ordering::SeqCst);

    // Build the worker's pollfd set: wake-pipe + caller-provided inputs.
    // Captured by value into the worker closure; the worker owns its set.
    let mut input_fds: Vec<i32> = Vec::with_capacity(1 + extra_input_fds.len());
    input_fds.push(read_fd);
    input_fds.extend_from_slice(extra_input_fds);

    // Spawn the shutdown-worker thread. It blocks on poll(2) over all
    // input FDs; first to fire wins. On wake → trigger_shutdown in
    // normal context (Sender drop is safe; not in signal handler).
    //
    // poll(2) over pipe FDs is the Linux primitive that gives lock-step
    // OS-event delivery without timing (per INTERSTITIAL Linux-only § —
    // signalfd/eventfd/epoll/poll are the load-bearing primitives).
    // POLLHUP fires on pipe-EOF (all writers closed) — that's how the
    // lifeline mechanism propagates parent-death without a signal.
    std::thread::Builder::new()
        .name("wat-shutdown-worker".to_string())
        .spawn(move || {
            let mut pollfds: Vec<libc::pollfd> = input_fds
                .iter()
                .map(|&fd| libc::pollfd {
                    fd,
                    events: libc::POLLIN | libc::POLLHUP,
                    revents: 0,
                })
                .collect();
            // Block forever (timeout = -1). EINTR retries; any FD ready
            // (POLLIN or POLLHUP) → break.
            loop {
                let n = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as _, -1) };
                if n > 0 {
                    break; // some FD ready — wake
                }
                // n == 0 cannot happen with timeout=-1 in normal operation;
                // n < 0 is typically EINTR (signal interrupted poll). Retry.
            }
            // Wake received.
            //
            // Arc 170 "stopping is a protocol", builder-corrected ordering — the worker MEASURES
            // and WAKES; it does not transition. This file's own signal doctrine ("the kernel
            // MEASURES; userland owns the transitions") applied to the SIGNAL HANDLER from the
            // start; it now applies to the worker too. An earlier revision of this stone had the
            // worker itself announce (`StopAccepted`) and ask each held service to stop — WRONG:
            // `ThreadOwnedCell` (`src/rust_deps/custodia.rs`) binds a Handle's admin `Peer'` to
            // whichever OS thread constructed it (main, via `bootstrap_wat_vm_process` →
            // `start-primed-stdio`); only THAT thread may legally `send'`/`recv'` on it. The
            // worker is a different OS thread and can never satisfy that check — every ask from
            // here failed, always, silently swallowed until the fix that stopped discarding the
            // error surfaced it (see the arc 170 report). The announce and the ask-then-await now
            // run on MAIN — `ProcessRuntime::ask_stop_and_collect_failures` (`src/freeze.rs`),
            // called from `invoke_user_main_orchestrated` on `:user::main`'s way out, while main
            // still owns the Handles.
            //
            // The wake byte is simultaneously the reason-free notice (contract rule 4: a client
            // never learns a service's crash/stop reason; `POLLIN` carries none) AND the unblock
            // that lets main be the one to act: Phase 1's `POLLIN | POLLHUP` split (readers no
            // longer wait for `POLLHUP`-only) means this write alone reaches every process-tier
            // reader — including main's own blocked `readln` / `read-frame` — WITHOUT tearing
            // anything down. Main's read returns `Stopped`, not `Disconnected`; `:user::main`
            // observes `(stopped?)` and returns normally, still holding its Handles, still able
            // to ask them to stop.
            let wake_byte: [u8; 1] = [0];
            unsafe {
                libc::write(
                    broadcast_w_fd.as_raw_fd(),
                    wake_byte.as_ptr() as *const _,
                    1,
                );
            }
            // Ground truth (driven by hand, not assumed — see the arc 170 report): calling
            // `trigger_shutdown()` here unconditionally, immediately after the wake byte,
            // reproduces the ORIGINAL bug on the very first try — main's ask-then-await
            // (`ProcessRuntime::ask_stop_and_collect_failures`, `src/freeze.rs`) sends
            // `Admin::Stop` down a THREAD-TIER `Peer'`, whose `recv'` for `Status::Stopped` is
            // cascade-aware (`comms::thread::Receiver::recv`, selects against `shutdown_rx()`).
            // If `SHUTDOWN_TX_PTR` is already dropped by the time that `recv'` runs, crossbeam's
            // `select!` can pick the disconnected shutdown arm over the real (already-sent, real)
            // reply — every ask spuriously fails with "process shutdown" (`RecvOutcome::Shutdown`)
            // instead of the true `Status::Stopped`. So this call is conditional: only when NO
            // `ProcessRuntime` is alive to race (`stdio_bootstrapped() == false` — a bare
            // library/test caller that spawned the shutdown infra directly, e.g.
            // `tests/process/shutdown_cascade_memory.rs`'s probes, which have no Handles to ask
            // and legitimately need their OWN blocked thread-tier recvs woken NOW). When a
            // `ProcessRuntime` IS alive, this is main's job, AFTER its ask completes —
            // `invoke_user_main_orchestrated` (`src/freeze.rs`) calls `trigger_shutdown()` itself,
            // once `ask_stop_and_collect_failures` has returned.
            if !stdio_bootstrapped() {
                trigger_shutdown();
            }
        })
        .expect("wat-shutdown-worker thread spawn failed");

    // Record this process as the owner. Done AFTER the worker spawns so
    // there is no window where the pid is set but the worker is not yet
    // running. (The ordering: new wake-fd stored, new rx_boxed stored,
    // worker spawned, pid stored — ensures any concurrent reader that
    // observes the new pid also observes the new wake-fd and new rx.)
    SHUTDOWN_INIT_PID.store(current_pid, Ordering::SeqCst);
}

/// Drop the global SHUTDOWN_TX_PTR. All SHUTDOWN_RX recvs wake with
/// crossbeam Disconnected (which typed_recv Slice B maps to
/// RecvOutcome::Shutdown). Idempotent — second call sees null pointer
/// and no-ops.
///
/// MUST be called from normal context (deallocator can run). The signal
/// handler MUST NOT call this directly — it writes to the wake pipe;
/// the worker thread calls trigger_shutdown.
pub fn trigger_shutdown() {
    let ptr = SHUTDOWN_TX_PTR.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !ptr.is_null() {
        // SAFETY: ptr was Box::into_raw'd in init_shutdown_signal and
        // is never accessed except via this swap. The swap to null
        // means no other thread can race us into Box::from_raw on the
        // same pointer.
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

// ── Arc 170 "stopping is a protocol" — no silent drop on the stop path ─────
//
// Builder ruling: "any failure must be loud and obvious." A failed announce
// or a failed ask is collected here (never discarded with `let _ =`) and
// reported by the exit path (`src/distribution/mod.rs`) once `:user::main`
// returns — as registered EDN (`StopFailed`) on stderr, immediately before a
// non-zero exit. No new channel: `StopFailed`/`StopFailure` are registered
// kernel records (`src/types.rs`), and stderr is the SAME dying-declaration
// channel `emit_panic_envelope` (`src/process/stdio.rs`) already writes on.
//
// Correction (builder-ruled): the announce + the ask-then-await themselves do
// NOT live here anymore. `ThreadOwnedCell` (`src/rust_deps/custodia.rs`) binds
// each Handle's admin `Peer'` to whichever thread constructed it —
// `bootstrap_wat_vm_process`, always the caller's own thread (main, in the
// CLI). The shutdown worker is a DIFFERENT OS thread, so it can never
// legally ask; only MAIN can. See `ProcessRuntime::ask_stop_and_collect_failures`
// (`src/freeze.rs`) for where the announce/ask actually run now — main, on
// `:user::main`'s way out, while it still owns the Handles. What remains
// here is thread-agnostic: building the registered `Fault`/`StopFailure`/
// `StopFailed` values, and the single-slot publish/take hand-off the exit
// path uses to read what main collected (kept as a plain global, not
// threaded through `invoke_user_main`'s signature — dozens of test callers
// use that signature directly and don't care about this).

::wat_source_derive::wat_field_names_from!(FAULT_FIELDS, "wat/core.wat", ":wat::core::Fault");
::wat_source_derive::wat_field_names_from!(
    STOP_FAILURE_FIELDS,
    "wat/kernel/diagnostics.wat",
    ":wat::kernel::StopFailure"
);
::wat_source_derive::wat_field_names_from!(
    STOP_FAILED_FIELDS,
    "wat/kernel/diagnostics.wat",
    ":wat::kernel::StopFailed"
);

/// `OnceLock` so a hot error path allocates the name vector once, not per raised fault.
pub(crate) fn fault_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(FAULT_FIELDS))
        .clone()
}
// Arc 109 Stone 4c — the freeze stop vocabulary — `stop_failure_names` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4c — the freeze stop vocabulary — `stop_failed_names` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Convert a `RuntimeError` into a `:wat::core::Fault` (`wat/core.wat`) — the canonical minimal
/// record that structurally satisfies the `:wat::core::Error` surface: `message`, `location` (a
/// `:wat::kernel::Location`, via [`value_from_span`]), `causes` (empty — a Fault is a leaf).
/// Chosen over round-tripping `RuntimeError`'s own `WatError::error_edn()` through `edn_to_value`,
/// which would require every possible `RuntimeErrorKind` variant tag to be independently
/// EDN-decodable; `Fault` is already a single, simple, always-registered record.
pub(crate) fn fault_from_runtime_error(err: &RuntimeError) -> Value {
    use crate::edn::contract::WatError;
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::core::Fault".to_string(),
        fault_names(),
        Arc::new(vec![
            Value::String(Arc::new(err.message())),
            value_from_span(err.span().clone()),
            Value::Vec(Arc::new(Vec::new())),
        ]),
    )))
}

// Arc 109 Stone 4c — the freeze stop vocabulary — `stop_failure_value` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Convert a caught panic payload into a `:wat::core::Fault`. Ground truth, verified by hand
/// (not assumed): `:wat::kernel::assertion-failed!` — the arm the generated `<fqdn>/stop` caller's
/// `RecvOutcome::Lost`/`Closed` branches raise on (`wat/service.wat` ~:1479), and the arm
/// `stdio-write-out` raises on a lost/closed StdOut peer — is `std::panic::panic_any`
/// (`src/assertion.rs::eval_kernel_assertion_failed`), NOT a returned `Err`. A real broken-pipe
/// stop failure was driven by hand to find this (see the arc 170 report): `apply_function` never
/// returned `Err` for it at all — it unwound. Without this arm, `stop_failure_value` alone would
/// silently miss the exact failure class it exists to report — this stone's whole point.
///
/// Downcast order mirrors `finish_forked_child`'s existing panic-payload handling
/// (`src/process/verbs.rs`): `AssertionPayload` (carries its own message + location) → `String` /
/// `&str` (a bare panic message, no location — falls back to this call site) → an opaque fallback.
pub(crate) fn fault_from_panic_payload(payload: &(dyn std::any::Any + Send)) -> Value {
    if let Some(p) = payload.downcast_ref::<crate::assertion::AssertionPayload>() {
        let span = p
            .location
            .clone()
            .unwrap_or_else(|| crate::rust_caller_span!());
        Value::Aggregate(Arc::new(AggregateValue::record(
            "wat::core::Fault".to_string(),
            fault_names(),
            Arc::new(vec![
                Value::String(Arc::new(p.message.clone())),
                value_from_span(span),
                Value::Vec(Arc::new(Vec::new())),
            ]),
        )))
    } else {
        let message = if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = payload.downcast_ref::<&str>() {
            (*s).to_string()
        } else {
            "<unknown panic payload>".to_string()
        };
        Value::Aggregate(Arc::new(AggregateValue::record(
            "wat::core::Fault".to_string(),
            fault_names(),
            Arc::new(vec![
                Value::String(Arc::new(message)),
                value_from_span(crate::rust_caller_span!()),
                Value::Vec(Arc::new(Vec::new())),
            ]),
        )))
    }
}

// Arc 109 Stone 4c — the freeze stop vocabulary — `stop_failure_from_panic` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4c — the freeze stop vocabulary — `stop_failed_value` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4c — the freeze stop vocabulary — `STOP_FAILURES_PTR` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4c — the freeze stop vocabulary — `publish_stop_failures` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4c — the freeze stop vocabulary — `take_stop_failures` moved to
// `src/freeze/stop.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// `true` while a `ProcessRuntime` (`src/freeze.rs`) is alive in this process — i.e. there are
/// Handles main might be about to (or already) ask to stop. Set by `bootstrap_wat_vm_process`
/// once the Handles are ready; cleared by `ProcessRuntime::Drop`.
///
/// The shutdown worker consults this exactly once, right after writing the wake byte, to decide
/// whether it is safe to call [`trigger_shutdown`] itself: ground-truthed by hand (see the arc 170
/// report) that calling it unconditionally there races main's ask-then-await and turns every ask
/// into a spurious "process shutdown" failure — so the worker only fires it when `false` (no
/// `ProcessRuntime` alive to race, e.g. a bare library/test caller with its own blocked
/// thread-tier recvs and nothing to ask). When `true`, `invoke_user_main_orchestrated`
/// (`src/freeze.rs`) calls `trigger_shutdown()` itself, AFTER `ask_stop_and_collect_failures`
/// returns — which is what makes it safe.
static STDIO_BOOTSTRAPPED: AtomicBool = AtomicBool::new(false);

/// Mark a `ProcessRuntime` as alive. Called once by `bootstrap_wat_vm_process` after the primed
/// stdio Handles are ready.
pub(crate) fn set_stdio_bootstrapped() {
    STDIO_BOOTSTRAPPED.store(true, Ordering::SeqCst);
}

/// Mark no `ProcessRuntime` as alive. Called by `ProcessRuntime::Drop`.
pub(crate) fn clear_stdio_bootstrapped() {
    STDIO_BOOTSTRAPPED.store(false, Ordering::SeqCst);
}

/// Read whether a `ProcessRuntime` is currently alive. See [`STDIO_BOOTSTRAPPED`]'s doc.
fn stdio_bootstrapped() -> bool {
    STDIO_BOOTSTRAPPED.load(Ordering::SeqCst)
}

// ── End arc 170 Slice A shutdown infrastructure ────────────────────────────

// Stone 251.2e — Value cluster (Value enum + Clause/ClauseSet/ClauseAttempt/ClauseFailureReason
// + StructValue/EnumValue + all impls) moved to src/value/value.rs. Re-exported here for
// zero-churn (Value used ×2156 internally). SpawnOutcome/ProgramHandleInner moved here too but
// were purged in arc 278's vacate-spawn-outcome strike (a locus has no return value).
use crate::types::Nature;
pub use crate::value::{
    AggregateValue, Clause, ClauseAttempt, ClauseFailureReason, ClauseSet, EnumValue, HolonForm,
    Value,
};

// Stone 251.2c — Function + Environment cluster moved to src/value/environment.rs.
// Stone 255.1a — FunctionBody added.
pub use crate::value::{BoundEntry, EnvBuilder, Environment, Function, FunctionBody, ReteContract};

use crate::value::EncodingCtx;

// Stone 251.2d — SymbolTable lifted to src/value/symbol_table.rs.
pub use crate::value::SymbolTable;

// Stone 251.2b — observe types (Provenance/TrackedValue/ValueSnapshot) moved to
// src/value/observe.rs. Re-exported here for zero-churn.
// Re-export principle applied per type (consumer counts from grep -rn 'runtime::{.*Type'):
// ValueSnapshot(74)/TrackedValue(10)/Provenance(6) → all > threshold handled via re-export
// (TrackedValue/Provenance repointed at the few ≤15 external test sites that were already
//  updated; the larger internal src/ cluster keeps the re-export path).
pub use crate::value::{Provenance, TrackedValue, ValueSnapshot};

// Stone 251.2b — signal types (EvalSignal/EvalBreak/RuntimeError/RuntimeErrorKind) moved to
// src/value/signal.rs. Re-exported here for zero-churn.
// RuntimeError(25+)/RuntimeErrorKind(22)/EvalBreak(3)/EvalSignal(3) → RE-EXPORT.
pub use crate::value::{EvalBreak, EvalSignal, RuntimeError, RuntimeErrorKind};

/// Arc 170 #13 — which of the three `register_defclause` effects a given call
/// lands. See [`register_defclause`]'s doc comment for the full shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseRegPhase {
    /// Pre-resolve: register the stub `Function` (0-arg, nil body) so the
    /// checker sees the defclause name as callable and does not report
    /// `UnknownCallee`. Does NOT touch `runtime_def_values`.
    Stub,
    /// Post-resolve / freeze (or eval-time, e.g. a REPL-typed defclause):
    /// register the real `ClauseSet` into `runtime_def_values`, removing any
    /// stub `Function` registered by a prior `Stub` call.
    Runtime,
}

// Arc 109 Stone 2 — `register_defclause` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_defines` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_extend_type_surface_impls` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_stdlib_runtime_defs` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `preregister_stdlib_defclause_stub` moved to `src/declare/preregister.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `AXIS_DECLARATION_KEYS` + `meta_has_doc_axis_key` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `record_binding_metadata` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_stdlib_defines` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `parametric_decl_type` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `restrictions_to_binding_metadata_ast` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_struct_methods` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_aggregate_methods` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_enum_methods` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_newtype_methods` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 293 inheritance annihilation: collect_all_record_fields DELETED.
// All types are flat (nature + own fields); inherited fields were always 0 after
// the parse-time nature-root guard. `register_aggregate_methods` uses `agg.fields` directly.

// Arc 293.R2.2 — register_record_methods DELETED.
// Accessor codegen for Record + HolonRecord is now in register_aggregate_methods
// (unified with Struct). The macro's accessor emission was removed from wat/Record.wat.
// freeze/env.rs calls register_aggregate_methods instead.

// Arc 109 Stone 2 — `register_type_predicates` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `parse_declare_acronyms_form` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `preregister_acronyms` moved to `src/declare/preregister.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_runtime_defs` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `RUNTIME_DECLARATION_HEADS` + `DECLARATION_HEADS` + `is_runtime_declaration_head` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `is_declaration_head` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `is_declaration_form` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_runtime_defs_form` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Stone 241.16 — `is_define_form` DELETED. Stone 241.11 already removed the caller;
// the function itself is now dead code. `:wat::core::define` is HARD CUT (total).

// Arc 109 Stone 2 — `parse_defalias_form` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `register_defalias` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `build_delegate_body` moved to `src/declare/register.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `is_struct_form` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `is_enum_form` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `preregister_struct_accessors_from_form` moved to `src/declare/preregister.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `preregister_enum_constructors_from_form` moved to `src/declare/preregister.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `try_parse_metadata_map` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `FnShapeMetadata` + `ParsedFnShapeDef` + `try_parse_fn_shape_def` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `try_parse_variadic_def_fn_form` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `try_parse_user_variadic_def_fn_form` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Stone 241.14 — `try_parse_fn_shape_def_restricted` DELETED.
// All callers (register_defines, preregister_fn_defs_in_do,
// preregister_fn_defs_in_let) had their def-restricted arms deleted.
// def-restricted is HARD CUT at check.rs.

// Arc 109 Stone 2 — `preregister_fn_defs_in_do` moved to `src/declare/preregister.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `preregister_fn_defs_in_let` moved to `src/declare/preregister.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Stone 241.16 — `ParsedDefineSignature`, `parse_define_form`, `parse_define_signature`,
// and `parse_param_pair` DELETED. `:wat::core::define` eval-time residue is now complete.
// These functions processed the old Scheme-style `(:wat::core::define sig body)` form.
// The canonical replacement is `:wat::core::defn` (Clojure-aligned). HARD CUT is TOTAL.
// See: Stone 241.11 (startup-check HARD CUT); Stone 241.16 (eval-time residue completion).

// Arc 109 Stone 2 — `parse_type_keyword` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `resolve_type_slot_args` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `parse_type_slot` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `angle_type_head_in_name` moved to `src/declare/typevar.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `angle_minted_name_reason` moved to `src/declare/typevar.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `is_type_arg_shaped` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `is_type_var_path` moved to `src/declare/parse.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `collect_free_type_vars` moved to `src/declare/typevar.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `collect_free_type_vars_in` moved to `src/declare/typevar.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 2 — `walk_free_type_vars` moved to `src/declare/typevar.rs` (the declare home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Evaluate `ast` in **tail position** with respect to the innermost
/// enclosing [`apply_function`]. When a user-defined function call
/// appears here, emit [`EvalSignal::TailCall`] instead of recursing
/// through `apply_function`; the enclosing loop catches the signal,
/// reassigns `cur_func`/`cur_args`, and re-iterates without stack
/// growth. Everything else delegates to [`eval`].
///
/// The tail-carrying forms (`if`, `match`, `let`) have sibling
/// tail-aware helpers (`eval_if_tail`, `eval_match_tail`,
/// `eval_let_tail`) that reuse the same validation as their non-tail
/// twins but dispatch the body through `eval_tail` rather than `eval`.
/// Arc 154 retired `let*`; `let` is the single-letform vocabulary (Clojure-faithful).
///
/// Three tail-call shapes are detected (Stage 2 covers all three):
///
/// 1. **Keyword head** resolving in `sym.functions` — a
///    `define`-registered named function (Stage 1's original scope).
/// 2. **Bare-symbol head** resolving to a fn value in `env` —
///    fn-valued params and let-bound fns. Enables
///    Y-combinator-lite self-recursion (fn passed as argument)
///    without a letrec mechanism.
/// 3. **Inline-fn-literal head** `((fn ...) args)` — the
///    head evaluates to a fn value directly.
///
/// Non-fn, non-registered, non-form heads delegate to [`eval`]
/// so error handling (NotCallable, UnboundSymbol, primitive
/// dispatch, `Some`/`Ok`/`Err` constructors) is unchanged.
pub(crate) fn eval_tail(ast: &WatAST, env: &Environment, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let (items, list_span) = match ast {
        WatAST::List(items, span) if !items.is_empty() => (items, span.clone()),
        _ => return eval_inner(ast, env, sym).map(|tv| tv.value_owned()),
    };
    let args = &items[1..];
    match &items[0] {
        WatAST::Keyword(k, _) => {
            let mut head = k.as_str();
            // Arc 278 #56 (S5) phase 1b — mirrors `dispatch_keyword_head_value`'s rete gate
            // (below, at the `RETE_PREFIX` check): a rete `Form` in TAIL POSITION must reach the
            // SAME `*_tail` routine its core twin does, or it silently trades TCO for a native
            // stack overflow (SIGSEGV at depth, proven by
            // `tests/rete/probe_arc278_55_slice_one_vocabulary.rs`'s TCO gate — NOT a located
            // error, because this function has no chance to raise one: the overflow happens in
            // the recursive Rust call stack itself). Resolving to `core_name` BEFORE the match
            // below — rather than adding a parallel rete-keyed arm per form — means a future Form
            // row that mirrors one of these four heads is covered with zero further edit here;
            // only a mirror of a head NOT in this match (e.g. `and`/`or`) falls through to the
            // catch-all `eval_inner`, unaffected, exactly as before this gate existed.
            if head.starts_with(crate::rete::vocabulary::RETE_PREFIX) {
                if let Some(op) = crate::rete::vocabulary::rete_op_for(head) {
                    if op.class == crate::rete::vocabulary::OpClass::Form {
                        head = op.core_name;
                    }
                }
            }
            // Arc 255 Stone the-tail-door — the registry's tail door. MUST sit here: after the
            // rete `Form` re-mapping above (so a `:wat::rete::core::*` spelling has already been
            // rewritten to its `:wat::core::*` `head` before this consults the registry — placed
            // any higher and every rete `Form` spelling of a registered tail form would silently
            // lose TCO, with every test staying green, DESIGN's "TWO placement facts") and before
            // `match head {` (so a registered tail impl runs BEFORE the literal arms it replaces,
            // not as a first arm inside the match — the guard-hoist contract's shape). `if`/`let`/
            // `match` are the only three registered tail impls as of this stone; every other head
            // (`do`/`and`/`or`/`ann-form`/`:wat::rete::insert`/a user fn/a defclause) has no
            // registry row and falls through unchanged to the match below.
            if let Some(entry) = crate::intrinsic::registry().lookup_entry(head) {
                if let Some(tail) = entry.tail_handler {
                    return tail(args, &list_span, env, sym);
                }
            }
            match head {
                // Arc 255 `DESIGN-STONE-the-tail-door.md` — this arm RETIRED; `:wat::core::if`
                // carries a registered `role = tail` handler now, so the registry-first tail
                // door above (`crate::intrinsic::registry().lookup_entry(head).tail_handler`)
                // already dispatches it to `eval_if_tail` (unchanged) before this match is ever
                // reached.
                // Arc 255.1c-kernel-remainder (home #8) — the `:wat::kernel::serve-dispatch-op`
                // tail-position special-case that used to live HERE moved to the intrinsic
                // registry (`src/intrinsic/kernel/serve.rs`); the fallthrough `_ =>
                // eval_inner(ast, env, sym)` arm at the bottom of this match now reaches it via
                // registry lookup, which calls `crate::runtime::eval_kernel_serve_dispatch_op_tail`
                // — the SAME delegate, still evaluating `body` via `eval_tail` internally, so the
                // `serve` self-recursion trampoline is preserved. See that module's doc for the
                // full derivation (verified safe against `apply_function`'s trampoline loop).
                // Arc 255 `DESIGN-STONE-the-tail-door.md` — this arm RETIRED; `:wat::core::let`
                // carries a registered `role = tail` handler now, so the registry-first tail
                // door above already dispatches it to `eval_let_tail` (unchanged, including the
                // `.map(|tv| tv.value_owned())` this arm used to do here — now performed inside
                // the macro-generated shim instead) before this match is ever reached.
                // Arc 255 `DESIGN-STONE-the-tail-door.md` — this arm RETIRED; `:wat::core::match`
                // carries a registered `role = tail` handler now (`eval_match_tail`, newly
                // annotated by this stone — it had no `#[wat_special_form_impl]` of any role
                // before), so the registry-first tail door above already dispatches it to
                // `eval_match_tail` (unchanged) before this match is ever reached.
                // Arc 255 Stone 1a-zeta (`DESIGN-STONE-1a-zeta-the-last-three-of-the-special-form-
                // table.md`) — this arm RETIRED; `:wat::core::do` carries a registered
                // `role = tail` handler now (`eval_do_tail`, annotated in place, same file), so
                // the registry-first tail door above already dispatches it to `eval_do_tail`
                // (unchanged) before this match is ever reached.
                // Arc 278 #59 — `and`/`or`/`ann-form` mirror the `if`/`match`/`let`/`do` pattern
                // above: each is a legitimate tail context (see eval_and_tail/eval_or_tail/
                // eval_ann_form_tail's docs for what each one does and, for and/or, the RULED
                // trade this makes).
                // Arc 255 Stone 1a-i — the `and`/`or` arms that used to sit here are RETIRED;
                // `:wat::core::and`/`:wat::core::or` carry registered `role = tail` handlers now
                // (`eval_and_tail`/`eval_or_tail` themselves — STOP-1's stacked-attribute pair, the
                // same fns this arm used to name), so the registry-first tail door above already
                // dispatches to them before this match is ever reached.
                // Arc 255 Stone 1a-zeta — the `ann-form` arm that used to sit here is ALSO
                // RETIRED (the comment above this one used to note it "keeps its own arm below"
                // — no longer true); `:wat::core::ann-form` carries a registered `role = tail`
                // handler now (`eval_ann_form_tail`, annotated in place, same file), so the
                // registry-first tail door above already dispatches it to `eval_ann_form_tail`
                // (unchanged) before this match is ever reached.
                // DESIGN-STONE-insert-prime-split — foldl's inner is tail; without this
                // arm the defclause TCO path apply_function's the wat 2-ary wrapper (~1.2 µs).
                ":wat::rete::insert" => {
                    crate::rete::kernel::eval_insert_public(args, &list_span, env, sym)
                }
                // A user-defined function call in tail position — signal.
                // Head resolves in sym.functions; anything else (kernel/
                // algebra/config primitive, :rust:: shim) runs through
                // regular eval.
                other if sym.has_function(other) => {
                    let func = sym.get(other).expect("contains_key above").clone();
                    emit_tail_call(func, args, env, sym, list_span)
                }
                // Clause-TCO stone — a `defclause` head in TAIL position.
                //
                // The arm above misses it: a defclause registers a `ClauseSet` as a def-value,
                // not an entry in `sym.functions`, so before this arm every clause head fell to
                // `_ => eval_inner(...)` and recursed on the REAL STACK. Measured: a 200,000-deep
                // tail-recursive `defclause` SIGSEGV'd while its byte-identical `defn` twin
                // completed. Every `defclause` in wat was affected — arithmetic, `into`, `conj`,
                // every user-written multi-arity verb.
                //
                // Selection routes through `select_defclause_clause` — the SAME door the ordinary
                // path uses. Duplicating that loop would be the "N ways to do a thing" defect.
                other
                    if sym
                        .def_value(other)
                        .is_some_and(|v| matches!(v, Value::wat__core__clauses(_))) =>
                {
                    let Some(Value::wat__core__clauses(cs)) = sym.def_value(other) else {
                        unreachable!("the guard above matched wat__core__clauses")
                    };
                    let cs = cs.clone();
                    let vals = args
                        .iter()
                        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
                        .collect::<Result<Vec<_>, _>>()?;
                    let (idx, _scope) = crate::function::select_defclause_clause(&cs, &vals, &list_span, sym)?;
                    let clause = &cs.clauses[idx];
                    // ⚠ `:ensure` is a POST-condition — it runs AFTER the body, and a tail call
                    // abandons the frame it would return into. An ensure-bearing clause therefore
                    // takes the ordinary path, DELIBERATELY. Tail-calling it would silently delete
                    // a post-condition the author wrote and the checker promised.
                    if clause.ensure_fn.is_none() {
                        if let Some(f) = &clause.func {
                            return Err(EvalBreak::Signal(EvalSignal::TailCall {
                                func: f.clone(),
                                args: vals,
                                call_span: list_span.clone(),
                            }));
                        }
                    }
                    // Ordinary path. It re-selects; that cost is paid ONLY by ensure-bearing
                    // clauses (and by the checker-built clauses that carry no compiled Function),
                    // and it keeps `:ensure` handling in exactly one place.
                    crate::function::eval_call_to_defclause_with_vals(cs, vals, &list_span, sym)
                }
                _ => eval_inner(ast, env, sym).map(|tv| tv.value_owned()),
            }
        }
        // Bare-symbol head: a fn-valued local binding. `Some`,
        // `Ok`, `Err` are constructor symbols that are NEVER bound in
        // env, so `env.lookup` returns None for them and we delegate
        // to eval (which special-cases the three constructors).
        WatAST::Symbol(ident, span) => {
            if let Some(tv) = env.lookup(crate::scope::env_key(ident).as_ref(), span) {
                if let Value::wat__core__fn(f) = tv.value() {
                    emit_tail_call(f.clone(), args, env, sym, list_span)
                } else {
                    // Already-fetched non-fn value: apply directly via the same
                    // path eval_list uses — avoids a second key derivation + lookup.
                    apply_tracked_callee(tv, args, env, sym).map(|tv| tv.value_owned())
                }
            } else {
                eval_inner(ast, env, sym).map(|tv| tv.value_owned())
            }
        }
        // Inline fn-literal head `((fn ...) args)`. Evaluate
        // the head non-tail; if the value is a fn, signal tail
        // call; otherwise delegate to `apply_value` with the
        // already-evaluated callee so we don't re-evaluate.
        WatAST::List(_, _) => {
            let callee = eval_inner(&items[0], env, sym)?.value_owned();
            match callee {
                Value::wat__core__fn(f) => emit_tail_call(f, args, env, sym, list_span),
                other => apply_value(&other, args, env, sym),
            }
        }
        // Literal head (int/float/bool/string) — not callable; let
        // eval raise the right error.
        _ => eval_inner(ast, env, sym).map(|tv| tv.value_owned()),
    }
}

/// Evaluate `raw_args` non-tail and emit a [`EvalSignal::TailCall`]
/// carrying `func`. Shared helper for all three tail-call shapes
/// (named define, bare-symbol fn, inline-fn literal). Arity
/// is enforced by [`apply_function`]'s trampoline loop on its next
/// iteration. Arc 016 slice 2: carries `call_span` so the trampoline
/// can refresh the call-stack frame with the new invocation's
/// location.
fn emit_tail_call(
    func: Arc<Function>,
    raw_args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    call_span: Span,
) -> Result<Value, EvalBreak> {
    let vals = raw_args
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    Err(EvalBreak::Signal(EvalSignal::TailCall {
        func,
        args: vals,
        call_span,
    }))
}

/// Tail-position twin of [`eval_if`]. Same validation; the selected
/// branch body is evaluated via [`eval_tail`] instead of [`eval`].
#[wat_special_form_impl(":wat::core::if", role = tail)]
fn eval_if_tail(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() == 3 {
        // Arc 258.1 — bare `(if cond then else)`.
        let cond_val = eval_inner(&args[0], env, sym)?.value_owned();
        return match cond_val {
            Value::bool(true) => eval_tail(&args[1], env, sym),
            Value::bool(false) => eval_tail(&args[2], env, sym),
            other => Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::BadCondition {
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into()),
        };
    }
    // Arc 258.4 — the `-> :T` ascription is retired; a stray `->` (the old 5-arg form)
    // is the retired shape; refuse it with a migration hint.
    if args.len() >= 2 && matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::if".into(),
            reason: "`:wat::core::if` no longer takes `-> :T`; the result type is inferred by unifying the branches. Write (:wat::core::if cond then else)".into()
        }).into());
    }
    Err(RuntimeError::new(
        list_span.clone(),
        RuntimeErrorKind::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond then else) — 3 args; got {}",
                args.len()
            ),
        },
    )
    .into())
}

/// Tail-position twin of [`eval_let`]. Bindings accumulate
/// sequentially (each RHS sees prior bindings); the LAST body form
/// runs through [`eval_tail`] so a tail-call inside it propagates.
/// Arc 154 — sequential semantics moved under the `:wat::core::let`
/// keyword (single-letform vocabulary; Clojure-faithful). Pre-arc-154
/// this body lived under `eval_let_star_tail` and dispatched on
/// `:wat::core::let*` (historical; that dispatch arm is gone).
/// Arc 168 — flat-vector bindings + implicit-do body (mirrors [`eval_let`]).
/// Arc 233 Stone 233.2.e: flipped from Result<Value> → Result<TrackedValue>
/// to close the 233.2.k honest delta (eval_let_tail was the remaining path
/// that stripped provenance from tail-call let bodies). Callers that need
/// bare Value use .value_owned().
///
/// Mirrors the 233.2.j eval_let pattern: the tail-call path now preserves
/// provenance through the trampoline boundary.
#[wat_special_form_impl(":wat::core::let", role = tail)]
fn eval_let_tail(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "expected (:wat::core::let [name expr ...] body ...); got {} args",
                    args.len()
                ),
            },
        )
        .into());
    }
    let bindings_form = &args[0];

    let mut scope = env.clone();
    match bindings_form {
        WatAST::Vector(items, _) => {
            if items.len() % 2 != 0 {
                return Err(RuntimeError::new(
                    bindings_form.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::let".into(),
                        reason: format!(
                            "let bindings vector must have an even number of elements; got {}",
                            items.len()
                        ),
                    },
                )
                .into());
            }
            let mut i = 0;
            while i < items.len() {
                let binder = &items[i];
                let rhs = &items[i + 1];
                let binding = parse_let_binding(binder, rhs)?;
                scope = bind_let_binding(binding, &scope, sym)?;
                i += 2;
            }
        }
        _ => {
            return Err(RuntimeError::new(
                bindings_form.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::let".into(),
                    reason: "let bindings must be a flat vector `[name expr ...]`".into(),
                },
            )
            .into());
        }
    };

    let body = &args[1..];
    if body.is_empty() {
        return Ok(TrackedValue::from(Value::Unit));
    }
    let last_idx = body.len() - 1;
    for form in &body[..last_idx] {
        let _ = eval_inner(form, &scope, sym)?;
    }
    // tail-call path: eval_tail returns Result<Value>; wrap with Unknown provenance.
    // The trampoline path uses Value directly; TrackedValue wraps at this boundary.
    eval_tail(&body[last_idx], &scope, sym).map(TrackedValue::from)
}

/// Tail-position twin of [`eval_do`]. Non-final forms are evaluated
/// for their side effects (results discarded); the FINAL form is
/// evaluated through [`eval_tail`] so a tail-call inside it propagates
/// through the trampoline. Arc 136 slice 1a.
///
/// Arc 255 Stone 1a-zeta — the `role = tail` pointer for `:wat::core::do`. Annotated IN PLACE
/// (signature already fits the canonical `TailHandler` shape), mirroring `if`/`let`/`match`'s
/// own tail-door annotations a few hundred lines above.
#[wat_special_form_impl(":wat::core::do", role = tail)]
fn eval_do_tail(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::do".into(),
                reason: "do form requires at least one form; got zero".into(),
            },
        )
        .into());
    }
    let last_idx = args.len() - 1;
    for arg in &args[..last_idx] {
        let _ = eval_inner(arg, env, sym)?;
    }
    eval_tail(&args[last_idx], env, sym)
}

/// Tail-position twin of [`eval_match`]. The matched arm's body is
/// evaluated via [`eval_tail`] — a tail-call inside an arm body
/// propagates through to `apply_function`'s trampoline.
///
/// Arc 255 Stone the-tail-door — this fn had NO `#[wat_special_form_impl]` annotation of any
/// role before this stone (its `role = check`/`role = eval` siblings live on `infer_match`/
/// `eval_match`; only the tail twin was never wired). Added here so `:wat::core::match` gets a
/// THIRD registered row and `eval_tail`'s literal arm for it can be deleted without losing TCO —
/// see this stone's report for the surprise this was.
#[wat_special_form_impl(":wat::core::match", role = tail)]
fn eval_match_tail(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Arc 258.5 — the `-> :T` ascription is retired (the result type is
    // inferred from the arm bodies). A stray `->` in ascription position
    // is the old form; refuse it with a migration hint.
    if args.len() >= 2 && matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::match".into(),
            reason: "`:wat::core::match` no longer takes `-> :T`; the result type is inferred by unifying the arm bodies (like `if`). Write (:wat::core::match scrut (pat body) ...)".into()
        }).into());
    }
    if args.len() < 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::match".into(),
            reason: format!(
                "expected (:wat::core::match scrut arm1 arm2 ...) — at least a scrutinee and one arm; got {}",
                args.len()
            )
        }).into());
    }
    let scrutinee = eval_inner(&args[0], env, sym)?.value_owned();
    for arm in &args[1..] {
        let arm_items = match arm {
            WatAST::List(items, _) => items,
            other => {
                return Err(RuntimeError::new(
                    other.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "each arm must be a list `(pattern body)`, got {}",
                            other.variant_name()
                        ),
                    },
                )
                .into());
            }
        };
        if arm_items.len() != 2 {
            return Err(RuntimeError::new(
                arm.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "each arm must have exactly (pattern body); got {} elements",
                        arm_items.len()
                    ),
                },
            )
            .into());
        }
        let pattern = &arm_items[0];
        let body = &arm_items[1];
        if let Some(arm_env) = try_match_pattern(pattern, &scrutinee, env, sym)? {
            return eval_tail(body, &arm_env, sym);
        }
    }
    Err(RuntimeError::new(
        args[0].span().clone(),
        RuntimeErrorKind::PatternMatchFailed {
            value_type: scrutinee.type_name(),
        },
    )
    .into())
}
#[wat_special_form_impl(":wat::core::and", role = eval)]
fn eval_and(
    args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — located elsewhere: non-bool operand errors locate at `arg.span()`, more precise than the coarse list span

    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Short-circuit: false wins.
    for arg in args {
        let arg_span = arg.span().clone();
        match eval_inner(arg, env, sym)?.value_owned() {
            Value::bool(false) => return Ok(Value::bool(false)),
            Value::bool(true) => continue,
            other => {
                return Err(RuntimeError::new(
                    arg_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::and".into(),
                        expected: "bool",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into())
            }
        }
    }
    Ok(Value::bool(true))
}

#[wat_special_form_impl(":wat::core::or", role = eval)]
fn eval_or(
    args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — located elsewhere: non-bool operand errors locate at `arg.span()`, more precise than the coarse list span

    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    for arg in args {
        let arg_span = arg.span().clone();
        match eval_inner(arg, env, sym)?.value_owned() {
            Value::bool(true) => return Ok(Value::bool(true)),
            Value::bool(false) => continue,
            other => {
                return Err(RuntimeError::new(
                    arg_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::or".into(),
                        expected: "bool",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into())
            }
        }
    }
    Ok(Value::bool(false))
}


/// Mirrors [`eval_if_tail`]'s shape: every operand but the LAST keeps the ordinary strict,
/// checked evaluation (short-circuiting `false`); the last operand is handed to [`eval_tail`] so
/// a self- or mutually-recursive tail call underneath it reuses the native stack frame instead of
/// growing it.
///
/// **Arc 278 #59 — the RULED weakening (builder, 2026-08-02), documented rather than hidden per
/// this arc's law that nothing weakens quietly.** The standalone `eval_and` this fn was once the
/// tail-position twin of (arc 255 Stone 1a-i deleted it — see this fn's own STOP-1 note below)
/// type-checked EVERY operand at runtime and raised a located `TypeMismatch` on a non-bool.
/// Tail-calling the last operand means its
/// value is never inspected here, so `eval_and_tail` cannot raise that check on the last operand —
/// there is no shape that keeps both the check and the TCO, and the TCO was chosen. This is safe
/// in all STATICALLY CHECKED source: `infer_boolean_shortcircuit` (check.rs) already forces every
/// `and` operand, including the last, to `:bool` before eval ever runs, so the runtime check this
/// skips is belt-and-braces duplicating a checker guarantee there. The difference is observable
/// only on a path that reaches the runtime WITHOUT going through that checker — `:wat::eval-ast!`
/// evaluates a `WatAST` value at runtime with no type-check pass ("trust-the-caller"), and a
/// `:wat::core::fn` literal built inside a `quote` is never visited by the checker at all (only
/// its own call site's result type is checked, not its body). Pinned by
/// `and_or_tail_skip_the_last_operand_check_on_the_unchecked_eval_ast_path` in
/// `tests/rete/probe_arc278_59_tco_and_or_ann_form.rs`.
///
/// Arc 255 Stone 1a-i, STOP-1 — this fn carries BOTH `role = eval` and `role = tail` for
/// `:wat::core::and`: its own signature is `TailHandler`-shaped exactly, so the registry's eval
/// door and tail door both point here now. There is no longer a separate `eval_and` handler for
/// the eval door — its literal arm in `dispatch_keyword_head_value` and the standalone `eval_and`
/// fn it called are both retired below, next to `eval_or`'s identical retirement.
#[wat_special_form_impl(":wat::core::and", role = tail)]
fn eval_and_tail(
    args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — located elsewhere: non-bool operand errors locate at `arg.span()`, more precise than the coarse list span
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Ok(Value::bool(true));
    }
    let last = args.len() - 1;
    for arg in &args[..last] {
        let arg_span = arg.span().clone();
        match eval_inner(arg, env, sym)?.value_owned() {
            Value::bool(false) => return Ok(Value::bool(false)),
            Value::bool(true) => continue,
            other => {
                return Err(RuntimeError::new(
                    arg_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::and".into(),
                        expected: "bool",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into())
            }
        }
    }
    eval_tail(&args[last], env, sym)
}

/// Sibling of [`eval_and_tail`] — same shape (short-circuit on every operand but the last,
/// tail-call the last), same RULED weakening (last operand's runtime bool check is traded for
/// TCO; see `eval_and_tail`'s doc for the full rationale and the pinning test,
/// `and_or_tail_skip_the_last_operand_check_on_the_unchecked_eval_ast_path`).
///
/// Arc 255 Stone 1a-i, STOP-1 — carries BOTH `role = eval` and `role = tail` for
/// `:wat::core::or`, mirroring `eval_and_tail`'s own stacking exactly (see its doc). The
/// standalone `eval_or` fn and its `dispatch_keyword_head_value` arm are retired below.
#[wat_special_form_impl(":wat::core::or", role = tail)]
fn eval_or_tail(
    args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — located elsewhere: non-bool operand errors locate at `arg.span()`, more precise than the coarse list span
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Ok(Value::bool(false));
    }
    let last = args.len() - 1;
    for arg in &args[..last] {
        let arg_span = arg.span().clone();
        match eval_inner(arg, env, sym)?.value_owned() {
            Value::bool(true) => return Ok(Value::bool(true)),
            Value::bool(false) => continue,
            other => {
                return Err(RuntimeError::new(
                    arg_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::or".into(),
                        expected: "bool",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into())
            }
        }
    }
    eval_tail(&args[last], env, sym)
}

/// Tail-position twin of [`eval_ann_form`]. A type ascription is a pure, checked, type-erased
/// pass-through — the wrapped expression's value is returned untouched — so TCO here is
/// observationally free. Unlike `eval_and_tail`/`eval_or_tail`, nothing is skipped or weakened:
/// the arity guard is unconditional (mirrors `eval_ann_form`) and the wrapped expression is handed
/// to `eval_tail` exactly as `eval_ann_form` hands it to `eval_inner`.
///
/// Arc 255 Stone 1a-zeta — the `role = tail` pointer for `:wat::core::ann-form`. Annotated IN
/// PLACE (signature already fits the canonical `TailHandler` shape), mirroring `if`/`let`/
/// `match`'s own tail-door annotations above.
#[wat_special_form_impl(":wat::core::ann-form", role = tail)]
fn eval_ann_form_tail(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::ann-form".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    eval_tail(&args[0], env, sym)
}

/// Evaluate a single form in the given scope. Internal implementation;
/// returns TrackedValue (Arc 233 Stone 233.2.j: cascaded from eval boundary).
/// Public boundary is `eval` (direct passthrough post-233.2.j).
pub(crate) fn eval_inner(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    match ast {
        // Arc 233 Stone 233.2.e: literal arms carry Provenance::Literal{span} so
        // errors on literal values name their source position.
        WatAST::IntLit(n, span) => Ok(TrackedValue::new(
            Value::i64(*n),
            Provenance::Literal { span: span.clone() },
        )),
        WatAST::FloatLit(x, span) => Ok(TrackedValue::new(
            Value::f64(*x),
            Provenance::Literal { span: span.clone() },
        )),
        // Arc 300 stone B — rational literal, representation only.
        WatAST::RationalLit(r, span) => Ok(TrackedValue::new(
            Value::wat__core__Rational(Box::new(r.clone())),
            Provenance::Literal { span: span.clone() },
        )),
        // Arc 300 stone C1 — bigint literal, full arithmetic type (mirrors
        // Rational immediately above, one type over).
        WatAST::BigIntLit(n, span) => Ok(TrackedValue::new(
            Value::wat__core__BigInt(Box::new(n.clone())),
            Provenance::Literal { span: span.clone() },
        )),
        // Arc 300 stone D — char literal, representation only (mirrors
        // BigInt/Rational immediately above, one type over). Was a
        // desugared `(:wat::core::char/of "c")` call before this stone.
        WatAST::CharLit(c, span) => Ok(TrackedValue::new(
            Value::wat__core__Char(*c),
            Provenance::Literal { span: span.clone() },
        )),
        WatAST::BoolLit(b, span) => Ok(TrackedValue::new(
            Value::bool(*b),
            Provenance::Literal { span: span.clone() },
        )),
        WatAST::StringLit(s, span) => Ok(TrackedValue::new(
            Value::String(Arc::new(s.clone())),
            Provenance::Literal { span: span.clone() },
        )),
        // Arc 244 — NilLit is the canonical nil VALUE literal; evals to Value::Unit.
        WatAST::NilLit(span) => Ok(TrackedValue::new(
            Value::Unit,
            Provenance::Literal { span: span.clone() },
        )),
        // Arc 215 stone 2 — `[...]` vector literals at expression position.
        // Check.rs already type-checked these items via infer_list_constructor
        // (T inferred from first element; all elements unified). At runtime,
        // each item is evaluated and collected into Value::Vec.
        //
        // HISTORICAL NOTE: Arc 167 slice 1 rejected these with "vector
        // literals at value position are not supported." Arc 215 stone 2
        // retires that restriction. The `WatAST::Vector` AST node that the
        // parser produces for `[...]` is now also the runtime-evaluated form
        // for expression-position vector literals.
        //
        // Arc 233 Stone 233.2.e: vector literal carries Provenance::Literal{span}.
        WatAST::Vector(items, span) => {
            let elems = items
                .iter()
                .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TrackedValue::new(
                Value::Vec(Arc::new(elems)),
                Provenance::Literal { span: span.clone() },
            ))
        }
        // Arc 257 slice 1 — first-class map literal `{k0 v0 k1 v1 …}`.
        // Reuses `eval_hashmap_ctor`'s inner loop (guard-and-insert) but
        // skips the `:K :V` type-keyword sentinel check — a literal carries
        // no explicit type sentinels.
        WatAST::Map(pairs, span) => {
            #[allow(clippy::mutable_key_type)]
            let mut map: std::collections::HashMap<Value, Value> =
                std::collections::HashMap::with_capacity(pairs.len());
            for (k_node, v_node) in pairs {
                let k = eval_inner(k_node, env, sym)?.value_owned();
                let v = eval_inner(v_node, env, sym)?.value_owned();
                if !value_is_key_hashable(&k) {
                    return Err(RuntimeError::new(k_node.span().clone(), RuntimeErrorKind::TypeMismatch {
                        op: "{…} map literal".into(),
                        expected: "hashable key (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                        got: Box::new(ValueSnapshot::of(&k))
                    }).into());
                }
                map.insert(k, v);
            }
            Ok(TrackedValue::new(
                Value::wat__std__HashMap(Arc::new(map)),
                Provenance::Literal { span: span.clone() },
            ))
        }
        // Arc 257 slice 1 — first-class set literal `#{x y z …}`.
        // Reuses `eval_hashset_ctor`'s inner loop (guard-and-insert) but
        // skips the `:T` type-keyword sentinel check.
        WatAST::Set(items, span) => {
            #[allow(clippy::mutable_key_type)]
            let mut set: std::collections::HashSet<Value> =
                std::collections::HashSet::with_capacity(items.len());
            for item in items {
                let v = eval_inner(item, env, sym)?.value_owned();
                if !value_is_set_hashable(&v) {
                    return Err(RuntimeError::new(item.span().clone(), RuntimeErrorKind::TypeMismatch {
                        op: "#{…} set literal".into(),
                        expected: "hashable value (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                        got: Box::new(ValueSnapshot::of(&v))
                    }).into());
                }
                set.insert(v);
            }
            Ok(TrackedValue::new(
                Value::wat__std__HashSet(Arc::new(set)),
                Provenance::Literal { span: span.clone() },
            ))
        }
        WatAST::Keyword(k, span) => {
            // Arc 153 slice 1a — `:wat::core::nil` at value
            // position evaluates to `Value::Unit` (the nil
            // singleton). The infer hook in check.rs types this
            // keyword as `:wat::core::nil` (singleton type);
            // evaluation here returns the singleton value.
            // Special-case is narrow: only the exact FQDN string.
            //
            // Arc 233 Stone 233.2.e: nil/None keyword special-cases carry
            // Provenance::Literal{span} (they appear as keyword literals in source).
            if k == ":wat::core::nil" {
                return Ok(TrackedValue::new(
                    Value::Unit,
                    Provenance::Literal { span: span.clone() },
                ));
            }
            // `:None` is the nullary constructor of the built-in
            // `(Option :- [T])` enum (058-030). Special-cased here so users
            // can write `:None` in expression position to produce
            // `Value::Option(None)` without requiring a keyword-path
            // call form.
            //
            // Arc 109 slice 1h — `:wat::core::None` is the canonical
            // FQDN form; `:None` is a bare-grammar exception that
            // retires (poisoned at type-check time, runtime keeps
            // working).
            if k == ":None" || k == ":wat::core::None" {
                return Ok(TrackedValue::new(
                    Value::Option(Arc::new(None)),
                    Provenance::Literal { span: span.clone() },
                ));
            }
            // Arc 048 — user-enum unit variants. Pre-built EnumValues
            // sit in `sym.unit_variants` keyed by their full keyword
            // path (`:enum::Variant`). When the keyword evaluates,
            // return the variant value directly (no function call).
            // Mirrors the `:None` shortcut for Option.
            if let Some(ev) = sym.unit_variant(k) {
                return Ok(TrackedValue::from(Value::Enum(Arc::new(ev.clone()))));
            }
            // Arc 157 — top-level `def` bindings. A keyword that was
            // bound via `(:wat::core::def :name expr)` at top-level
            // position resolves to the value computed when freeze
            // evaluated the def form (populated in `runtime_def_values`
            // by `register_runtime_defs` during `FrozenWorld::freeze`).
            // Checked AFTER unit_variants (enum constructors take
            // precedence) and BEFORE the function-value lift (so a
            // def-bound closure is returned as the stored Value rather
            // than re-lifted through `sym.get`).
            if let Some(v) = sym.def_value(k) {
                return Ok(TrackedValue::from(v.clone()));
            }
            // Arc 009 — names are values. If the keyword is a registered
            // user/stdlib define, lift it to a callable Function value.
            // Parallels `:wat::kernel::spawn`'s long-standing accept-by-
            // name convention — generalized here so every `:fn(...)`-
            // typed parameter accepts a bare keyword-path reference.
            // Primitives (kernel/algebra/config/io) stay call-only at
            // runtime; they can pass the type check but won't evaluate
            // to a Function until a caller demands that extension.
            if let Some(func) = sym.get(k) {
                return Ok(TrackedValue::from(Value::wat__core__fn(func.clone())));
            }
            Ok(TrackedValue::from(Value::wat__core__keyword(Arc::new(
                k.clone(),
            ))))
        }
        // Stone 242.2 — Doctrine 1: bare `nil` is the value form for the nil singleton.
        // The type-check arm (check.rs `is_primitive_type_keyword_in_value_position`)
        // now rejects `:wat::core::nil` as a keyword in value position; bare `nil`
        // (WatAST::Symbol) is the canonical value form. Evaluate to Value::Unit.
        WatAST::Symbol(ident, span) if ident.as_str() == "nil" => Ok(TrackedValue::new(
            Value::Unit,
            Provenance::Literal { span: span.clone() },
        )),
        WatAST::Symbol(ident, span) => env
            .lookup(crate::scope::env_key(ident).as_ref(), span)
            .ok_or_else(|| {
                RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::UnboundSymbol(ident.as_str().to_owned()),
                )
            })
            .map_err(EvalBreak::from),
        // Arc 233 Stone 233.2.j: eval_list now returns TrackedValue (producers propagate
        // TrackedValue directly; non-producer arms wrap with .into_tracked()).
        WatAST::List(items, span) => eval_list(items, span, env, sym),
    }
}

/// Public eval boundary — returns `Result<TrackedValue, EvalBreak>`.
///
/// Arc 233 Stone 233.2.j: direct passthrough to eval_inner, which now returns
/// TrackedValue directly. The pre-233.2.j unwrap-and-rewrap of Value::Tracked
/// is removed entirely — eval_inner never produces Value::Tracked variants;
/// producers construct TrackedValue::new directly.
///
/// This is the ONLY public eval boundary; external callers always receive
/// `TrackedValue` or a `RuntimeError` diagnostic. Signals (`EvalSignal`) are
/// an eval-loop internal mechanism — `apply_function` catches them before
/// unwinding past its trampoline; they must NEVER escape through this boundary
/// to external callers. If a signal escapes here, that is an interpreter bug.
///
/// See arc 233 Stone 233.2.j (substrate-errors-as-values).
/// Arc 243 Stone 243.7b: this function is a signal catch-boundary for the
/// public API surface. The signal subgraph (`eval_inner` and below) carries
/// `EvalBreak`; this boundary unwraps to `RuntimeError` for all callers
/// outside `apply_function`.
pub fn eval(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, RuntimeError> {
    match eval_inner(ast, env, sym) {
        Ok(v) => Ok(v),
        Err(EvalBreak::Diagnostic(e)) => Err(*e),
        // A signal escaping through the public eval boundary is an interpreter
        // bug — apply_function should catch signals before they reach here.
        // Convert to a diagnostic so external callers get a RuntimeError.
        Err(EvalBreak::Signal(s)) => Err(RuntimeError::new(
            ast.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: "eval".into(),
                reason: format!("internal: eval-loop signal escaped apply_function boundary: {s}"),
            },
        )),
    }
}

// Arc 233 Stone 233.2.j: eval_list now returns TrackedValue so producer provenance
// (from eval_keyword_from_string, eval_holon_from_holon, eval_edn_read) propagates
// through dispatch_keyword_head to eval_inner without losing the attached provenance.
// Non-producer arms wrap their bare Value return with .into_tracked() (Provenance::Unknown).
fn eval_list(
    items: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    // `()` evaluates to Unit. Natural reading: the empty list /
    // empty tuple IS the unit value. Lets `(if cond do-work ())`
    // cleanly express "if else unit" without awkward placeholder
    // calls. Matches the type-level `:()` keyword (unit type) at
    // the value level.
    let head = match items.first() {
        Some(h) => h,
        None => return Ok(TrackedValue::from(Value::Unit)),
    };
    let rest = &items[1..];

    // Arc 109 slice 1h's FQDN keyword-guard arms for `:wat::core::Some`/`Ok`/`Err`
    // (a `match head { WatAST::Keyword(k, _) if k == "…" => … }` block, right here)
    // are RETIRED — arc 255 Stone A-2-ii-b-1. All three are now `#[wat_intrinsic]`-
    // registered (`src/intrinsic/option.rs`, `src/intrinsic/result.rs`), and
    // `dispatch_keyword_head` below already checks the registry FIRST
    // (`crate::intrinsic::registry().lookup(head)`) before falling through to its own
    // match — so a `WatAST::Keyword` head naming any of the three now reaches the SAME
    // `eval_some_ctor`/`eval_ok_ctor`/`eval_err_ctor` bodies (unchanged; arc 109 Stone
    // the-last-two-map-items moved them to `src/option/mod.rs`/`src/result/mod.rs`)
    // through the registry instead of through this now-redundant guard.
    //
    // STONE: the bare-symbol shorthand dies — the bare-Symbol constructor exceptions
    // (`Some`/`Ok`/`Err` as a bare identifier, not a keyword) that used to sit right
    // here are DELETED, not retired-with-a-guard: the checker refuses this spelling at
    // both constructor and match-pattern sites (arc 109 slice 1h/1i's retirement,
    // finally closed on both doors), so a bare `(Some 1)`/`(Ok 1)`/`(Err e)` reaching
    // this dispatch now falls to the `WatAST::Symbol(ident, span)` arm just below —
    // an unbound-symbol lookup, the same failure any other undefined bare callable
    // gets. `eval-ast!` (an unchecked runtime path) previously reached these arms and
    // evaluated the shorthand anyway; deleting them is what makes the runtime agree
    // with the checker even off the checked corridor.
    match head {
        // dispatch_keyword_head now returns TrackedValue (propagates producer provenance).
        WatAST::Keyword(k, _) => dispatch_keyword_head(k, rest, list_span, env, sym),
        WatAST::Symbol(ident, span) => {
            // Bare symbol as head — look up a callable in the env.
            // Arc 233 Stone 233.2.k: keep TrackedValue so NotCallable errors
            // preserve producer provenance (of_tracked reads provenance intact).
            // Arc 233 Stone 233.2.e: pass span so lookup constructs SymbolBound.
            let tv = env
                .lookup(crate::scope::env_key(ident).as_ref(), span)
                .ok_or_else(|| {
                    RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::UnboundSymbol(ident.as_str().to_owned()),
                    )
                })?;
            apply_tracked_callee(tv, rest, env, sym)
        }
        WatAST::List(_, _) => {
            // Inline fn call: ((fn ...) arg1 arg2)
            let callee_tv = eval_inner(head, env, sym)?;
            apply_tracked_callee(callee_tv, rest, env, sym)
        }
        other => Err(RuntimeError::new(
            other.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: other.variant_name().into(),
                reason: "call head must be a keyword, symbol, or list".into(),
            },
        )
        .into()),
    }
}

// Arc 233 Stone 233.2.j: dispatch_keyword_head returns TrackedValue so
// producer provenance (eval_keyword_from_string, eval_holon_from_holon,
// eval_edn_read) propagates to eval_inner without Value::Tracked wrapping.
// Routes the 3 producers directly (they return TrackedValue); all other
// arms go through dispatch_keyword_head_value and wrap with .into_tracked().
fn dispatch_keyword_head(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    // Arc 255 Stone G — the registry-first door, hoisted to THIS TrackedValue-returning
    // function (not `dispatch_keyword_head_value`, whose `Result<Value, _>` signature would
    // force a discard of whatever provenance the handler stamped). `NativeHandler` now returns
    // `TrackedValue` directly, so a registered producer's `Provenance::RuntimeBuilt` survives
    // un-rewrapped here; a non-producer handler still yields `Provenance::Unknown`, exactly as
    // the shim's default arm always has (`crates/wat-macros/src/wat_intrinsic.rs`). Registered
    // wins, always — same order guarantee `dispatch_keyword_head_value`'s own registry door
    // documents (`DESIGN-STONE-255.1c-guard-hoist.md`); consulting it a second time there (for
    // callers that reach that function directly, e.g. `dispatch_rete_op`) is redundant for
    // heads that land here first but not incorrect, since a lookup is idempotent.
    if let Some(handler) = crate::intrinsic::registry().lookup(head) {
        return handler(args, list_span, env, sym);
    }
    // Arc 255 Stone the-hand-rolled-arms-retire — the registry-first door above answers
    // every row with a handler; a row that reaches THIS point and declares
    // `@Purity Unevaluated` has no handler by construction
    // (`unevaluated_purity_carries_no_route_to_evaluation`, `src/intrinsic/mod.rs`) — it is
    // consumed before evaluation (registered or spliced at freeze time) and was never meant
    // to land here at all. Keyed on `purity`, not `@Category` and not a hand list: the
    // 2026-06-24 note's refused antipattern is a `const DECLARATION_FORMS: &[&str]`, and
    // `@Category` is the wrong key too — `:wat::core::use!` is `@Category Declaration` and
    // legally evaluates to `Unit` (`use_form.rs:76-77`), so a category-keyed guard would
    // refuse a form that works today.
    if let Some(entry) = crate::intrinsic::registry().lookup_entry(head) {
        // Arc 255 Stone 2a — the alias field IS the dispatch, not documentation of one
        // (`DESIGN-STONE-2a-the-alias-field-and-why-1b-was-blocked-twice.md`'s ★★★ contract).
        // Checked FIRST, ahead of the `Unevaluated` guard just below: an alias row answers "this
        // name means that name," which is the more specific, positive fact about what this row
        // IS — the `Unevaluated` guard is a REFUSAL for rows with no route to evaluation at all,
        // and an alias row is never that (it has a route: the target's own). The two guards
        // read disjoint fields (`alias_of` vs `purity`) and neither's declared row is provable
        // as satisfying the other today, so ordering does not change behavior for any existing
        // row — this placement is a readability choice: resolve "what does this name mean" before
        // asking "does this name refuse to run at all."
        //
        // This is also the ONLY point in either dispatch door where a rete-namespaced alias's
        // head is intercepted BEFORE `dispatch_keyword_head_value`'s `RETE_PREFIX` gate — see
        // that function's own copy of this check for why the same placement there does NOT
        // reach a rete-prefixed head the same way.
        if let Some(core) = entry.alias_of {
            return dispatch_keyword_head_value(core, args, list_span, env, sym).map(TrackedValue::from);
        }
        if entry.purity == wat_doc::Purity::Unevaluated {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::DeclarationInExpressionPosition(head.to_string()),
            )
            .into());
        }
    }
    // Producers + forms that preserve provenance: return TrackedValue directly.
    //
    // ⛔ THIS WAS A `match` UNTIL 2026-09-01. Arc 255's wave-3 homing took its last
    // non-`let` arm, leaving one live arm plus `_ => {}`, and clippy's `single_match`
    // fired: "you seem to be trying to use `match` for an equality check." That lint is
    // EARNED — it is the campaign's thesis showing up in the code's shape. The
    // retirement record below is kept verbatim: it is the history of what left this
    // match, and it outlives the match itself. When `let` finds a home too, this `if`
    // and its whole comment block retire together and the function becomes a single
    // delegation to `dispatch_keyword_head_value`.
    {
        // Arc 255 Stone HOME-11 — `:wat::edn::{read,read-json,read-foreign}` RETIRED as literal
        // arms this stone; registry-routed via `src/intrinsic/edn.rs` (the registry-first door
        // above already reaches them, same reasoning as `keyword/from-string`'s note in
        // `src/intrinsic/keyword.rs`).
        // Arc 255 Stone HOME-12 — `:wat::core::{read-string, ast->source, ast->children,
        // ast-kind, ast-name, ast-span, ast-end-span, symbol-node, fresh-symbol, keyword-node}`
        // RETIRED as literal arms this stone; registry-routed via `src/intrinsic/ast.rs` (the
        // registry-first door above already reaches them, same reasoning as above). `write-forms`
        // and `with-children` stayed literal arms here at the time — they were not that stone's
        // ten.
        // Arc 255 Stone the-registry-answers-first-wave-3 — `:wat::core::write-forms` /
        // `:wat::core::with-children` RETIRED as literal arms this stone; registry-routed via
        // `src/intrinsic/ast.rs` (the registry-first door above already reaches them, joining
        // HOME-12's ten — bodies unchanged, still `crate::edn::render::eval_write_forms` /
        // `eval_with_children`). `:wat::core::macro-error` RETIRED the same stone; registry-
        // routed via `src/intrinsic/macro_error.rs` (its body — previously inline here, the only
        // one of the five with no pre-existing named fn — moved verbatim into
        // `eval_macro_error`, logic unchanged).
        // Arc 255 Stone E-iv — `:wat::core::keyword/{to-symbol,to-type-form,
        // to-type-form-colon}` RETIRED this stone; their replacements
        // (`:wat::keyword::{to-symbol,to-type-form,to-type-form-colon}`,
        // `src/intrinsic/keyword.rs`) are registry-routed, not literal arms here — same
        // reasoning as `keyword/from-string`'s note above (`RuntimeBuilt` provenance no longer
        // survives; downgraded to `Provenance::Unknown`, the same shape every other
        // registry-routed verb already has).
        // Arc 233 Stone 233.2.k: let must return TrackedValue directly so provenance
        // from the last body expression flows through (not stripped by dispatch_keyword_head_value).
        if head == ":wat::core::let" {
            return eval_let(args, list_span, env, sym);
        }
    }
    // All other arms: dispatch through value-returning inner and wrap.
    dispatch_keyword_head_value(head, args, list_span, env, sym).map(TrackedValue::from)
}

// Inner dispatch: returns Result<Value, EvalBreak>.
// Called by dispatch_keyword_head for all non-producer arms.
fn dispatch_keyword_head_value(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Arc 278 #55 (S3b+S4) slice one — THE ONE TABLE (`rete::vocabulary::RETE_OPS`), consulted
    // FIRST for rete-namespaced heads. Routes generically by `class` (`dispatch_rete_op`, below)
    // — never a per-op match arm added to the giant match that follows (STOP-2: no rete op named
    // in more than one file). This is the "a `where` traverses `dispatch_keyword_head_value`"
    // path proven by `wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat` — the SAME
    // function `:wat::i64::+` resolves through (the registry-first door a few lines down),
    // so a rete op registered here is automatically reachable from a `where`, with no
    // `:4829`/`:9753` kernel unification needed.
    // The namespace gate comes FIRST and is the whole cost for non-rete heads: ONE prefix
    // compare, which is false for essentially every form any wat program evaluates. Without it
    // `rete_op_for`'s linear scan over the table runs on EVERY keyword dispatch in the runtime —
    // and this function is that dispatch, not the `where` path, so compiling `where` (#49a) would
    // never have removed the tax. Cheaper than the benchmark that would have justified caring.
    // DESIGN-STONE-insert-prime-split — before the intrinsic registry and before
    // the wat defclause in `sym`. 2-ary is insert$native; 3+ is insert-all$native.
    if head == ":wat::rete::insert" {
        return crate::rete::kernel::eval_insert_public(args, list_span, env, sym);
    }
    if head.starts_with(crate::rete::vocabulary::RETE_PREFIX) {
        if let Some(op) = crate::rete::vocabulary::rete_op_for(head) {
            return dispatch_rete_op(op, head, args, list_span, env, sym);
        }
    }
    // Arc 255 Stone 255.1c-guard — the registry is consulted BEFORE the literal
    // table, not as a guard arm partway down it. Registered wins, always: a
    // literal arm below this point can no longer shadow a registration by
    // sitting higher in the match (it was shadowable at HEAD — see
    // docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-255.1c-guard-hoist.md).
    // Arc 255 Stone G — `NativeHandler` now returns `TrackedValue`; THIS function's signature
    // is the bare-`Value` inner dispatch (shared by `dispatch_rete_op`'s recursive calls, which
    // have no use for provenance), so any provenance a producer handler stamped is discarded
    // here via `value_owned()` — the caller that wants it, `dispatch_keyword_head`, now
    // consults the registry itself BEFORE ever reaching this function, so a provenance-bearing
    // producer never actually flows through this discard on that path.
    if let Some(handler) = crate::intrinsic::registry().lookup(head) {
        return handler(args, list_span, env, sym).map(TrackedValue::value_owned);
    }
    // Arc 255 Stone the-hand-rolled-arms-retire — same guard as `dispatch_keyword_head`'s
    // copy above (this function's own registry-first door above proves no handler exists for
    // an `Unevaluated` row, same as there); duplicated here rather than shared because this
    // function is also reached directly by callers that bypass `dispatch_keyword_head`
    // (e.g. `dispatch_rete_op`'s recursive calls) — a lookup is idempotent, so the repeat
    // costs nothing when the first door already caught it. Retires the two hand-rolled arms
    // (`def`, `defclause`) that used to be this match's only declaration-position refusals;
    // of the 11 rows declaring `@Purity Unevaluated`, only `def` had one — the other 10
    // (`defalias`, `defenum`, `defmacro`, `defsurface`, `newtype`, `structtype`, `typealias`,
    // `load-file!`, `digest-load!`, `signed-load!`) fell through to `UnknownFunction` before
    // this guard existed. `defclause` is a separate case — see the retirement comment at its
    // old arm site, a few dozen lines below. See the sibling guard's comment above for the
    // full predicate rationale.
    if let Some(entry) = crate::intrinsic::registry().lookup_entry(head) {
        // Arc 255 Stone 2a — the alias field IS the dispatch; see `dispatch_keyword_head`'s
        // identical check for the full placement rationale (alias-before-`Unevaluated`, same
        // reasoning, same disjoint fields). Recurses into THIS function (not
        // `dispatch_keyword_head`) — the bare-`Value` re-invoke, exactly the shape
        // `dispatch_rete_op`'s own `Alias | Form | Redispatch` arm already uses
        // (`dispatch_keyword_head_value(op.core_name, args, list_span, env, sym)`), so an alias
        // registered here composes with every OTHER caller of this function, not only the one
        // reached via `dispatch_keyword_head`.
        //
        // ⚠ This check sits AFTER the `RETE_PREFIX` gate a few lines above (arc 278 #55's THE
        // ONE TABLE consult, hoisted to the top of this function) — so for a head that starts
        // with the rete namespace prefix and has an existing `RETE_OPS` row, THAT row answers
        // first, via `dispatch_rete_op`, and this alias check is never reached for it ON THIS
        // PATH. `dispatch_keyword_head` (this function's caller, for every top-level keyword-
        // headed call) has NO `RETE_PREFIX` gate of its own, so its copy of this check — placed
        // before it ever calls into this function — is what actually intercepts a rete-
        // namespaced alias ahead of `dispatch_rete_op`; see that function's comment for the
        // proof this stone's STOP-1 requires.
        if let Some(core) = entry.alias_of {
            return dispatch_keyword_head_value(core, args, list_span, env, sym);
        }
        if entry.purity == wat_doc::Purity::Unevaluated {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::DeclarationInExpressionPosition(head.to_string()),
            )
            .into());
        }
    }
    match head {
        // Arc 232 Stone 232.0 — `:wat::core::apply` substrate primitive.
        // Universal escape hatch: takes a keyword head + [-> :T] annotation +
        // optional leading positional args + a trailing :Vector (spread as
        // trailing args). Routes EARLY before all other arms so apply is
        // unambiguous at the dispatch level.
        ":wat::core::apply" => eval_apply(args, env, sym, list_span.clone()),
        // Arc 255 Stone A-2-ii-b-0 — `:wat::core::type` (arc 234 Stone 234.0's polymorphic
        // type-name primitive: HolonAST classifier-wrap → extract_classifier; other →
        // `Value::declared_type_name`; consumed by surface-method dispatch and all arc 234.x
        // record-y verbs) moved into a `#[wat_intrinsic]` handler (`src/intrinsic/reflect.rs`)
        // with its real (1) arity declared; the pre-match registry check above (arc
        // 255.1c-guard) intercepts the name before reaching here. Ruled
        // `Pure ∧ Deterministic ∧ Total` — `eval_type`'s only `return Err` is the arity check,
        // which retires on homing; the body then delegates unconditionally to
        // `Value::declared_type_name`, an exhaustive, non-`Result` match over every `Value`
        // variant (`src/value/value.rs:1705`) — no remaining domain-failure path.
        // Arc 255 Stone P6-c-W6 — `:wat::core::length`/`empty?` moved into `#[wat_intrinsic]`
        // handlers (above this match, still in this file) with their real (1/1) arities
        // declared; the pre-match registry check above (arc 255.1c-guard) intercepts both
        // names before reaching here.
        // Arc 237 Stone 237.7b-ii — `:wat::core::contains?` ∀T intrinsic with custom inference arm.
        // Polymorphic membership predicate: (Vector :- [T]) / (HashSet :- [T]) / (HashMap :- [K V]) → bool.
        // Tier B: element-typing enforced at check by infer_contains (src/check.rs); behavior-preserving.
        ":wat::core::contains?" => eval_contains(args, list_span, env, sym),
        // Arc 237 Stone 237.7b-iv — `:wat::core::get` ∀T intrinsic with custom inference arm.
        // Polymorphic indexed/keyed lookup: (Vector :- [T]) + i64 → (Option :- [T]); (HashMap :- [K V]) + K → (Option :- [V]).
        // Tier B: (Option :- [element]) precision enforced at check by infer_get (src/check.rs); behavior-preserving.
        // NO HashSet arm — HashSet has no positional get.
        ":wat::core::get" => eval_get(args, list_span, env, sym),
        // Arc 255 Stone the-collection-readers — `:wat::core::conj`/`assoc` moved into
        // `#[wat_intrinsic]` handlers (`src/intrinsic/collection.rs`), thin delegates over
        // `eval_conj`/`eval_assoc` (in place, unmoved); the pre-match registry check above
        // (arc 255.1c-guard) intercepts both names before reaching here.
        // Arc 237 Stone 237.5 — `:wat::core::conforms?` general type-conformance primitive.
        // Recursive walker over the TypeExpr grammar (Path / Parametric / Tuple / Alias / Union).
        // Signature: (value :TypeExpr) -> :wat::core::bool
        // Error contract: well-formed type + no-match → false; unknown/Fn/Var type → Err.
        ":wat::core::conforms?" => eval_conforms(args, list_span, env, sym),
        // Arc 255 Stone HOME-11 — `:wat::edn::validate` RETIRED as a literal arm this stone;
        // registry-routed via `src/intrinsic/edn.rs` (`eval_edn_validate`'s body — the DEEP
        // shape check `conforms?` structurally cannot do — is untouched and un-moved; only the
        // dispatch route changed, plus a visibility widening to `pub(crate) fn` so the registry
        // handler can reach it).
        // Arc 237 Stone S-A — `:wat::core::subtype?` is-a hierarchy predicate.
        // Directional, transitive, reflexive walk over the `typesub` child→parent registry.
        // Signature: (:TypeKeyword :TypeKeyword) -> :wat::core::bool
        // Error contract: well-formed known type names → bool; unknown name → Err.
        ":wat::core::subtype?" => eval_subtype(args, list_span, env, sym),
        // Arc 255 Stone the-registry-answers-first-wave-3 — `:wat::core::aggregate-new` /
        // `:wat::core::kwargs-construct` RETIRED as literal arms this stone; registry-routed via
        // `src/intrinsic/record.rs` (the registry-first door above already reaches them, joining
        // the rest of the record family — bodies unchanged, still `eval_aggregate_new` /
        // `eval_kwargs_construct` below, now `pub(crate)`).
        // Arc 293 K3-revise — the TWO projection verbs (the PAIR): project a satisfier's
        // surface attributes into a new backing record at the pure tier the caller names.
        // Projection is ONE-WAY UP — you never project down to a struct (you already have
        // the struct; an in-locus copy of in-locus data buys nothing).
        //   (:wat::core::to-record  x :S) → :S$core-record  (portable EDN; Record nature)
        //   (:wat::holon::to-record x :S) → :S$holon-record (portable EDN + hologram)
        // arg0 `x` is evaluated; arg1 `:S` is a literal surface keyword (NOT evaluated).
        // RETIRED 293 K3-revise: `:wat::core::to-struct` — projection is ONE-WAY UP, never
        // down; `$struct` is the impure tier; you already have the struct in locus.
        // Arc 255 Stone the-record-family — `:wat::core::to-record` moved into a
        // `#[wat_intrinsic]` handler (`src/intrinsic/record.rs`) with its real (2) arity
        // declared; the pre-match registry check above (arc 255.1c-guard) intercepts the
        // name before reaching here. Ruled `Pure ∧ Deterministic ∧ Partial` — measured at
        // `project_surface_attrs` (`:17828`), whose `sym.get(&method_key)` miss raises
        // `UnknownFunction` when a surface member names no accessor on the concrete type.
        // Arc 296 G-1b — `:wat::core::Record::of` / `:wat::holon::Record::of` DELETED (finish
        // the kill, arc 294.c.2a): both retired constructors, zero/one live callers, superseded
        // by `aggregate-new` (the one nature-dispatched ctor).
        // Arc 255 Stone A-2-ii-b-0 — `:wat::core::Record/field-at` (arc 234 Stone 234.2a's
        // accessor: (record index) -> field-value at fields[index]) moved into a
        // `#[wat_intrinsic]` handler (`src/intrinsic/record.rs`) with its real (2) arity
        // declared; the pre-match registry check above (arc 255.1c-guard) intercepts the name
        // before reaching here. Ruled `Pure ∧ Deterministic ∧ Partial` — measured at the site:
        // `if index < 0 || (index as usize) >= fields.len()` returns `Err`.
        // Arc 234 Stone 234.3a — polymorphic record read verbs.
        // record?   :: ∀T. T -> bool          — true iff input is Value::Aggregate (Record/HolonRecord nature)
        // record->map :: :wat::core::Record -> (HashMap :- [keyword T]) — extract field-name/value map
        // Arc 255 Stone the-record-family — `:wat::core::record->map` moved into a
        // `#[wat_intrinsic]` handler (`src/intrinsic/record.rs`) with its real (1) arity
        // declared; the pre-match registry check above (arc 255.1c-guard) intercepts the
        // name before reaching here. Ruled `Pure ∧ Deterministic ∧ Partial` — measured at
        // `record_field_map` (`:18125`), which raises `MalformedForm` when the receiver's
        // class is not registered in the TypeEnv — a hole the `:wat::core::Record` umbrella
        // param type does not close (the same container-gate/value-hole shape `assoc`'s
        // Record arm carries).
        // Arc 255 `DESIGN-STONE-a-registered-row-may-not-keep-its-arm.md` — this arm RETIRED;
        // `:wat::core::record?` carries a registered handler, so the registry-first door above
        // (`crate::intrinsic::registry().lookup(head)`) already dispatches it to
        // `eval_record_q` (unchanged) before this match is ever reached.
        // Arc 249 Stone 249.3a — form-shape predicate over WatAST::List form-values.
        // List? :: ∀T. T -> bool — true iff input is Value::wat__WatAST wrapping WatAST::List.
        // core form-shape predicate over WatAST::List; distinct from
        // :wat::holon::is-List? (a classifier over HolonAST). The name
        // diverges on purpose — the form-vs-holon distinction is the
        // reason this exists. Do not "harmonize" the two names.
        ":wat::core::List?" => crate::record::access::eval_list_q(args, list_span, env, sym),
        // Arc 234 Stone 234.3b — write verb in the polymorphic record-y family.
        // assoc :: :wat::core::Record × :wat::core::keyword × :T -> :wat::core::Record
        // Returns a new record with one field replaced; original is unchanged (immutable).
        // Arc 255 Stone the-record-family — moved into a `#[wat_intrinsic]` handler
        // (`src/intrinsic/record.rs`) with its real (3) arity declared; the pre-match
        // registry check above (arc 255.1c-guard) intercepts the name before reaching here.
        // Ruled `Pure ∧ Deterministic ∧ Partial` — measured at `record_assoc_inner`
        // (`:18319`'s `UnknownField` miss on the field-name lookup; `:18337`'s `TypeMismatch`
        // when the new value's type variant differs from the old field's) — `Record/assoc`
        // is `assoc`'s sibling and shares exactly that shape.
        // Arc 237 Stone S-C.2d — type-BLIND record data equality.
        // same-data? :: :wat::core::Record × :wat::core::Record -> :wat::core::bool
        // Compares field-name→value maps (record->map); type-blind and flavor-blind.
        // Distinct from `=` (type-strict, arc 238): Pt[0,0] same-data? Coord[0,0] → true.
        // Arc 255 Stone the-record-family — moved into a `#[wat_intrinsic]` handler
        // (`src/intrinsic/record.rs`) with its real (2) arity declared; the pre-match
        // registry check above (arc 255.1c-guard) intercepts the name before reaching here.
        // Ruled `Pure ∧ Deterministic ∧ Partial` — reaches `record_field_map` (`:18125`)
        // twice, the same unregistered-class hole `record->map` carries.
        // Language forms
        // Arc 157 slice 1a-ii — config setters. These are top-level forms
        // that update the SymbolTable carrier flags at freeze time (via
        // `register_runtime_defs_form`). At eval time (this arm), the flag
        // has already been processed at freeze time; return Unit as a no-op.
        // The `dispatch_keyword_head` takes `sym: &SymbolTable` (immutable),
        // so there is no way to mutate the flag here — and no need to,
        // because freeze-time processing already set it.
        // ⛔ Arc 255 Stone 1a-ε — the `":wat::config::set-redef!" | ":wat::config::set-eval-redef!"
        // => Ok(Value::Unit)` arm that stood here is DELETED. Both are registered rows carrying a
        // `role = eval` handler now, so the registry-first door hoisted above this match answers
        // them by name and this arm could never fire — the "a registered row may not keep its
        // literal arm" gate (`intrinsic/mod.rs`) demanded the deletion and named both rows. The
        // no-op semantics are unchanged; they moved to `intrinsic/special/config_set_redef.rs`,
        // whose own doc records WHY the arm is a no-op: the flag was already applied at freeze.
        // Arc 170 Gap I-B — `:wat::core::def` at expression position. The permissive arm
        // (evaluate RHS, return Unit) that used to live here, then this stone's own literal
        // `":wat::core::def" => Err(DeclarationInExpressionPosition(...))` arm that replaced
        // it, is now RETIRED (Arc 255 Stone the-hand-rolled-arms-retire,
        // `BRIEF-STONE-the-hand-rolled-arms-retire.md`). `def` is a registered
        // `#[wat_special_form]` row declaring `@Purity Unevaluated`, so the registry-first
        // `Unevaluated`-keyed guard above this match now answers it by the SAME name before
        // this match is ever reached — a hand-rolled arm can no longer shadow the guard by
        // sitting higher in the match, same "registered wins, always" contract the 255.1c
        // guard-hoist established. Top-level defs are still processed by
        // `register_runtime_defs_form` (freeze-time), which never routes through `eval` /
        // `dispatch_keyword_head`.
        // Stone 241.14 — `:wat::core::def-restricted` eval arm DELETED.
        // HARD CUT at check.rs fires before eval; no form reaches here.
        // Stone 237.2's `:wat::core::defclause` at expression-position arm is ALSO RETIRED
        // this stone, but NOT by the guard above — `defclause` carries no
        // `#[wat_special_form]` registration at all (it is parsed as a declaration only by
        // `register_runtime_defs_form`/`preregister_defclause_in_env`, never entered into
        // `crate::intrinsic::registry()`), so `lookup_entry(":wat::core::defclause")` is
        // `None` and the purity guard does not fire for it — measured live: the literal head
        // `:wat::core::defclause` in expression position does not even reach this dispatch
        // through a check-passing program; `check.rs`'s resolve pass already refuses it as an
        // `UnresolvedReference` (not a registered call head), so this arm was reachable only
        // by an AST that bypasses `check.rs` entirely (no test in this repo exercised that
        // path for `defclause` — `def`'s sibling probe,
        // `tests/wat_lang/probe_def_not_special.rs`, only ever covered `def`). Retired anyway
        // per the brief rather than kept as an orphaned special case: any raw-AST encounter of
        // `:wat::core::defclause` now falls through to the ordinary `UnknownFunction` fallback,
        // same as any other unregistered head — a narrower, honest answer (`defclause` really
        // isn't a registry-known function) in place of a name this stone's guard cannot derive
        // without the exact hand-list the 2026-06-24 note refused.
        // Stone 241.16 — `:wat::core::define` eval dispatch arm DELETED.
        // HARD CUT at check.rs (Stone 241.11 + 241.16) fires before eval;
        // no define-headed form reaches this dispatch. DefineInExpressionPosition
        // variant retired with this arm.
        // Arc 155 — `:wat::core::fn` is the canonical operator for
        // function values (Clojure-faithful lowercase verb; mirrors
        // arc 154's let retirement recipe). Routes to `eval_fn`
        // (formerly `eval_lambda`).
        // Arc 255 `DESIGN-STONE-every-role-carries-its-pointer.md` — this arm RETIRED;
        // `:wat::core::fn` carries a registered `role = eval` handler now (`eval_fn_form`,
        // `src/intrinsic/special/fn_form.rs`), so the registry-first door above
        // (`crate::intrinsic::registry().lookup(head)`) already dispatches it to `eval_fn`
        // (unchanged) before this match is ever reached.
        // Arc 155 slice 2 — `:wat::core::lambda` dispatch arm retired.
        // Single-letform vocabulary; lambda is dead (Clojure-faithful;
        // `fn` replaces `lambda` per user direction 2026-05-07).
        // BareLegacyLambda variant + Display retained as orphaned
        // scaffolding (arc 113 precedent); runtime fall-through retired.
        // Arc 255 `DESIGN-STONE-every-role-carries-its-pointer.md` — this arm RETIRED;
        // `:wat::core::let` carries a registered `role = eval` handler now, so the
        // registry-first door above (`crate::intrinsic::registry().lookup(head)`) already
        // dispatches it to `eval_let` (unchanged) before this match is ever reached.
        // Arc 255 Stone 1a-zeta (`DESIGN-STONE-1a-zeta-the-last-three-of-the-special-form-
        // table.md`) — this arm RETIRED; `:wat::core::do` carries a registered `role = eval`
        // handler now (`eval_do`, annotated in place, same file), so the registry-first door
        // above (`crate::intrinsic::registry().lookup(head)`) already dispatches it to
        // `eval_do` (unchanged) before this match is ever reached.
        // Arc 255 `DESIGN-STONE-every-role-carries-its-pointer.md` — this arm RETIRED;
        // `:wat::core::if` carries a registered `role = eval` handler now, so the
        // registry-first door above (`crate::intrinsic::registry().lookup(head)`) already
        // dispatches it to `eval_if` (unchanged) before this match is ever reached.
        // Arc 255 Stone 1a-zeta — this arm RETIRED; `:wat::core::ann-form` carries a
        // registered `role = eval` handler now (`eval_ann_form`, annotated in place, same
        // file), so the registry-first door above (`crate::intrinsic::registry().lookup(head)`)
        // already dispatches it to `eval_ann_form` (unchanged) before this match is ever
        // reached.
        // Arc 255 Stone 1a-gamma-i — this arm RETIRED; `:wat::core::quote` carries a
        // registered `role = eval` handler now (`eval_quote_form`,
        // `src/intrinsic/special/quote.rs`), so the registry-first door above
        // (`crate::intrinsic::registry().lookup(head)`) already dispatches it to `eval_quote`
        // (unchanged) before this match is ever reached.
        // Arc 118 — lazy-seq foundation primitives.
        // `seq-empty`/`cons`/`next` (arc 255 Stone P6-c-W2) moved into `#[wat_intrinsic]`
        // handlers (`src/intrinsic/stream.rs`); the pre-match registry check above
        // (arc 255.1c-guard) intercepts all three names before reaching here.
        // Arc 255 Stone 1a-zeta — the `:wat::stream::lazy` arm that used to sit here (`=>
        // eval_lazy_seq(...)`) is RETIRED; the row carries a registered `role = eval` handler
        // now (`eval_lazy_seq_form`, `src/intrinsic/special/stream_lazy.rs` — a thin delegate,
        // since `eval_lazy_seq`'s own 3-param signature doesn't fit the canonical 4-param
        // `NativeHandler` shape), so the registry-first door above already dispatches it before
        // this match is ever reached. `lazy-seq` is a SPECIAL FORM (capture-don't-eval): wrap
        // the body in a 0-arg closure over the current env → Stream::Thunk. Mirrors `quote`.
        // Arc 294.b — `#holon <form>` / `(:wat::holon::literal <form>)`.
        // Capture the body as data via `eval_quote` (→ `Value::wat__WatAST`),
        // then lower to a hologram via `to_holon_inner` (which dispatches
        // `Value::wat__WatAST` through `watast_to_holon` at runtime.rs:14437).
        // Arc 255 Stone 1a-gamma-i — this arm RETIRED; `:wat::core::quasiquote` carries a
        // registered `role = eval` handler now (`eval_quasiquote`, annotated in place, same
        // file), so the registry-first door above already dispatches it before this match is
        // ever reached.
        // Arc 255 Stone 1a-gamma-i — this arm RETIRED; `:wat::core::struct->form` carries a
        // registered `role = eval` handler now (`eval_struct_to_form`,
        // `src/reflect/render.rs`, annotated in place), so the registry-first door above
        // already dispatches it before this match is ever reached.
        // Arc 143 slice 1 / slice 3, Arc 201 slice 5 — `lookup-define`, `signature-of-defn`,
        // `signature-of-fn`, `return-type-of`, `body-of`, `rename-callable-name`,
        // `extract-arg-names`, `extract-arg-types` (arc 255 Stone P6-c-W3) moved into
        // `#[wat_intrinsic]` handlers (above this match, still in this file); the pre-match
        // registry check above (arc 255.1c-guard) intercepts all eight names before reaching
        // here. Real arity declared for each (1/1/1/1/1/3/1/1) — no hand-rolled arity guard
        // survives.
        // Stone 241.7 / Arc 170 Strike B — `metadata-of`, `field-names-of`, `field-types-of`
        // (arc 255 Stone P6-c-W4) moved into `#[wat_intrinsic]` handlers (above this match,
        // still in this file); the pre-match registry check above (arc 255.1c-guard)
        // intercepts all three names before reaching here. Real arity declared for each
        // (1/1/1) — no hand-rolled arity guard survives. Unlike W3's eight, these three carry
        // NO checker `TypeScheme` (`FROZEN_CHECKER_DEBT_LEDGER`, `src/intrinsic/mod.rs`) —
        // homing them grows that ledger by three, not zero; `src/check.rs` itself is
        // untouched (`field-names-of`/`field-types-of` keep their `infer_list` special-case;
        // `metadata-of` keeps having no check-side treatment at all).
        // Arc 201 slice 2 — general-purpose Bundle accessors. The
        // leaf-unwrap counterpart (`:wat::core::atom-value`) was already
        // minted by arc 057; SCORE-SLICE-2 § Sibling check documents the
        // decision to reuse it rather than mint `Atom/value` as a duplicate.
        // Arc 232 Stone 232.0a — typed-entities reflection layer.
        // Three verbs that lift existing Rust helpers (extract_classifier)
        // and mint new structural accessors (bind_left + bind_right) as
        // wat-callable primitives. Surface-method dispatch
        // (Stone 232.1) consumes extract-classifier; defrecord accessor
        // synthesis (separate stone) composes Bind/left + Bind/right +
        // Bundle/children. Naming convention: Bind/left + Bind/right are
        // positional (structural fact); extract-classifier is semantic
        // (classifier-wrap convention). Per intueri cast 2026-05-23 night.
        // Arc 259 — The Forced Hand: ambient program environment.
        // `:wat::program::env` (arc 255 Stone P6-c-W2) moved into a `#[wat_intrinsic]`
        // handler (`src/intrinsic/program.rs`); the pre-match registry check above
        // (arc 255.1c-guard) intercepts the name before reaching here.
        // Arc 209 C0b.3a-0 — process child owner-link.
        // Reads the calling thread's SELF_PEER slot (installed at the
        // child-only seam run_forms_as_server_child, before :user::main).
        // Root → clean MalformedForm error (no owner-link). Two checker-only
        // type-keyword args (:S :R) validated but not evaluated.
        ":wat::program::self-peer" => eval_program_self_peer(args, list_span),
        // Arc 170 slice 1e — `:wat::runtime::argv`/`current-thread` (ambient runtime values
        // per REALIZATIONS pass 7) moved into `#[wat_intrinsic]` handlers (arc 255 Stone
        // P6-c-W3, both above this match, still in this file) with their real (0) arity
        // declared — no `args`/`list_span` params survive (true nullary, like
        // `:wat::stream::empty`); the pre-match registry check above (arc 255.1c-guard)
        // intercepts both names before reaching here.
        // arc 255 Stone P6-c-1 — `:wat::program::cpu-count` and `:wat::form::matches?`
        // homed to `#[wat_intrinsic]` (both above this match, still in this file); no
        // arm needed here anymore. Type-checking (`check.rs::infer_form_matches` for
        // `matches?`; the hand-registered `TypeScheme` for `cpu-count`) is unaffected.
        // Arc 278 Stone 2a — rete single-fact alpha matcher. Pure data-in/data-out: cond
        // (WatAST from quote) × fact (Record) → (Option :- [(PersistentMap :- [String Value])]).
        // Bindings keyed by logic-var name string ("?t" → bound value).
        // Arc 255 Stone P6-c-W5a — `:wat::rete::alpha-match`/`alpha-match-local`/
        // `alpha-match-under`/`cond-has-deferred-constraint?` moved into `#[wat_intrinsic]`
        // handlers (`src/intrinsic/rete.rs`) with their real (2/2/3/1) arities declared; the
        // pre-match registry check above (arc 255.1c-guard) intercepts all four names before
        // reaching here. The pure inner matcher (`alpha_match_inner`/`*_local`/`*_seeded`,
        // `src/rete/matcher.rs`) is unchanged.
        // Arc 278 Stone 4a — rete RHS insert evaluator (the dual of alpha-match): insert-form
        // (WatAST) × bindings (PersistentMap) → Record, resolving ?var/literal fact-args via
        // resolve_operand, OR (Stone B widening) falling through to a fenced fn-call eval —
        // no longer pure data-in/data-out, see `eval_insert`'s own doc.
        // Arc 255 Stone P6-c-W5b — `:wat::rete::eval-insert` moved into a `#[wat_intrinsic]`
        // handler (`src/rete/eval_insert.rs`, in place — not relocated to `src/intrinsic/`)
        // with its real (2) arity declared; the pre-match registry check above (arc
        // 255.1c-guard) intercepts the name before reaching here.
        // Arc 278 Stone P2 — native Rust single-pass fire cycle (the differential harness).
        // (:wat::rete::fire-once <session>) → :wat::rete::Session
        // Equivalent to fire-once$oracle on AST Sessions; Export is native-only
        // (the oracle refuses an imported Export).
        ":wat::rete::fire-once$native" | ":wat::rete::fire-once" => {
            crate::rete::kernel::eval_fire_once_native(args, list_span, env, sym)
        }
        // Public `fire-rules` is a wat Fn (first-class). Keyword-head and the
        // Fn body both reach rust through `$native`.
        ":wat::rete::fire-rules$native" | ":wat::rete::fire-rules" => {
            crate::rete::kernel::eval_fire_rules_native(args, list_span, env, sym)
        }
        // Arc 278 — intern the rust InternedNetwork when compile-all returns a Session
        // (`DESIGN-STONE-arm-at-compile`). Value unchanged. First fire-rules HIT.
        // rune:circumspicere(accepted-by-design) — intern hangup mouths are keyword
        // primitives (TypeScheme + runtime dispatch), not dual-impl wat Fns; bound in
        // DESIGN-STONE-intern-eviction.md (keyword primitives; oracle has no intern).
        // Arc 278 — drop one intern lease (`DESIGN-STONE-intern-eviction`).
        // Value unchanged. Last lease removes the intern entry.
        // Arc 255 Stone P6-c-W5b — `:wat::rete::arm-session`/`release-session` moved into
        // `#[wat_intrinsic]` handlers (`src/rete/kernel/arm.rs`, in place) with their real (1)
        // arity declared each; the pre-match registry check above (arc 255.1c-guard)
        // intercepts both names before reaching here.
        // Arc 278 — native `insert-all` (oracle is `insert-all$oracle`).
        // 2-ary `insert` is handled above (`eval_insert_public`).
        ":wat::rete::insert-all$native" | ":wat::rete::insert-all" => {
            crate::rete::kernel::eval_insert_all_native(args, list_span, env, sym)
        }
        ":wat::rete::insert$native" => {
            crate::rete::kernel::eval_session_insert(args, list_span, env, sym)
        }
        ":wat::rete::fire-rules-explain$native" | ":wat::rete::fire-rules-explain" => {
            crate::rete::kernel::eval_fire_rules_explain(args, list_span, env, sym)
        }
        // Arc 278 Stone P12c — per-edge explain payload builder.
        // (:wat::rete::step-payload session alpha-id bindings sfact supporting) → :wat::rete::DerivationStep
        // REUSES resolve_operand + the clause classifier from matcher.rs (faithful by construction).
        // Arc 255 Stone P6-c-W5c — `:wat::rete::step-payload` moved into a `#[wat_intrinsic]`
        // handler (`src/rete/step_payload.rs`, in place) with its real (5) arity declared; the
        // pre-match registry check above (arc 255.1c-guard) intercepts the name before reaching
        // here.
        // Arc 255 Stone P6-c-W5b — `:wat::rete::export`/`import` moved into `#[wat_intrinsic]`
        // handlers (`src/rete/export.rs`, in place) with their real (1) arity declared each;
        // the pre-match registry check above (arc 255.1c-guard) intercepts both names before
        // reaching here.
        // Arc 278 Stone 6a — the rete condition fence: four orthogonal classifiers, each
        // default-deny + transitive over user-fn bodies. A rete condition must be
        // (pure AND deterministic AND total AND a rete primitive).
        //   pure?          = effect-free (no IO/mutation). Uuid/v4 IS pure (does no IO).
        //   deterministic? = same inputs → same output. Uuid/v4 is NOT (random); Uuid/v5 IS.
        //   total?         = defined on all inputs. ARMED: `compile-condition` consults it.
        // Arc 255 Stone P6-c-W5a — `:wat::rete::pure?`/`deterministic?`/`total?`/`primitive?`
        // and the admission test `vocabulary-admitted?` all moved into `#[wat_intrinsic]`
        // handlers (`src/intrinsic/rete.rs`) with their real (1) arity declared; the pre-match
        // registry check above (arc 255.1c-guard) intercepts all five names before reaching
        // here. `axis-violation` below runs the SAME walk and IS part of Stone P6-c-W5c.
        // BRIEF-the-fence-names-the-head — the SAME walk pure?/deterministic? run, surfacing the
        // first violating leaf instead of discarding it.
        // (:wat::rete::axis-violation <quoted-expr> <axis: :wat::rete::Axis>) -> (:wat::core::Option :- [wat::rete::AxisViolation])
        // Arc 255 Stone P6-c-W5c — `:wat::rete::axis-violation` moved into a `#[wat_intrinsic]`
        // handler (`src/rete/purity.rs`, in place) with its real (2) arity declared; the
        // pre-match registry check above (arc 255.1c-guard) intercepts the name before reaching
        // here.
        // Arc 278 Stone 6b-i — the runtime evaluator for where/:test predicates.
        // (:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :wat::core::bool
        // Arc 255 Stone P6-c-W5b — moved into a `#[wat_intrinsic]` handler
        // (`src/rete/eval_test.rs`, in place) with its real (2) arity declared; the pre-match
        // registry check above (arc 255.1c-guard) intercepts the name before reaching here.
        // #49 — rule-compile refuse: lower the where expr or raise. Returns nil on success.
        // Arc 255 Stone P6-c-W5c — `:wat::rete::lower` moved into a `#[wat_intrinsic]` handler
        // (`src/rete/expr_ir.rs`, in place) with its real (1) arity declared; the pre-match
        // registry check above (arc 255.1c-guard) intercepts the name before reaching here.
        // Arc 255 Stone P6-c-W5c — `:wat::rete::collect-rules` moved into a `#[wat_intrinsic]`
        // handler (`src/rete/collect.rs`, in place) with its real (1) arity declared; the
        // pre-match registry check above (arc 255.1c-guard) intercepts the name before reaching
        // here.
        // Arc 255 Stone 1a-gamma-i — this arm RETIRED; `:wat::core::forms` carries a
        // registered `role = eval` handler now (`eval_forms_form`,
        // `src/intrinsic/special/forms.rs`), so the registry-first door above already
        // dispatches it to `eval_forms` (unchanged) before this match is ever reached.
        // Arc 255 Stone 1a-gamma-i — this arm RETIRED; `:wat::core::macroexpand-1` carries a
        // registered `role = eval` handler now (`eval_macroexpand_1`,
        // `src/reflect/expand.rs`, annotated in place), so the registry-first door above
        // already dispatches it before this match is ever reached.
        // Arc 255 Stone 1a-gamma-i — this arm RETIRED; `:wat::core::macroexpand` carries a
        // registered `role = eval` handler now (`eval_macroexpand`, `src/reflect/expand.rs`,
        // annotated in place), so the registry-first door above already dispatches it before
        // this match is ever reached.
        // Arc 255 Stone HOME-8 — ":wat::holon::from-holon" (the one holon producer) is now
        // registered via `#[wat_intrinsic]` (`src/intrinsic/holon/atom.rs`); the registry-first
        // door at the top of `dispatch_keyword_head` finds it before this match is ever reached.
        // Arc 255 `DESIGN-STONE-every-role-carries-its-pointer.md` — this arm RETIRED;
        // `:wat::core::match` carries a registered `role = eval` handler now, so the
        // registry-first door above (`crate::intrinsic::registry().lookup(head)`) already
        // dispatches it to `eval_match` (unchanged) before this match is ever reached.
        // Arc 255.1c-kernel-remainder (home #8) — `:wat::kernel::serve-dispatch-op`'s
        // non-tail literal arm (and its `eval_kernel_serve_dispatch_op` delegate, which
        // evaluated `body` via `eval_inner`) moved to the intrinsic registry, WHICH NOW
        // REGISTERS THE TAIL DELEGATE (`eval_kernel_serve_dispatch_op_tail`) for this FQDN —
        // a two-arm collapse to one handler. The `eval_inner`-based non-tail delegate was
        // already "defensive parity… reached only if ever evaluated outside serve's tail
        // position" per its own doc (codegen never places it there); with both literal arms
        // gone there is no second dispatch path left for it to be parity FOR, so it is
        // deleted rather than kept as unreachable duplicate code. See
        // `src/intrinsic/kernel/serve.rs`'s doc for the full derivation, including why
        // routing serve-dispatch-op through the registry via the TAIL delegate preserves
        // `serve`'s TCO (verified against `apply_function`'s trampoline loop).
        // Arc 109 slice 1j — § D' Option/Result method forms.
        // Stone 241.15: the three retiring verbs (:wat::core::try,
        // :wat::core::option::expect, :wat::core::result::expect) are now
        // HARD CUT (MalformedForm rejection at check time); dispatch arms deleted.
        // Arc 255 Stone A-2-ii-b-0 — `:wat::core::Option/expect` moved into a `#[wat_intrinsic]`
        // handler (`src/intrinsic/option.rs`) with its real (2) arity declared; the pre-match
        // registry check above (arc 255.1c-guard) intercepts the name before reaching here.
        // Ruled `Pure ∧ Deterministic ∧ Partial` — it raises on `None`
        // (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`).
        // Arc 255 Stone — the option/result siblings — `:wat::core::Option/try`,
        // `:wat::core::Result/expect`, `:wat::core::Result/try` moved into `#[wat_intrinsic]`
        // handlers (`src/intrinsic/option.rs`, `src/intrinsic/result.rs`) with their real
        // arities declared (`Result/expect` 2, both `try` verbs 1); the registry check above
        // intercepts each name before reaching here, so their literal dispatch arms are gone.
        // `Result/expect` ruled `Pure ∧ Deterministic ∧ Partial` — same shape as
        // `Option/expect`, it `expect_panic`s on `Err`. Both `try` verbs ruled
        // `Pure ∧ Deterministic ∧ Total` — the propagate signal they raise
        // (`EvalSignal::TryPropagate`/`OptionPropagate`) is not a panic; `apply_function`
        // (`:19490-19495` below) catches it and wraps it as the enclosing function's own
        // matchable `Err`/`:None` return, guaranteed type-correct by the checker whenever the
        // body contains a `try` (see the comment at `:19458`).
        // Arc 255 Stone the-record-family — `:wat::core::struct-new`, `:wat::core::struct-field`,
        // `:wat::core::variant` moved into `#[wat_intrinsic]` handlers (`src/intrinsic/record.rs`)
        // with their real arities declared; the pre-match registry check above (arc 255.1c-guard)
        // intercepts each name before reaching here.
        // `struct-new` — `Variadic`. ⚠ Its own first guard (`if args.is_empty()`, above at
        // `:15765`) enforces a minimum of ONE argument (the type-name keyword; zero fields is
        // admitted), not two — a measured correction, not the two-argument minimum an earlier
        // draft of this stone's brief assumed (STOP-1). Ruled `Pure ∧ Deterministic ∧ Partial` —
        // raises `MalformedForm` when arg0 is not a keyword literal (`:15779`) or names an
        // unregistered struct/newtype (`:15810`).
        // `struct-field` — `Exact(2)` (`if args.len() != 2`, above at `:16086`). Ruled
        // `Pure ∧ Deterministic ∧ Partial` — raises `TypeMismatch` on a non-`Aggregate`
        // receiver (`:16109`) and `MalformedForm` on an out-of-range field index (`:16148`).
        // `variant` — `Variadic`, real minimum TWO (`if args.len() < 2`, above at `:15852`;
        // confirmed against §1's table). Ruled `Pure ∧ Deterministic ∧ Partial` — raises
        // `MalformedForm` when the type path is not a registered enum (`:15917`) or the
        // variant name is unknown on that enum (`:15937`).
        // Arc 255.1c-kernel-remainder (home #8) — `:wat::kernel::retag-op` moved to the
        // intrinsic registry (`src/intrinsic/kernel/serve.rs`); dispatch now reaches
        // `eval_retag_op` (unchanged) via the registry lookup above, not a literal arm here.
        ":wat::core::first" => {
            eval_positional_accessor(args, list_span, env, sym, ":wat::core::first", 0)
        }
        ":wat::core::second" => {
            eval_positional_accessor(args, list_span, env, sym, ":wat::core::second", 1)
        }
        ":wat::core::third" => {
            eval_positional_accessor(args, list_span, env, sym, ":wat::core::third", 2)
        }
        // Stone 118.B4-0 — `nth` promoted from a wat `defclause` to a Rust intrinsic so a
        // `defmacro` program body (which evaluates only through this dispatcher) can call it.
        // The runtime-index generalization of first/second/third, above. `:wat::core::nth-spec`
        // (`wat/core.wat`) is the wat ORACLE kept honest by a differential test.
        // Arc 255 Stone P6-c-W6 — moved into a `#[wat_intrinsic]` handler (above this match,
        // still in this file) with its real (2) arity declared; the pre-match registry check
        // above (arc 255.1c-guard) intercepts the name before reaching here.
        // Stone 118.B5 — `stream->vec`/`stream->pvec` promoted from wat `defn` to Rust
        // intrinsics: the native kernel underneath `into`'s two Stream clause arms
        // (`wat/seq.wat:166`; the clause bodies are UNCHANGED — they already named these two
        // verbs, which simply stop being interpreted). `:wat::core::stream->vec-spec` /
        // `-pvec-spec` (`wat/seq.wat`) are the retained wat ORACLES, kept honest by a
        // differential (`wat-tests/core/core-stream-materializers-differential.wat`). Same
        // shape as `nth` immediately above and `foldl` (B6): the fast native kernel, the wat
        // spec keeps it honest.
        ":wat::core::stream->vec" => {
            crate::collection::transform::eval_stream_to_vec(args, list_span, env, sym)
        }
        ":wat::core::stream->pvec" => {
            crate::collection::transform::eval_stream_to_pvec(args, list_span, env, sym)
        }
        // Vec last + find-last-index. Arc 047.
        // Arc 255 Stone P6-c-W6 — `:wat::core::last` moved into a `#[wat_intrinsic]` handler
        // (`src/collection/transform.rs`, in place) with its real (1) arity declared; the
        // pre-match registry check above (arc 255.1c-guard) intercepts the name before
        // reaching here. `find-last-index` is a HOF wearing a reader's name (calls
        // `apply_function` on a caller fn) and stays in this giant match — W7's family, not W6.
        ":wat::core::find-last-index" => {
            crate::collection::transform::eval_vec_find_last_index(args, list_span, env, sym)
        }
        // Arc 255 Stone P6-c-W6 — `:wat::core::rest` moved into a `#[wat_intrinsic]` handler
        // (`src/collection/eval.rs`, in place) with its real (1) arity declared; the pre-match
        // registry check above (arc 255.1c-guard) intercepts the name before reaching here.
        // Arc 255 Stone HOME-9 — `:wat::std::list::map-with-index`'s dispatch arm (which lived
        // here) is DELETED, not moved. `:wat::core::map-indexed` is its non-drop-in Seqable-
        // generic replacement (arg order flips, result is a lazy Stream) — see
        // `src/collection/transform.rs`'s note beside the deleted `eval_vec_map_with_index`.

        // :u8 range-checked cast from :i64. Arc 008 slice 1.
        // Arc 255 `DESIGN-STONE-a-registered-row-may-not-keep-its-arm.md` — this arm RETIRED;
        // `:wat::core::u8` carries a registered handler, so the registry-first door above
        // (`crate::intrinsic::registry().lookup(head)`) already dispatches it to
        // `eval_u8_cast` (unchanged) before this match is ever reached. `step_list`'s own
        // `:wat::core::u8` arm (a different match, tail position) is untouched by this stone.

        // Arc 255 Stone C — the old `:wat::core::i64::{+,-,*,/,mod,rem,quot}` arms
        // that lived here are RETIRED. The registry-first door above
        // (`crate::intrinsic::registry().lookup(head)`) already dispatches the
        // surviving `:wat::i64::*` spelling to `intrinsic/i64.rs`'s handlers, which
        // call the SAME shared op fns (`i64_add_op` etc., defined beside
        // `eval_i64_arith` above) this match's old arms used to call directly — so
        // there is nothing left for this match to do for i64 arithmetic; a wrong
        // per-type name is now a check-time retirement error, not a fallthrough
        // here.
        // Arc 255 Stone D — the old `:wat::core::bigint::{+,-,*,/,to-f64,to-rational}`
        // and `:wat::core::rational::{+,-,*,/,to-f64}` /
        // `:wat::core::rational/{numerator,denominator}` arms that lived here are
        // RETIRED. The registry-first door above (`crate::intrinsic::registry().lookup(head)`)
        // already dispatches the surviving `:wat::bigint::*` / `:wat::rational::*` spellings to
        // `intrinsic/bigint.rs` / `intrinsic/rational.rs`'s handlers, which call the SAME shared
        // fns (`eval_bigint_arith`, `bigint_div`, `eval_bigint_to_{f64,rational}`,
        // `eval_rational_arith`, `rational_div`, `eval_rational_{to_f64,numerator,denominator}`,
        // all still defined above/below) this match's old arms used to call directly — so there
        // is nothing left for this match to do for bigint/rational; a wrong per-type name is now
        // a check-time retirement error, not a fallthrough here.
        // arc 237 Stone 237.8a — mixed-type binary leaf arms DELETED
        // under THE DECISION (`feedback_no_implicit_coercion`).
        // +'i64'f64, -'i64'f64, *'i64'f64, /'i64'f64,
        // +'f64'i64, -'f64'i64, *'f64'i64, /'f64'i64 — all retired.
        // Their old eval helpers and Value-level inner helpers are also deleted.
        //
        // Arc 255 Stone C — the old `:wat::core::f64::{+,-,*,/,max,min,abs,clamp,
        // max-of,min-of}` arms and the old `:wat::core::{i64,f64}::to-*` scalar
        // conversion arms that lived here are RETIRED. The registry-first door
        // above already dispatches every surviving `:wat::i64::*` / `:wat::f64::*`
        // spelling to `intrinsic/i64.rs` / `intrinsic/f64.rs`, which call the same
        // shared arithmetic/conversion fns these arms used to call directly.
        // Arc 237 follow-on — derive is a no-op at runtime (edge already registered
        // at splice/pre-check time by splice_type_decls in types.rs). Accept as unit.
        ":wat::core::derive" => Ok(Value::Unit),
        // Arc 255 home #4 phase 2 (the string carve, builder-amended to all four
        // `string_ops.rs` families + the fifth unnamed one, `List/of`) — the 19
        // `:wat::string::*` verbs (including `declare-acronyms`), the 7
        // `:wat::uuid::*` verbs, `:wat::core::char`, `:wat::core::List`,
        // and `:wat::regex::matches?` are REGISTERED now (`intrinsic/string.rs`,
        // `intrinsic/uuid.rs`, `intrinsic/char.rs`, `intrinsic/list.rs`,
        // `intrinsic/regex.rs`) and resolve via the registry hoist above
        // (`crate::intrinsic::registry().lookup(head)`, this fn's `Arc 255 Stone
        // 255.1c-guard` a few dozen lines up) — this match no longer carries their
        // arms. `string_ops.rs` itself is deleted; do not re-add a match arm here for
        // any of these FQDNs, or it becomes silently-dead code the registry always
        // shadows (see that hoist's own comment).
        //
        // Arc 234 Stone 234.4 — slash-form alias for i64::to-f64 (untouched — a
        // different naming scheme, not part of this stone's `::`-retirement).
        ":wat::core::i64/to-f64" => crate::numeric::convert::eval_i64_to_f64(args, list_span, env, sym, ":wat::core::i64/to-f64"),
        // `:wat::string::to-i64` / `to-f64` / `to-bool` are REGISTERED now
        // (`intrinsic/string.rs`, arc 255 home #4 phase 2) — no arm here; see the
        // registry-hoist note a few dozen lines up this match.
        // Arc 255 `DESIGN-STONE-a-registered-row-may-not-keep-its-arm.md` — this arm RETIRED;
        // `:wat::core::bool::to-string` carries a registered handler, so the registry-first door
        // above (`crate::intrinsic::registry().lookup(head)`) already dispatches it to
        // `eval_bool_to_string` (unchanged) before this match is ever reached.
        // Arc 170 slice 3 Gap A — keyword reflection primitives. Arc 255 Stone E-iv —
        // `:wat::core::keyword/to-string` RETIRED this stone; `:wat::keyword::to-string`
        // (`src/intrinsic/keyword.rs`) is registry-routed — no arm here (the registry-first
        // door above this match, `crate::intrinsic::registry().lookup(head)`, already
        // dispatches it).

        // Comparison — return :bool
        // Stone 237.8b — `=`/`not=` stay here (migrate to 8c defclauses later).
        // Stone 245.8 — `<`/`>`/`<=`/`>=` PROMOTED from wat defclauses to relational
        // intrinsic. Runtime dispatch arms added here (routed directly to `eval_compare`).
        // The defclauses in wat/core.wat are retired; the type-locked per-Type leaves
        // (`:wat::i64::<` etc.) are registered intrinsics now (`intrinsic/i64.rs` /
        // `intrinsic/f64.rs`), dispatched via the registry-first door above — arc 255
        // Stone C retired the old `:wat::core::{i64,f64}::{<,>,<=,>=,=,not=}` arms
        // that used to live here (Stone 237.3's per-Type aliases, restored by
        // DESIGN-STONE-per-type-equality-restored.md), same `eval_compare` /
        // `eval_f64_compare` engines, just reached through the registry now.
        ":wat::core::=" => eval_eq(head, args, list_span, env, sym),
        ":wat::core::not=" => eval_not_eq(head, args, list_span, env, sym),
        ":wat::core::<" => eval_compare(head, args, list_span, env, sym, |o| {
            o == std::cmp::Ordering::Less
        }),
        ":wat::core::>" => eval_compare(head, args, list_span, env, sym, |o| {
            o == std::cmp::Ordering::Greater
        }),
        ":wat::core::<=" => eval_compare(head, args, list_span, env, sym, |o| {
            o != std::cmp::Ordering::Greater
        }),
        ":wat::core::>=" => eval_compare(head, args, list_span, env, sym, |o| {
            o != std::cmp::Ordering::Less
        }),

        // Stone 237.3 — slash-form alias for i64/to-string (probe 14).
        ":wat::core::i64/to-string" => crate::numeric::convert::eval_i64_to_string(args, list_span, env, sym, ":wat::core::i64/to-string"),

        // Arc 255 Stone F — the `String/` namespace aliases (Stone 237.3) that lived here
        // (concat/starts-with?/ends-with?/contains?/empty?) are RETIRED. Their replacement is
        // `:wat::string::*`, reached through the ordinary registry-first door above (each is a
        // `#[wat_intrinsic]` in `intrinsic/string.rs`) — no explicit match arm needed here at
        // all, unlike this Stone 237.3 shim which had to arity-check for itself. See
        // `src/remedy/retirement.rs`'s five new rows for the old-spelling error message.

        // Stone 237.8b — HARD CUT: explicit `+`/`-`/`*`/`/` arms removed.
        // These ops are now wat defclauses (registered in runtime_def_values)
        // and dispatched via the `other =>` fallback arm below.
        // The `<`/`>`/`<=`/`>=` arms were never explicit here (they routed
        // through eval_compare); those ops are now defclauses too.

        // Boolean
        // Arc 255 `DESIGN-STONE-a-registered-row-may-not-keep-its-arm.md` — this arm RETIRED;
        // `:wat::core::not` carries a registered handler, so the registry-first door above
        // (`crate::intrinsic::registry().lookup(head)`) already dispatches it to `eval_not`
        // (unchanged) before this match is ever reached.
        // Arc 255 Stone 1a-i — these two arms RETIRED; `:wat::core::and`/`:wat::core::or` carry
        // registered `role = eval` handlers now (`eval_and_tail`/`eval_or_tail`, STOP-1's
        // stacked-attribute pair), so the registry-first door above already dispatches to them
        // before this match is ever reached. The standalone `eval_and`/`eval_or` fns these arms
        // used to call are deleted (their only callers) —
        // `registry_first_door_owns_every_handler_row_no_literal_arm_survives` (`intrinsic/mod.rs`)
        // is the gate that fires if a literal arm like this survives registration.

        // List construction
        // Arc 163 slice 3d — `:wat::core::Vector` is canonical;
        // legacy `:wat::core::vec` and `:wat::core::list` runtime
        // arms retired. Type-checker Pattern 2 poison (check.rs:3840,
        // 3858) still surfaces friendly redirect for users typing
        // legacy keywords; runtime arm gone for defense-in-depth.
        ":wat::core::Vector" => {
            // Arc 109 step ① Room 3 — accept `(Vector [T] …)` alongside the existing
            // positional `(Vector :T …)`; see `crate::check::unwrap_type_param_bracket`.
            // Splice at the dispatch call site, mirroring check.rs's Room 2 arm exactly —
            // `eval_vector_ctor` itself stays untouched.
            let spliced_args = crate::check::unwrap_type_param_bracket(args);
            crate::collection::eval::eval_vector_ctor(&spliced_args, list_span, env, sym)
        }
        // Arc 146 slice 3 — `:wat::core::conj` is now a Dispatch
        // (declared in `wat/core.wat`). The dispatch_keyword_head
        // guard above intercepts before reaching this arm; the
        // per-Type impls (`:wat::core::Vector/conj` / `HashSet/conj`)
        // sit further down in this match.
        // Post-arc-165: `:wat::core::Tuple` is canonical PascalCase
        // per slice 1f's vec→Vector playbook completed. Legacy
        // `:wat::core::tuple` arm retired; Pattern 2 poison in
        // check.rs handles any remaining consumer sites at type-check.
        //
        // Arc 109 step ①b Room 3 — accept `(Tuple :- [T1 T2 …] …)`. Still NOT wired to
        // `crate::check::unwrap_type_param_bracket` (splicing would evaluate the bracket's
        // type keywords as VALUES — same reasoning as step ①'s STOP-3, unchanged). Instead:
        // strip a genuine leading bracket via `crate::check::split_type_param_bracket` —
        // the SAME discriminator check.rs's `infer_tuple_constructor` uses, so check and
        // eval never disagree on which forms have a bracket. A literal `WatAST::Vector`
        // that is NOT a type-keyword bracket (e.g. `(Tuple [1 2 3] "tag")`,
        // `tests/collection/probe_arc216_stone7_tuple_roundtrip.rs`) is left as an ordinary
        // first element, unchanged. Types are erased at runtime (mirrors `eval_ann_form`) —
        // once check.rs has validated the bracket, only the VALUES matter here.
        //
        // Stone ②-i-b — one case `eval_tuple_ctor` cannot take: a `:-`-declared EMPTY
        // bracket (`(Tuple :- [])`) strips to zero values, and `eval_tuple_ctor` treats
        // `args.is_empty()` as the illegal bare `(Tuple)` head. A `:-`-declared empty
        // bracket is different: it is the empty tuple VALUE this stone makes writable
        // (measured: today `(Tuple [])` — a literal Vector element, not a param-spec,
        // since arc 109 "THE LAST DOORS" retired bracket-sniffing entirely — builds
        // `[[]]`, a 1-tuple holding an empty vector; only `(Tuple :- [])` now means the
        // empty tuple). Build it directly here rather than teaching `eval_tuple_ctor` to
        // disambiguate "no bracket, zero args" from "bracket, zero args" — it cannot, by
        // the time it sees only `values`.
        ":wat::core::Tuple" => {
            match crate::check::split_type_param_bracket(args) {
                // The empty tuple is a ZERO-LENGTH param-spec with zero values —
                // `(Tuple :- [])`. Guard on `inner` too, not `rest` alone: a
                // declared-but-unpopulated `(Tuple :- [A B])` is an arity mismatch
                // (check.rs `infer_tuple_constructor` checks bracket arity against
                // VALUE arity), and answering it with an empty tuple here would be a
                // check-says-no / runtime-says-yes divergence — the exact class step
                // ①b's Room 3 was found by. `inner.is_empty()` also confines this arm
                // to the `:-` spelling for free: `split_type_param_bracket` only ever
                // returns `Some` for the `:-`-marked spelling now.
                Some((inner, _bspan, rest)) if inner.is_empty() && rest.is_empty() => {
                    Ok(Value::Tuple(Arc::new(vec![])))
                }
                Some((_inner, _bspan, rest)) => eval_tuple_ctor(rest, list_span, env, sym),
                None => eval_tuple_ctor(args, list_span, env, sym),
            }
        }
        // ═══ PARTITION (runtime side) — see the CLAUSE vs INTRINSIC marker in
        // check.rs `infer_list`. INTRINSIC = type-level computation; two flavors.
        // See `docs/DISPATCH.md`.
        //
        //   PROJECTIVE — type flows from argument type parameters into the return.
        //   The per-Type collection impls below (`eval_<container>_<op>`, routed by
        //   `dispatch_keyword_head`) are the runtime arm: `get`/`conj`/`assoc`/
        //   `contains`/`length`/`empty?` whose return depends on the container's
        //   type params — a `defclause` cannot express them. Warded home: arc 246
        //   (`src/collection/`).
        //
        //   RELATIONAL — a constraint flows between the arguments (∀T). Equality
        //   (`eval_eq` / `eval_not_eq`, via `:wat::core::=` / `:wat::core::not=`
        //   above) is the runtime arm: `values_equal` handles all T structurally.
        //
        //   NOTE: arc 241.13 retired `:wat::core::define-dispatch`; routing is via
        //   `dispatch_keyword_head` + custom `infer_*` arms, not a wat-declared entity.
        // ══════════════════════════════════════════════════════════════════════════
        // Arc 146 slice 2 — `:wat::core::length` is now a Dispatch
        // (declared in `wat/core.wat`). The dispatch routes to one of
        // these per-Type impls by inspecting the arg's value tag.
        // Direct calls to the per-Type names are also legal and
        // bypass the dispatch hop.
        // Arc 255 Stone E-ii — `:wat::core::Vector/length` and `:wat::core::PersistentVector/length`
        // RETIRED this stone; `:wat::vec::length`/`:wat::vector::length` (`src/intrinsic/{vec,vector}.rs`)
        // are their replacements, reached via the registry-first door (`dispatch_keyword_head_value`'s
        // `crate::intrinsic::registry().lookup(head)`, above this match) — no arm needed here.
        // Arc 255 Stone E-iii — `:wat::core::HashSet/length` and `:wat::core::List/length` RETIRED
        // this stone; `:wat::hashset::length`/`:wat::linkedlist::length`
        // (`src/intrinsic/{hashset,linkedlist}.rs`) are their replacements, reached the same
        // registry-first way — no arm needed here either.
        //
        // Arc 146 slice 3 — empty? / contains? / get / conj are now
        // Dispatches (declared in `wat/core.wat`). Per-Type impls also
        // directly callable; the dispatch_keyword_head guard above
        // intercepts the polymorphic surface name first.
        // Arc 255 Stone E-ii — `:wat::core::Vector/empty?` and `:wat::core::PersistentVector/empty?`
        // RETIRED this stone; `:wat::vec::empty?`/`:wat::vector::empty?` are their replacements.
        // Arc 255 Stone E-iii — `:wat::core::HashSet/empty?` and `:wat::core::List/empty?` RETIRED
        // this stone; `:wat::hashset::empty?`/`:wat::linkedlist::empty?` are their replacements.
        //
        // Arc 255 Stone E-ii — `:wat::core::Vector/contains?` and `:wat::core::PersistentVector/contains?`
        // RETIRED this stone; `:wat::vec::contains?`/`:wat::vector::contains?` are their replacements.
        // Arc 255 Stone E-iii — `:wat::core::HashSet/contains?` and `:wat::core::List/contains?`
        // RETIRED this stone; `:wat::hashset::contains?`/`:wat::linkedlist::contains?` are their
        // replacements.
        //
        // Arc 255 Stone E-ii — `:wat::core::Vector/get` and `:wat::core::PersistentVector/get`
        // RETIRED this stone; `:wat::vec::get`/`:wat::vector::get` are their replacements.
        // Arc 255 Stone E-iii — `:wat::core::List/get` RETIRED this stone; `:wat::linkedlist::get`
        // is its replacement. (HashSet has no direct-call `get` verb — its "get-by-equality" is
        // `contains?`, reached only via the generic `:wat::core::get` polymorphic surface.)
        //
        // Arc 255 Stone E-ii — `:wat::core::Vector/conj` and `:wat::core::PersistentVector/conj`
        // RETIRED this stone; `:wat::vec::conj`/`:wat::vector::conj` are their replacements.
        // Arc 255 Stone E-iii — `:wat::core::HashSet/conj` and `:wat::core::List/conj` (PREPENDS —
        // Clojure semantic, distinct from Vector's/HashSet's APPEND/insert `conj`) RETIRED this
        // stone; `:wat::hashset::conj`/`:wat::linkedlist::conj` are their replacements.
        //
        // Arc 146 slice 4 — per-Type assoc / dissoc / keys / values / concat.
        // Single-impl-per-container ops. Surface short names
        // (:assoc / :dissoc / :keys / :values / :concat) become user-define
        // aliases via `wat/core-aliases.wat`; they delegate to these per-Type
        // impls (each is also directly callable as `:HashMap/assoc` etc.).
        // Arc 255 Stone E-ii — `:wat::core::Vector/concat`, `:wat::core::Vector/extend`, and
        // `:wat::core::PersistentVector/concat` RETIRED this stone; `:wat::vec::concat`,
        // `:wat::vec::extend`, and `:wat::vector::concat` are their replacements.
        // Arc 255 Stone P6-c-W6 — `:wat::core::reverse`/`range` moved into `#[wat_intrinsic]`
        // handlers (`src/collection/transform.rs`, in place) with their real (1/2) arities
        // declared; the pre-match registry check above (arc 255.1c-guard) intercepts both
        // names before reaching here.
        // Arc 255 Stone the-collection-readers — `:wat::core::take`/`drop` moved into
        // `#[wat_intrinsic]` handlers (`src/intrinsic/collection.rs`), thin delegates over
        // `eval_vec_take`/`eval_vec_drop` (`src/collection/transform.rs`, in place, unmoved);
        // the pre-match registry check above (arc 255.1c-guard) intercepts both names before
        // reaching here.
        // Arc 255 Stone A-2-ii-b — `:wat::core::sort$native` moved into a `#[wat_intrinsic]`
        // handler (`src/intrinsic/collection.rs`), a thin delegate over
        // `crate::collection::transform::eval_vec_sort_by` (in place, unmoved — the gate this
        // stone shipped lives there); the pre-match registry check above (arc 255.1c-guard)
        // intercepts the name before reaching here.
        ":wat::core::map" => crate::collection::transform::eval_vec_map(args, list_span, env, sym),
        ":wat::core::mapv" => crate::collection::transform::eval_mapv(args, list_span, env, sym),
        // Arc-278 DESIGN-STONE seq-traversal-one-door, Strike 1 — the private eager→lazy
        // normalizer, native now (was a wat `defclause`, `wat/seq.wat`, that stepped its
        // source via repeated `rest` — O(n^2) on every eager container). Steps by position;
        // see `eval_seqable_to_stream`'s doc for the per-container shape.
        ":wat::core::seqable->stream" => {
            crate::collection::transform::eval_seqable_to_stream(args, list_span, env, sym)
        }
        ":wat::core::foldl" => {
            crate::collection::transform::eval_vec_foldl(args, list_span, env, sym)
        }
        // Arc-278 DESIGN-STONE seq-traversal-one-door, Strike 2a — `:wat::core::filter` is
        // native again (was the arc-118.2a wat `defclause`, `wat/seq.wat`, which stepped its
        // source via repeated `rest` — O(n^2) on every eager container). Composes through
        // `seqable->stream`'s per-container normalization; see `eval_filter`'s doc.
        ":wat::core::filter" => {
            crate::collection::transform::eval_filter(args, list_span, env, sym)
        }
        // Arc 255 Stone HOME-9 moved `:wat::seq::{zip,window,remove-at}` off the dead
        // `:wat::std::list::` namespace and made them Seqable-generic (Vector |
        // PersistentVector | List | Stream). Arc 255 Stone HOME-10 carved their dispatch
        // arms into `#[wat_intrinsic]` handlers (`src/intrinsic/seq.rs`) — the pre-match
        // registry check above (arc 255.1c-guard) intercepts all three names before
        // reaching here, same shape as `:wat::time::*` a few dozen lines up.
        ":wat::core::HashMap" => {
            // Arc 109 step ① Room 3 — accept `(HashMap [K V] …)` alongside the existing
            // positional `(HashMap :K :V …)`; see `crate::check::unwrap_type_param_bracket`.
            let spliced_args = crate::check::unwrap_type_param_bracket(args);
            crate::collection::eval::eval_hashmap_ctor(&spliced_args, list_span, env, sym)
        }
        // Arc 109 step ①b Room 3 — accept `(PersistentMap :- [K V] …)`. Still NOT wired
        // to `crate::check::unwrap_type_param_bracket` (splicing would misalign the
        // `args.chunks(2)` pairing, same as check-time — unchanged reasoning). Instead:
        // strip a genuine leading bracket via `crate::check::split_type_param_bracket`,
        // the same discriminator `infer_persistentmap_constructor` uses at check time, so
        // check and eval agree on which forms have a bracket. Types are erased at
        // runtime; `eval_persistentmap_ctor` itself stays untouched.
        ":wat::core::PersistentMap" => {
            let values = match crate::check::split_type_param_bracket(args) {
                Some((_inner, _bspan, rest)) => rest,
                None => args,
            };
            crate::collection::eval::eval_persistentmap_ctor(values, list_span, env, sym)
        }
        // Arc 109 step ①b Room 3 — accept `(PersistentVector :- [T] …)`. Same reasoning
        // and mechanism as `PersistentMap` above: strip a genuine leading bracket via
        // `crate::check::split_type_param_bracket`; `eval_persistentvector_ctor` itself
        // stays untouched.
        ":wat::core::PersistentVector" => {
            let values = match crate::check::split_type_param_bracket(args) {
                Some((_inner, _bspan, rest)) => rest,
                None => args,
            };
            crate::collection::eval::eval_persistentvector_ctor(values, list_span, env, sym)
        }
        ":wat::core::HashSet" => {
            // Arc 109 step ① Room 3 — accept `(HashSet [T] …)` alongside the existing
            // positional `(HashSet :T …)`; see `crate::check::unwrap_type_param_bracket`.
            let spliced_args = crate::check::unwrap_type_param_bracket(args);
            crate::collection::eval::eval_hashset_ctor(&spliced_args, list_span, env, sym)
        }
        // Arc 146 slice 3 — `:wat::core::get` and `:wat::core::contains?`
        // are now Dispatches (declared in `wat/core.wat`). The
        // dispatch_keyword_head guard above intercepts them; the
        // per-Type impls (`Vector/get`, `HashMap/get`,
        // `Vector/contains?`, `HashMap/contains-key?`,
        // `HashSet/contains?`) sit in the per-Type block above.
        //
        // Arc 146 slice 4 — `:wat::core::concat` / `:assoc` / `:dissoc` /
        // `:keys` / `:values` retired here; each is now a user-define
        // alias (declared in `wat/core-aliases.wat`) that delegates to
        // the per-Type impl (`:HashMap/assoc` etc., `:Vector/concat`).
        // Aliases dispatch through env.get; the per-Type primitives
        // sit in the per-Type block above.
        // :wat::io:: — abstract IO substrate (arc 008 slice 2) — CLOSED.
        // Every `:wat::io::` verb (`IOReader/*`, `IOWriter/*`, the two RAII
        // temp handles, the filesystem one-shots) has been carved to
        // `#[wat_intrinsic]` registrations — see `src/intrinsic/io/`
        // (`reader.rs`, `writer.rs`, `fs.rs`). No `:wat::io::` literal-match
        // arm remains here. `:wat::stdlib::sources`, below, is a different
        // family (arc 275 Stone 275.1's baked stdlib load order) and was
        // never carved here.
        // Arc 275 Stone 275.1 — baked stdlib load order for deporder.
        ":wat::stdlib::sources" => {
            crate::io::eval_stdlib_sources(args, list_span, env, sym).map_err(Into::into)
        }
        // Arc 255 Stone HOME-8 — every `:wat::holon::*` verb (the algebra-core
        // constructors, classifier predicates/projections, the term/Thermometer
        // surface, `Hologram/*`, the measurement primitives, `OnlineSubspace/*`,
        // `Reckoner/*`, `Engram*/*`) has been carved to `#[wat_intrinsic]`
        // registrations — see `src/intrinsic/holon/` (`atom.rs`, `hologram.rs`,
        // `engram.rs`, `subspace.rs`, `reckoner.rs`). No `:wat::holon::` literal-
        // match arm remains here; the registry-first door at the top of
        // `dispatch_keyword_head`/`dispatch_keyword_head_value` reaches them all.
        // Arc 255 `DESIGN-STONE-a-registered-row-may-not-keep-its-arm.md` — this arm RETIRED;
        // `:wat::core::show` carries a registered handler, so the registry-first door above
        // (`crate::intrinsic::registry().lookup(head)`) already dispatches it to `eval_show`
        // (unchanged) before this match is ever reached.
        // Arc 279 — unquoted display: String→itself, i64/f64/bool→digits. Unlike `show`,
        // which wraps strings in `"..."`, `str` renders values as format fills them.
        ":wat::core::str" => eval_str(args, list_span, env, sym),
        // Arc 255 Stone HOME-11 — the remaining 9 `:wat::edn::` verbs (the 4 `write*`
        // renderers, and the 5 `ForeignRecord`/`ForeignVariant` accessors) RETIRED as literal
        // arms this stone; registry-routed via `src/intrinsic/edn.rs`. The registry-first door
        // at the top of `dispatch_keyword_head`/`dispatch_keyword_head_value` reaches them all
        // (same shape the `:wat::holon::*` carve comment above documents).

        // Constrained runtime eval — four forms, matching the load
        // pipeline's discipline on source interface and verification.
        //
        // STONE-the-binder-must-be-universal (arc 109) — all TEN of these root-level
        // eval forms pass through this one cluster, so the call-site `:- […]` binder
        // (arc 109's fourth position) is peeled HERE, once, rather than inside each
        // helper below. Types are erased at runtime — check.rs has already validated
        // and bound the binder against the callee's declared type params (that is why
        // the call type-checks clean today even though the runtime previously refused
        // it) — so every helper below only ever needs the value ARGS. Peeling once
        // means a form added later to this cluster inherits the fix instead of
        // re-earning the bug; peeling inside each helper was ten edits, ten chances to
        // miss one, and no guarantee for an eleventh.
        head @ (":wat::eval-ast!" | ":wat::eval-with-defs!" | ":wat::eval-step!"
        | ":wat::eval::walk" | ":wat::eval-edn!" | ":wat::eval-file!" | ":wat::eval-digest!"
        | ":wat::eval-digest-string!" | ":wat::eval-signed!" | ":wat::eval-signed-string!") => {
            let (_binder, args) = crate::types::peel_param_spec(args);
            match head {
                ":wat::eval-ast!" => eval_form_ast(args, env, sym, list_span),
                // Arc 170 — the sibling that supplies the WORLD as well as the form.
                ":wat::eval-with-defs!" => eval_form_with_defs(args, env, sym, list_span),
                ":wat::eval-step!" => eval_form_step(args, env, sym, list_span),
                ":wat::eval::walk" => eval_walk(args, env, sym, list_span),
                ":wat::eval-edn!" => eval_form_edn(args, env, sym, list_span),
                ":wat::eval-file!" => eval_form_file(args, env, sym, list_span),
                ":wat::eval-digest!" => eval_form_digest(args, env, sym, list_span),
                ":wat::eval-digest-string!" => eval_form_digest_string(args, env, sym, list_span),
                ":wat::eval-signed!" => eval_form_signed(args, env, sym, list_span),
                ":wat::eval-signed-string!" => eval_form_signed_string(args, env, sym, list_span),
                _ => unreachable!("outer match arm restricts head to these ten forms"),
            }
        }

        // Kernel primitives — channel IO + stop flag + user signals.
        // Arc 255.1c-kernel-ambient — stopped?/sigusr1?/sigusr2?/sighup?/reset-sigusr1!/
        // reset-sigusr2!/reset-sighup! moved to the intrinsic registry
        // (`src/intrinsic/kernel/ambient.rs`); dispatch now reaches them via the
        // registry lookup above, not a literal arm here.
        // Arc 255.1c-kernel-remainder (home #8) — call-site/macro-call-site moved to the
        // intrinsic registry (`src/intrinsic/kernel/source.rs`); dispatch now reaches
        // them via the registry lookup above, not a literal arm here.
        // Arc 255.1c-kernel-stdio — println/pprintln/eprintln/epprintln/readln'/read-frame
        // moved to the intrinsic registry (`src/intrinsic/kernel/stdio.rs`); dispatch now
        // reaches them via the registry lookup above, not a literal arm here.
        // Arc 255.1c-kernel-resource — HandlePool::{new,pop,finish}, pipe,
        // spawn-thread, spawn-process, after, close, signal, listener, connect,
        // accept, allow, deny (fifteen verbs, `:Resource`'s whole population) moved
        // to the intrinsic registry (`src/intrinsic/kernel/resource.rs`); dispatch
        // now reaches them via the registry lookup above, not a literal arm here.
        // :wat::kernel::spawn / :wat::kernel::join / :wat::kernel::join-result
        // retired in arc 114. spawn-thread + Thread/join-result are the
        // canonical replacements; the type-checker poisons every call site
        // with a self-describing migration hint. Runtime impls deleted
        // alongside the dispatch — no callers reach this layer post-poison.
        // Arc 255.1c-kernel-error — LociDiedError/message, Failure/message,
        // Failure/location, LociDiedError/to-failure moved to the intrinsic
        // registry (`src/intrinsic/kernel/error.rs`); dispatch now reaches
        // them via the registry lookup above, not a literal arm here.
        // Arc 170 CULMINATION (arc 278 IPC de-prime) — `:wat::kernel::extract-panics`
        // ANNIHILATED with the run-sandboxed family (its only callers were the
        // deleted manual sandbox drivers; the primed peer wire delivers the
        // LociDiedError chain directly via recv' Lost, no stderr-scrape needed).
        // Arc 105c — substrate `:wat::kernel::run-sandboxed` /
        // `-ast` dispatch arms are GONE. The wat-level defines in
        // `wat/kernel/sandbox.wat` (bundled in `src/stdlib.rs`) atop
        // arc 105a's Result-returning spawn-program +
        // arc 105b's ThreadDiedError/message accessor are now
        // canonical. Vec<String> exits the substrate boundary; the
        // wat-level helper is the only place collected-output-as-
        // Vec<String> survives, where it earns its keep as the
        // test assertion target.
        // Arc 255.1c-kernel-remainder (home #8) — assertion-failed!/raise! moved to the
        // intrinsic registry (`src/intrinsic/kernel/abort.rs`); here/fn-forms moved to
        // (`src/intrinsic/kernel/source.rs`); dispatch now reaches them via the registry
        // lookup above, not a literal arm here.
        // Arc 259 S2c-ii-b — spawn-program' is now a wat defclause in wat/spawn.wat.
        // The 3-arg Rust intrinsic is RETIRED; the defclause dispatches on the host
        // type (ThreadOpts → spawn-thread'; ProcessOpts → spawn-process').
        // Arc 259 S2c-i — per-tier 1-arg primitives (no tier keyword, no env arg).
        // spawn-thread' : fn([(Peer' :- [S R])]) -> nil -> (Thread' :- [R S])
        // spawn-process' : forms -> (Process' :- [I O])
        // Both delegate to the shared spawn_thread_peer / spawn_process_peer helpers.
        // Arc 255.1c-kernel-message — send/try-send/recv/select/poll moved to the
        // intrinsic registry (`src/intrinsic/kernel/message.rs`); dispatch now
        // reaches them via the registry lookup above, not a literal arm here.
        //
        // Arc 214 Stone 4.6a-ii — close': intrinsic (∀-parametric: (peer :- [∀I ∀O]));
        // see docs/DISPATCH.md + check.rs ~4814 for the CLAUSE-vs-INTRINSIC
        // partition. Downcasts the peer RustOpaque by sentinel (Thread' first,
        // then Process', else TypeMismatch).
        // DESIGN-STONE-process-signal-owner-to-child.md; BRIEF-process-signal-p2-mint.md
        // — owner-to-child signal delivery. STOP-1: (Process :- [I O]) only, no shared
        // codegen with Thread'/Peer'. STOP-3: routes through Pidfd::send_signal, never
        // kill(pid, sig). See eval_signal.
        // Arc 255.1c-kernel-remainder (home #8) — peer-process/peer-wire?/address-wire?/
        // require-wire-address moved to the intrinsic registry
        // (`src/intrinsic/kernel/identity.rs`); dispatch now reaches them via the
        // registry lookup above, not a literal arm here.
        // Arc 209 Stone C0b.1 — thread-tier connection: listener'/connect'/accept'.
        // listener' mints the crossbeam rendezvous (Listener'=rx, Address'=tx).
        // connect' mints the connection pairs, wraps the client Peer' end locally,
        // ships the server's raw halves over the rendezvous.
        // accept' receives the server's raw halves from the rendezvous, wraps the
        // server Peer' end on this thread.  No Peer' cell ever crosses a thread.
        // Arc 255.1c-kernel-remainder (home #8) — peer-pid moved to the intrinsic
        // registry (`src/intrinsic/kernel/identity.rs`); dispatch now reaches it via
        // the registry lookup above, not a literal arm here. Still type-invisible to
        // `check.rs` (no scheme, no `infer_*` arm) — registration documents the verb,
        // it does not close that hole (task #110 / 255.1b-iv).
        // Arc 209 C0b.3b-b — allow'/deny': mutate the SocketListener's allow-set.
        // allow' : [(Listener' :- [S R]) i64 :-> nil]  — insert pid; process-tier only.
        // deny'  : [(Listener' :- [S R]) i64 :-> nil]  — remove pid; process-tier only.
        // :wat::kernel::wait-child retired in arc 112 — replaced by
        // :wat::kernel::Process/join-result returning (Result :- [()
        // ProcessDiedError]). The orphaned eval body in src/fork.rs
        // was removed in arc 214 Stone 6.2.
        // Arc 255.1c-kernel-ambient — sigusr1?/sigusr2?/sighup?/reset-sigusr1!/
        // reset-sigusr2!/reset-sighup! (plus stopped? above, near call-site) moved to
        // the intrinsic registry (`src/intrinsic/kernel/ambient.rs`); dispatch now
        // reaches them via the registry lookup above, not a literal arm here.

        // :wat::core::use! — resolve-pass declaration, no-op at runtime.
        // Validation happens during resolve; by the time eval runs, the
        // declaration has done its job. Returns :() for the value
        // position (if a user writes it inside an expression — unusual
        // but not illegal).
        // ⛔ Arc 255 Stone 1a-ε — the `":wat::core::use!" => Ok(Value::Unit)` arm that stood here
        // is DELETED, for the same reason and by the same gate as the two config setters above:
        // `use!` is a registered row with a `role = eval` handler, so the registry-first door
        // answers it and this arm was unreachable. The no-op moved to
        // `intrinsic/special/use_form.rs`; the declaration's real work happens at the resolve
        // pass (`collect_use_declarations`), which that row names as its `role = declare`.

        // Config accessors (:wat::config::dim-count/dim-capacity/global-seed/noise-floor) —
        // arc 255 Stone P6-c-W1 moved their dispatch arms into `#[wat_intrinsic]` handlers
        // (`src/intrinsic/config.rs`); the pre-match registry check above (arc 255.1c-guard)
        // intercepts all four names before reaching here, same shape as `:wat::math::*` a
        // few dozen lines down.

        // Stdlib math (:wat::math::ln/exp/sqrt/sin/cos/pi) and stat
        // (:wat::stat::mean/variance/stddev) — arc 255 Stone HOME-9 moved these off the dead
        // `:wat::std::` namespace; arc 255 Stone HOME-10 carved their dispatch arms into
        // `#[wat_intrinsic]` handlers (`src/intrinsic/math.rs`, `src/intrinsic/stat.rs`). The
        // pre-match registry check above (arc 255.1c-guard) intercepts all 9 names before
        // reaching here, same shape as `:wat::time::*` a few dozen lines down.
        // `log` is DELETED, not moved: it was wired to the SAME `f64::ln` as `ln` (a level-1
        // lie), had zero call sites, and does not carry forward under a new address.

        // Time primitives — arc 056/097, carved to the registry at
        // `src/intrinsic/time.rs` (arc 255.1c-time, home #2). The
        // pre-match registry check above (arc 255.1c-guard) intercepts
        // all 41 `:wat::time::*` names before reaching here.

        // :rust::* — dispatch through the rust-deps registry. Each
        // symbol's shim handles its own arg evaluation and marshaling.
        other if other.starts_with(":rust::") => {
            // rune:sequi(ambient-context) — rust-deps registry is a write-once dispatch
            // table installed at startup; threading it through every resolver/eval
            // signature would bloat every call site for a read-only config surface,
            // not domain state.
            let registry = crate::rust_deps::registry();
            match registry.get_symbol(other) {
                Some(sym_entry) => (sym_entry.dispatch)(args, env, sym).map_err(Into::into),
                None => Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::UnknownFunction(format!(
                        "{} is not registered in the rust-deps registry",
                        other
                    )),
                )
                .into()),
            }
        }

        // Anything else: user-defined function lookup.
        other => {
            // STONE-exactly-one-call-position (arc 109) — position 4's runtime peel
            // (previously below, at the generic user-function dispatch only) is hoisted
            // to the TOP of this whole arm, before anything branches on whether the
            // head is a surface-method call (`other.contains('/')`). The surface-method
            // dispatch block just below used to see the RAW `args` — a call carrying
            // `:- [T…]` had the marker land in `args[0]`/`args[1]` where the receiver
            // was expected. Types are erased at runtime — nothing here needs the peeled
            // type nodes themselves (check.rs already validated and bound them against
            // the callee's declared type params); every consumer below (the Peer-nature
            // send/recv forwarding, the aggregate-dispatch receiver read, and the
            // generic user-function lookup) only needs the REST.
            let (_, args) = crate::types::peel_param_spec(args);

            if other.contains('/') {
                let protocol_fqdn = wat_reader::identifier::receiver(other);
                // STONE reap-the-angle-machinery (arc 109) — Stone 6b-DEP used to strip an
                // explicit type-args suffix `<T1,T2>` off the call-head's method name here
                // (`mk<i64,i64>` → bare `mk`). That embedded-in-the-call-head spelling is
                // unexpressible now (a keyword containing `<` is a LEXER error, verified
                // directly), and explicit type args travel as the separate `:- [T…]` marker
                // peeled by `peel_param_spec` just above — so `method_name_raw` can never
                // carry a suffix; use it directly.
                let method_name = wat_reader::identifier::method(other);

                // Arc 293.4b — surface-method dispatch.
                // Arc 293.4c — also handles foreign types taught via `extend-type` (monkeypatch).
                // Arc 293.4d — broadened to Field members too (every surface member is an accessor).
                //
                // A head `:S/name` where `S` is a `TypeDef::Surface` with ANY member named `name`
                // (Field OR Method) routes to `:<T>/<name>` in `sym.functions`. For a record's
                // Field member, `:<T>/<field>` is the auto-generated field accessor registered by
                // `register_record_methods`; for a Method member (defn or extend-type), the same
                // key holds. The check layer has already verified satisfaction; here we only need
                // the concrete FQDN. Both paths use the identical dispatch lookup.
                //
                // `sym.types` is populated at freeze time (`FrozenWorld::freeze` → `symbols.set_types`).
                if let Some(types) = sym.types() {
                    if let Some(crate::types::TypeDef::Surface(s)) = types.get(protocol_fqdn) {
                        if let Some(member) = s.members.iter().find(|m| match m {
                            crate::types::SurfaceMember::Method { name, .. } => name == method_name,
                            crate::types::SurfaceMember::Field { name, .. } => name == method_name,
                        }) {
                            // Arc 293 S3-Nature-4 (Path B) — a `:nature :Peer` surface has no
                            // aggregate satisfier to look up; instead it COMPOSES the generic
                            // `send'`/`recv'` peer primitives with its own S1-synthesized
                            // `Op`/`Reply` enums: `(let [__op (:S::Op::<Variant> req) _ (send'
                            // peer __op) __r (recv' peer)] (match __r -> <ret> ((:S::Reply::
                            // <Variant> resp) resp)))`. This branch fires ONLY for
                            // `Nature::Peer` — every other nature (aggregate dispatch) falls
                            // through to the unchanged `:<T>/<method>` lookup below.
                            if s.nature == Some(crate::types::Nature::Peer) {
                                if let crate::types::SurfaceMember::Method { ret, .. } = member {
                                    use crate::scope::Identifier;
                                    if args.len() < 2 {
                                        return Err(RuntimeError::new(
                                            list_span.clone(),
                                            RuntimeErrorKind::ArityMismatch {
                                                op: other.to_string(),
                                                expected: 2,
                                                got: args.len(),
                                            },
                                        )
                                        .into());
                                    }
                                    let variant = crate::string::kebab_to_pascal_with_acronyms(
                                        method_name,
                                        &[],
                                    );
                                    let op_ctor = format!("{}::Op::{}", protocol_fqdn, variant);
                                    let reply_ctor =
                                        format!("{}::Reply::{}", protocol_fqdn, variant);
                                    // DESIGN-STONE-the-client-validates-locally.md — Path B is a
                                    // SECOND, independently-drifting copy of the send-then-recv
                                    // forwarding `wat/service.wat`'s `op-methods` also builds (this
                                    // is the mechanism `:S/method` calls ACTUALLY run through — every
                                    // corpus fixture calls the SURFACE name, never the service's own
                                    // `<fqdn>/<op>`). It needs the identical local-budget strike:
                                    // `cap_const_kw` mirrors `build_op_budget_constants`'s emitted
                                    // name exactly (`src/types.rs:3041`).
                                    //
                                    // `rtl_ctor_kw` — NOT a guess. `ret` is the op's declared
                                    // return type, READ (not looked up) straight from the
                                    // `SurfaceMember::Method` this `member` binding already holds —
                                    // the same field `synthesize_surface_protocol`'s RTL lock reads
                                    // (`src/types.rs` ~2804, `if let TypeExpr::Path(resp_path) = ret`).
                                    // Arc 278 #74 — this comment's ORIGINAL justification is now
                                    // RETIRED and is recorded here rather than silently dropped: it
                                    // read *"a response type is free to be named anything"*, citing
                                    // `probe-repl-durable-forms.wat`'s `EvalResponse`. Both halves
                                    // are false today. The builder ruled the name into LAW
                                    // (`<Op>Response`, checker-enforced in
                                    // `synthesize_surface_protocol`), and that probe is now the
                                    // fixture PROVING the refusal
                                    // (`tests/services/probe_arc278_repl_durable_forms_response_law.wat.bad`).
                                    // Path B's CODE is unaffected and needs no change — it already
                                    // READ `ret` rather than guessing, which is why the law found
                                    // nothing to fix here. Reading the declaration stays correct;
                                    // it is simply no longer the only thing standing between us and
                                    // a wrong name. `Path`/`Parametric`
                                    // both carry the base name with no `<...>` suffix baked in
                                    // (unlike the wat-level string split elsewhere in this codebase,
                                    // Rust's `TypeExpr` keeps head and args structurally separate) —
                                    // but they do NOT carry the leading `:` the same way, and the
                                    // difference is DELIBERATE, not an accident: a parametric
                                    // head is stored BARE so its two parse paths (`(Head :- [args])`
                                    // and `(Ctor arg…)`) produce a byte-identical string for
                                    // unification (src/types.rs ~4450; stated outright at ~4287,
                                    // "We must produce the SAME string for unification"). `Path`
                                    // re-prepends the colon instead (`format!(":{}", s)`, ~4494)
                                    // — a different, equally intentional convention for a
                                    // different variant. Normalize HERE, at this one read site,
                                    // never upstream in the parser/storage: a parametric response
                                    // (e.g. `(PCache::GetResponse :- [K V])`) fed a colon-less `head`
                                    // into this keyword string, missing its first real character.
                                    let resp_base_raw: &str = match ret {
                                        crate::types::TypeExpr::Path(p) => p.as_str(),
                                        crate::types::TypeExpr::Parametric { head, .. } => {
                                            head.as_str()
                                        }
                                        // A serviceable op's Response is locked to Path/Parametric by
                                        // `synthesize_surface_protocol`'s RTL enforcement; this arm is
                                        // unreached in practice and only guards against a genuinely
                                        // malformed declaration reaching here some other way.
                                        _ => protocol_fqdn,
                                    };
                                    let resp_base =
                                        crate::types::parametric_head_fqdn(resp_base_raw);
                                    let cap_const_kw = format!(
                                        "{}::{}-MAX-REQUEST-BYTES",
                                        protocol_fqdn,
                                        method_name.to_uppercase()
                                    );
                                    let rtl_ctor_kw = format!("{resp_base}::RequestTooLarge");
                                    let span = list_span.clone();

                                    // Eval the peer + request ONCE (avoids double-evaluating the
                                    // caller's arg expressions); bind them into a child env that
                                    // the synthesized forwarding AST references by name.
                                    let peer_val = eval_inner(&args[0], env, sym)?.value_owned();
                                    let req_val = eval_inner(&args[1], env, sym)?.value_owned();
                                    let call_env = env
                                        .child()
                                        .bind_unknown_span("__peer", TrackedValue::from(peer_val))
                                        .bind_unknown_span("__req", TrackedValue::from(req_val))
                                        .build();

                                    let send_recv_ast = WatAST::List(vec![
                                        WatAST::Keyword(":wat::core::let".into(), span.clone()),
                                        WatAST::Vector(vec![
                                            WatAST::Symbol(Identifier::bare("__op"), span.clone()),
                                            WatAST::List(vec![
                                                WatAST::Keyword(op_ctor, span.clone()),
                                                WatAST::Symbol(Identifier::bare("__req"), span.clone()),
                                            ], span.clone()),
                                            WatAST::Symbol(Identifier::bare("__send"), span.clone()),
                                            WatAST::List(vec![
                                                WatAST::Keyword(":wat::kernel::send".into(), span.clone()),
                                                WatAST::Symbol(Identifier::bare("__peer"), span.clone()),
                                                WatAST::Symbol(Identifier::bare("__op"), span.clone()),
                                            ], span.clone()),
                                            WatAST::Symbol(Identifier::bare("__r"), span.clone()),
                                            WatAST::List(vec![
                                                WatAST::Keyword(":wat::kernel::recv".into(), span.clone()),
                                                WatAST::Symbol(Identifier::bare("__peer"), span.clone()),
                                            ], span.clone()),
                                        ], span.clone()),
                                        WatAST::List(vec![
                                            // Arc 278 the recv'-outcome wall — `recv'` returns a
                                            // matchable `(RecvOutcome :- [Reply])`, NEVER a raise. This
                                            // Path-B intrinsic RE-WRAPS it into a
                                            // `(RecvOutcome :- [<Op>Response])` the caller faces as a value
                                            // (we are ADT; no try/catch): ::Message unwraps the reply
                                            // to its Response and re-wraps as ::Message; ::Lost maps
                                            // to a REASON-FREE ::Lost (arc-294 client = reason-free
                                            // 500 — the client never gets the service's internal
                                            // cause; the owner keeps the full cause on its crash
                                            // channel); ::Closed passes through.
                                            WatAST::Keyword(":wat::core::match".into(), span.clone()),
                                            WatAST::Symbol(Identifier::bare("__r"), span.clone()),
                                            // ::Message arm — unwrap the reply variant to its
                                            // Response, re-wrap in RecvOutcome::Message.
                                            WatAST::List(vec![
                                                WatAST::List(vec![
                                                    WatAST::Keyword(":wat::kernel::RecvOutcome::Message".into(), span.clone()),
                                                    WatAST::Symbol(Identifier::bare("__m"), span.clone()),
                                                ], span.clone()),
                                                WatAST::List(vec![
                                                    WatAST::Keyword(":wat::kernel::RecvOutcome::Message".into(), span.clone()),
                                                    WatAST::List(vec![
                                                        WatAST::Keyword(":wat::core::match".into(), span.clone()),
                                                        WatAST::Symbol(Identifier::bare("__m"), span.clone()),
                                                        WatAST::List(vec![
                                                            WatAST::List(vec![
                                                                WatAST::Keyword(reply_ctor, span.clone()),
                                                                WatAST::Symbol(Identifier::bare("resp"), span.clone()),
                                                            ], span.clone()),
                                                            WatAST::Symbol(Identifier::bare("resp"), span.clone()),
                                                        ], span.clone()),
                                                    ], span.clone()),
                                                ], span.clone()),
                                            ], span.clone()),
                                            // ::Lost arm — scrub the cause; a REASON-FREE Failure via the
                                            // ONE canonical constructor (arc 278 Strike A —
                                            // :wat::kernel::message-only-failure, wat/spawn.wat). Mirrors
                                            // runtime::message_only_failure's 5-field shape (that Rust fn
                                            // is what the wat helper itself mirrors). Was emitted as a
                                            // `:wat::core::struct-new` (wrong nature: Struct, not Record —
                                            // Failure/message couldn't read it back); the helper mints the
                                            // canonical Record.
                                            WatAST::List(vec![
                                                WatAST::List(vec![
                                                    WatAST::Keyword(":wat::kernel::RecvOutcome::Lost".into(), span.clone()),
                                                    WatAST::Symbol(Identifier::bare("_cause"), span.clone()),
                                                ], span.clone()),
                                                WatAST::List(vec![
                                                    WatAST::Keyword(":wat::kernel::RecvOutcome::Lost".into(), span.clone()),
                                                    // Arc 170 — SCRUB THE DEATH, PASS THE STOP.
                                                    //
                                                    // The scrub is arc-294's ruling: a client learns no server
                                                    // internals. But `Shutdown` is NOT a server internal — it is
                                                    // the CLIENT'S OWN process being asked to stop, and rewriting
                                                    // it as a peer death re-destroys, one layer up, the exact
                                                    // distinction `kernel/peer.rs` was just fixed to carry.
                                                    //
                                                    // Emits: (match _cause
                                                    //          (LociDiedError::Stopped LociDiedError::Stopped)
                                                    //          (_ LociDiedError::Disconnected))
                                                    //
                                                    // The `_` here is deliberate INFORMATION HIDING at a trust
                                                    // boundary (every genuine death → one reason-free value),
                                                    // not the accidental variant-erasure this arc is killing.
                                                    // The difference is that the client is not entitled to what
                                                    // is dropped, and IS entitled to know its own process is
                                                    // stopping.
                                                    WatAST::List(vec![
                                                        WatAST::Keyword(":wat::core::match".into(), span.clone()),
                                                        WatAST::Symbol(Identifier::bare("_cause"), span.clone()),
                                                        WatAST::List(vec![
                                                            WatAST::Keyword(":wat::kernel::LociDiedError::Stopped".into(), span.clone()),
                                                            WatAST::Keyword(":wat::kernel::LociDiedError::Stopped".into(), span.clone()),
                                                        ], span.clone()),
                                                        WatAST::List(vec![
                                                            WatAST::Symbol(Identifier::bare("_"), span.clone()),
                                                            WatAST::Keyword(":wat::kernel::LociDiedError::Disconnected".into(), span.clone()),
                                                        ], span.clone()),
                                                    ], span.clone()),
                                                ], span.clone()),
                                            ], span.clone()),
                                            // ::Stopped arm — arc 278 #73. Pass the stop through AS
                                            // ITSELF. This is the fact the `Lost` arm above went to
                                            // such lengths to rescue from the scrub: the nested
                                            // `match _cause` exists ONLY because a stop had nowhere
                                            // to ride except inside a death report. It now has a
                                            // top-level home and arrives here directly.
                                            //
                                            // The nested dig is deliberately LEFT IN PLACE as a
                                            // second line: a `Stopped` decoded off a wire reason
                                            // could still surface inside `Lost`. Now that the
                                            // primary path is direct, collapsing that dig is a
                                            // follow-up worth its own grounding, not a change to
                                            // make in passing.
                                            WatAST::List(vec![
                                                WatAST::Keyword(":wat::kernel::RecvOutcome::Stopped".into(), span.clone()),
                                                WatAST::Keyword(":wat::kernel::RecvOutcome::Stopped".into(), span.clone()),
                                            ], span.clone()),
                                            // ::Closed arm — pass the reason-free terminal through.
                                            WatAST::List(vec![
                                                WatAST::Keyword(":wat::kernel::RecvOutcome::Closed".into(), span.clone()),
                                                WatAST::Keyword(":wat::kernel::RecvOutcome::Closed".into(), span.clone()),
                                            ], span.clone()),
                                        ], span.clone()),
                                    ], span.clone());

                                    // DESIGN-STONE-the-client-validates-locally.md — THE STRIKE,
                                    // Path B's copy. STOP-3: `peer-wire?` gates the whole
                                    // measure+guard behind "is there a wire" (a thread-tier `__peer`
                                    // never reaches `:wat::edn::write` at all — zero encodes, exactly
                                    // as today). Under budget (or no wire to measure against) falls
                                    // through to the UNCHANGED `send_recv_ast`, reached from ONLY one
                                    // of the two branches below (STOP-2: no uniform fall-through).
                                    let n_sym = Identifier::bare("__n");
                                    let wrapped_ast = WatAST::List(vec![
                                        WatAST::Keyword(":wat::core::if".into(), span.clone()),
                                        WatAST::List(vec![
                                            WatAST::Keyword(":wat::kernel::peer-wire?".into(), span.clone()),
                                            WatAST::Symbol(Identifier::bare("__peer"), span.clone()),
                                        ], span.clone()),
                                        WatAST::List(vec![
                                            WatAST::Keyword(":wat::core::let".into(), span.clone()),
                                            WatAST::Vector(vec![
                                                WatAST::Symbol(n_sym.clone(), span.clone()),
                                                WatAST::List(vec![
                                                    WatAST::Keyword(":wat::string::length".into(), span.clone()),
                                                    WatAST::List(vec![
                                                        WatAST::Keyword(":wat::edn::write".into(), span.clone()),
                                                        WatAST::Symbol(Identifier::bare("__req"), span.clone()),
                                                    ], span.clone()),
                                                ], span.clone()),
                                            ], span.clone()),
                                            WatAST::List(vec![
                                                WatAST::Keyword(":wat::core::if".into(), span.clone()),
                                                WatAST::List(vec![
                                                    WatAST::Keyword(":wat::i64::>".into(), span.clone()),
                                                    WatAST::Symbol(n_sym.clone(), span.clone()),
                                                    WatAST::Keyword(cap_const_kw.clone(), span.clone()),
                                                ], span.clone()),
                                                // OVER budget — the SAME RequestTooLarge{bytes,cap} a
                                                // server would send, with NO send and therefore NO recv.
                                                WatAST::List(vec![
                                                    WatAST::Keyword(":wat::kernel::RecvOutcome::Message".into(), span.clone()),
                                                    WatAST::List(vec![
                                                        WatAST::Keyword(rtl_ctor_kw, span.clone()),
                                                        WatAST::Symbol(n_sym, span.clone()),
                                                        WatAST::Keyword(cap_const_kw.clone(), span.clone()),
                                                    ], span.clone()),
                                                ], span.clone()),
                                                send_recv_ast.clone(),
                                            ], span.clone()),
                                        ], span.clone()),
                                        send_recv_ast,
                                    ], span.clone());

                                    return eval_inner(&wrapped_ast, &call_env, sym)
                                        .map(|tv| tv.value_owned());
                                }
                            }
                            // Must have at least 1 arg (the receiver).
                            if args.is_empty() {
                                return Err(RuntimeError::new(
                                    list_span.clone(),
                                    RuntimeErrorKind::ArityMismatch {
                                        op: other.to_string(),
                                        expected: 1,
                                        got: 0,
                                    },
                                )
                                .into());
                            }
                            // Eval the receiver (arg 0).
                            let receiver = eval_inner(&args[0], env, sym)?.value_owned();
                            // Read the receiver's concrete type FQDN.
                            // Record/holon-Record: class_fqdn has no leading colon — add it.
                            // Struct: type_name is already colon-prefixed (instance-specific FQDN).
                            // RustOpaque: type_path is already colon-prefixed.
                            // Arc 293.4c — other types (String, i64, etc.): derive FQDN from
                            // type_name() with a colon prefix. This enables foreign types taught
                            // via `extend-type` to dispatch surface methods. The check layer has
                            // already verified satisfaction; here we only need the concrete FQDN.
                            // Arc 293.R2.1 — Aggregate: class is colon-free; add ':' for method key.
                            let concrete_type_fqdn: String = match &receiver {
                                Value::Aggregate(a) => format!(":{}", a.class),
                                Value::RustOpaque(inner) => inner.type_path.to_string(),
                                other_val => {
                                    // Foreign type: type_name() returns the FQDN without leading colon.
                                    format!(":{}", other_val.type_name())
                                }
                            };
                            // Look up `:<T>/<method>` as a plain function in sym.functions.
                            // Arc 293.4c: extend-type on a surface also registers under this key,
                            // so both user-defn and extend-provided methods are found here.
                            // STONE reap-the-angle-machinery (arc 109) — `method_key` used to be
                            // stripped via `canonical_callable_name`; angle syntax is unexpressible
                            // now, so neither `concrete_type_fqdn` nor `method_name` can carry a
                            // suffix to strip — look it up directly.
                            let method_key = format!("{}/{}", concrete_type_fqdn, method_name);
                            let func = match sym.get(&method_key) {
                                Some(f) => f.clone(),
                                None => {
                                    return Err(RuntimeError::new(
                                        list_span.clone(),
                                        RuntimeErrorKind::UnknownFunction(format!(
                                            "type `{}` does not implement surface method `{}` — \
                                             expected a `defn {}` but none is registered",
                                            concrete_type_fqdn, method_name, method_key
                                        )),
                                    )
                                    .into());
                                }
                            };
                            // Eval the remaining args.
                            let rest_vals: Vec<Value> = args[1..]
                                .iter()
                                .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
                                .collect::<Result<Vec<_>, _>>()?;
                            // Full arg list: receiver + rest (receiver is arg 0 as declared in `defn :T/<method>`).
                            let mut all_vals = Vec::with_capacity(1 + rest_vals.len());
                            all_vals.push(receiver);
                            all_vals.extend(rest_vals);
                            return apply_function(func, all_vals, sym, list_span.clone())
                                .map_err(Into::into);
                        }
                    }
                }
            }

            // STONE-exactly-one-call-position (arc 109) — position 4's RUNTIME peel now
            // happens once, hoisted to the top of this arm (see the comment there) —
            // this generic user-function dispatch reads the same already-peeled `args`
            // the surface-method dispatch block above it does; neither extracts a
            // second time.

            // STONE reap-the-angle-machinery (arc 109) — this used to strip `<T,...>`
            // turbofish from the head via `canonical_callable_name` before lookup. Angle
            // syntax is unexpressible now, so `other` can never carry a suffix; look it up
            // directly (`def_value(other)` just below already does, unstripped).
            let func = match sym.get(other) {
                Some(f) => f.clone(),
                None => {
                    // Arc 157 — before the UnknownFunction path, check
                    // whether the call head names a `def`-bound value.
                    // A `def`-bound value (e.g. `:get-config`) may be a
                    // fn (closure); calling it via `(:get-config ...)`
                    // should dispatch through `apply_function` rather than
                    // erroring. The canonical strip is skipped here —
                    // `def` names are registered verbatim.
                    if let Some(v) = sym.def_value(other) {
                        match v {
                            Value::wat__core__fn(f) => {
                                let func = f.clone();
                                let vals = args
                                    .iter()
                                    .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
                                    .collect::<Result<Vec<_>, _>>()?;
                                return apply_function(func, vals, sym, list_span.clone())
                                    .map_err(Into::into);
                            }
                            // Stone 237.2 — defclause-bound value dispatch.
                            Value::wat__core__clauses(cs) => {
                                let cs = cs.clone();
                                return crate::function::eval_call_to_defclause(cs, args, list_span, env, sym);
                            }
                            other_val => {
                                return Err(RuntimeError::new(
                                    list_span.clone(),
                                    RuntimeErrorKind::NotCallable {
                                        got: Box::new(ValueSnapshot::of(other_val)),
                                    },
                                )
                                .into());
                            }
                        }
                    }
                    // Arc 140 slice 1 — sandbox-scope leak detection.
                    // Inner-scope lookup missed; if this SymbolTable
                    // belongs to a sub-program (outer_symbols is
                    // attached at spawn time), check whether the
                    // name resolves in the outer scope. If yes — fire
                    // the teaching SandboxScopeLeak diagnostic with both
                    // spans (offending invocation + outer-scope define).
                    // Otherwise fall through to the generic
                    // UnknownFunction.
                    if let Some(outer) = sym.outer_symbols.as_ref() {
                        if let Some(outer_func) = outer.get(other) {
                            // Stone 255.1a — Native builtins carry no span; use crate::rust_caller_span!().
                            let outer_define_span = match &outer_func.body {
                                FunctionBody::Wat(ast) => ast.span().clone(),
                                // rune:lint(span-substitution) — outer_define_span names WHERE the
                                // shadowed outer binding was DEFINED, not where this call happened;
                                // a Native builtin has no wat definition site to point at, so
                                // list_span (the call site) would misattribute the definition.
                                FunctionBody::Native => crate::rust_caller_span!(),
                            };
                            return Err(RuntimeError::new(
                                list_span.clone(),
                                RuntimeErrorKind::SandboxScopeLeak {
                                    offending_name: other.to_string(),
                                    outer_define_span,
                                },
                            )
                            .into());
                        }
                    }
                    // Arc 278 BRIEF-construction-total-three-walls.md #1 — nested surface
                    // aggregate-constructor dispatch. `build_insert_fact` special-cases a
                    // `:then`/`:when` item's OWN top-level `(:Type arg…)` shape before ever
                    // reaching this generic evaluator, but a constructor written as a NESTED
                    // operand (e.g. `:then [(:usr::Outer :inner (:usr::Inner :x 1))]`) reaches
                    // here with `other` = the bare aggregate-type keyword and no fn registered
                    // under that name — it used to fall all the way to UnknownFunction below.
                    // Nothing about the form is illegal (STOP: this is the one wall that gets
                    // WIRED, not tightened) — a bare aggregate-type keyword head is unambiguous
                    // (TypeEnv keys carry the leading colon, matching `other` verbatim — `<K,V>`
                    // is unexpressible, arc 109 ③'s wall at `src/types.rs:4688`, so `other` is
                    // already the bare name; used directly, never stripped, same as
                    // `construct_aggregate`'s own `bare_name` derivation), so delegate to the
                    // SAME kwargs/positional dispatch `:wat::core::kwargs-construct` already
                    // gives the macro-expanded form — a nested surface constructor now evaluates
                    // identically to its expanded-form twin, arity/field-name errors included.
                    if matches!(
                        sym.types().and_then(|t| t.get(other)),
                        Some(crate::types::TypeDef::Aggregate(_))
                    ) {
                        let mut synth_args: Vec<WatAST> = Vec::with_capacity(args.len() + 1);
                        synth_args.push(WatAST::Keyword(other.to_string(), list_span.clone()));
                        synth_args.extend(args.iter().cloned());
                        return crate::record::construct::eval_kwargs_construct(
                            &synth_args, list_span, env, sym,
                        );
                    }
                    // Arc 234 Stone 234.3c — keyword-as-accessor fall-through.
                    // When head is an unknown verb AND args.len() == 1 AND receiver is
                    // {Value::Aggregate (Record/HolonRecord/Struct), wat__std__HashMap}, dispatch as field accessor.
                    // Fires LAST: after user-fn lookup, after def-bound check, after sandbox
                    // leak detection. Only unknown single-arg keyword calls reach here.
                    if args.len() == 1 {
                        let receiver = eval_inner(&args[0], env, sym)?.value_owned();
                        let bare_name = other.strip_prefix(':').unwrap_or(other);
                        match receiver {
                            // Arc 293.R2.1 — Aggregate: dispatch on nature.
                            // Record/HolonRecord → keyword_accessor_record (field_names path).
                            // Struct → keyword_accessor_struct (TypeDef path).
                            Value::Aggregate(a) if a.nature != Nature::Struct => {
                                let class_arc = Arc::new(a.class.to_string());
                                return keyword_accessor_record(
                                    bare_name,
                                    class_arc,
                                    a.fields.clone(),
                                    sym,
                                    list_span,
                                );
                            }
                            Value::Aggregate(a) => {
                                return keyword_accessor_struct(bare_name, a, sym, list_span);
                            }
                            Value::wat__std__HashMap(map) => {
                                // HashMap accessor: keyword key → (Option :- [V]).
                                // Equivalent to (:wat::core::HashMap/get map :key).
                                // Never errors on miss — missing key = None (per D5 / T7).
                                let key = Value::wat__core__keyword(Arc::new(other.to_string()));
                                return match map.get(&key) {
                                    Some(v) => Ok(Value::Option(Arc::new(Some(v.clone())))),
                                    None => Ok(Value::Option(Arc::new(None))),
                                };
                            }
                            _other_receiver => {
                                // Non-receiver type — fall through to UnknownFunction below.
                            }
                        }
                    }
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::UnknownFunction(other.to_string()),
                    )
                    .into());
                }
            };
            let vals = args
                .iter()
                .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
                .collect::<Result<Vec<_>, _>>()?;
            apply_function(func, vals, sym, list_span.clone()).map_err(Into::into)
        }
    }
}

// ─── Arc 234 Stone 234.3c — keyword-as-accessor helpers ──────────────────────

/// Resolve `bare_name` → field index via the class's `RecordDef.field_names` and
/// return the corresponding value from `fields`. Miss → `UnknownField`.
///
/// Stone S-C.2b re-route: name→index now goes through `RecordDef.field_names`
/// (the CLASS property, Ruby model). Variant-agnostic:
/// works for holonic records today; will work for base records (S-C.2c) without change.
/// Parity: for holonic records the answer is IDENTICAL (same positions, new source).
fn keyword_accessor_record(
    bare_name: &str,
    class_fqdn: Arc<String>,
    fields: Arc<Vec<Value>>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = "keyword-as-accessor (record)";
    // Look up the RecordDef in the TypeEnv — class_fqdn has no leading ':', TypeEnv keys do.
    let type_key = format!(":{}", class_fqdn);
    let types = sym.types().ok_or_else(|| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "record keyword-accessor requires the type registry".into(),
            },
        )
    })?;
    // Arc 293.2b — record aggregates (kind != Struct) replace TypeDef::Record.
    let record_def = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature != crate::types::Nature::Struct => a,
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "record class :{} is not registered in the TypeEnv",
                        class_fqdn
                    ),
                },
            )
            .into());
        }
    };
    let available: Vec<String> = record_def.field_names().map(|s| s.to_string()).collect();
    match record_def.field_names().position(|n| n == bare_name) {
        Some(i) => Ok(fields[i].clone()),
        None => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::UnknownField {
                record_class: class_fqdn.as_ref().to_string(),
                field: bare_name.to_string(),
                available,
            },
        )
        .into()),
    }
}

/// Look up `bare_name` in `sv`'s TypeDef field list and return the field value.
/// Miss → `UnknownField`.
/// Arc 293.R2.1 — `sv` is now `Arc<AggregateValue>` with `nature == Nature::Struct`.
fn keyword_accessor_struct(
    bare_name: &str,
    sv: Arc<AggregateValue>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = "keyword-as-accessor (struct)";
    let types = sym.types().ok_or_else(|| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "struct keyword-accessor requires the type registry".into(),
            },
        )
    })?;
    // Arc 293.2b/R2.1 — struct aggregates (kind==Struct) via TypeDef::Aggregate.
    // class is colon-free; TypeEnv keys have leading ':'.
    let type_key = format!(":{}", sv.class);
    let struct_def = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature == crate::types::Nature::Struct => a,
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("struct type :{} is not registered in the TypeEnv", sv.class),
                },
            )
            .into());
        }
    };
    let available: Vec<String> = struct_def.fields.iter().map(|(n, _)| n.clone()).collect();
    match struct_def.fields.iter().position(|(n, _)| n == bare_name) {
        Some(i) => Ok(sv.fields[i].clone()),
        None => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::UnknownField {
                record_class: format!(":{}", sv.class),
                field: bare_name.to_string(),
                available,
            },
        )
        .into()),
    }
}

// ─── Language forms ─────────────────────────────────────────────────────

// Stone 241.18a — DELETED: eval_fn DELETED.
// `:wat::core::fn` evaluator MIGRATED to `src/function/eval.rs`.
// Caller at runtime dispatch arm (line ~5311) updated to `crate::function::eval_fn`.
// `synthesize_fn_body` stays here (also used by try_parse_fn_shape_def + defclause).

/// Arc 168 — collapse fn body forms (implicit-do) into a single
/// `WatAST` for `Function::body`. Mirrors `synthesize_let_body` but
/// reused at call sites that want the same rule (let, fn,
/// `try_parse_fn_shape_def`).
///
/// - Empty body → `NilLit` (canonical nil value literal). The fn's
///   declared `-> :T` constrains `:T` to be `:wat::core::nil` for
///   this to type-check; substrate allows it, idiom doesn't
///   encourage it. Arc 244: was the nil-type Keyword heresy;
///   now `WatAST::nil()`.
/// - Single form → the form itself (zero-overhead pass-through;
///   pre-arc-168 code shape preserved exactly).
/// - Multi-form → wrap in `(:wat::core::do f1 f2 ... fN)`.
pub(crate) fn synthesize_fn_body(forms: &[WatAST]) -> WatAST {
    if forms.is_empty() {
        // Arc 244 — canonical nil value literal (not the type keyword).
        return WatAST::nil();
    }
    if forms.len() == 1 {
        return forms[0].clone();
    }
    let mut do_items: Vec<WatAST> = Vec::with_capacity(forms.len() + 1);
    do_items.push(WatAST::Keyword(
        ":wat::core::do".into(),
        crate::rust_caller_span!(),
    ));
    do_items.extend(forms.iter().cloned());
    WatAST::List(do_items, crate::rust_caller_span!())
}

// Stone 241.18a — DELETED: parse_fn_signature DELETED.
// fn-form signature parser MIGRATED to `src/function/parse.rs`.
// Caller in try_parse_fn_shape_def (line ~4129) updated to `crate::function::parse_fn_signature`.
// HARD CUT: no backward-compat re-export.

// Arc 109 Stone — the defclause-into-function-home — `parse_defclause_clause` moved to
// `src/function/parse.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `mod arc109_two_iii_defclause_return_slot`
// (parse_defclause_clause's own #[cfg(test)] probe) moved to `src/function/parse.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `parse_defclause_form` moved to
// `src/function/parse.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `parse_extend_type_form` moved to
// `src/function/parse.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `parse_derive_form` moved to
// `src/function/parse.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `is_defclause_form` moved to
// `src/function/parse.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `eval_call_to_defclause` moved to
// `src/function/eval.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `select_defclause_clause` moved to
// `src/function/eval.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `eval_call_to_defclause_with_vals` moved to
// `src/function/eval.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `declared_type_subsumes` moved to
// `src/function/subsume.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `value_matches_type_by_name` moved to
// `src/function/subsume.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the defclause-into-function-home — `val_type_path` moved to
// `src/function/subsume.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// `(:wat::core::let [n1 e1 n2 e2 ...] body1 body2 ... bodyN)` —
/// sequential let with flat-vector bindings + implicit-do body
/// (arc 168). Arc 154 retired `let*`; `let` is the single-letform
/// vocabulary (Clojure-faithful: Clojure's user-facing `let` IS the
/// sequential primitive). Arc 168 reshapes the binding form to a
/// `WatAST::Vector` of alternating `(binder, expr)` pairs and adds
/// implicit-do body semantics.
///
/// Each RHS is evaluated in an environment that includes the PRIOR
/// bindings. `n2`'s RHS can refer to `n1`; `n3`'s RHS can refer to both.
///
/// Body is 1+ trailing forms — implicit-do semantics. All but last
/// evaluated for side effect; last form's value IS the let's value.
/// Empty body → `:wat::core::nil` (Clojure-faithful).
///
/// Rust-level semantics: cumulative `Environment` chain. Each binding
/// commits to the env chain before the next RHS evaluates, so subsequent
/// bindings can reference earlier ones.
///
/// Non-Vector outer shape (e.g. legacy `((n e) ...)` nested-pair list)
/// produces a clean `MalformedForm` naming the canonical shape.
/// Arc 168 slice 3 retired the legacy outer-List fall-through arm.
#[wat_special_form_impl(":wat::core::let", role = eval)]
fn eval_let(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "expected (:wat::core::let [name expr ...] body ...); got {} args",
                    args.len()
                ),
            },
        )
        .into());
    }
    let bindings_form = &args[0];

    // Iterate (binder, expr) pairs into the cumulative scope chain.
    let mut scope = env.clone();
    match bindings_form {
        WatAST::Vector(items, _) => {
            // Canonical flat-vector outer. Even-length required.
            if items.len() % 2 != 0 {
                return Err(RuntimeError::new(bindings_form.span().clone(), RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::let".into(),
                    reason: format!(
                        "let bindings vector must have an even number of elements (alternating name expr name expr ...); got {}",
                        items.len()
                    )
                }).into());
            }
            let mut i = 0;
            while i < items.len() {
                let binder = &items[i];
                let rhs = &items[i + 1];
                let binding = parse_let_binding(binder, rhs)?;
                scope = bind_let_binding(binding, &scope, sym)?;
                i += 2;
            }
        }
        _ => {
            return Err(RuntimeError::new(
                bindings_form.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::let".into(),
                    reason: "let bindings must be a flat vector `[name expr ...]`".into(),
                },
            )
            .into());
        }
    };

    // Implicit-do body: args[1..]. Empty body → :wat::core::nil singleton.
    let body = &args[1..];
    if body.is_empty() {
        return Ok(TrackedValue::from(Value::Unit));
    }
    let last_idx = body.len() - 1;
    for form in &body[..last_idx] {
        let _ = eval_inner(form, &scope, sym)?;
    }
    // Arc 233 Stone 233.2.k: return TrackedValue directly so provenance from the
    // last body expression flows through (e.g., a let-bound producer value used
    // as the let body's result retains its RuntimeBuilt provenance).
    eval_inner(&body[last_idx], &scope, sym)
}

/// Apply a single parsed `LetBinding` to a scope, returning the
/// extended scope chain. Shared between flat-vector and
/// legacy-outer-List paths; centralizes the eval semantics.
fn bind_let_binding(
    binding: LetBinding<'_>,
    scope: &Environment,
    sym: &SymbolTable,
) -> Result<Environment, EvalBreak> {
    match binding {
        LetBinding::Single {
            name,
            name_span,
            rhs,
        } => {
            // Arc 233 Stone 233.2.k: Environment stores TrackedValue directly.
            // Arc 233 Stone 233.2.e: bind with name_span so env.lookup can
            // construct SymbolBound provenance when the name is referenced.
            let tv = eval_inner(rhs, scope, sym)?;
            Ok(scope.child().bind(name, name_span, tv).build())
        }
        LetBinding::Destructure { names, rhs } => {
            let value = eval_inner(rhs, scope, sym)?.value_owned();
            let elements = destructure_tuple(&value, names.len(), ":wat::core::let")?;
            let mut builder = scope.child();
            for ((name, name_span), elem) in names.into_iter().zip(elements) {
                // Arc 233 Stone 233.2.e: each destructure slot is bound with its
                // name_span from the LHS pattern. Lookup yields SymbolBound with
                // binding_span pointing at the slot's position in the pattern.
                // The per-element provenance within the tuple (deeper tracing)
                // is out of scope per sub-DESIGN Decision 3.
                builder = builder.bind(name, name_span, TrackedValue::from(elem));
            }
            Ok(builder.build())
        }
        // Arc 169 slice 1 — struct destructure. The 12-word rule:
        // *bind the field's value to the field's name in this
        // scope*. RHS must evaluate to a `Value::Aggregate(Struct)`; each
        // requested field-name resolves against the struct type's
        // declared fields (looked up via the SymbolTable's TypeEnv);
        // the field's value is bound to the local of the same name.
        //
        // The type-checker (arc 169 check arm) catches struct-type
        // mismatches and unknown field names ahead of time; runtime
        // posture is defense-in-depth — clear diagnostics for
        // programs that reach here without having been checked.
        LetBinding::StructDestructure { field_names, rhs } => {
            let value = eval_inner(rhs, scope, sym)?.value_owned();
            // Arc 293.R2.1 — Aggregate with nature==Struct.
            let sv = match &value {
                Value::Aggregate(a) if a.nature == Nature::Struct => a.clone(),
                other => {
                    return Err(RuntimeError::new(
                        rhs.span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: ":wat::core::let".into(),
                            expected: "wat::core::Struct",
                            got: Box::new(ValueSnapshot::of(other)),
                        },
                    )
                    .into());
                }
            };
            // Resolve field-name → field-index via the struct's
            // declared field list. SymbolTable carries the TypeEnv
            // post-freeze; runtime callers always have it attached.
            let types = sym.types().ok_or_else(|| RuntimeError::new(rhs.span().clone(), RuntimeErrorKind::MalformedForm {
                head: ":wat::core::let".into(),
                reason: "struct destructure requires the type registry, but the SymbolTable has no TypeEnv attached (programmer error: this build path didn't go through startup_from_source / freeze)".into()
            }))?;
            // Arc 293.2b/R2.1 — class is colon-free; TypeEnv keys have leading ':'.
            let type_key = format!(":{}", sv.class);
            let struct_def = match types.get(&type_key) {
                Some(crate::types::TypeDef::Aggregate(a))
                    if a.nature == crate::types::Nature::Struct =>
                {
                    a
                }
                _ => {
                    return Err(RuntimeError::new(rhs.span().clone(), RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::let".into(),
                        reason: format!(
                            "struct destructure: rhs type :{} is not registered as a struct in the TypeEnv (programmer error: a Value::Aggregate{{nature=Struct}} exists at runtime without a corresponding AggregateDef{{kind=Struct}})",
                            sv.class
                        )
                    }).into());
                }
            };
            let mut builder = scope.child();
            for (fname, fname_span) in &field_names {
                let idx = struct_def
                    .fields
                    .iter()
                    .position(|(n, _)| n == fname)
                    .ok_or_else(|| RuntimeError::new(rhs.span().clone(), RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::let".into(),
                        reason: format!(
                            "struct destructure: field {:?} is not declared on struct :{} (declared fields: {})",
                            fname,
                            sv.class,
                            struct_def
                                .fields
                                .iter()
                                .map(|(n, _)| n.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }))?;
                let elem = sv
                    .fields
                    .get(idx)
                    .cloned()
                    .ok_or_else(|| RuntimeError::new(rhs.span().clone(), RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::let".into(),
                        reason: format!(
                            "struct destructure: field {:?} index {} is out of range on struct :{} (value has {} fields, declaration has {})",
                            fname,
                            idx,
                            sv.class,
                            sv.fields.len(),
                            struct_def.fields.len()
                        )
                    }))?;
                // Arc 233 Stone 233.2.e: bind with fname_span so lookup yields SymbolBound.
                builder = builder.bind(fname.clone(), fname_span.clone(), TrackedValue::from(elem));
            }
            Ok(builder.build())
        }
        // Arc 234 Stone 234.4 — hash-destructure.
        // Evaluates the RHS once; dispatches on Value variant:
        //   Aggregate (Record/HolonRecord) → look up field index via AggregateDef; bind fields[i]
        //   Aggregate (Struct)             → look up field in TypeDef; bind fields[i]
        //   wat__std__HashMap              → keyword key lookup; bind to Value::Option(Some/None)
        //   Other          → TypeMismatch
        //
        // Reuses keyword_accessor_record / keyword_accessor_struct helpers
        // from Stone 234.3c. HashMap arm uses the same key-build pattern
        // as the keyword-as-accessor fall-through (runtime.rs line ~5939).
        LetBinding::HashDestructure { bindings, rhs } => {
            let value = eval_inner(rhs, scope, sym)?.value_owned();
            let mut builder = scope.child();
            match &value {
                // Arc 293.R2.1 — Aggregate: dispatch on nature.
                // Record/HolonRecord → keyword_accessor_record; Struct → keyword_accessor_struct.
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    // Record receiver — resolve field names via RecordDef.field_names.
                    for (var_name, bare_field, var_span) in &bindings {
                        let field_val = keyword_accessor_record(
                            bare_field,
                            Arc::new(a.class.to_string()),
                            a.fields.clone(),
                            sym,
                            rhs.span(),
                        )?;
                        builder = builder.bind(
                            var_name.clone(),
                            var_span.clone(),
                            TrackedValue::from(field_val),
                        );
                    }
                }
                Value::Aggregate(a) => {
                    // Struct receiver — look up each field in TypeDef.
                    for (var_name, bare_field, var_span) in &bindings {
                        let field_val =
                            keyword_accessor_struct(bare_field, a.clone(), sym, rhs.span())?;
                        builder = builder.bind(
                            var_name.clone(),
                            var_span.clone(),
                            TrackedValue::from(field_val),
                        );
                    }
                }
                Value::wat__std__HashMap(map) => {
                    // HashMap receiver — keyword key lookup returning (Option :- [V]).
                    // Consistent with keyword-as-accessor fall-through and
                    // :wat::core::HashMap/get (miss = None, never an error).
                    for (var_name, bare_field, var_span) in &bindings {
                        let key_str = format!(":{}", bare_field);
                        let key = Value::wat__core__keyword(Arc::new(key_str));
                        let opt_val = match map.get(&key) {
                            Some(v) => Value::Option(Arc::new(Some(v.clone()))),
                            None => Value::Option(Arc::new(None)),
                        };
                        builder = builder.bind(
                            var_name.clone(),
                            var_span.clone(),
                            TrackedValue::from(opt_val),
                        );
                    }
                }
                other => {
                    return Err(RuntimeError::new(
                        rhs.span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: ":wat::core::let hash-destructure".into(),
                            expected: "wat::core::Record, Struct, or wat::core::HashMap",
                            got: Box::new(ValueSnapshot::of(other)),
                        },
                    )
                    .into());
                }
            }
            Ok(builder.build())
        }
    }
}

/// `(:wat::core::do f1 f2 ... fN)` — Clojure-faithful sequential
/// evaluation form. Arc 136 slice 1a.
///
/// Each non-final form is evaluated for its side effect; the resulting
/// value is discarded. The FINAL form is evaluated; its value is
/// returned as the do form's value. Empty arg list → MalformedForm
/// (belt-and-suspenders for programs reaching the dispatcher without
/// the checker having run; the type checker fires the same diagnostic).
///
/// Arc 255 Stone 1a-zeta — the `role = eval` pointer for `:wat::core::do`. Annotated IN PLACE
/// (signature already fits the canonical `NativeHandler` shape) — see
/// `intrinsic/special/do_form.rs` for the doc-only struct and the `role = check`/`role = tail`
/// pointers.
#[wat_special_form_impl(":wat::core::do", role = eval)]
fn eval_do(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::do".into(),
                reason: "do form requires at least one form; got zero".into(),
            },
        )
        .into());
    }
    let last_idx = args.len() - 1;
    for arg in &args[..last_idx] {
        let _ = eval_inner(arg, env, sym)?;
    }
    eval_inner(&args[last_idx], env, sym).map(|tv| tv.value_owned())
}

/// Verify `value` is a tuple of the expected arity and return its
/// elements cloned. Used by `let` destructure bindings.
fn destructure_tuple(
    value: &Value,
    expected_arity: usize,
    op: &str,
) -> Result<Vec<Value>, EvalBreak> {
    match value {
        Value::Tuple(items) => {
            if items.len() != expected_arity {
                // arc 138: no span — destructure_tuple is called from let
                // binding evaluators with no per-binding span context;
                // the enclosing let form's span is one frame up.
                Err(RuntimeError::new(
                    crate::rust_caller_span!(),
                    RuntimeErrorKind::MalformedForm {
                        head: op.into(),
                        reason: format!(
                        "destructure arity mismatch: binder has {} names, tuple has {} elements",
                        expected_arity,
                        items.len()
                    ),
                    },
                )
                .into())
            } else {
                Ok((**items).clone())
            }
        }
        // arc 138: no span — same rationale as above.
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::core::Tuple",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// One let binding form (arc 168 + arc 169).
///
/// Three canonical shapes — all honest about types. Bare-single
/// `(name rhs)` is NOT accepted at the bare layer: every bound name's
/// type must be derivable from a declaration somewhere in the program,
/// not from the checker guessing at a literal.
///
/// - **Single** — binder is a `WatAST::Symbol`. Name's type is inferred
///   from the RHS at check time.
/// - **Destructure** — binder is a `WatAST::Vector` of bare symbols.
///   RHS must have a declared tuple return type (from a primitive or
///   user function); each binder name receives the matching
///   tuple-element type from that declaration. Structural destructure
///   — types flow from the RHS's declared shape through the pattern;
///   no inference from literals.
/// - **StructDestructure** (arc 169 / arc 257.2) — binder is a
///   `WatAST::Map` with `:keys`-destructure form (`{:keys [f1 f2 ...]}`);
///   each name is BOTH the field-name AND the local binding-name.
///   RHS must be a struct-typed expression; each field name resolves
///   against the struct type's registered fields.
///
/// Arc 233 Stone 233.2.e: added per-name spans to all three variants so
/// bind_let_binding can store binding_span in BoundEntry and env.lookup
/// can construct SymbolBound provenance at lookup time.
enum LetBinding<'a> {
    Single {
        name: String,
        /// Source position of the LHS name in the let binder (e.g., `x` in `[x 42]`).
        name_span: Span,
        rhs: &'a WatAST,
    },
    Destructure {
        /// Per-name spans for each slot in the destructure pattern (e.g., `a`, `b` in `[[a b] ...]`).
        names: Vec<(String, Span)>,
        rhs: &'a WatAST,
    },
    StructDestructure {
        /// Per-field-name spans for each slot in the struct destructure pattern.
        field_names: Vec<(String, Span)>,
        rhs: &'a WatAST,
    },
    /// Arc 234 Stone 234.4 — Clojure-style hash-destructure.
    /// `{var :field  var2 :field2 ...}` in let-binding position.
    /// Receiver-polymorphic over Value::Aggregate (all natures) and wat__std__HashMap.
    ///
    /// Each binding carries (var_name, bare_field_name, var_span).
    /// Runtime evaluates the RHS once and dispatches on Value variant
    /// to extract each field by name.
    HashDestructure {
        /// (var_name, bare_field_name, var_name_span) triples.
        bindings: Vec<(String, String, Span)>,
        rhs: &'a WatAST,
    },
}

/// Parse a single (binder, rhs) chunk from a flat-vector let
/// bindings form (arc 168 + arc 169). The caller (`eval_let`) walks
/// the outer `WatAST::Vector` 2-at-a-time and calls this for each
/// pair.
///
/// Binder is one of:
/// - `WatAST::Symbol` → `LetBinding::Single { name, rhs }` (canonical)
/// - `WatAST::Vector` of bare symbols → `LetBinding::Destructure`
///   (tuple destructure; arc 168)
/// - `WatAST::Map` with `:keys`-destructure →
///   `LetBinding::StructDestructure` (struct destructure; arc 169 / 257.2)
fn parse_let_binding<'a>(binder: &'a WatAST, rhs: &'a WatAST) -> Result<LetBinding<'a>, EvalBreak> {
    match binder {
        // Arc 233 Stone 233.2.e: extract name_span from WatAST::Symbol(_, span).
        WatAST::Symbol(ident, name_span) => Ok(LetBinding::Single {
            name: crate::scope::env_key(ident).into_owned(),
            name_span: name_span.clone(),
            rhs,
        }),
        WatAST::Vector(inner, _) => {
            // Destructure binder: every element must be a bare symbol.
            // Arc 233 Stone 233.2.e: capture per-name spans for SymbolBound provenance.
            let mut names = Vec::with_capacity(inner.len());
            for item in inner {
                match item {
                    WatAST::Symbol(ident, name_span) => {
                        names.push((crate::scope::env_key(ident).into_owned(), name_span.clone()))
                    }
                    other => {
                        return Err(RuntimeError::new(
                            other.span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: ":wat::core::let".into(),
                                reason: format!(
                                    "destructure binder must be a vector of bare symbols; got {}",
                                    other.variant_name()
                                ),
                            },
                        )
                        .into());
                    }
                }
            }
            if names.is_empty() {
                return Err(RuntimeError::new(
                    binder.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::let".into(),
                        reason:
                            "destructure binder cannot be empty — at least one name is required"
                                .into(),
                    },
                )
                .into());
            }
            Ok(LetBinding::Destructure { names, rhs })
        }
        // Arc 257.2 — Map binder: keys-destructure ({:keys [x y z]}) or
        // hash-destructure ({var :field ...}). classify_map_destructure is
        // the ONE authoritative helper (ast.rs) for both forms.
        WatAST::Map(pairs, _) => {
            let md = WatAST::classify_map_destructure(pairs).ok_or_else(|| {
                RuntimeError::new(
                    binder.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::let".into(),
                        reason: "map in binder position must be a keys-destructure \
                        ({:keys [field1 field2 ...]}) or a hash-destructure \
                        ({var :field  var2 :field2 ...}); got neither"
                            .into(),
                    },
                )
            })?;
            match md.kind {
                crate::ast::MapDestructureKind::Keys => {
                    // Keys-destructure: same semantics as the old StructDestructure —
                    // each binding name IS the field name.
                    let field_names: Vec<(String, Span)> = md
                        .bindings
                        .into_iter()
                        .map(|(ident, _field, sp)| (crate::scope::env_key(&ident).into_owned(), sp))
                        .collect();
                    if field_names.is_empty() {
                        return Err(RuntimeError::new(
                            binder.span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: ":wat::core::let".into(),
                                reason:
                                    "keys-destructure {:keys [...]} cannot have an empty vector"
                                        .into(),
                            },
                        )
                        .into());
                    }
                    Ok(LetBinding::StructDestructure { field_names, rhs })
                }
                crate::ast::MapDestructureKind::Hash => {
                    // Hash-destructure: (var_name, bare_field_name, var_span).
                    let bindings: Vec<(String, String, Span)> = md
                        .bindings
                        .into_iter()
                        .map(|(ident, field, sp)| {
                            (crate::scope::env_key(&ident).into_owned(), field, sp)
                        })
                        .collect();
                    Ok(LetBinding::HashDestructure { bindings, rhs })
                }
            }
        }
        other => Err(RuntimeError::new(
            other.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                "let binder must be a Symbol (single), a Vector of symbols (tuple destructure), \
                 a Map keys-destructure ({{:keys [f1 f2 ...]}}), or a Map hash-destructure \
                 ({{var :field ...}}); got {}",
                other.variant_name()
            ),
            },
        )
        .into()),
    }
}

/// `(:wat::core::if cond then else)` — typed conditional per
/// the 2026-04-20 INSCRIPTION. Both branches must produce `:T`; the
/// annotation is check-time only (runtime ignores it but validates
/// the form's arity).
///
/// Arity: exactly 3 args — `[cond, then, else]`.
///
/// ⛔ THIS DOC WAS INVERTED UNTIL 2026-08-28. It read *"Arity: exactly 5 args.
/// Positions: [cond, `->`, `:T`, then, else]. The old 3-arg form is refused"* —
/// the precise opposite of the code beneath it. **Arc 258.4 retired the `-> :T`
/// ascription**: the 3-arg form is the live path (the `args.len() == 3` arm
/// below) and a stray `->` is what gets refused now. The comment never moved.
///
/// It was caught because **arc 255 Stone P6-a made this comment PUBLIC**:
/// `(:wat::core::show-source :wat::core::if)` now prints this fn, doc comment
/// included, where it used to print `""`. A buried inverted claim became a
/// published one the moment the source became reachable — so on this fn, and on
/// every fn a `#[wat_special_form_impl]` names, the doc comment is USER-FACING
/// DOCUMENTATION and stale prose here is a shipped lie.
#[wat_special_form_impl(":wat::core::if", role = eval)]
fn eval_if(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() == 3 {
        // Arc 258.1 — bare `(if cond then else)`.
        let cond_val = eval_inner(&args[0], env, sym)?.value_owned();
        return match cond_val {
            Value::bool(true) => eval_inner(&args[1], env, sym).map(|tv| tv.value_owned()),
            Value::bool(false) => eval_inner(&args[2], env, sym).map(|tv| tv.value_owned()),
            other => Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::BadCondition {
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into()),
        };
    }
    // Arc 258.4 — the `-> :T` ascription is retired; a stray `->` (the old 5-arg form)
    // is the retired shape; refuse it with a migration hint.
    if args.len() >= 2 && matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::if".into(),
            reason: "`:wat::core::if` no longer takes `-> :T`; the result type is inferred by unifying the branches. Write (:wat::core::if cond then else)".into()
        }).into());
    }
    Err(RuntimeError::new(
        list_span.clone(),
        RuntimeErrorKind::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond then else) — 3 args; got {}",
                args.len()
            ),
        },
    )
    .into())
}

// ─── Built-ins ──────────────────────────────────────────────────────────

// Arc 109 Stone 1 — `eval_i64_arith` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `i64_add_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `i64_sub_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `i64_mul_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `i64_div_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `i64_quot_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `i64_rem_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `i64_mod_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Arc 278 #55 (S3b+S4) slice one — generic dispatch for every row of THE ONE TABLE
/// (`rete::vocabulary::RETE_OPS`). The `class`, never a per-op FQDN, drives the routing — this
/// function's own body names classes, not rete ops (STOP-2).
///
/// `Alias`/`Form` re-invoke `dispatch_keyword_head_value` on `core_name` with the SAME `args` —
/// literally the identical path the core op already uses (`:wat::i64::>`'s `eval_compare`
/// call, reached via the registry-first door; `:wat::core::and`'s `eval_and_tail`, registered
/// arc 255 Stone 1a-i), never a second implementation.
///
/// `Fallback` is the one class with its own logic, and it is a SECOND TERMINAL HANDLER over the
/// shared kernel, not a duplicate: it re-invokes `dispatch_keyword_head_value` on `core_name`
/// (reaching the exact `eval_i64_arith` call `:wat::i64::+` uses via the registry — the SAME
/// function the design stone's STOP-A probe proved a `where` traverses) and catches ONLY the
/// `IntegerOverflow` it can raise, substituting the caller's `:undefined` value instead of
/// propagating. **STOP-3 did NOT fire**: the design stone anticipated needing either a
/// `:4829`-onto-`:9753` kernel unification or an apply-only surface (`arith_i64_i64_inner` /
/// `I64ArithErr`, the pre-evaluated substrate table, is reached only via `apply` — not via a
/// `where`). Recursing through `dispatch_keyword_head_value` sidesteps that entirely: it is the
/// SAME function `:4829`'s arm already lives in, so no kernel move was needed at all — the
/// measured size of the anticipated refactor is zero, not merely "contained."
fn dispatch_rete_op(
    op: &crate::rete::vocabulary::ReteOp,
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    use crate::rete::vocabulary::OpClass;
    match op.class {
        // Arc 278 #57 round 1b — `Redispatch` joins `Alias`/`Form` here: same generic
        // re-invoke on `core_name`, zero new runtime logic (the checker's routing is the
        // only thing round 1b changes; `dispatch_keyword_head_value` already re-dispatches
        // by `core_name` for any class, per this fn's own header doc).
        OpClass::Alias | OpClass::Form | OpClass::Redispatch => {
            dispatch_keyword_head_value(op.core_name, args, list_span, env, sym)
        }
        OpClass::Fallback => {
            // BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — arity DERIVED
            // from `op.params.len()`, not hardcoded to 4. Every Fallback row before `first` took
            // exactly TWO real args before the `:undefined` marker (i64/f64 arithmetic, holon
            // cosine/dot: `[X, X, Keyword, X]`), so `4`/`&args[0..2]`/`args[2]`/`args[3]` were
            // literally correct — but `first` takes exactly ONE real arg (the container:
            // `[Container<T>, Keyword, Var(T)]`, `params.len() == 3`). Deriving the split from
            // the row's own declared shape is behavior-preserving for every pre-existing row
            // (all have `params.len() == 4`, same split as before: marker at 2, fallback at 3,
            // real args `[0..2]`) and now correct for a 3-param row too.
            let total_arity = op.params.len();
            let marker_idx = total_arity - 2;
            let fallback_idx = total_arity - 1;
            if args.len() != total_arity {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::ArityMismatch {
                        op: head.into(),
                        expected: total_arity,
                        got: args.len(),
                    },
                )
                .into());
            }
            // The literal keyword `:undefined` is a mandatory marker in the second-to-last slot —
            // a kwargs SURFACE would lower this away before an intrinsic ever saw it (proven by
            // `wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat` row A); this op is a
            // plain positional Rust intrinsic instead (no `wat/` defmacro — out of this slice's
            // scope), so the marker is inspected here, directly, on the raw AST.
            match &args[marker_idx] {
                WatAST::Keyword(k, _) if k == ":undefined" => {}
                other => return Err(RuntimeError::new(other.span().clone(), RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: "the fallback-carrying rete op requires the literal keyword `:undefined` as its second-to-last argument, e.g. `(:wat::rete::i64::+ a b :undefined fallback)`".into(),
                }).into()),
            }
            // ONE classification, shared with the two rete walks — see
            // `classify_fallback_outcome`, which this arm's body became. Only the
            // recursion into the caller's `:undefined` arg stays here.
            match classify_fallback_outcome(
                dispatch_keyword_head_value(
                    op.core_name,
                    &args[0..marker_idx],
                    list_span,
                    env,
                    sym,
                ),
                &op.ret,
                op.core_name,
                head,
                list_span,
            )? {
                FallbackVerdict::Value(v) => Ok(v),
                FallbackVerdict::UseFallback => {
                    eval_inner(&args[fallback_idx], env, sym).map(|tv| tv.value_owned())
                }
            }
        }
    }
}

// Arc 109 Stone 1 — `eval_bigint_arith` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `bigint_div` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `to_bigrational` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `collapse_bigrational` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_rational_arith` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `rational_div` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_i64_to_rational` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_bigint_to_rational` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_rational_to_f64` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `bigint_component_to_value` moved to `src/numeric/ops.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_rational_numerator` moved to `src/numeric/ops.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_rational_denominator` moved to `src/numeric/ops.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_u8_cast` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_f64_arith` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `f64_add_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `f64_sub_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `f64_mul_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `f64_div_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `f64_max_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `f64_min_op` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// arc 237 Stone 237.8a — eval_i64_f64_arith and eval_f64_i64_arith
// DELETED under THE DECISION (`feedback_no_implicit_coercion`).
// Mixed-type leaf eval helpers are dead; their only callers were the
// +'i64'f64 / +'f64'i64 (etc.) match arms, which are also deleted.

// ─── Scalar conversions (arc 014) ───────────────────────────────────
//
// :wat::core::<source>::to-<target> — explicit named casts between
// the four scalar tiers (i64, f64, bool, String). Infallible ones
// return the target directly; fallible ones return (Option :- [T]). No
// implicit coercion at arithmetic / comparison sites; users opt in
// to each conversion by name at the call site.

pub(crate) fn eval_one_arg<T>(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    expected: &'static str,
    extract: impl Fn(Value) -> Result<T, Value>,
) -> Result<T, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: head.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let arg_span = args[0].span().clone();
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    extract(v).map_err(|other| {
        EvalBreak::from(RuntimeError::new(
            arg_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected,
                got: Box::new(ValueSnapshot::of(&other)),
            },
        ))
    })
}

// Arc 109 Stone 1 — `eval_i64_to_string` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_i64_to_f64` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_i64_to_bigint` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_bigint_to_f64` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_f64_to_string` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_f64_to_i64` moved to `src/numeric/convert.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_f64_round` moved to `src/numeric/ops.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_f64_unary` moved to `src/numeric/ops.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `eval_f64_clamp` moved to `src/numeric/ops.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// `(:wat::core::bool::to-string b)` → the literal `"true"` or `"false"` for `b`.
///
/// **Purity ground —** the sole arg is evaluated by ordinary call-by-value
/// (`eval_one_arg`'s `eval_inner`, not itself an effect); past that the body only matches
/// `Value::bool` and formats one of two fixed string literals — no `eval_inner`/
/// `apply_function` on caller-supplied code beyond the initial argument evaluation.
///
/// **Totality ground —** every `bool` is one of exactly two values, and each maps to its own
/// fixed literal with no failure path inside the domain; the only error `eval_one_arg` can
/// raise is a `TypeMismatch` for a non-bool argument, which is outside the declared `bool ->
/// String` domain (the same reasoning `:wat::i64::to-f64`/`:wat::i64::to-string` use for
/// their own `Total`).
///
/// **Expand-time ground —** on `macros/eval.rs`'s `is_expand_time_legal` residue list today
/// (the "value/control-flow ops" group names `bool::to-string` explicitly), so it is legal
/// inside a macro body today; registering it here REPLACES that residue entry, so it must
/// declare the SAME verdict — `Legal` — or the registration silently revokes today's
/// legality (arc 255 the `fn` lesson).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     args :wat::core::bool the boolean to render
/// @ret     :wat::core::String `"true"` when `args` is `true`, `"false"` otherwise
/// @example (:wat::core::bool::to-string true) #=> "true"
#[wat_intrinsic(":wat::core::bool::to-string")]
fn eval_bool_to_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let b = eval_one_arg(
        ":wat::core::bool::to-string",
        args,
        list_span,
        env,
        sym,
        "bool",
        |v| match v {
            Value::bool(b) => Ok(b),
            other => Err(other),
        },
    )?;
    Ok(Value::String(Arc::new(
        if b { "true" } else { "false" }.to_string(),
    )))
}

// ─── Arc 170 slice 3 Gap A — keyword reflection primitives ───────────────
//
// `:wat::keyword::to-string`  → extracts keyword text WITHOUT leading colon.
// `:wat::keyword::from-string` → constructs a keyword Value from text;
//     text MUST NOT start with ':' (diagnostic error if it does).
//
// These two primitives are the substrate that keyword/of (macro special-form)
// is built on top of conceptually. They also stand as first-class runtime
// verbs usable in user code.

/// `(:wat::keyword::to-string k)` — extract the text of a keyword value,
/// without the leading colon sigil.
///
/// Examples:
///   `(keyword/to-string :foo)`            → `"foo"`
///   `(keyword/to-string :wat::core::i64)` → `"wat::core::i64"`
// Arc 255 Stone E-iv — bumped to `pub(crate)` so `src/intrinsic/keyword.rs`'s registry-home
// shim (`:wat::keyword::to-string`) can call the SAME algorithm; the algorithm stays here
// (untouched), only its home's dispatch route moves.
pub(crate) fn eval_keyword_to_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::keyword::to-string".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let arg_span = args[0].span().clone();
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // The keyword string always starts with ':'; strip it.
    let raw: String = match &v {
        Value::wat__core__keyword(k) => k.to_string(),
        // Arc 249 Stone 249.4a — keyword FORM-value (bound in a macro body as
        // Value::wat__WatAST(Keyword)): same stripping as the keyword-value arm.
        Value::wat__WatAST(ast) => match &**ast {
            WatAST::Keyword(k, _) => k.clone(),
            _ => {
                return Err(RuntimeError::new(
                    arg_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::keyword::to-string".into(),
                        expected: "keyword",
                        got: Box::new(ValueSnapshot::of(&v)),
                    },
                )
                .into())
            }
        },
        _ => {
            return Err(RuntimeError::new(
                arg_span,
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::keyword::to-string".into(),
                    expected: "keyword",
                    got: Box::new(ValueSnapshot::of(&v)),
                },
            )
            .into())
        }
    };
    let text = raw.strip_prefix(':').unwrap_or(&raw);
    Ok(Value::String(Arc::new(text.to_string())))
}

/// `(:wat::keyword::from-string s)` — construct a keyword Value from
/// a text string. The text MUST NOT start with ':' (the colon is the sigil,
/// not part of the payload). Returns a MalformedForm error with a helpful
/// diagnostic if the string starts with ':'.
///
/// Round-trip property: `(from-string (to-string k)) == k` for any keyword `k`.
// Arc 233 Stone 233.2.j: returns TrackedValue directly (no Value::Tracked wrap).
// Arc 255 Stone E-iv — bumped to `pub(crate)` so `src/intrinsic/keyword.rs`'s registry-home
// shim (`:wat::keyword::from-string`) can call the SAME algorithm.
// Arc 255 Stone G — the shim now forwards this fn's returned `TrackedValue` un-rewrapped
// (`NativeHandler` sniffs the handler's declared return type), so the registry-routed call
// carries this fn's own `RuntimeBuilt` provenance again, not `Provenance::Unknown`.
pub(crate) fn eval_keyword_from_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    let s = eval_one_arg(
        ":wat::keyword::from-string",
        args,
        list_span,
        env,
        sym,
        "String",
        |v| match v {
            Value::String(s) => Ok(s),
            other => Err(other),
        },
    )?;
    if angle_type_head_in_name(&s) {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::keyword::from-string".into(),
            reason: angle_minted_name_reason(&s),
        })
        .into());
    }
    if s.starts_with(':') {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::keyword::from-string".into(),
            reason: format!(
                "input string {:?} starts with ':' — keyword text must not include the leading colon sigil; \
                 use keyword/to-string to produce a colon-free string, or strip the ':' before calling from-string",
                s.as_str()
            )
        }).into());
    }
    // Prepend ':' to form the canonical keyword string.
    // Arc 233 Stone 233.2.j: construct TrackedValue directly (no Value::Tracked wrap).
    let kw = Value::wat__core__keyword(Arc::new(format!(":{}", s.as_str())));
    Ok(TrackedValue::new(
        kw,
        Provenance::RuntimeBuilt {
            producer: ":wat::keyword::from-string",
            call_span: list_span.clone(),
        },
    ))
}

// ─── Arc 232 Stone 232.0 — :wat::core::apply ────────────────────────────────

/// Validate the `[-> :T]` annotation vector that `:wat::core::apply` expects
/// at position 1. Returns `Ok(())` on a valid shape; returns
/// `Err(MalformedForm)` on any structural violation. Extracted as a helper
/// so both the fn-valued fast path and the keyword-valued slow path can reuse
/// the same validation without duplication.
/// `:wat::core::apply` — dynamic keyword-head invocation (Clojure's apply
/// contract; convergence #16).
///
/// Shape (inline `-> :T` typed-expect pattern; mirrors
/// `:wat::core::Result/expect -> :T <value> <msg>` from arc 108):
///
///   `(:wat::core::apply -> :T <head> <a1>...<an> <args-vec>)`
///
/// - `->`       : MUST be the `->` symbol (position 0; inline annotation marker).
/// - `:T`       : MUST be a type keyword (position 1; declared return type).
///   Consumed by the checker; runtime validates shape only.
/// - `head`     : expression at position 2; evaluates to `:wat::core::keyword`
///   (FQDN of callable) OR `:wat::core::fn` (Arc 009 lift / let-bound).
/// - `a1..an`   : zero or more leading positional args (positions 3..n-1).
/// - `args-vec` : LAST positional arg MUST be `:wat::core::Vector`; its
///   elements are spread as trailing args. May be empty.
///
/// Two head-dispatch paths:
///   • `Value::wat__core__fn`     — Arc 009 lift OR let-bound fn-value;
///                                   dispatched directly via `apply_function`.
///   • `Value::wat__core__keyword` — runtime-built keyword (e.g. via
///                                   `keyword/from-string`); dispatched via the
///                                   full keyword-name lookup chain (functions /
///                                   def-bound / dispatch_registry / substrate).
/// Other head types rejected with TypeMismatch.
///
/// Dispatch chain (mirrors `dispatch_keyword_head`'s `other` arm):
/// 1. `sym.functions` (user-defined functions / defn)
/// 2. `sym.runtime_def_values` (def-bound callable values)
/// 3. dispatch_registry (dispatch entities)
/// 4. `dispatch_substrate_impl` (pre-evaluated substrate arithmetic arms)
/// 5. Error — UnknownFunction
///
/// Special-form rejection: if head names a declaration form or language
/// special form, error immediately with MalformedForm diagnostic (STOP-8).
fn eval_apply(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: Span,
) -> Result<Value, EvalBreak> {
    // Arc 258 — the `-> :T` ascription is retired; a stray `->` (the old form) is
    // refused with a migration hint.
    if !args.is_empty() && matches!(&args[0], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError::new(list_span, RuntimeErrorKind::MalformedForm {
            head: ":wat::core::apply".into(),
            reason: "`:wat::core::apply` no longer takes `-> :T`; the result is the applied fn's return type. Write (:wat::core::apply <fn> <a1>...<an> <args-vec>)".into()
        }).into());
    }
    // `(apply <head> <a1>...<an> <args-vec>)` — minimum a fn head and the args-vector.
    if args.len() < 2 {
        return Err(RuntimeError::new(list_span, RuntimeErrorKind::MalformedForm {
            head: ":wat::core::apply".into(),
            reason: format!(
                "expected (:wat::core::apply <fn> <a1>...<an> <args-vec>) — at least a fn and an args-vector; got {} arg(s)",
                args.len()
            )
        }).into());
    }

    // Step 1 — evaluate the head fn (args[0]).
    //
    // Arc 009 "names are values": a literal keyword that names a registered
    // user define evaluates to `Value::wat__core__fn`; runtime-built keywords
    // remain `Value::wat__core__keyword`. Both are valid apply heads.
    let head_val = eval_inner(&args[0], env, sym)?.value_owned();

    // Step 2 — evaluate leading positional args (args[1..last]). Empty if none.
    let leading_ast = &args[1..args.len() - 1];
    let mut combined: Vec<Value> = leading_ast
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    // Step 4 — evaluate last arg; must be :wat::core::Vector (spread).
    let spread_ast = &args[args.len() - 1];
    let spread_val = eval_inner(spread_ast, env, sym)?.value_owned();
    let spread_vec = match spread_val {
        Value::Vec(ref v) => v.clone(),
        ref other => {
            return Err(RuntimeError::new(
                spread_ast.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::apply".into(),
                    expected: "wat::core::Vector",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    combined.extend((*spread_vec).iter().cloned());

    // Step 5 — fast path: fn-valued head (Arc 009 lift OR let-bound fn).
    if let Value::wat__core__fn(func) = &head_val {
        return apply_function(func.clone(), combined, sym, list_span).map_err(Into::into);
    }

    // Stone O-ii — clause-set head. `dispatch_keyword_head` has had this arm since Stone 237.2
    // (runtime.rs:6758); `apply` never grew it, so every defclause — `+`, `reduce`, `sort` — was
    // refused by the keyword gate below. `combined` is already the evaluated args, which is
    // precisely what the value-level entry wants.
    if let Value::wat__core__clauses(cs) = &head_val {
        return crate::function::eval_call_to_defclause_with_vals(cs.clone(), combined, &list_span, sym);
    }

    // Step 6 — keyword-valued head: extract name + dispatch chain.
    let head_kw = match &head_val {
        Value::wat__core__keyword(k) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::apply".into(),
                    expected: "wat::core::keyword",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };

    // Step 7 — special-form rejection (STOP-8). Apply cannot dispatch to
    // declaration forms or language special forms — they require AST-level
    // structural parsing that eval_apply does not perform. Attempting to
    // apply them would silently misfire; reject with a clean diagnostic.
    const SPECIAL_FORMS: &[&str] = &[
        ":wat::core::def",
        // Stone 241.14 — ":wat::core::def-restricted" removed (HARD CUT; can't reach eval).
        // Stone 241.16 — ":wat::core::define" removed (HARD CUT total; eval-time residue completed).
        ":wat::core::defn",
        ":wat::core::fn",
        ":wat::core::let",
        ":wat::core::if",
        ":wat::core::do",
        ":wat::core::match",
        ":wat::core::quote",
        ":wat::core::quasiquote",
        // Arc 294.b — holon literal is a special form (body is data, not a callable).
        ":wat::holon::literal",
        // Arc 118 — lazy-seq is a special form (body is captured unevaluated, not a callable).
        ":wat::stream::lazy",
    ];
    if SPECIAL_FORMS.contains(&head_kw.as_str()) {
        return Err(RuntimeError::new(
            list_span,
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::apply".into(),
                reason: format!(
                    "cannot apply special form {:?} — apply only dispatches callable \
                 verbs and user-defined functions, not declaration or language forms",
                    head_kw.as_str()
                ),
            },
        )
        .into());
    }

    // Step 7 — dispatch (mirrors dispatch_keyword_head's `other` arm).
    // (a) User-defined function (defn / define registered in sym.functions).
    // STONE reap-the-angle-machinery (arc 109) — `head_kw` used to be stripped via
    // `canonical_callable_name`; angle syntax is unexpressible now, so it can never carry
    // a suffix — look it up directly (mirrors `def_value(head_kw.as_str())` just below,
    // which was already unstripped).
    if let Some(func) = sym.get(head_kw.as_str()) {
        return apply_function(func.clone(), combined, sym, list_span).map_err(Into::into);
    }

    // (b) def-bound callable value.
    if let Some(v) = sym.def_value(head_kw.as_str()) {
        match v {
            Value::wat__core__fn(f) => {
                return apply_function(f.clone(), combined, sym, list_span).map_err(Into::into);
            }
            other_val => {
                return Err(RuntimeError::new(
                    list_span,
                    RuntimeErrorKind::NotCallable {
                        got: Box::new(ValueSnapshot::of(other_val)),
                    },
                )
                .into());
            }
        }
    }

    // (c) substrate arithmetic / dispatch-impl verbs (pre-evaluated path).
    // Arc 255 Stone Q — pass the call's own `list_span`, not a synthesized one.
    if let Some(result) = dispatch_substrate_impl(head_kw.as_str(), &combined, &list_span) {
        return result;
    }

    // (d) Registered, but with no value-level door. Stone O-iv-a — `apply` used to call
    // these "unknown function", which is false: the registry holds the name. A BINDING
    // handler takes `&[WatAST]` and evaluates its own arguments; `apply` has already
    // evaluated its arguments and holds `&[Value]`, so there is no AST left to hand it.
    if crate::intrinsic::registry()
        .lookup_entry(head_kw.as_str())
        .is_some()
    {
        return Err(RuntimeError::new(
            list_span,
            RuntimeErrorKind::NotValueDispatchable {
                name: head_kw.as_str().to_string(),
            },
        )
        .into());
    }

    // (e) Genuinely not registered anywhere — UnknownFunction, and now it means it.
    Err(RuntimeError::new(
        list_span,
        RuntimeErrorKind::UnknownFunction(head_kw.as_str().to_string()),
    )
    .into())
}

/// `:wat::core::=` — structural equality. Composites (Vec, Tuple,
/// Option, Result, Struct) compare element-/field-wise; primitives
/// fall through to the `eval_compare` path. Split from `eval_compare`
/// because equality generalizes cleanly over composite values while
/// ordering (`<`, `>`, `<=`, `>=`) does not — a Vec of structs has no
/// canonical ordering worth inventing here.
fn eval_eq(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: head.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let a_span = args[0].span().clone();
    let a = eval_inner(&args[0], env, sym)?.value_owned();
    let b = eval_inner(&args[1], env, sym)?.value_owned();
    match values_equal(&a, &b) {
        Some(eq) => Ok(Value::bool(eq)),
        None => Err(RuntimeError::new(
            a_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "matching comparable pair",
                got: Box::new(ValueSnapshot::of(&a)),
            },
        )
        .into()),
    }
}

/// `(:wat::core::not= a b)` — Clojure-tradition inequality.
///
/// Inverse of `:wat::core::=`. Same polymorphism (cross-numeric
/// promotion, structural equality on composites, Enum equality post-
/// arc-056-companion). The runtime is `not(=)`; the type checker
/// shares `infer_comparison` so call-site type rules are
/// identical.
///
/// `(not= a b)` reads more naturally aloud than `(not (= a b))` and
/// follows the Clojure lineage. The C-family alternative `!=` was
/// passed over to keep the substrate's operator naming Lisp-shaped.
fn eval_not_eq(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    match eval_eq(head, args, list_span, env, sym)? {
        Value::bool(eq) => Ok(Value::bool(!eq)),
        // Unreachable — eval_eq always returns Value::bool on success.
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "bool from inner eq",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Structural equality on [`Value`] — returns `Some(bool)` for pairs
/// whose types support equality, `None` for pairs whose shapes aren't
/// comparable at all (e.g., comparing a `Value::Function` to anything;
/// two values of different top-level kinds; a struct to a tuple).
///
/// f64 uses `PartialEq`; `NaN == NaN` is false (Rust's standard
/// IEEE-754 semantics). Callers who need exact bit equality should
/// encode through an integer representation.
pub(crate) fn values_equal(a: &Value, b: &Value) -> Option<bool> {
    match (a, b) {
        (Value::i64(x), Value::i64(y)) => Some(x == y),
        (Value::u8(x), Value::u8(y)) => Some(x == y),
        (Value::f64(x), Value::f64(y)) => Some(x == y),
        // Stone 237.8c — the (i64,f64)/(f64,i64) cross-numeric arms (arc 050) deleted.
        // THE DECISION (237.8a): the checker rejects mixed-numeric `=` before eval;
        // these arms are unreachable. HARD CUT.
        //
        // Arc 300 stone C4 — REINSTATED, category-aware: an i64 and an f64 are
        // different numeric categories (mirrors bigint↔f64 / rational↔f64
        // below, one type-pair over) — `Some(false)`, never a TypeMismatch.
        // This path IS reachable: `eval_in_frozen` (the frozen/oracle eval
        // path) does not run the checker, only `runtime::eval` directly —
        // the checker's rejection is a separate, complementary gate, not the
        // only path to `values_equal`. clj: `(= 1 1.0)` => false.
        (Value::i64(_), Value::f64(_)) => Some(false),
        (Value::f64(_), Value::i64(_)) => Some(false),
        // Arc 300 stone C1 — bigint equality. Same-type: structural
        // (`num_bigint::BigInt` implements `PartialEq`). Category-aware
        // cross-type: bigint↔i64 compares by VALUE (both INTEGER category —
        // clj: `(= 1N 1)` => true); bigint↔f64 is a DIFFERENT category and is
        // cleanly `false` (clj: `(= 1N 1.0)` => false), never a TypeMismatch —
        // this is the one deliberate exception to "cross-numeric falls to
        // None" above: `=`'s category-awareness is bigint's pinned contract.
        (Value::wat__core__BigInt(x), Value::wat__core__BigInt(y)) => Some(x == y),
        (Value::wat__core__BigInt(x), Value::i64(y)) => Some(x.as_ref() == &BigInt::from(*y)),
        (Value::i64(x), Value::wat__core__BigInt(y)) => Some(&BigInt::from(*x) == y.as_ref()),
        (Value::wat__core__BigInt(_), Value::f64(_)) => Some(false),
        (Value::f64(_), Value::wat__core__BigInt(_)) => Some(false),
        // Arc 300 stone C2 — rational equality. Same-type: structural
        // (`BigRational` implements `PartialEq`, already-reduced). Category-
        // aware cross-type: a genuine rational always has denominator >= 2
        // (Stone B's invariant — an integer-valued ratio already collapsed
        // at construction), so it is NEVER integer-valued and NEVER equal to
        // an i64/bigint (clj: `(= 1/2 1)` => false); rational↔f64 is a
        // DIFFERENT category too (clj: `(= 1/2 0.5)` => false) — same
        // deliberate cross-numeric-falls-to-`Some(false)` exception bigint
        // established immediately above, one type over.
        (Value::wat__core__Rational(x), Value::wat__core__Rational(y)) => Some(x == y),
        (Value::wat__core__Rational(_), Value::i64(_)) => Some(false),
        (Value::i64(_), Value::wat__core__Rational(_)) => Some(false),
        (Value::wat__core__Rational(_), Value::wat__core__BigInt(_)) => Some(false),
        (Value::wat__core__BigInt(_), Value::wat__core__Rational(_)) => Some(false),
        (Value::wat__core__Rational(_), Value::f64(_)) => Some(false),
        (Value::f64(_), Value::wat__core__Rational(_)) => Some(false),
        (Value::String(x), Value::String(y)) => Some(x == y),
        (Value::bool(x), Value::bool(y)) => Some(x == y),
        (Value::wat__core__keyword(x), Value::wat__core__keyword(y)) => Some(x == y),
        // Arc 207 — Uuid equality. `uuid::Uuid` implements `PartialEq`.
        // Two Uuid values with the same content are equal; a Uuid and a
        // String holding the same 36 chars are NOT equal (cross-type
        // falls through to `_ => None`). UUIDs are identifiers, not ordinals;
        // no `values_compare` arm is added (correct: same as keyword/Enum/Struct).
        (Value::wat__core__Uuid(x), Value::wat__core__Uuid(y)) => Some(x == y),
        // Arc 220 — Char equality. `char` implements `PartialEq`.
        // Two Char values with the same codepoint are equal; a Char and a
        // String are NOT equal (cross-type falls through to `_ => None`).
        (Value::wat__core__Char(x), Value::wat__core__Char(y)) => Some(x == y),
        (Value::Unit, Value::Unit) => Some(true),
        (Value::Vec(xs), Value::Vec(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        // Arc 220 Stone 220.4 — List same-type structural equality.
        (Value::wat__core__List(xs), Value::wat__core__List(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        // Arc 220 Stone 220.4 — Cross-type sequence equality per EDN spec §282-289.
        // `List(1,2,3) == Vector(1,2,3)` returns true in the structural-equality path.
        (Value::Vec(xs), Value::wat__core__List(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        (Value::wat__core__List(xs), Value::Vec(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        (Value::Tuple(xs), Value::Tuple(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        (Value::Option(x), Value::Option(y)) => match (&**x, &**y) {
            (None, None) => Some(true),
            (Some(_), None) | (None, Some(_)) => Some(false),
            (Some(xv), Some(yv)) => values_equal(xv, yv),
        },
        (Value::Result(x), Value::Result(y)) => match (&**x, &**y) {
            (Ok(xv), Ok(yv)) => values_equal(xv, yv),
            (Err(xv), Err(yv)) => values_equal(xv, yv),
            _ => Some(false),
        },
        // Arc 048 — user-defined enum equality. Two Enum values are
        // equal iff they have the same `type_path`, the same
        // `variant_name`, and structurally-equal fields. This makes
        // `(=)` and `(not=)` work on PhaseLabel / PhaseDirection /
        // any user enum without callers writing match-by-match
        // boilerplate.
        (Value::Enum(a), Value::Enum(b)) => {
            if a.type_path != b.type_path || a.variant_name != b.variant_name {
                return Some(false);
            }
            if a.fields.len() != b.fields.len() {
                return Some(false);
            }
            for (x, y) in a.fields.iter().zip(b.fields.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        // Arc 052 — Vector equality is bit-exact: dim must match and
        // every i8 element must match. Forced by the Hash + Eq contract
        // for use as HashMap/LruCache keys. For graded similarity, reach
        // for cosine / presence? / simhash.
        (Value::Vector(a), Value::Vector(b)) => {
            if a.dimensions() != b.dimensions() {
                return Some(false);
            }
            Some(a.data() == b.data())
        }
        // Arc 073 — HolonAST structural equality. The closed algebra
        // already implements PartialEq/Eq with the f64-to_bits NaN
        // dance; values_equal exposes that to wat-side `:wat::core::=`.
        // Distinct from `coincident?` (which encodes both sides and
        // compares on the algebra grid via cosine + sigma): this is
        // the bit-exact structural predicate, the one a HashMap or a
        // term-store template-key dispatch lookup needs.
        (Value::holon__HolonAST(a), Value::holon__HolonAST(b)) => Some(a == b),
        // Arc 293.R2.1 — Aggregate (all natures). Cross-nature → false (nature check first).
        (Value::Aggregate(x), Value::Aggregate(y)) => {
            if x.nature != y.nature {
                return Some(false);
            }
            if x.class != y.class {
                return Some(false);
            }
            if x.fields.len() != y.fields.len() {
                return Some(false);
            }
            for (xf, yf) in x.fields.iter().zip(y.fields.iter()) {
                match values_equal(xf, yf) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        // Arc 238 Stone 238.1 — HashMap structural equality.
        // Delegates to Value's PartialEq (arc 216.5a; storage is Arc<HashMap<Value,Value>>).
        // Order-independent + structural + total. No numeric promotion (Hash-keyed storage
        // is type-sensitive; #{1} != #{1.0} is honest and documented in DESIGN.md).
        (Value::wat__std__HashMap(a), Value::wat__std__HashMap(b)) => Some(a == b),
        // Arc 238 Stone 238.1 — HashSet structural equality.
        // Delegates to Value's PartialEq (arc 216.5b; storage is Arc<HashSet<Value>>).
        // Order-independent (set semantics).
        (Value::wat__std__HashSet(a), Value::wat__std__HashSet(b)) => Some(a == b),
        // DESIGN-STONE-into-pv-from-vector.md — PersistentVector same-type structural
        // equality (order-dependent; a vector's order is semantic). A genuine pre-existing
        // gap surfaced by this stone's own test: `rpds::VectorSync<Value>` already implements
        // PartialEq and Value's manual Rust-level PartialEq impl already delegates to it
        // (value.rs:616, arc-278-0b) — but the wat-level `=` (`values_equal`, this fn) had no
        // arm at all, so no corpus test had ever compared two PersistentVectors before this
        // stone tried to `assert-eq` its own result. Deliberately NO cross-kind arm
        // (PersistentVector vs Vector/List) — unlike List<->Vector's EDN-spec cross-type
        // equality below, PersistentVector stays a genuinely distinct kind; the whole point of
        // this stone is that its receiver's kind is never conflated with a Vector's.
        (Value::wat__core__PersistentVector(a), Value::wat__core__PersistentVector(b)) => {
            Some(a == b)
        }
        // Arc 238 Stone 238.1 — Instant equality. chrono::DateTime<Utc> implements Eq.
        // Mirrors the values_compare Instant arm (runtime.rs:9609).
        // Closes the orderable-but-not-equatable asymmetry (Instant had values_compare but not values_equal).
        (Value::Instant(a), Value::Instant(b)) => Some(a == b),
        // Arc 238 Stone 238.1 — Duration equality. i64 nanoseconds; mirrors values_compare.
        (Value::Duration(a), Value::Duration(b)) => Some(a == b),
        // Arc 238 Stone 238.1 — WatAST structural equality.
        // WatAST derives PartialEq (ast.rs:33; span-agnostic — two nodes with same structure
        // but different spans compare equal). Symmetry with the holon__HolonAST arm above.
        (Value::wat__WatAST(a), Value::wat__WatAST(b)) => Some(a == b),
        _ => None,
    }
}

/// Structural ordering on [`Value`] — returns `Some(Ordering)` for pairs
/// whose types support ord, `None` for pairs whose shapes aren't ordered
/// at all (e.g., HashMap, HashSet, Enum, Struct, HolonAST, unit, fn).
///
/// Mirrors [`values_equal`]'s recursive shape (arc 148 slice 3): the
/// arms accepted here are the ord-comparable subset of the arms accepted
/// by `values_equal`. The two functions are kept in lockstep —
/// `values_equal`'s recursive arms are extended here for `Vec`, `Tuple`,
/// `Option`, `Result`, and `Vector`; new leaf arms cover `Instant`,
/// `Duration`. `Bytes` (which is `(:wat::core::Vector :- [wat::core::u8])` at
/// the type level and `Value::Vec` of `Value::u8` at runtime) is covered
/// by the `Vec` arm recursing into the `u8` arm — no separate Bytes
/// variant exists.
///
/// Variant-ordered semantics (matching Rust's stdlib defaults):
/// - `Option`: `None < Some(_)`; `Some(x) cmp Some(y) = x cmp y`.
/// - `Result`: `Err < Ok`; same-variant pairs recurse on their payload.
///
/// Vec / Tuple / Vector use lexicographic order with shorter-is-less
/// tie-break (matches Rust's `Vec::cmp` / slice cmp). f64 uses
/// `partial_cmp` and treats NaN-involved comparisons as `Equal` (the
/// same posture `eval_compare` adopted before slice 3).
///
/// Returns `None` for any pair that values_equal would also return
/// `None` for, plus pairs whose type lacks a canonical order
/// (HashMap / HashSet / Enum / Struct / HolonAST / unit). Callers
/// (currently `eval_compare`) translate `None` into `TypeMismatch`.
fn values_compare(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    match (a, b) {
        (Value::i64(x), Value::i64(y)) => Some(x.cmp(y)),
        (Value::u8(x), Value::u8(y)) => Some(x.cmp(y)),
        (Value::f64(x), Value::f64(y)) => Some(x.partial_cmp(y).unwrap_or(Ordering::Equal)),
        // Arc 300 stone C5b — the 6 lossy f64-cross arms (i64/BigInt/Rational vs f64,
        // both directions) all route through the one exact ordering door instead of
        // coercing the exact operand down to f64 (arc 050's original coerce-down
        // arms lost precision above 2^53: `(< 9007199254740992.0 9007199254740993)`
        // was `false`; the door promotes the f64 UP to an exact `BigRational` via
        // `from_f64`, never approximates). Policy here: NaN preserved as `Equal`
        // (this caller's existing posture, unwrap_or(Equal), byte-identical to
        // before). NotNumeric cannot occur for these type-guaranteed pairs.
        (Value::i64(_), Value::f64(_))
        | (Value::f64(_), Value::i64(_))
        | (Value::wat__core__BigInt(_), Value::f64(_))
        | (Value::f64(_), Value::wat__core__BigInt(_))
        | (Value::wat__core__Rational(_), Value::f64(_))
        | (Value::f64(_), Value::wat__core__Rational(_)) => {
            match crate::value::numeric_order::numeric_order(a, b) {
                crate::value::numeric_order::NumOrd::Ord(o) => Some(o),
                crate::value::numeric_order::NumOrd::Incomparable => Some(Ordering::Equal),
                crate::value::numeric_order::NumOrd::NotNumeric => {
                    unreachable!(
                        "numeric_order given a pair the match pattern already proved numeric"
                    )
                }
            }
        }
        // Arc 300 stone C1 — bigint total order. Same-type: `BigInt`
        // implements `Ord`. Cross-type with i64: promote i64 to bigint
        // before comparing (mirrors the i64↔f64 promotion pattern above;
        // `cf i64↔f64 :8369` per the design's room table).
        (Value::wat__core__BigInt(x), Value::wat__core__BigInt(y)) => Some(x.cmp(y)),
        (Value::wat__core__BigInt(x), Value::i64(y)) => Some(x.as_ref().cmp(&BigInt::from(*y))),
        (Value::i64(x), Value::wat__core__BigInt(y)) => Some(BigInt::from(*x).cmp(y.as_ref())),
        // Arc 300 stone C4 — bigint↔f64 total order (was missing; grounding
        // showed `(< 1N 2.0)` had no arm). The bigint↔f64 and rational↔f64 mixed
        // arms both route through the C5b door above (combined with the
        // i64↔f64 arm) — see the `numeric_order` match arm just above `Value::f64`.
        // Arc 300 stone C2 — rational total order. Same-type: `BigRational`
        // implements `Ord` (cross-multiplication, exact — no float rounding).
        // Cross-type with i64/bigint: promote the integer side to a
        // `BigRational` before comparing (mirrors the i64↔bigint promotion
        // pattern immediately above, one type over). Cross-type with f64: see
        // the C5b door above — no longer coerced down to f64.
        (Value::wat__core__Rational(x), Value::wat__core__Rational(y)) => Some(x.cmp(y)),
        (Value::wat__core__Rational(x), Value::i64(y)) => {
            Some(x.as_ref().cmp(&BigRational::from_integer(BigInt::from(*y))))
        }
        (Value::i64(x), Value::wat__core__Rational(y)) => {
            Some(BigRational::from_integer(BigInt::from(*x)).cmp(y.as_ref()))
        }
        (Value::wat__core__Rational(x), Value::wat__core__BigInt(y)) => {
            Some(x.as_ref().cmp(&BigRational::from_integer((**y).clone())))
        }
        (Value::wat__core__BigInt(x), Value::wat__core__Rational(y)) => {
            Some(BigRational::from_integer((**x).clone()).cmp(y.as_ref()))
        }
        (Value::String(x), Value::String(y)) => Some(x.cmp(y)),
        (Value::bool(x), Value::bool(y)) => Some(x.cmp(y)),
        (Value::wat__core__keyword(x), Value::wat__core__keyword(y)) => Some(x.cmp(y)),
        // Arc 148 slice 3 — time ord. chrono::DateTime<Utc> implements Ord
        // (chronological); Duration is a non-negative i64 nanosecond count
        // and uses i64 ord directly.
        (Value::Instant(x), Value::Instant(y)) => Some(x.cmp(y)),
        (Value::Duration(x), Value::Duration(y)) => Some(x.cmp(y)),
        // Arc 148 slice 3 — Vec lex ord, recursive. Element-wise
        // comparison; first non-Equal element decides; on a prefix tie,
        // shorter < longer (matches Rust's `Vec::cmp`). Returns None if
        // any pair of elements isn't ord-comparable. Bytes (Value::Vec of
        // Value::u8) flows through here.
        (Value::Vec(xs), Value::Vec(ys)) => {
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_compare(x, y)? {
                    Ordering::Equal => continue,
                    non_eq => return Some(non_eq),
                }
            }
            Some(xs.len().cmp(&ys.len()))
        }
        // Arc 148 slice 3 — Tuple lex ord, recursive. Same shape as Vec
        // (lexicographic with shorter-is-less tie-break). Tuples of
        // different declared arities still hit this arm because they
        // share Value::Tuple; the type checker would normally reject
        // mixed arities upstream, but the lex semantics is honest if it
        // ever arrives.
        (Value::Tuple(xs), Value::Tuple(ys)) => {
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_compare(x, y)? {
                    Ordering::Equal => continue,
                    non_eq => return Some(non_eq),
                }
            }
            Some(xs.len().cmp(&ys.len()))
        }
        // Arc 148 slice 3 — Option variant-order: None < Some(_); same
        // variant recurses on payload (matches Rust's stdlib).
        (Value::Option(x), Value::Option(y)) => match (&**x, &**y) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (Some(xv), Some(yv)) => values_compare(xv, yv),
        },
        // Arc 148 slice 3 — Result variant-order: Err < Ok; same variant
        // recurses on payload (matches Rust's stdlib `Result::cmp`).
        (Value::Result(x), Value::Result(y)) => match (&**x, &**y) {
            (Err(xv), Err(yv)) => values_compare(xv, yv),
            (Ok(xv), Ok(yv)) => values_compare(xv, yv),
            (Err(_), Ok(_)) => Some(Ordering::Less),
            (Ok(_), Err(_)) => Some(Ordering::Greater),
        },
        // Arc 148 slice 3 — algebra Vector ord (bit-exact element-wise
        // i8 lex; matches the bit-exact `values_equal` Vector arm).
        // Different-dim Vectors fall to length lex on the i8 slice; the
        // type checker would normally enforce dim equality upstream, but
        // honest lex if mismatched values arrive.
        (Value::Vector(x), Value::Vector(y)) => Some(x.data().cmp(y.data())),
        _ => None,
    }
}

pub(crate) fn eval_compare<F: Fn(std::cmp::Ordering) -> bool>(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    pred: F,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: head.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let a_span = args[0].span().clone();
    let a = eval_inner(&args[0], env, sym)?.value_owned();
    let b = eval_inner(&args[1], env, sym)?.value_owned();
    // Arc 300 stone C5c — consult the exact ordering door FIRST. IEEE 754: every
    // ordered comparison involving NaN is false, for ALL FOUR of `< > <= >=` — so
    // `Incomparable` short-circuits to `false` regardless of which predicate `pred`
    // is (a `!= Greater` spelling of `<=` would otherwise read NaN's collapsed
    // `Equal` as true). Non-numeric pairs (`NotNumeric`) fall through to the
    // existing `values_compare` path, byte-identical to before this stone.
    // `values_compare` itself is NOT changed — its own NaN->`Equal` posture is the
    // collection-totality seam this stone deliberately leaves (see the design stone).
    let result = match crate::value::numeric_order::numeric_order(&a, &b) {
        crate::value::numeric_order::NumOrd::Ord(o) => pred(o),
        crate::value::numeric_order::NumOrd::Incomparable => false,
        crate::value::numeric_order::NumOrd::NotNumeric => match values_compare(&a, &b) {
            Some(o) => pred(o),
            None => {
                return Err(RuntimeError::new(
                    a_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: head.into(),
                        expected: "matching comparable pair",
                        got: Box::new(ValueSnapshot::of(&a)),
                    },
                )
                .into());
            }
        },
    };
    Ok(Value::bool(result))
}

// Arc 109 Stone 1 — `eval_f64_compare` moved to `src/numeric/compare.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Stone 237.8b — HARD CUT:
//   - ArithOp enum + impl (Add/Sub/Mul/Div, allows_zero_ary, one_ary_inserts_identity, identity_i64)
//   - apply_arith_pair (binary step for variadic fold)
//   - eval_arithmetic_variadic (the variadic evaluator itself)
//
// All replaced by wat defclauses in wat/core.wat. The defclause clauses fold
// over per-Type binary primitives (i64::+, f64::+ etc.) which remain as
// 2-ary Rust intrinsics. No variadic arithmetic dispatch needed at Rust level.
//
// Tombstone for grep-history only (search these names to find this comment):
//   ArithOp, apply_arith_pair, eval_arithmetic_variadic
//   allows_zero_ary, one_ary_inserts_identity, identity_i64

// (tombstone end)

/// `(:wat::core::not b)` → the boolean negation of `b`.
///
/// **Purity ground —** the sole arg is evaluated by ordinary call-by-value (`eval_inner`, not
/// itself an effect); past that the body only matches `Value::bool` and returns its inverse —
/// no `eval_inner`/`apply_function` on caller-supplied code beyond the initial evaluation.
///
/// **Totality ground —** every `bool` is one of exactly two values and each maps to its own
/// inverse with no failure path inside the domain; the only error this fn can raise is a
/// `TypeMismatch` for a non-bool argument, outside the declared `bool -> bool` domain (same
/// reasoning as `:wat::core::bool::to-string`'s `Total`, registered alongside this verb).
///
/// **Expand-time ground —** on `macros/eval.rs`'s `is_expand_time_legal` residue list today
/// (the "value/control-flow ops" group names `not` explicitly), so it is legal inside a macro
/// body today; registering it here REPLACES that residue entry, so it must declare the SAME
/// verdict — `Legal` — or the registration silently revokes today's legality (arc 255 the
/// `fn` lesson).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     args :wat::core::bool the boolean to negate
/// @ret     :wat::core::bool the inverse of `args`
/// @example (:wat::core::not true) #=> false
#[wat_intrinsic(":wat::core::not")]
fn eval_not(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::not".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let arg_span = args[0].span().clone();
    match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::bool(b) => Ok(Value::bool(!b)),
        other => Err(RuntimeError::new(
            arg_span,
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::not".into(),
                expected: "bool",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

// Arc 255 Stone 1a-i — `eval_and`/`eval_or` DELETED. Their only callers were the
// `dispatch_keyword_head_value` match arms retired above; `:wat::core::and`/`:wat::core::or`
// now dispatch through the registry's `role = eval` handler, which is `eval_and_tail`/
// `eval_or_tail` themselves (STOP-1's stacked eval+tail attribute pair), not a separate fn.
// This is the one place this stone's "no verb changes behaviour" promise narrows: `eval_and`
// raised a runtime `TypeMismatch` on a non-bool LAST operand; `eval_and_tail` tail-calls that
// operand away instead (the arc 278 #59 RULED weakening, previously observable only from tail
// position). Deleting `eval_and` extends that same weakening to every call position — observable
// only on the same already-exotic bypass the #59 pinning test exercises (a `quote`d `fn` literal
// invoked via `:wat::eval-ast!`, never type-checked), never on statically-checked source, where
// `infer_boolean_shortcircuit` already forces every operand, including the last, to `:bool`.
// Flagged for the orchestrator rather than decided silently; see the stone's report.

// Arc 146 slice 3 — `eval_conj` retired. The polymorphism is honest
// now: a Dispatch (declared in `wat/core.wat`) routes
// `:wat::core::conj` to `:Vector/conj` and `:HashSet/conj` per-Type
// impls (above). HashMap doesn't conj — it requires key+value
// pairing, so `:wat::core::assoc` is the right verb there.

/// `(:wat::core::Tuple a b c ...)` — build a heterogeneous tuple
/// `Value::Tuple`. Arity 1+; the 0-tuple is the unit `:()` handled
/// elsewhere. Ships 2026-04-19 to support wat-source programs that
/// need to RETURN tuples (earlier slices saw tuples only as
/// primitive return values; Path-B Console needs to construct
/// `(pool, driver-handle)` in wat source).
/// Post-arc-165: `:wat::core::Tuple` is the canonical PascalCase
/// spelling per slice 1f's vec→Vector playbook completed.
fn eval_tuple_ctor(
    args: &[WatAST],
    list_span: &Span,

    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        // The user's own `(:wat::core::Tuple …)` form — `list_span` was a parameter the whole
        // time. The prior "arc 138: no span — leaf helper" was false, and the rune that cited
        // `rust_caller_span!()` as being "located elsewhere" was citing the harm as the cure.
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::Tuple".into(),
                reason: "tuple must have at least one element; the 0-tuple is :() (Unit)".into(),
            },
        )
        .into());
    }
    let items = args
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Tuple(Arc::new(items)))
}

/// Require a `Vec` argument. Used by list primitives that take one
/// Vec as their sole / first arg.
pub(crate) fn require_vec(op: &'static str, v: Value) -> Result<Arc<Vec<Value>>, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok(xs),
        // arc 138: no span — require_vec is a value-level helper without
        // AST context; the caller's arg span isn't threaded through every
        // call site (would expand the helper signature across ~30 callers).
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::core::Vector",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Require an `i64` argument. Used by list primitives whose second
/// arg is a count / index.
pub(crate) fn require_i64(op: &'static str, v: Value) -> Result<i64, EvalBreak> {
    match v {
        Value::i64(n) => Ok(n),
        // arc 138: no span — same rationale as `require_vec` above.
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Arc 146 slice 2 — substrate-primitive impl dispatch from
/// `eval_dispatch_call` when an arm's impl is NOT a user-define
/// `Function` (i.e., not present in `sym.functions`). Routes to the
/// per-Type Rust handler, passing the already-evaluated values so
/// side effects fire exactly once (sym.get path also fires once
/// via apply_function with pre-evaluated values).
///
/// Returns `Some(result)` when the impl is a known substrate
/// primitive; `None` when no substrate impl matches (caller surfaces
/// `UnknownFunction`).
///
/// Arc 146 slice 3 extends this with the per-Type empty? / contains? /
/// get / conj impls (10 new arms; 3+3+2+2).
///
/// Arc 146 slice 4 extends this with the per-Type assoc / dissoc /
/// keys / values / concat impls (5 new arms). These ops aren't
/// dispatched (alias-expansion goes through user-define), but the
/// substrate-impl entries here let the alias body resolve when the
/// per-Type impl is the named target.
///
/// Arc 255 Stone C — the per-type numeric arms below are keyed DIRECTLY on the
/// surviving `:wat::i64::*` / `:wat::f64::*` spelling. Through Stone B, this
/// function folded the new spelling onto its `:wat::core::` twin before matching
/// (`fold_numeric_home`, deleted this stone) rather than carrying a second copy of
/// 36 arms; now that the old spelling is retired, the fold has nothing left to
/// fold onto, so the arms are the new spelling's ONLY implementation and the fold
/// is gone. Reached via `apply` of a BOUND keyword — `(let [plus :wat::i64::+]
/// (apply plus [2 3]))` — which arrives at this table directly.
///
/// Arc 255 Stone N — **the registry is consulted FIRST.** HOME-13 (retracted)
/// found this fn was the second of two dispatch tables — `apply`'s substrate
/// fallback, entirely registry-blind. Every one of the 44 named arms below
/// now ALSO carries a `value_handler` registered under the SAME fqdn
/// (`IntrinsicSubmission::value_handler`, `src/intrinsic/mod.rs`) — the exact
/// value-level implementation each literal arm below already called (the
/// SAME `*_inner` / `arith_*_inner` fn; no new arithmetic or algorithm). So
/// for all 44, the registry lookup below fires and the literal match never
/// runs; the match is kept, byte-for-byte, as the fallback for any name NOT
/// (yet) registered with a `value_handler` — this stone makes the 44 arms
/// **removable**, it does not remove them (STOP-2). A verb whose
/// `value_handler` is sabotaged and whose result changes under `apply` is
/// the proof the registry — not this match — now serves that verb.
///
/// Arc 255 Stone HOME-13 (reinstated) — the 44 arms Stone N made removable
/// are now REMOVED; the registry lookup below is this fn's entire body for
/// those fqdns, and unregistered names fall through to `None`. Before
/// removal, the `:wat::vec::concat` / `:wat::vector::concat` pair (and
/// `:wat::vec::extend` alongside it) carried a note worth keeping: this fn
/// is also `eval_apply`'s substrate fallback, not only a const-eval path,
/// so a per-Type leaf registered ONLY on `dispatch_keyword_head_value`'s
/// keyword-dispatch arm would have left `apply` unable to reach it — an
/// avoidable split-brain (see `docs/arc/2026/06/278-rules-engine/
/// DESIGN-STONE-into-pv-from-vector.md` for the concat op itself; the
/// split-brain risk was runtime.rs-local and undocumented there). The
/// `value_handler` registrations above close that risk for all 44 at once,
/// so it no longer needs a per-arm note now that there is no per-arm table.
// Arc 255 Stone Q — gained a trailing `&Span` param. `apply` (the one caller, below)
// already holds `list_span`; this fn simply forwards it to `handler` now that
// `ValueHandler` has somewhere to put it.
// Arc 255 Stone Q-2 — the arity-mismatch diagnostic just below now USES `span` instead
// of synthesizing `rust_caller_span!()`: Q threaded the span, Q-2 is the stone that
// spends it. A wrong-arity `apply` now points at the user's `.wat` call site.
pub(crate) fn dispatch_substrate_impl(
    impl_name: &str,
    vals: &[Value],
    span: &Span,
) -> Option<Result<Value, EvalBreak>> {
    // Arc 255 Stone O-i — the value door gets the same arity guard the AST
    // door has always had (`crates/wat-macros/src/wat_intrinsic.rs`'s
    // generated shim). Without this, every value handler's opening
    // `vals.first().expect("arity-checked")` names a check that happened on
    // the OTHER door only, and a wrong-arity `apply` panics the process
    // instead of returning the clean `ArityMismatch` the direct call gives.
    //
    // ONE lookup, not two. `lookup_value` had exactly one caller — this line — so
    // consulting the entry for BOTH the handler and its arity retires it rather than
    // paying a second `HashMap::get` to keep a single-purpose accessor alive. Two ways
    // to ask the registry one question is the shape this arc exists to delete.
    let entry = crate::intrinsic::registry().lookup_entry(impl_name)?;
    let handler = entry.value_handler?;
    if let crate::intrinsic::Arity::Exact(n) = entry.arity {
        if vals.len() != n {
            return Some(Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::ArityMismatch {
                    op: impl_name.into(),
                    expected: n,
                    got: vals.len(),
                },
            )
            .into()));
        }
    }
    Some(handler(vals, span))
}

// Arc 109 Stone 1 — `I64ArithErr` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `arith_i64_i64_inner` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `arith_f64_f64_inner` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `arith_bigint_bigint_inner` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 1 — `arith_rational_rational_inner` moved to `src/numeric/arith.rs` (the numeric home;
// docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// arc 237 Stone 237.8a — arith_i64_f64_inner and arith_f64_i64_inner
// DELETED under THE DECISION (`feedback_no_implicit_coercion`).
// Their only callers were the +'i64'f64 / +'f64'i64 dispatch arms,
// which are also deleted.

// Arc 146 slice 3 — `eval_empty_q` retired. The polymorphism is
// honest now: a Dispatch (declared in `wat/core.wat`) routes
// `:wat::core::empty?` to the per-Type `:Vector/empty?` /
// `:HashMap/empty?` / `:HashSet/empty?` impls above.
//
// Arc 146 slice 4 — `eval_concat` retired. The single-impl polymorphism
// is honest now: an alias (declared in `wat/core-aliases.wat`) maps
// `:wat::core::concat` to the per-Type `:wat::core::Vector/concat`
// impl above. Variadic 1+ arg shape collapsed to honest binary; callers
// nest for >2 args (or fold).

// Stone 216.5b — runtime hashability guard.
// Returns `false` for the 14 opaque-handle `Value` variants that carry
// `unreachable!()` in `impl Hash for Value`. These variants are not
// atomizable and should not be inserted into a `HashSet<Value>` at the WAT surface.
// Called by `eval_hashset_ctor` and `hashset_conj_inner` BEFORE `HashSet::insert`
// so that a user-visible `TypeMismatch` is returned instead of an `unreachable!()`
// panic. The `is_atomizable` check-time predicate (src/check.rs:3623) is the static
// guarantee; this guard is the runtime defence-in-depth for inferred types.
/// Stone 216.5c — shared hashability predicate.
///
/// Returns `false` for the 14 opaque-handle variants (those that receive
/// `unreachable!()` in `impl Hash for Value`). All other variants — including
/// structurally-hashable non-atomizable ones like `u8`, `Tuple`, `Option`, etc. —
/// return `true`. Callers rely on this before inserting into `HashSet<Value>`
/// or `HashMap<Value, _>` to preserve WAT-surface TypeMismatch behavior
/// instead of hitting the `unreachable!()` panic.
///
/// **Unification decision:** `value_is_set_hashable` and `value_is_key_hashable`
/// have identical bodies (same 14 opaque-handle variants). They are both thin
/// wrappers over this function. Separate names are kept for call-site clarity
/// (set insert vs. map key insert) but the predicate logic is defined once.
pub fn value_is_hashable(v: &Value) -> bool {
    !matches!(
        v,
        Value::wat__core__fn(_)
            | Value::wat__kernel__Sender(_)
            | Value::wat__kernel__Receiver(_)
            | Value::wat__kernel__HandlePool { .. }
            | Value::wat__kernel__ChildHandle(_)
            | Value::RustOpaque(_)
            | Value::io__IOReader(_)
            | Value::io__IOWriter(_)
            | Value::OnlineSubspace(_)
            | Value::Reckoner(_)
            | Value::Engram(_)
            | Value::EngramLibrary(_)
            | Value::Hologram(_)
    )
}

/// Guard for `HashSet<Value>` insert sites. Delegates to `value_is_hashable`.
/// Preserves WAT-surface TypeMismatch for opaque-handle elements (they can
/// never be inserted into a HashSet; contains? on them is always false).
pub fn value_is_set_hashable(v: &Value) -> bool {
    value_is_hashable(v)
}

/// Guard for `HashMap<Value, _>` key insert sites. Delegates to `value_is_hashable`.
/// Parallel to `value_is_set_hashable` (Stone 216.5b) for HashMap keys.
/// Preserves WAT-surface TypeMismatch instead of hitting `unreachable!()` in Hash.
pub fn value_is_key_hashable(v: &Value) -> bool {
    value_is_hashable(v)
}

// Arc 146 slice 3 — `eval_get` retired. The polymorphism is honest
// now: a Dispatch (declared in `wat/core.wat`) routes
// `:wat::core::get` to `:Vector/get` (Vec×i64 → (Option :- [T])) and
// `:HashMap/get` ((HashMap :- [K V])×K → (Option :- [V])). HashSet's
// "get-by-equality" is just `:contains?` per arc 146 DESIGN audit
// table.

// Arc 146 slice 4 — `eval_dissoc` / `eval_keys` / `eval_values` retired.
// The single-impl polymorphism is honest now: aliases (declared in
// `wat/core-aliases.wat`) map the short surface names to the per-Type
// `:wat::core::HashMap/dissoc` / `keys` / `values` impls (defined above
// adjacent to the slice 2/3 per-Type block). The pre-arc-146 Vec branch on
// assoc was a Vec-as-HashMap-anachronism per arc 146 DESIGN audit table;
// Vec/set is the honest verb for "replace at index" and lives independently.
//
// Arc 237 Stone 237.7c — `eval_assoc` is LIVE below (see fn eval_assoc).

/// `(:wat::core::assoc coll key new-value)` — arc 237 Stone 237.7c.
///
/// Polymorphic write verb spanning two heterogeneous collection families:
///
/// - `Value::wat__std__HashMap(_)` → `hashmap_assoc_inner` (functional clone-insert).
/// - `Value::Aggregate (Record/HolonRecord nature)` →
///   `eval_record_assoc` (base early-return rebuilds fields only; holonic fallthrough
///   rebuilds BOTH fields + hologram in parity — the PARITY invariant).
///   Flavor is preserved: base → base, holonic → holonic.
/// - else → teaching `RuntimeError::TypeMismatch`.
///
/// Arc 255 Stone the-collection-readers — homed into a thin `#[wat_intrinsic]` delegate
/// (`src/intrinsic/collection.rs`) with its real (3) arity declared; the shim's own arity
/// check makes this fn's hand-rolled `args.len() != 3` guard dead, so it retires here. `pub(crate)`
/// so the delegate can call it.
pub(crate) fn eval_assoc(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::assoc";
    let arg0_val = eval_inner(&args[0], env, sym)?.value_owned();
    let arg1_val = eval_inner(&args[1], env, sym)?.value_owned();
    let arg2_val = eval_inner(&args[2], env, sym)?.value_owned();
    use crate::collection::map_container::MapContainer;
    match MapContainer::of_value(&arg0_val) {
        Some(m) if m.can_assoc() => match m {
            // exhaustive over MapContainer, no `_`
            MapContainer::HashMap => {
                crate::collection::eval::hashmap_assoc_inner(&arg0_val, &arg1_val, &arg2_val)
            }
            MapContainer::PersistentMap => {
                crate::collection::eval::persistentmap_assoc_inner(&arg0_val, &arg1_val, &arg2_val)
            }
            MapContainer::Record => {
                crate::record::update::record_assoc_inner(arg0_val, arg1_val, arg2_val, list_span, sym)
            }
        },
        Some(_) => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(HashMap :- [K V]), (PersistentMap :- [K V]), or :wat::core::Record",
                got: Box::new(ValueSnapshot::of(&arg0_val)),
            },
        )
        .into()), // can_assoc()==false (none today; the slot)
        None => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(HashMap :- [K V]), (PersistentMap :- [K V]), or :wat::core::Record",
                got: Box::new(ValueSnapshot::of(&arg0_val)),
            },
        )
        .into()),
    }
}

// Arc 146 slice 3 — `eval_contains_q` retired. The polymorphism is
// honest now: a Dispatch (declared in `wat/core.wat`) routes
// `:wat::core::contains?` to per-Type impls with MIXED VERBS:
// `:Vector/contains?` (element membership), `:HashMap/contains-key?`
// (key membership), and `:HashSet/contains?` (element membership).
// The pre-arc-146 Vec×i64-as-valid-index check was retired with the
// semantic correction (use `(< i (length xs))` for index validity).

/// `(:wat::core::quote <expr>)` — capture an unevaluated AST.
///
/// This is the mechanism that places a wat program into the algebra as
/// data. The inner form is NOT evaluated at quote time — no side effects
/// fire, no functions are called. The AST is wrapped as a
/// `Value::wat__WatAST` and can be passed to `:wat::holon::Atom`,
/// `:wat::eval-ast!`, stored in environments, etc.
///
/// Quote is how programs become holons without running.
pub(crate) fn eval_quote(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::quote".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    Ok(Value::wat__WatAST(Arc::new(args[0].clone())))
}

// `eval_seq_empty`/`eval_cons` that used to live here (`:wat::stream::empty`/`cons`) moved to
// `src/intrinsic/stream.rs` — arc 255 Stone P6-c-W2, the P6-c campaign's second wave. Both
// were declaring a variadic `&[WatAST]` they used only to reject (a hand-rolled length
// check); homing them meant declaring the real arity (0 and 2) so `#[wat_intrinsic]`'s
// generated shim owns the check and `metadata-of` reports the true arity instead of a lie.

/// Arc 118 — `(:wat::stream::lazy <body>) -> (Stream :- [T])`. SPECIAL FORM (capture-don't-eval).
///
/// The body is NOT evaluated here. Instead it is captured as a 0-arg wat closure
/// over the current environment (`env.clone()` in `closed_env`), and wrapped in a
/// `Stream::Thunk(LazyCell{ thunk })`. The body runs ONLY when the seq is forced.
///
/// ⚠ It is NOT memoized. Stone 118.B3 deleted the `forced: OnceLock` cache this comment used to
/// promise ("runs at most ONCE"): forcing the same cell twice now runs its body twice. The cache
/// existed only to hide the three-call `first`/`rest`/`empty?` walk the stdlib itself used, and its
/// cost was retaining every cell ever forced — O(n) memory for a pipeline whose entire purpose is
/// not to have any. The stdlib walks with `:wat::stream::next` (one force per cell) since 118.B2b.
///
/// Mirrors `eval_quote`'s capture-don't-eval shape (runtime.rs) + the fn-closure
/// construction in `function::eval_fn` (a 0-param `Function` with `closed_env`).
// Arc 255 Stone 1a-zeta — widened `fn` -> `pub(crate) fn` so
// `intrinsic/special/stream_lazy.rs`'s thin `role = eval` delegate (needed because this fn's
// own 3-param signature — no `sym` — does not fit the canonical 4-param `NativeHandler` shape,
// same asymmetry `eval_quote`/`eval_fn` hit) can call it from another module. Body untouched.
pub(crate) fn eval_lazy_seq(args: &[WatAST], list_span: &Span, env: &Environment) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::stream::lazy".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    // Capture the body as a 0-arg closure over the current env (NOT evaluated now).
    let thunk = Arc::new(Function {
        name: None,
        params: Vec::new(),
        type_params: Vec::new(),
        param_types: Vec::new(),
        ret_type: crate::types::TypeExpr::Parametric {
            head: "wat::stream::Stream".into(),
            args: vec![crate::types::TypeExpr::Var(0)],
        },
        rest_param: None,
        rest_param_type: None,
        body: crate::value::FunctionBody::Wat(Arc::new(args[0].clone())),
        closed_env: Some(env.clone()),
        rete: None,
        synthesized_for: None,
    });
    Ok(Value::wat__stream__Stream(Arc::new(
        crate::stream::Stream::Thunk(crate::stream::LazyCell::new(thunk)),
    )))
}

// `NEXT_OUTCOME_TYPE`/`next_outcome_item`/`next_outcome_exhausted`/`eval_stream_next` that
// used to live here (`:wat::stream::next`) moved to `src/intrinsic/stream.rs` — arc 255
// Stone P6-c-W2. Declared a variadic `&[WatAST]` used only to reject (a hand-rolled length
// check); homing it meant declaring the real arity (1) so `#[wat_intrinsic]`'s generated
// shim owns the check.

/// `(:wat::core::ann-form <expr> <type>) -> T` — arc 251 Stone 251.4b.
///
/// Checked, type-erased identity. The type slot is ERASED at runtime;
/// only `expr` is evaluated and its value returned. The arity guard here
/// is belt-and-suspenders (the checker enforces arity 2 before runtime;
/// a well-typed program always has exactly 2 args).
///
/// Arc 255 Stone 1a-zeta — the `role = eval` pointer for `:wat::core::ann-form`. Annotated IN
/// PLACE (signature already fits the canonical `NativeHandler` shape) — see
/// `intrinsic/special/ann_form.rs` for the doc-only struct and the `role = check`/`role = tail`
/// pointers.
#[wat_special_form_impl(":wat::core::ann-form", role = eval)]
fn eval_ann_form(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::ann-form".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // Evaluate expr; erase the type slot (args[1] is ignored at runtime).
    eval_inner(&args[0], env, sym).map(|tv| tv.value_owned())
}

/// `(:wat::core::quasiquote <template>) -> :wat::WatAST`.
///
/// Arc 091 slice 8. Runtime quasiquote — same template shape
/// `defmacro` bodies use, but at expression position. Walks the
/// template; at each `(:wat::core::unquote X)` site evaluates X
/// in the surrounding environment and converts the resulting Value
/// to a WatAST literal node; returns the assembled form as a
/// `Value::wat__WatAST`.
///
/// Differs from `eval_quote` in that unquoted expressions get
/// evaluated AND substituted; differs from `expand_template` (the
/// macro-expansion-time walker in `macros.rs`) in that the unquote
/// substitution comes from runtime values, not macro-bound AST args.
///
/// Supported value-to-AST conversions at unquote sites:
/// - `:i64` / `:f64` / `:bool` / `:String` / `:wat::core::keyword` →
///   matching literal node
/// - `:wat::WatAST` → the inner form directly (already an AST)
///
/// Other Value shapes (Struct, Enum, Vec, HashMap, HolonAST) error
/// at the unquote site — those don't have a single canonical AST
/// representation and the caller should pass a wat::WatAST shape
/// (typically constructed via a nested quasiquote or `forms`).
///
/// Nested quasiquote tracks depth like `walk_template` does: a
/// `(:wat::core::quasiquote X)` inside the body bumps depth + 1
/// and preserves the wrapper; `(:wat::core::unquote X)` fires only
/// at depth 1.
///
/// Arc 255 Stone 1a-gamma-i — the `role = eval` pointer for `:wat::core::quasiquote`.
/// Annotated IN PLACE (signature already fits the canonical `NativeHandler` shape) — see
/// `intrinsic/special/quasiquote.rs` for the doc-only struct and the `role = check` pointer.
#[wat_special_form_impl(":wat::core::quasiquote", role = eval)]
fn eval_quasiquote(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::quasiquote";
    if args.len() != 1 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let walked = walk_quasiquote(&args[0], env, sym, 1)?;
    Ok(Value::wat__WatAST(Arc::new(walked)))
}

/// Recursive walker for runtime quasiquote — inverse of macros.rs's
/// expansion-time walker, but evaluating unquotes against the
/// runtime environment instead of substituting macro bindings.
///
/// rune:solvere(load-bearing-coupling) — qq depth-walk is mirrored in 3 sites
/// (walk_template / validate_quasiquote_template / walk_quasiquote); the depth
/// rule (nested +1, fire-at-depth-1, peel-deeper) is one contract that must
/// change in all three in sync; a unifying visitor would obscure three readable
/// single-purpose walkers.
fn walk_quasiquote(
    form: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    depth: u32,
) -> Result<WatAST, EvalBreak> {
    match form {
        WatAST::List(items, span) => {
            // Nested quasiquote — bump depth, preserve wrapper.
            if let Some(arg) = match_qq_head(items, ":wat::core::quasiquote") {
                let inner = walk_quasiquote(arg, env, sym, depth + 1)?;
                return Ok(WatAST::List(
                    vec![
                        WatAST::Keyword(":wat::core::quasiquote".into(), span.clone()),
                        inner,
                    ],
                    span.clone(),
                ));
            }
            // Unquote — fires at depth 1; preserves+peels deeper.
            if let Some(arg) = match_qq_head(items, ":wat::core::unquote") {
                if depth == 1 {
                    let v = eval_inner(arg, env, sym)?.value_owned();
                    return value_to_watast(":wat::core::unquote", v, span.clone());
                }
                let inner = walk_quasiquote(arg, env, sym, depth - 1)?;
                return Ok(WatAST::List(
                    vec![
                        WatAST::Keyword(":wat::core::unquote".into(), span.clone()),
                        inner,
                    ],
                    span.clone(),
                ));
            }
            // Plain list — walk children with splice support.
            // Arc 249 Stone 249.3a — `~@`-splice at eval-time:
            // at depth 1, a child `(:wat::core::unquote-splicing E)` evaluates E
            // and flattens its elements into the parent's child-vector (1-to-N),
            // mirroring expand-time `splice_argument` semantics (expand.rs:1097).
            let mut walked: Vec<WatAST> = Vec::with_capacity(items.len());
            for child in items.iter() {
                // Detect `(:wat::core::unquote-splicing E)` at depth 1.
                if depth == 1 {
                    if let Some(splice_expr) =
                        match_qq_head_named(child, ":wat::core::unquote-splicing")
                    {
                        let v = eval_inner(splice_expr, env, sym)?.value_owned();
                        match v {
                            // Vec: convert each element to WatAST and splice all.
                            // Mirrors splice_argument's computed-Vec case (expand.rs:1152).
                            Value::Vec(elems) => {
                                for elem in elems.iter() {
                                    walked.push(value_to_watast(
                                        ":wat::core::unquote-splicing",
                                        elem.clone(),
                                        span.clone(),
                                    )?);
                                }
                            }
                            // WatAST List: splice the inner list's children.
                            // Threading case: `~@step` where step is a list-form value.
                            Value::wat__WatAST(ref ast) => {
                                if let WatAST::List(ref children, _) = **ast {
                                    walked.extend(children.iter().cloned());
                                } else {
                                    return Err(RuntimeError::new(
                                        span.clone(),
                                        RuntimeErrorKind::TypeMismatch {
                                            op: ",@".into(),
                                            expected: "sequence (Vec value or list form)",
                                            got: Box::new(ValueSnapshot::of(&v)),
                                        },
                                    )
                                    .into());
                                }
                            }
                            // Any other shape: honest refusal (not a sequence).
                            other => {
                                return Err(RuntimeError::new(
                                    span.clone(),
                                    RuntimeErrorKind::TypeMismatch {
                                        op: ":wat::core::unquote-splicing".into(),
                                        expected: "sequence (Vec value or list form)",
                                        got: Box::new(ValueSnapshot::of(&other)),
                                    },
                                )
                                .into());
                            }
                        }
                        continue;
                    }
                }
                // Below depth 1 or not a splice form: walk normally.
                walked.push(walk_quasiquote(child, env, sym, depth)?);
            }
            Ok(WatAST::List(walked, span.clone()))
        }
        // Arc 212: bracketed `[a b c]` Vector form (let-binding vectors,
        // fn-signature parameter vectors, template-position vector
        // literals). Walks children identically to Lists but preserves
        // the Vector wrapper — without this, an unquote inside any
        // bracketed shape stays literal and the child sees
        // `:wat::core::unquote` as an unknown function.
        //
        // Splice stone: Vector children now get the same `~@`-splice support
        // as List children (Arc 249 Stone 249.3a extended to Vector context).
        // At depth 1, a child `(:wat::core::unquote-splicing E)` evaluates E
        // and flattens its elements into the Vector — enabling program-body
        // quasiquotes to splice env-bound list form-values element-wise into
        // fn-argspec vectors (e.g. `[~@params]` where `params` is a
        // `Value::Vec` of `Value::wat__WatAST` form-values).
        WatAST::Vector(items, span) => {
            let mut walked: Vec<WatAST> = Vec::with_capacity(items.len());
            for child in items.iter() {
                // Detect `(:wat::core::unquote-splicing E)` at depth 1.
                if depth == 1 {
                    if let Some(splice_expr) =
                        match_qq_head_named(child, ":wat::core::unquote-splicing")
                    {
                        let v = eval_inner(splice_expr, env, sym)?.value_owned();
                        match v {
                            // Vec: convert each element to WatAST and splice all.
                            // Mirrors the List arm's Vec case and splice_argument's computed-Vec case.
                            Value::Vec(elems) => {
                                for elem in elems.iter() {
                                    walked.push(value_to_watast(
                                        ":wat::core::unquote-splicing",
                                        elem.clone(),
                                        span.clone(),
                                    )?);
                                }
                            }
                            // WatAST List: splice the inner list's children.
                            Value::wat__WatAST(ref ast) => {
                                if let WatAST::List(ref children, _) = **ast {
                                    walked.extend(children.iter().cloned());
                                } else {
                                    return Err(RuntimeError::new(
                                        span.clone(),
                                        RuntimeErrorKind::TypeMismatch {
                                            op: ",@".into(),
                                            expected: "sequence (Vec value or list form)",
                                            got: Box::new(ValueSnapshot::of(&v)),
                                        },
                                    )
                                    .into());
                                }
                            }
                            // Any other shape: honest refusal.
                            other => {
                                return Err(RuntimeError::new(
                                    span.clone(),
                                    RuntimeErrorKind::TypeMismatch {
                                        op: ":wat::core::unquote-splicing".into(),
                                        expected: "sequence (Vec value or list form)",
                                        got: Box::new(ValueSnapshot::of(&other)),
                                    },
                                )
                                .into());
                            }
                        }
                        continue;
                    }
                }
                // Below depth 1 or not a splice form: walk normally.
                walked.push(walk_quasiquote(child, env, sym, depth)?);
            }
            Ok(WatAST::Vector(walked, span.clone()))
        }
        // Arc 257 slice 1 — Map/Set literals: walk all k/v and elements
        // at the same depth (may contain unquote forms in quasiquote templates).
        WatAST::Map(pairs, span) => {
            let mut walked_pairs: Vec<(WatAST, WatAST)> = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                let wk = walk_quasiquote(k, env, sym, depth)?;
                let wv = walk_quasiquote(v, env, sym, depth)?;
                walked_pairs.push((wk, wv));
            }
            Ok(WatAST::Map(walked_pairs, span.clone()))
        }
        WatAST::Set(items, span) => {
            let mut walked: Vec<WatAST> = Vec::with_capacity(items.len());
            for child in items {
                walked.push(walk_quasiquote(child, env, sym, depth)?);
            }
            Ok(WatAST::Set(walked, span.clone()))
        }
        // Leaves (IntLit, FloatLit, BoolLit, StringLit, NilLit,
        // Keyword, Symbol) are preserved verbatim — no unquotes inside.
        other => Ok(other.clone()),
    }
}

/// Pattern-match `(:wat::core::quasiquote X)` or
/// `(:wat::core::unquote X)` — return Some(X) when items has exactly
/// 2 entries and items[0] is the expected keyword.
fn match_qq_head<'a>(items: &'a [WatAST], head: &str) -> Option<&'a WatAST> {
    if items.len() != 2 {
        return None;
    }
    if let WatAST::Keyword(k, _) = &items[0] {
        if k == head {
            return Some(&items[1]);
        }
    }
    None
}

/// Pattern-match a `WatAST` node as `(<head> X)` — return Some(&X) when the
/// node is a List with exactly 2 entries and the first is the expected keyword.
/// Used by the splice-aware child walk in `walk_quasiquote`.
fn match_qq_head_named<'a>(node: &'a WatAST, head: &str) -> Option<&'a WatAST> {
    if let WatAST::List(items, _) = node {
        match_qq_head(items, head)
    } else {
        None
    }
}

/// Convert a runtime Value to a literal WatAST node — used by
/// `walk_quasiquote` at unquote sites. Inverse of the eval-eval
/// path: this is "value back to source" for the supported leaf
/// shapes.
pub fn value_to_watast(op: &str, v: Value, span: Span) -> Result<WatAST, EvalBreak> {
    match v {
        Value::i64(n) => Ok(WatAST::IntLit(n, span)),
        Value::f64(x) => Ok(WatAST::FloatLit(x, span)),
        Value::bool(b) => Ok(WatAST::BoolLit(b, span)),
        Value::String(s) => Ok(WatAST::StringLit((*s).clone(), span)),
        // Arc 244 — Value::Unit (nil) → NilLit; closes the quasiquote ~nil gap (AUDIT §3 site 9).
        Value::Unit => Ok(WatAST::NilLit(span)),
        Value::wat__core__keyword(k) => Ok(WatAST::Keyword((*k).clone(), span)),
        Value::wat__WatAST(a) => Ok((*a).clone()),
        Value::holon__HolonAST(h) => Ok(holon_to_watast(&h)),
        other => Err(RuntimeError::new(
            span,
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "primitive (i64/f64/bool/String/keyword/nil) or :wat::WatAST",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

// Arc 109 Stone — the reflect home — `eval_struct_to_form` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `type_expr_to_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `binder_head_nodes` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `function_to_signature_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `function_to_define_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `type_scheme_to_signature_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `primitive_to_define_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `macrodef_to_signature_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `macrodef_to_define_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `typedef_to_signature_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `typedef_to_define_ast` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `name_from_keyword_or_fn` moved to `src/reflect/render.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `Binding` moved to `src/reflect/lookup.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `lookup_form` moved to `src/reflect/lookup.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_lookup_define` moved to `src/reflect/lookup.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_signature_of_defn` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_signature_of_fn` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_return_type_of` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_body_of` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// `(:wat::runtime::metadata-of <name :keyword>) -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])])`
///
/// Stone 241.7. Returns the binding's metadata-map as Option:
/// - Some(baseline-map) when `name` is a registered Rust intrinsic (arc 255.1b-iii): the
///   auto-derived `:name`/`:kind`/`:defined-in`/`:layer`/`:arity`/`:purity`/`:determinism`/
///   `:totality`/`:expand-time`/`:doc`/`:added`/`:ret`/`:category` fields
/// - Some(doc-map) when `name` is a wat `defn`/`def` whose `{...}` metadata map carries any
///   doc-axis key (arc 255 Stone "metadata-of answers in one shape"): `:purity`/
///   `:determinism`/`:totality`/`:expand-time`/`:category`/`:defined-in` come back as the SAME
///   `Value::Enum` shape the intrinsic branch above produces (both read through the one
///   decoder, `wat_doc::from_metadata`), plus `:doc`/`:added`/`:ret` as `Value::String` — never
///   the raw, un-decoded `Value::wat__WatAST` this branch used to hand back for these keys
/// - Some({:k1 v1 ...}) when metadata was attached at def time but carries NO doc-axis key
///   (e.g. `{:restricted-to […]}`, a capability restriction unrelated to the doc contract):
///   read and stored exactly as authored, raw and un-decoded, wrapped as `Value::wat__WatAST`
/// - None when binding exists but no metadata
/// - None when binding doesn't exist
///
/// Accepts any binding name (def + defn alike). The argument is read as a
/// binding-name keyword: if the WatAST arg is a Keyword literal, its string
/// is used directly (without evaluating through runtime_def_values — which
/// would resolve a `def :my::x 42` to `42`, losing the name). If the arg
/// evaluates to a named fn value, `name_from_keyword_or_fn` recovers the
/// name from the fn (supporting `(metadata-of my-fn-var)` call style).
///
/// ★ Doc correction (arc 255 Stone P6-c-W4): the prior header claimed a uniform
/// `(:Option :- [(HashMap :- [Keyword HolonAST])])` — false on two counts, checked against the
/// body below. First, the map's VALUES are never `HolonAST`: the intrinsic-baseline branch
/// inserts plain scalar/enum `Value`s (`Value::String`, `Value::i64`, `Value::Enum` for
/// `:kind`/`:defined-in`/`:layer`/`:purity`/`:determinism`/`:totality`/`:expand-time`/
/// `:category` — see the `put` calls below, whose own comment already said "PLAIN wat Values
/// (no HolonAST wrapping)"), while the user-metadata branch (`sym.binding_metadata`) wraps a
/// CAPABILITY-only map's values as `Value::wat__WatAST` — WatAST, not HolonAST (arc
/// 201/251/294.f retired the HolonAST carrier on this whole reflection surface — the same
/// finding W3 made for `lookup-define`/`body-of`). A user-metadata map that instead carries a
/// doc-axis key takes the SAME `Value::Enum`/`Value::String` path the intrinsic branch does
/// (Stone "metadata-of answers in one shape" — both call `wat_doc::from_metadata`, neither
/// decodes the AST itself). Second, the branches remain heterogeneous with EACH OTHER when a
/// capability-only map is in play (plain values vs. WatAST-wrapped), so no single element type
/// describes all three cases; `:wat::core::Value` (this surface's existing convention for "any
/// wat value", e.g. `edn::get-field`'s `@ret`) is what actually fits.
/// `metadata-of` carries NO checker `TypeScheme` (absent from `register_builtins` — see the
/// `FROZEN_CHECKER_DEBT_LEDGER` entry below), so this claim was never verified by anything; it
/// is corrected here, not newly enforced.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @arg     name_ast :wat::core::keyword the binding name (or intrinsic FQDN) whose metadata is read (a literal keyword; a named fn value also resolves via its stored name)
/// @ret     (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])]) the metadata map, or `:None` when the binding is unregistered or carries no metadata
/// @example (:wat::core::match (:wat::runtime::metadata-of :wat::runtime::lookup-define) ((:wat::core::Some _) true) (:wat::core::None false)) #=> true
/// @example (:wat::core::match (:wat::runtime::metadata-of :probe::totally-unknown-xyz) ((:wat::core::Some _) true) (:wat::core::None false)) #=> false
/// @see     :wat::runtime::lookup-define
/// @see     :wat::runtime::field-names-of
#[wat_intrinsic(":wat::runtime::metadata-of")]
#[allow(clippy::mutable_key_type)]
fn eval_metadata_of(
    name_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::metadata-of";
    // Extract the binding name. Prefer the keyword string directly from
    // the WatAST (avoids runtime_def_values resolution that would lose the
    // name for non-fn defs). Fall back to eval + name_from_keyword_or_fn
    // for the fn-value case (e.g. a fn passed via a symbol binding).
    let name: String = match name_ast {
        WatAST::Keyword(k, _) => k.clone(),
        _ => {
            let v = eval_inner(name_ast, env, sym)?.value_owned();
            match crate::reflect::render::name_from_keyword_or_fn(&v) {
                Some(n) => n,
                None => {
                    return Err(RuntimeError::new(
                        name_ast.span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: ":wat::core::keyword or named function",
                            got: Box::new(ValueSnapshot::of(&v)),
                        },
                    )
                    .into());
                }
            }
        }
    };
    // Arc 255.1b-iii — the intrinsic branch. If `name` is a registered Rust
    // intrinsic, answer `metadata-of` with the SAME shape as the user path:
    // `Some((HashMap :- [keyword Value]))`, carrying the auto-derived baseline.
    // Arc 255.1b-iv-c: all values are PLAIN wat Values (not holon-AST-wrapped);
    // the three closed-domain fields use Value::Enum (Kind/DefinedIn/Layer).
    // Seamless reflection parity — a `:wat::core::Bytes::to-hex` reflects like
    // a user `defn`. ZERO eval behavior change: the handler dispatch route is
    // untouched; this only READS the baseline the registry already carries.
    if let Some(entry) = crate::intrinsic::registry().lookup_entry(&name) {
        // 13 `put`s below (`:name`/`:kind`/`:defined-in`/`:layer`/`:arity`/`:purity`/
        // `:determinism`/`:totality`/`:expand-time`/`:doc`/`:added`/`:ret`/`:category`) —
        // bumped from a stale `8` (already undercounting pre-`:totality`/`:expand-time`)
        // while touching this block for the "metadata-of answers in one shape" stone.
        let mut map: std::collections::HashMap<Value, Value> =
            std::collections::HashMap::with_capacity(13);
        // iv-c: put inserts PLAIN values (no HolonAST wrapping).
        let mut put = |key: &str, val: Value| {
            map.insert(Value::wat__core__keyword(Arc::new(key.to_string())), val);
        };
        // :name — the FQDN as a plain keyword value.
        put(
            ":name",
            Value::wat__core__keyword(Arc::new(entry.name.to_string())),
        );
        // :kind / :defined-in / :layer — closed-domain Value::Enum (iv-c §5).
        put(
            ":kind",
            crate::intrinsic::ToEnumValue::to_enum_value(&entry.kind),
        );
        put(
            ":defined-in",
            crate::intrinsic::ToEnumValue::to_enum_value(&crate::intrinsic::DefinedIn::Rust),
        );
        put(
            ":layer",
            crate::intrinsic::ToEnumValue::to_enum_value(&crate::intrinsic::Layer::Substrate),
        );
        // :arity — Exact(N) → N as i64; Variadic → -1 (sentinel for "variadic").
        let arity_val = match entry.arity {
            crate::intrinsic::Arity::Exact(n) => Value::i64(n as i64),
            crate::intrinsic::Arity::Variadic => Value::i64(-1),
        };
        put(":arity", arity_val);
        // :purity / :determinism — declared enum values from the doc, not derived bools.
        let purity_val = crate::intrinsic::ToEnumValue::to_enum_value(&entry.purity);
        let determinism_val = crate::intrinsic::ToEnumValue::to_enum_value(&entry.determinism);
        put(":purity", purity_val);
        put(":determinism", determinism_val);
        // :totality / :expand-time — Stone "metadata-of answers in one shape": these two axes
        // have lived on `IntrinsicEntry` since the T3/expand-T3 stones but were never `put` here,
        // so an intrinsic's `metadata-of` answered `:purity`/`:determinism`/`:category` and
        // silently OMITTED `:totality`/`:expand-time` — the same "answers in two shapes" defect
        // this stone closes, one level up: absent here, present (raw AST) on the wat branch.
        // Wired in now so both axes are comparable across BOTH branches, per the acceptance bar
        // ("the other axes too ... converging one key only MOVES the defect").
        let totality_val = crate::intrinsic::ToEnumValue::to_enum_value(&entry.totality);
        let expand_time_val = crate::intrinsic::ToEnumValue::to_enum_value(&entry.expand_time);
        put(":totality", totality_val);
        put(":expand-time", expand_time_val);
        // :doc — the GFM prose body from the structured doc contract (iv-b1).
        // :added — the @added version string.
        // :ret — the @ret description.
        // (Vector-valued keys :args/:examples/:see are CARRIED on the entry
        //  but rendered by the iv-b2 verifier seam, not here — scope cut.)
        put(":doc", Value::String(Arc::new(entry.prose.to_string())));
        put(":added", Value::String(Arc::new(entry.added.to_string())));
        put(":ret", Value::String(Arc::new(entry.ret.to_string())));
        // :category — closed-domain Value::Enum (iv-c / arc 255.1b-iv-c Part C).
        let category_val = crate::intrinsic::ToEnumValue::to_enum_value(&entry.category);
        put(":category", category_val);
        return Ok(Value::Option(Arc::new(Some(Value::wat__std__HashMap(
            Arc::new(map),
        )))));
    }
    match sym.binding_metadata.get(&name) {
        // Arc 255 Stone "metadata-of answers in one shape" — a metadata map carrying any
        // doc-axis key is a doc DECLARATION (same predicate as the registration gate,
        // `meta_has_doc_axis_key`, so the two cannot disagree on what counts as one). It is
        // run through the ONE decoder, `wat_doc::from_metadata` — already called at
        // registration to VALIDATE the same map; here its result is finally READ, not
        // discarded. Emission below reuses the registry branch's own `to_enum_value` /
        // `Value::String` shapes key for key, so the two branches cannot drift apart by
        // inspection: a `:purity` (etc.) from either branch is the same `Value::Enum` over
        // the same `wat_doc` type, not a raw `Value::wat__WatAST` the registry branch never
        // produces for these keys.
        Some(meta) if !meta.is_empty() && meta_has_doc_axis_key(meta) => {
            let map_ast = WatAST::Map(
                meta.iter()
                    .map(|(k, v)| (WatAST::Keyword(k.clone(), v.span().clone()), v.clone()))
                    .collect(),
                name_ast.span().clone(),
            );
            let doc = match wat_doc::from_metadata(&map_ast) {
                Ok(d) => d,
                // Unreachable in practice — registration (`register_stdlib_defines`) already
                // ran this SAME map through this SAME decoder and would have refused to
                // register a def whose doc contract doesn't hold. Handled, not unwrapped,
                // because a defensive `?` here costs nothing and an `unwrap` would turn a
                // decoder disagreement into a panic instead of a diagnosable error.
                Err(e) => {
                    return Err(RuntimeError::new(
                        name_ast.span().clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: name.clone(),
                            reason: format!(
                                "metadata-map doc contract violation (wat_doc::from_metadata): {e:?}"
                            ),
                        },
                    )
                    .into());
                }
            };
            let mut map: std::collections::HashMap<Value, Value> =
                std::collections::HashMap::with_capacity(9);
            let mut put = |key: &str, val: Value| {
                map.insert(Value::wat__core__keyword(Arc::new(key.to_string())), val);
            };
            // :purity / :determinism / :totality / :expand-time / :category — the SAME
            // `ToEnumValue::to_enum_value` calls the registry branch makes, fed from
            // `DocComment`'s typed fields instead of `IntrinsicEntry`'s. Same `Value::Enum`
            // over the same `wat_doc` enum type either way — the fix the NOTE asked for.
            put(":purity", crate::intrinsic::ToEnumValue::to_enum_value(&doc.purity));
            put(
                ":determinism",
                crate::intrinsic::ToEnumValue::to_enum_value(&doc.determinism),
            );
            put(":totality", crate::intrinsic::ToEnumValue::to_enum_value(&doc.totality));
            put(
                ":expand-time",
                crate::intrinsic::ToEnumValue::to_enum_value(&doc.expand_time),
            );
            put(":category", crate::intrinsic::ToEnumValue::to_enum_value(&doc.category));
            // :defined-in — a fact at THIS site: this branch is reached only from
            // `sym.binding_metadata`, which only a wat `defn`/`def` populates (STOP-4). Not a
            // default beside a derived field — the registry branch above is the ONLY other
            // way into this function, and it is reached only by a `#[wat_intrinsic]` entry.
            put(
                ":defined-in",
                crate::intrinsic::ToEnumValue::to_enum_value(&crate::intrinsic::DefinedIn::Wat),
            );
            // :doc / :added / :ret — same `Value::String` shape as the registry branch;
            // `:args`/`:examples`/`:see`/`:yields`/`:deprecated` are deliberately NOT emitted,
            // matching the registry branch's own scope cut (its comment: "CARRIED on the
            // entry but rendered by the iv-b2 verifier seam, not here").
            put(":doc", Value::String(Arc::new(doc.prose.clone())));
            put(":added", Value::String(Arc::new(doc.added.clone())));
            put(":ret", Value::String(Arc::new(doc.ret.clone())));
            Ok(Value::Option(Arc::new(Some(Value::wat__std__HashMap(
                Arc::new(map),
            )))))
        }
        // No doc-axis key (e.g. `{:restricted-to […]}`, 4 live in the corpus) — STOP-2: keeps
        // today's behaviour EXACTLY, raw and un-decoded. Same predicate as the branch above,
        // so a capability-only map can never accidentally take the doc path.
        Some(meta) if !meta.is_empty() => {
            let mut map: std::collections::HashMap<Value, Value> =
                std::collections::HashMap::with_capacity(meta.len());
            for (k, v) in meta {
                map.insert(
                    Value::wat__core__keyword(Arc::new(k.clone())),
                    Value::wat__WatAST(Arc::new(v.clone())),
                );
            }
            Ok(Value::Option(Arc::new(Some(Value::wat__std__HashMap(
                Arc::new(map),
            )))))
        }
        _ => Ok(Value::Option(Arc::new(None))),
    }
}

// ─── Arc 143 slice 3 — HolonAST manipulation primitives ─────────────────────

/// Destructure a `HolonAST` into its `Bundle` children. Returns a
/// `RuntimeError` if the AST is not a `Bundle` variant.
pub(crate) fn require_bundle<'a>(
    op: &'static str,
    holon: &'a HolonAST,
    arg_span: &Span,
) -> Result<&'a Vec<HolonAST>, EvalBreak> {
    match holon {
        HolonAST::Bundle(children) => Ok(children),
        _ => Err(RuntimeError::new(
            arg_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "Bundle (signature head HolonAST)",
                got: Box::new(ValueSnapshot::unavailable("non-Bundle HolonAST variant")),
            },
        )
        .into()),
    }
}

// Arc 109 Stone — the reflect home — `require_ast_children` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_rename_callable_name` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_extract_arg_names` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_extract_arg_types` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_field_names_of` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_field_types_of` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `resolve_type_keyword_arg` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `resolve_aggregate_def_for_reflection` moved to `src/reflect/verbs.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.


// Arc 109 Stone — the reflect home — `eval_form_matches` moved to `src/reflect/match.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `walk_match_clause` moved to `src/reflect/match.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_forms` moved to `src/reflect/match.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_macroexpand_1` moved to `src/reflect/expand.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the reflect home — `eval_macroexpand` moved to `src/reflect/expand.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// `(:wat::core::first xs)` / `second` / `third` — positional
/// accessor polymorphic over `Vec<T>` and tuples (user direction
/// 2026-04-19: "both are index-accessed data structs").
///
/// **Polymorphic return shape (arc 047):**
/// - On `Tuple`: returns the element at `index`, cloned, as `T`.
///   Tuples are fixed-arity and type-known; out-of-range is a
///   type error caught at compile time.
/// - On `Vec`: returns bare `T` — `items[index]` if in-range, else
///   raises (arc-278 flip: like `nth`; was `(Option :- [T])` before
///   that flip). Empty/short Vec is a runtime fact, so the raise
///   surfaces it honestly.
///
/// `third` covers 3-tuples + Vecs-of-length-≥-3 (when in-range);
/// higher indices go through `:wat::core::get`.
fn eval_positional_accessor(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &'static str,
    index: usize,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: op.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // Classify via the registry — the only Value→container map for sequence ops.
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&v) {
        Some(container) if container.indexable() => {
            // Dispatch: each container provides its element at `index` or raises out-of-range.
            // Tuple is the heterogeneous special case; the others are homogeneous.
            match container {
                StreamContainer::Tuple => {
                    let Value::Tuple(items) = v else {
                        unreachable!("of_value⇒Tuple")
                    };
                    items.get(index).cloned().ok_or_else(|| {
                        EvalBreak::from(RuntimeError::new(
                            args[0].span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: op.into(),
                                reason: format!(
                                    "tuple has {} element(s); no element at index {}",
                                    items.len(),
                                    index
                                ),
                            },
                        ))
                    })
                }
                // Arc-278 flip: bare T, raise on out-of-range (like nth; was Option).
                StreamContainer::Vector => {
                    let Value::Vec(items) = v else {
                        unreachable!("of_value⇒Vector")
                    };
                    items.get(index).cloned().ok_or_else(|| {
                        EvalBreak::from(RuntimeError::new(
                            args[0].span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: op.into(),
                                reason: format!(
                                    "{op}: sequence has {} element(s); no element at index {index}",
                                    items.len()
                                ),
                            },
                        ))
                    })
                }
                // Arc 220 Stone 220.4 — List: O(N) nth via iterator.
                // Arc-278 flip: bare T, raise on out-of-range (like nth; was Option).
                StreamContainer::List => {
                    let Value::wat__core__List(items) = v else {
                        unreachable!("of_value⇒List")
                    };
                    items.iter().nth(index).cloned().ok_or_else(|| {
                        EvalBreak::from(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                            head: op.into(),
                            reason: format!("{op}: sequence has fewer than {} element(s); no element at index {index}", index + 1)
                        }))
                    })
                }
                // Arc-278-0b — PersistentVector: O(log n) index access via rpds VectorSync.
                // Arc-278 flip: bare T, raise on out-of-range (like nth; was Option).
                StreamContainer::PersistentVector => {
                    let Value::wat__core__PersistentVector(pv) = v else {
                        unreachable!("of_value⇒PersistentVector")
                    };
                    pv.get(index).cloned().ok_or_else(|| {
                        EvalBreak::from(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                            head: op.into(),
                            reason: format!("{op}: sequence has fewer than {} element(s); no element at index {index}", index + 1)
                        }))
                    })
                }
                // Arc 249 Stone 249.3a-ii — WatAstList: sequence of child forms.
                // of_value guarantees this is a WatAST::List; positional access returns bare WatAST.
                // Arc-278 flip: bare T, raise on out-of-range (was Option).
                StreamContainer::WatAstList => {
                    let Value::wat__WatAST(ast) = v else {
                        unreachable!("of_value⇒WatAstList")
                    };
                    match &*ast {
                        WatAST::List(children, _) => children.get(index)
                            .map(|c| Value::wat__WatAST(Arc::new(c.clone())))
                            .ok_or_else(|| {
                                EvalBreak::from(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                                    head: op.into(),
                                    reason: format!("{op}: WatAST List has {} child(ren); no child at index {index}", children.len())
                                }))
                            }),
                        // Unreachable: of_value only returns WatAstList for List forms.
                        _ => unreachable!("StreamContainer::of_value guarantees WatAST::List for WatAstList"),
                    }
                }
                // Stone 118.B4-iii — THE WALL: `indexable()` is FALSE for Stream now, so this
                // arm is dead — no `container` value can reach it as `Stream`. Named, not `_`,
                // so a future capability change that reopens Stream here is a compile error, not
                // a silent revival. Built one stone ago (B4-0); retired on purpose, per the wall.
                StreamContainer::Stream => unreachable!(
                    "indexable() gate excludes Stream (Stone 118.B4-iii — THE WALL: use :wat::stream::next)"
                ),
                // indexable() gate excludes HashSet — named arm, genuinely dead, compiler-forced:
                StreamContainer::HashSet => unreachable!("indexable() gate excludes HashSet"),
            }
        }
        // ∅ N/A: HashSet is unordered — no canonical "first". Stone 118.B4-iii — THE WALL:
        // Stream lands here too now (indexable()==false) — a lazy seq has no first/second/third;
        // advance it with `:wat::stream::next`, whose `(NextOutcome :- [T]) = Item(value, rest) |
        // Exhausted` is the only door a Stream yields through.
        Some(StreamContainer::Stream) => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "Tuple, (Vector :- [T]), (List :- [T]), (PersistentVector :- [T]), or WatAST — a lazy (Stream :- [T]) has no first/second/third; advance it with :wat::stream::next ((NextOutcome :- [T]) = Item(value, rest) | Exhausted)",
                got: Box::new(ValueSnapshot::of(&v)),
            },
        )
        .into()),
        Some(_) => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "tuple, Vec, List, or PersistentVector",
                got: Box::new(ValueSnapshot::of(&v)),
            },
        )
        .into()),
        // None: not a sequence container. Preserve the specific MalformedForm for non-List WatAST
        // (a non-List form has no positional children — distinct from a type mismatch).
        None => match v {
            Value::wat__WatAST(_) => Err(EvalBreak::from(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: op.into(),
                    reason: format!(
                        "{op}: WatAST is not a List form; cannot take positional child"
                    ),
                },
            ))),
            other => Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "tuple, Vec, List, or PersistentVector",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into()),
        },
    }
}

// Arc 255 Stone P6-c-W6 — `:wat::core::nth` moved verbatim into a `#[wat_intrinsic]`
// handler (`src/intrinsic/collection.rs`) with its real (2) arity declared; the
// pre-match registry check intercepts the name before reaching the giant match.

// Arc 255 Stone C — `eval_f64_reduce` (the shared implementation for the OLD
// single-`Vector`-arg `:wat::core::f64::max-of` / `min-of`) is DELETED: its
// only two callers were those retired `runtime.rs` dispatch arms. The
// surviving `:wat::f64::max-of` / `min-of` are variadic and reduce via
// `f64_variadic_reduce` (`src/intrinsic/f64.rs`) instead — a genuinely
// different calling convention, not a renaming of this fn (see that module's
// header). `require_vec` (used here) has other live callers and is untouched.

// Arc 109 Stone — the last two map items — `eval_some_ctor` moved to
// `src/option/mod.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the last two map items — `eval_ok_ctor` moved to
// `src/result/mod.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the last two map items — `eval_err_ctor` moved to
// `src/result/mod.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the last two map items — `eval_try` moved to
// `src/result/mod.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the last two map items — `eval_option_try` moved to
// `src/option/mod.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the last two map items — `eval_option_expect` moved to
// `src/option/mod.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the last two map items — `eval_result_expect` moved to
// `src/result/mod.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the last two map items — `extract_panics` moved to
// `src/assertion.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged;
// shared by `eval_option_expect`/`eval_result_expect`, so it lives with the
// `AssertionPayload` type it destructures, not with either mover alone.

// Arc 109 Stone — the last two map items — `expect_panic` moved to
// `src/assertion.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged;
// shared by `eval_option_expect`/`eval_result_expect`, so it lives with the
// `AssertionPayload` type it builds, not with either mover alone.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_kernel_raise` moved to
// `src/kernel/abort.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_kernel_here` moved to
// `src/kernel/source.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_struct_new` moved to `src/record/construct.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_variant` moved to `src/record/construct.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_retag_op` moved to
// `src/kernel/serve.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_struct_field` moved to `src/record/access.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// `(:wat::core::match <scrutinee> <arm>...)` — pattern-match over
/// enum values. MVP-scoped to `(:Option :- [T])` (the only built-in enum);
/// user-declared enums graduate in a later slice.
///
/// Each arm is `(pattern body)`. Pattern forms:
/// - `:None` — matches `Value::Option(None)`, no binding.
/// - `(Some binder)` — matches `Value::Option(Some(v))`, binds `binder`
///   to `v` in the body's scope. Exactly one binder; further pattern
///   nesting is a future slice.
/// - bare identifier — wildcard that binds the scrutinee as that name.
/// - `_` — wildcard, no binding.
///
/// Arms are tried in order; the first match fires. If no arm matches
/// the scrutinee, returns `PatternMatchFailed`. (Exhaustiveness is
/// enforced statically by the type checker; this runtime error fires
/// only when the type check hasn't run.)
/// `(:wat::core::match scrutinee arm1 arm2 ...)` — typed
/// pattern match per the 2026-04-20 INSCRIPTION. Every arm body must
/// produce `:T`; mismatches are reported per-arm. The annotation is
/// check-time only at runtime (validated for shape, ignored for
/// dispatch).
///
/// Arity: at least 4 args (scrutinee, `->`, `:T`, one arm). The old
/// no-annotation form — `(match scrutinee arm1 ...)` — is refused
/// with a migration-hint MalformedForm. Hard break, no deprecation.
#[wat_special_form_impl(":wat::core::match", role = eval)]
fn eval_match(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Arc 258.5 — the `-> :T` ascription is retired (the result type is
    // inferred from the arm bodies). A stray `->` in ascription position
    // is the old form; refuse it with a migration hint.
    if args.len() >= 2 && matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::match".into(),
            reason: "`:wat::core::match` no longer takes `-> :T`; the result type is inferred by unifying the arm bodies (like `if`). Write (:wat::core::match scrut (pat body) ...)".into()
        }).into());
    }
    if args.len() < 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::match".into(),
            reason: format!(
                "expected (:wat::core::match scrut arm1 arm2 ...) — at least a scrutinee and one arm; got {}",
                args.len()
            )
        }).into());
    }
    let scrutinee = eval_inner(&args[0], env, sym)?.value_owned();
    for arm in &args[1..] {
        let arm_items = match arm {
            WatAST::List(items, _) => items,
            other => {
                return Err(RuntimeError::new(
                    other.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "each arm must be a list `(pattern body)`, got {}",
                            other.variant_name()
                        ),
                    },
                )
                .into());
            }
        };
        if arm_items.len() != 2 {
            return Err(RuntimeError::new(
                arm.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "each arm must have exactly (pattern body); got {} elements",
                        arm_items.len()
                    ),
                },
            )
            .into());
        }
        let pattern = &arm_items[0];
        let body = &arm_items[1];
        if let Some(arm_env) = try_match_pattern(pattern, &scrutinee, env, sym)? {
            return eval_inner(body, &arm_env, sym).map(|tv| tv.value_owned());
        }
    }
    Err(RuntimeError::new(
        args[0].span().clone(),
        RuntimeErrorKind::PatternMatchFailed {
            value_type: scrutinee.type_name(),
        },
    )
    .into())
}

/// Attempt to match `pattern` against `value`. Returns:
/// - `Ok(Some(env))` — pattern matches; `env` extends `outer` with any
///   pattern-introduced bindings.
/// - `Ok(None)` — pattern doesn't match this value; try the next arm.
/// - `Err(_)` — pattern is malformed.
///
/// Arc 055 — patterns are recursive over the algebra. List sub-patterns
/// dispatch on the value's shape (Option/Result/Enum/Tuple); literal
/// sub-patterns compare for equality; bare symbols bind, `_` discards.
/// Linear-shadowing semantics — a name bound twice in one pattern
/// keeps the second binding (later recursion overwrites earlier).
pub(crate) fn try_match_pattern(
    pattern: &WatAST,
    value: &Value,
    outer: &Environment,
    sym: &SymbolTable,
) -> Result<Option<Environment>, EvalBreak> {
    match pattern {
        // `:None` / `:wat::core::None` — matches Option(None) only.
        // Arc 109 slice 1h: FQDN form is canonical; bare form
        // continues to work at runtime (poisoned at type-check time).
        WatAST::Keyword(k, _) if k == ":None" || k == ":wat::core::None" => match value {
            Value::Option(opt) if opt.is_none() => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        // Arc 055 — literal sub-patterns compare by equality.
        WatAST::IntLit(n, _) => match value {
            Value::i64(v) if v == n => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        WatAST::FloatLit(f, _) => match value {
            Value::f64(v) if v == f => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        // Arc 300 stone B — rational literal sub-pattern; compares by
        // structural equality (both sides are already-reduced BigRationals).
        WatAST::RationalLit(r, _) => match value {
            Value::wat__core__Rational(v) if v.as_ref() == r => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        // Arc 300 stone C1 — bigint literal sub-pattern; compares by structural
        // equality (mirrors the Rational arm immediately above, one type over).
        WatAST::BigIntLit(n, _) => match value {
            Value::wat__core__BigInt(v) if v.as_ref() == n => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        // Arc 300 stone D — char literal sub-pattern; compares by equality
        // (mirrors the BigInt/Rational arms immediately above).
        WatAST::CharLit(c, _) => match value {
            Value::wat__core__Char(v) if v == c => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        WatAST::BoolLit(b, _) => match value {
            Value::bool(v) if v == b => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        WatAST::StringLit(s, _) => match value {
            Value::String(v) if v.as_str() == s => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        // Arc 048 — user-enum unit variant. Pattern `:enum::Variant`
        // matches `Value::Enum` whose `type_path::variant_name`
        // composes to the same path. The scrutinee's type is enforced
        // upstream by the checker; here we just compare paths.
        WatAST::Keyword(k, _) => match value {
            Value::Enum(ev) => {
                let composed = format!("{}::{}", ev.type_path, ev.variant_name);
                if composed == *k && ev.fields.is_empty() {
                    Ok(Some(outer.clone()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        },
        // `_` wildcard — matches any value, no binding.
        WatAST::Symbol(ident, _) if ident.as_str() == "_" => Ok(Some(outer.clone())),
        // Bare identifier — binds the scrutinee to that name.
        WatAST::Symbol(ident, _) => Ok(Some(
            outer
                .child()
                .bind_unknown_span(
                    crate::scope::env_key(ident),
                    TrackedValue::from(value.clone()),
                )
                .build(),
        )),
        // `(Some binder)` — matches Option(Some(v)), binds `binder` to v.
        WatAST::List(items, _) => {
            let head = items.first().ok_or_else(|| {
                RuntimeError::new(
                    pattern.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: "empty list pattern".into(),
                    },
                )
            })?;
            // STONE: the bare-symbol shorthand dies — only the FQDN keyword
            // form is recognized here now. The bare-Symbol "Some" alternative
            // is DELETED (arc 109 slice 1h's match-pattern half, never closed
            // until now): the checker refuses it at `pattern_coverage` /
            // `check_subpattern`, and a bare `((Some v) ...)` reaching this
            // runtime dispatch anyway (e.g. via `eval-ast!`, unchecked) now
            // falls through to the tuple-destructure arm below, same as any
            // other unrecognized list-pattern head — it no longer evaluates
            // the retired shorthand.
            let head_is_some = matches!(
                head,
                WatAST::Keyword(k, _) if k == ":wat::core::Some"
            );
            if head_is_some {
                if items.len() != 2 {
                    return Err(RuntimeError::new(
                        pattern.span().clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "(Some _) takes exactly one field, got {}",
                                items.len() - 1
                            ),
                        },
                    )
                    .into());
                }
                return match value {
                    Value::Option(opt) => match &**opt {
                        Some(inner) => try_match_pattern(&items[1], inner, outer, sym),
                        None => Ok(None),
                    },
                    _ => Ok(None),
                };
            }
            // STONE: the bare-symbol shorthand dies — only the FQDN keyword
            // form is recognized here now (see the "Some" comment above for
            // the full rationale; identical for "Ok"/"Err").
            let head_is_ok = matches!(
                head,
                WatAST::Keyword(k, _) if k == ":wat::core::Ok"
            );
            if head_is_ok {
                if items.len() != 2 {
                    return Err(RuntimeError::new(
                        pattern.span().clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "(Ok _) takes exactly one field, got {}",
                                items.len() - 1
                            ),
                        },
                    )
                    .into());
                }
                return match value {
                    Value::Result(r) => match &**r {
                        Ok(inner) => try_match_pattern(&items[1], inner, outer, sym),
                        Err(_) => Ok(None),
                    },
                    _ => Ok(None),
                };
            }
            let head_is_err = matches!(
                head,
                WatAST::Keyword(k, _) if k == ":wat::core::Err"
            );
            if head_is_err {
                if items.len() != 2 {
                    return Err(RuntimeError::new(
                        pattern.span().clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "(Err _) takes exactly one field, got {}",
                                items.len() - 1
                            ),
                        },
                    )
                    .into());
                }
                return match value {
                    Value::Result(r) => match &**r {
                        Err(inner) => try_match_pattern(&items[1], inner, outer, sym),
                        Ok(_) => Ok(None),
                    },
                    _ => Ok(None),
                };
            }
            match head {
                // Arc 048 — user-enum tagged variant. Pattern
                // `(:enum::Variant pat1 pat2 ...)` matches `Value::Enum`
                // whose `type_path::variant_name` composes to the same
                // path AND whose `fields` count matches.
                // Arc 055 — each sub-pattern is recursive (was: bare
                // symbol only). Linear shadowing — each sub-pattern's
                // bindings layer on top of the previous via Environment
                // chaining.
                WatAST::Keyword(variant_path, _) => match value {
                    Value::Enum(ev) => {
                        let composed = format!("{}::{}", ev.type_path, ev.variant_name);
                        if composed != *variant_path {
                            return Ok(None);
                        }
                        let sub_pats = &items[1..];
                        if sub_pats.len() != ev.fields.len() {
                            return Err(RuntimeError::new(
                                pattern.span().clone(),
                                RuntimeErrorKind::MalformedForm {
                                    head: ":wat::core::match".into(),
                                    reason: format!(
                                        "({} ...) takes {} field(s) for variant {}, got {}",
                                        variant_path,
                                        ev.fields.len(),
                                        ev.variant_name,
                                        sub_pats.len()
                                    ),
                                },
                            )
                            .into());
                        }
                        let mut env = outer.clone();
                        for (sub_pat, field_value) in sub_pats.iter().zip(ev.fields.iter()) {
                            match try_match_pattern(sub_pat, field_value, &env, sym)? {
                                Some(new_env) => env = new_env,
                                None => return Ok(None),
                            }
                        }
                        Ok(Some(env))
                    }
                    _ => Ok(None),
                },
                // Arc 055 — tuple destructure. Pattern is a list with no
                // recognized variant constructor at head; value must be
                // a tuple of matching arity. Each sub-pattern matches
                // one element by position.
                _ => match value {
                    Value::Tuple(elems) => {
                        if items.len() != elems.len() {
                            return Ok(None);
                        }
                        let mut env = outer.clone();
                        for (sub_pat, sub_val) in items.iter().zip(elems.iter()) {
                            match try_match_pattern(sub_pat, sub_val, &env, sym)? {
                                Some(new_env) => env = new_env,
                                None => return Ok(None),
                            }
                        }
                        Ok(Some(env))
                    }
                    _ => Ok(None),
                },
            }
        }
        // Arc 167 slice 1 — vector sub-patterns are not admitted
        // in arc 167. Slice 2 wires fn / defn signature consumers;
        // pattern-match positions are not legal Vector consumers.
        WatAST::Vector(_, _) => Err(RuntimeError::new(
            pattern.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "vector sub-patterns are not supported in arc 167".into(),
            },
        )
        .into()),
        // Arc 257.2 — Map brace-forms in match-arm pattern position.
        // classify_map_destructure detects hash-destructure ({var :field ...});
        // keys-destructure ({:keys [x y z]}) is not a valid match sub-pattern
        // (let-binding position only). Plain map literals are not match patterns.
        WatAST::Map(map_pairs, span) => {
            let md = WatAST::classify_map_destructure(map_pairs);
            let is_hash = matches!(&md, Some(m) if m.kind == crate::ast::MapDestructureKind::Hash);
            if is_hash {
                let md = md.unwrap();
                // Collect (var_name, bare_field_name) pairs from classifier.
                let pairs: Vec<(String, String)> = md
                    .bindings
                    .into_iter()
                    .map(|(ident, field, _)| (crate::scope::env_key(&ident).into_owned(), field))
                    .collect();
                // Dispatch on scrutinee receiver type. Arc 293.R2.1 — Aggregate.
                match value {
                    // Record/HolonRecord → keyword_accessor_record.
                    Value::Aggregate(a) if a.nature != Nature::Struct => {
                        let mut env = outer.clone();
                        for (var_name, bare_field) in &pairs {
                            let field_val = keyword_accessor_record(
                                bare_field,
                                Arc::new(a.class.to_string()),
                                a.fields.clone(),
                                sym,
                                span,
                            )?;
                            env = env
                                .child()
                                .bind_unknown_span(var_name.clone(), TrackedValue::from(field_val))
                                .build();
                        }
                        Ok(Some(env))
                    }
                    Value::Aggregate(a) => {
                        // Struct → keyword_accessor_struct.
                        let mut env = outer.clone();
                        for (var_name, bare_field) in &pairs {
                            let field_val =
                                keyword_accessor_struct(bare_field, a.clone(), sym, span)?;
                            env = env
                                .child()
                                .bind_unknown_span(var_name.clone(), TrackedValue::from(field_val))
                                .build();
                        }
                        Ok(Some(env))
                    }
                    Value::wat__std__HashMap(map) => {
                        let mut env = outer.clone();
                        for (var_name, bare_field) in &pairs {
                            let key_str = format!(":{}", bare_field);
                            let key = Value::wat__core__keyword(Arc::new(key_str));
                            let opt_val = match map.get(&key) {
                                Some(v) => Value::Option(Arc::new(Some(v.clone()))),
                                None => Value::Option(Arc::new(None)),
                            };
                            env = env
                                .child()
                                .bind_unknown_span(var_name.clone(), TrackedValue::from(opt_val))
                                .build();
                        }
                        Ok(Some(env))
                    }
                    // Any other value type: arm does not match; fall to next arm.
                    _ => Ok(None),
                }
            } else {
                // Not a hash-destructure (keys-destructure or plain map literal
                // in match-arm position): not a valid match sub-pattern.
                Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: "map in match-arm position must be a hash-destructure \
                        ({var :field ...}); keys-destructure and plain map literals \
                        are not valid match sub-patterns"
                            .into(),
                    },
                )
                .into())
            }
        }
        // Arc 244 — NilLit pattern matches Value::Unit (the nil value).
        WatAST::NilLit(_) => match value {
            Value::Unit => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        // Set literals are not match sub-patterns.
        WatAST::Set(_, span) => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "set literal is not a valid match sub-pattern".into(),
            },
        )
        .into()),
    }
}


/// `(:wat::core::type <any-value>) -> :wat::core::String` — arc 234 Stone 234.0.
///
/// Polymorphic runtime primitive that extracts a Value's record-type FQDN as a String.
/// Works on every Value variant that exists at Stone 234.0 time. Dispatch table:
///
/// - `Value::holon__HolonAST(h)` → `extract_classifier(h)` (classifier-wrap FQDN)
///   with fallback to `"wat::holon::HolonAST"` for non-classifier-wrapped HolonAST.
/// - `Value::Aggregate(a)` → `a.class` (per-instance FQDN, colon-free; covers Struct/Record/HolonRecord).
/// - Any other Value → `Value::type_name()` (existing Rust method; returns FQDN per
///   arc 224 Stone 224.5 naming audit).
///
/// Routes through `Value::declared_type_name` which is the ONE exhaustive authority (arc 237 Stone 237.5).
///
/// Arc 255 Stone A-2-ii-b-0 — `pub(crate)` so `src/intrinsic/reflect.rs`'s thin
/// `#[wat_intrinsic]` delegate can call straight into this unchanged body.
pub(crate) fn eval_type(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::type";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let arg_val = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc 237 Stone 237.5.fix-nominal-identity — route through the ONE authority.
    // declared_type_name is exhaustive and wildcard-free; covers Enum/Struct/Record/HolonAST
    // and all primitives.  No inline dispatch here — the authority is the single source.
    let type_str = arg_val.declared_type_name();
    Ok(Value::String(Arc::new(type_str)))
}

// Arc 255 Stone P6-c-W6 — `:wat::core::length`/`empty?` moved verbatim into
// `#[wat_intrinsic]` handlers (`src/intrinsic/collection.rs`) with their real (1/1)
// arities declared; the pre-match registry check intercepts both names before
// reaching the giant match.

// ─── Arc 237 Stone 237.7b-ii — :wat::core::contains? ────────────────────────

/// `(:wat::core::contains? <collection> <elem-or-key>) -> :wat::core::bool` — arc 237 Stone 237.7b-ii.
///
/// Polymorphic collection-membership predicate: ∀T. (T, elem) -> bool.
/// Mirrors `eval_empty` in shape: arity-2, eval args, match Value variant.
/// Delegates to the existing per-type inner helpers for correct semantics:
/// - `Value::Vec(..)` → vector element membership (PartialEq scan)
/// - `Value::wat__std__HashSet(..)` → set membership (Hash+Eq)
/// - `Value::wat__std__HashMap(..)` → KEY membership (contains-key?, not value)
///
/// All other variants produce a teaching `RuntimeError::TypeMismatch`.
fn eval_contains(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::contains?";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let arg0_val = eval_inner(&args[0], env, sym)?.value_owned();
    let arg1_val = eval_inner(&args[1], env, sym)?.value_owned();
    // Arc-278 strike A — map-family arms route through MapContainer (has_key capability).
    // The capability DRIVES the accepted set: the `if m.has_key()` guard is the genuine gate,
    // not a debug_assert. Exhaustive match over the closed MapContainer enum — NO `_`. Adding a
    // new keyed container forces this arm to be updated before the code compiles.
    use crate::collection::map_container::MapContainer;
    match MapContainer::of_value(&arg0_val) {
        Some(m) if m.has_key() => return match m {
            MapContainer::HashMap => crate::collection::eval::hashmap_contains_key_q_inner(&arg0_val, &arg1_val),
            // Arc-278-0a — PersistentMap: contains? checks KEY membership (same as HashMap).
            MapContainer::PersistentMap => crate::collection::eval::persistentmap_contains_key_q_inner(&arg0_val, &arg1_val),
            // Arc-278-A2 — Record: contains? tests field existence by keyword name.
            MapContainer::Record => crate::collection::eval::record_contains_field_q_inner(&arg0_val, &arg1_val, list_span, sym),
        },
        Some(_) => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), or (HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(&arg0_val))
        }).into()),
        None => {}
    }
    // Arc-278 seq-1a — seq-family arms route through StreamContainer (searchable capability).
    // The capability DRIVES the accepted set: the `if c.searchable()` guard is the genuine gate.
    // Exhaustive match over the closed StreamContainer enum — NO `_`. Adding a new seq container
    // forces this arm to be updated before the code compiles.
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&arg0_val) {
        Some(c) if c.searchable() => match c {
            StreamContainer::Vector => crate::collection::eval::vector_contains_q_inner(&arg0_val, &arg1_val),
            StreamContainer::HashSet => crate::collection::eval::hashset_contains_q_inner(&arg0_val, &arg1_val),
            StreamContainer::PersistentVector => crate::collection::eval::persistentvector_contains_q_inner(&arg0_val, &arg1_val),
            // seq-1b — filled
            StreamContainer::List => crate::collection::eval::list_contains_q_inner(&arg0_val, &arg1_val),
            StreamContainer::Tuple => crate::collection::eval::tuple_contains_q_inner(&arg0_val, &arg1_val),
            StreamContainer::WatAstList => crate::collection::eval::watastlist_contains_q_inner(&arg0_val, &arg1_val),
            // Arc 118 — searchable() gate excludes Stream (contains? forces the whole seq):
            StreamContainer::Stream => unreachable!("searchable() gate excludes Stream"),
        },
        Some(_) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), or (HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(&arg0_val))
        }).into()),
        None => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), or (HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(&arg0_val))
        }).into()),
    }
}

// ─── Arc 237 Stone 237.7b-iii — :wat::core::conj ────────────────────────────

/// `(:wat::core::conj <collection> <elem>) -> <collection>` — arc 237 Stone 237.7b-iii.
///
/// Polymorphic type-preserving append/insert: ∀T. ((coll :- [T]), T) -> (coll :- [T]).
/// Mirrors `eval_contains` in shape: arity-2, eval args, match Value variant.
/// Delegates to the existing per-type inner helpers for correct semantics:
/// - `Value::Vec(..)` → vector append (clone + push; functional, not mutating)
/// - `Value::wat__std__HashSet(..)` → set insert (clone + insert; functional)
///
/// HashMap excluded — HashMap insertion requires key+value pair (`assoc`).
/// All other variants produce a teaching `RuntimeError::TypeMismatch`.
///
/// Arc 255 Stone the-collection-readers — homed into a thin `#[wat_intrinsic]` delegate
/// (`src/intrinsic/collection.rs`) with its real (2) arity declared; the shim's own arity
/// check makes this fn's hand-rolled `args.len() != 2` guard dead, so it retires here. `pub(crate)`
/// so the delegate can call it.
pub(crate) fn eval_conj(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::conj";
    let arg0_val = eval_inner(&args[0], env, sym)?.value_owned();
    let arg1_val = eval_inner(&args[1], env, sym)?.value_owned();
    // Arc-278 strike 2 — classify via the registry (StreamContainer::of_value + has_append()).
    // The registry is the single source of truth; per-type inner helpers below do the work.
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&arg0_val) {
        Some(container) if container.has_append() => {
            match container {
                StreamContainer::Vector => {
                    crate::collection::eval::vector_conj_inner(&arg0_val, &arg1_val)
                }
                StreamContainer::HashSet => {
                    crate::collection::eval::hashset_conj_inner(&arg0_val, &arg1_val)
                }
                // Arc-278-0b — PersistentVector: generic conj dispatches to persistentvector_conj_inner.
                StreamContainer::PersistentVector => {
                    crate::collection::eval::persistentvector_conj_inner(&arg0_val, &arg1_val)
                }
                // Arc 220 Stone 220.4 — List: generic conj dispatches to list_conj_inner (PREPEND).
                StreamContainer::List => {
                    crate::collection::eval::list_conj_inner(&arg0_val, &arg1_val)
                }
                // has_append() gate excludes these — named arms, genuinely dead, compiler-forced:
                StreamContainer::Tuple | StreamContainer::WatAstList | StreamContainer::Stream => {
                    unreachable!("has_append() gate excludes Tuple/WatAstList/Stream")
                }
            }
        }
        // ∅ N/A or ○ gap: container has no append capability (Tuple, WatAstList — nature forbids / gap).
        Some(_) | None => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Vector :- [T]), (HashSet :- [T]), (PersistentVector :- [T]), or (List :- [T])",
                got: Box::new(ValueSnapshot::of(&arg0_val)),
            },
        )
        .into()),
    }
}

// ─── Arc 237 Stone 237.7b-iv — :wat::core::get ──────────────────────────────

/// `(:wat::core::get <collection> <index-or-key>) -> (Option :- [element])` — arc 237 Stone 237.7b-iv.
///
/// Polymorphic indexed/keyed lookup: ∀T. (coll, idx-or-key) -> (Option :- [element]).
/// Mirrors `eval_conj` in shape: arity-2, eval args, match Value variant.
/// Delegates to the existing per-type inner helpers for correct semantics:
/// - `Value::Vec(..)` → `vector_get_inner` (index i64 → (Option :- [T]); inner already wraps in Value::Option)
/// - `Value::wat__std__HashMap(..)` → `hashmap_get_inner` (key → (Option :- [V]); inner already wraps)
///
/// HashSet excluded — HashSet has no positional get (use `contains?`).
/// All other variants produce a teaching `RuntimeError::TypeMismatch`.
fn eval_get(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::get";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let arg0_val = eval_inner(&args[0], env, sym)?.value_owned();
    let arg1_val = eval_inner(&args[1], env, sym)?.value_owned();
    // Arc-278 strike A — map-family arms route through MapContainer (keyed_lookup capability).
    // The capability DRIVES the accepted set: the `if m.keyed_lookup()` guard is the genuine gate,
    // not a debug_assert. Exhaustive match over the closed MapContainer enum — NO `_`. Adding a
    // new keyed container forces this arm to be updated before the code compiles.
    use crate::collection::map_container::MapContainer;
    match MapContainer::of_value(&arg0_val) {
        Some(m) if m.keyed_lookup() => {
            return match m {
                MapContainer::HashMap => {
                    crate::collection::eval::hashmap_get_inner(&arg0_val, &arg1_val)
                }
                // Arc-278-0a — PersistentMap: generic get dispatches to persistentmap_get_inner.
                MapContainer::PersistentMap => {
                    crate::collection::eval::persistentmap_get_inner(&arg0_val, &arg1_val)
                }
                // Arc-278-A2 — Record: get by keyword resolves field index via RecordDef.
                MapContainer::Record => {
                    crate::collection::eval::record_get_inner(&arg0_val, &arg1_val, list_span, sym)
                }
            };
        }
        Some(_) => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), or (PersistentVector :- [T])",
                    got: Box::new(ValueSnapshot::of(&arg0_val)),
                },
            )
            .into())
        }
        None => {}
    }
    // Arc-278 seq-1a — seq-family arms route through StreamContainer (gettable capability).
    // The capability DRIVES the accepted set: the `if c.gettable()` guard is the genuine gate.
    // Exhaustive match over the closed StreamContainer enum — NO `_`. Adding a new seq container
    // forces this arm to be updated before the code compiles.
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&arg0_val) {
        Some(c) if c.gettable() => match c {
            StreamContainer::Vector => {
                crate::collection::eval::vector_get_inner(&arg0_val, &arg1_val)
            }
            StreamContainer::PersistentVector => {
                crate::collection::eval::persistentvector_get_inner(&arg0_val, &arg1_val)
            }
            // seq-1b — filled
            StreamContainer::List => crate::collection::eval::list_get_inner(&arg0_val, &arg1_val),
            StreamContainer::WatAstList => {
                crate::collection::eval::watastlist_get_inner(&arg0_val, &arg1_val)
            }
            StreamContainer::HashSet => {
                crate::collection::eval::hashset_get_inner(&arg0_val, &arg1_val)
            }
            // ∅ N/A — Tuple: heterogeneous product; runtime-index cannot be typed
            StreamContainer::Tuple => {
                unreachable!("gettable() gate excludes Tuple (∅ N/A — heterogeneous product)")
            }
            // Arc 118 — gettable() gate excludes Stream (no O(1) random access; walk via rest):
            StreamContainer::Stream => {
                unreachable!("gettable() gate excludes Stream (∅ N/A — no random access)")
            }
        },
        Some(_) => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), or (PersistentVector :- [T])",
                got: Box::new(ValueSnapshot::of(&arg0_val)),
            },
        )
        .into()),
        None => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), or (PersistentVector :- [T])",
                got: Box::new(ValueSnapshot::of(&arg0_val)),
            },
        )
        .into()),
    }
}

// ─── Arc 237 Stone 237.5 — :wat::core::conforms? ─────────────────────────────

/// `(:wat::core::conforms? <value> :TypeExpr)` → `:wat::core::bool` — arc 237 Stone 237.5.
///
/// Recursive type-conformance check over the `TypeExpr` grammar:
/// - `Path` → nominal identity (record class_fqdn / value.type_name()) or
///   alias expansion or union-membership recursion.
/// - `Parametric` → classifier match + element-wise recursion.
/// - `Tuple` → same-arity + per-position recursion.
/// - `Fn` / `Var` → error (unsupported / synthetic).
///
/// Error contract: well-formed type + no match → `false`.
/// Unknown/unregistered type name, Fn, or Var → `Err` (bad input, not negative result).
fn eval_conforms(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::conforms?";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // Evaluate the value (arg 0).
    let value = eval_inner(&args[0], env, sym)?.value_owned();
    // Parse the type expression from arg 1 (type-position keyword — labels-are-ASTs).
    let texpr = parse_type_slot(&args[1]).map_err(|e| {
        RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("second arg must be a type keyword: {}", e),
            },
        )
    })?;
    // Acquire the runtime TypeEnv for Path/Alias/Union resolution.
    let types = sym.types().ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: "conforms? requires the type registry, but the SymbolTable has no TypeEnv attached (programmer error: this build path didn't go through startup_from_source / freeze)".into()
    }))?;
    let result = conforms_check(&value, &texpr, types).map_err(|reason| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason,
            },
        )
    })?;
    Ok(Value::bool(result))
}

// ─── Arc 278 the REQUEST-MALFORMED wall — :wat::edn::validate ────────────────

/// `(:wat::edn::validate <value> :DeclaredType)` → `:wat::edn::Validation` —
/// arc 278 Stone 1 (DESIGN-request-malformed-input-sanitization.md).
///
/// **Input sanitization at a trust boundary.** `conforms?` (immediately above)
/// cannot serve here: for an Aggregate its `TypeExpr::Path` arm is a NOMINAL
/// identity check (`concrete_type_name_matches`) that never recurses into the
/// record's FIELDS — so `#dos.Bag/PutRequest {:items [1 2 3]}` conforms? TRUE
/// against `items <- (Vector :- [String])`. That gap is the denial of service: the
/// handler then uses the field at its declared type and the whole service dies.
///
/// This is a THIN WRAPPER over the deep walker that already existed and had zero
/// production callers since arc 258 Stone 258.5b deleted its last one on the
/// trusted-wire premise: `edn::render::edn_to_typed_value` walks the declared
/// `TypeExpr` per-field / per-element and yields the offending path
/// (`.items.[0]`). We render the runtime `Value` to EDN (`value_to_edn_with` —
/// the same writer `:wat::edn::write` uses, so both tiers present identically:
/// the process tier's decoded `Value` and the thread tier's verbatim crossbeam
/// `Value` reach this the same way) and hand it to that walker. No new
/// validation logic is minted — the two halves are merely connected.
///
/// Never raises on a bad *value*: a mismatch is the matchable
/// `Validation::Invalid[path expected got]`. A bad *type keyword* (unparseable /
/// no registry) is a programmer error and still raises.
///
/// Arc 255 Stone HOME-11 — widened from a bare `fn` to `pub(crate) fn` so
/// `src/intrinsic/edn.rs`'s registry handler can call it directly. Visibility-only change; the
/// body (and its `src/check.rs` special-cased type contract) is untouched.
pub(crate) fn eval_edn_validate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::edn::validate";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let value = eval_inner(&args[0], env, sym)?.value_owned();
    let texpr = parse_type_slot(&args[1]).map_err(|e| {
        RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("second arg must be a type keyword: {}", e),
            },
        )
    })?;
    if sym.types().is_none() {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "validate requires the type registry, but the SymbolTable has no TypeEnv attached (programmer error: this build path didn't go through startup_from_source / freeze)".into()
        }).into());
    }
    let edn = crate::edn::render::value_to_edn_with(&value, sym.types().map(|a| a.as_ref()));
    Ok(
        match crate::edn::render::edn_to_typed_value(&texpr, &edn, sym) {
            Ok(_) => Value::Enum(Arc::new(EnumValue {
                type_path: ":wat::edn::Validation".into(),
                variant_name: "Valid".into(),
                names: no_field_names(),
                fields: vec![],
            })),
            Err(e) => Value::Enum(Arc::new(EnumValue {
                type_path: ":wat::edn::Validation".into(),
                variant_name: "Invalid".into(),
                names: builtin_enum_variant_names(":wat::edn::Validation", "Invalid"),
                fields: vec![
                    Value::Vec(Arc::new(edn_coerce_path_segments(&e.path))),
                    Value::String(Arc::new(e.expected)),
                    Value::String(Arc::new(e.got)),
                ],
            })),
        },
    )
}

/// Split an `EdnCoerceError.path` (`".items.[0]"` — dot-joined, built leaf-upward
/// by `EdnCoerceError::at`) into its SEGMENTS (`["items" "[0]"]`).
///
/// The segments are the structured half of the `Invalid` payload: a caller can
/// index/walk them, which is why `path` is a `(Vector :- [String])` while
/// `expected`/`got` are rendered Strings. An empty path (the mismatch is the
/// value itself, not a sub-field) yields an empty vector — honest, not a `[""]`.
fn edn_coerce_path_segments(path: &str) -> Vec<Value> {
    wat_reader::identifier::dot_path_segments(path)
        .into_iter()
        .map(|seg| Value::String(Arc::new(seg.to_string())))
        .collect()
}

/// Recursive conformance walker over the `TypeExpr` grammar.
///
/// Returns `Ok(true)` / `Ok(false)` for well-formed types; `Err(reason)`
/// for unknown type names, Fn types, and Var (synthetic) expressions.
fn conforms_check(
    value: &Value,
    texpr: &crate::types::TypeExpr,
    types: &crate::types::TypeEnv,
) -> Result<bool, String> {
    use crate::types::{TypeDef, TypeExpr};

    match texpr {
        // ── Path arm ─────────────────────────────────────────────────────
        TypeExpr::Path(name) => {
            // Resolve in the TypeEnv.
            match types.get(name) {
                // Alias → expand and recurse.
                Some(TypeDef::Alias(alias)) => conforms_check(value, &alias.expr, types),
                // Union → value must conform to ANY member.
                Some(TypeDef::Union(union)) => {
                    let members = crate::types::collect_union_members(union, types);
                    for member in &members {
                        if conforms_check(value, member, types).unwrap_or(false) {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                // Struct / Enum / Newtype / Record → nominal identity check.
                // Special case: :wat::core::Record is the root record supertype (opaque umbrella
                // struct). Every value from `(:wat::core::defrecord ...)` is a subtype; conforms?
                // against :wat::core::Record returns true for any record value — NOT just exact match.
                // Same for :wat::holon::Record (the holonic-record umbrella). Arc 259.
                // Arc 293.3-core — Surface: runtime structural conformance (conforms? against a
                // surface) is not yet implemented; falls through as false. The static type checker
                // (`assignable`) handles structural surface satisfaction at compile time.
                Some(TypeDef::Surface(_)) => Ok(false),
                // Arc 293.2b — Aggregate (Struct + Record collapsed) + Enum + Newtype.
                Some(TypeDef::Aggregate(_))
                | Some(TypeDef::Enum(_))
                | Some(TypeDef::Newtype(_)) => {
                    let stripped_name = name.strip_prefix(':').unwrap_or(name.as_str());
                    if stripped_name == "wat::core::Record" || stripped_name == "wat::holon::Record"
                    {
                        return Ok(matches!(
                            value,
                            Value::Aggregate(a) if a.nature != Nature::Struct
                        ));
                    }
                    Ok(concrete_type_name_matches(value, name))
                }
                // Not in the TypeEnv — check built-in primitive paths.
                None => {
                    let stripped = name.strip_prefix(':').unwrap_or(name);
                    if is_builtin_primitive(stripped) {
                        Ok(concrete_type_name_matches(value, name))
                    } else {
                        // Not a TypeDef, not a built-in.
                        // Record classes declared via `(:wat::core::defrecord ...)` expand
                        // to `defn` forms and are NOT registered in the TypeEnv — their
                        // type identity lives in `Value::Aggregate.class` (colon-free FQDN).
                        // For an Aggregate value, check class directly (the value
                        // carries its own type tag — it is the ground truth). For all other
                        // value kinds, the name is genuinely unknown → Err (per error contract).
                        // Arc 293.R2.1 — Aggregate: class is colon-free.
                        match value {
                            Value::Aggregate(a) => {
                                Ok(a.class.as_ref() == stripped)
                            }
                            _ => Err(format!(
                                "unknown type name '{}' is not registered in the TypeEnv and is not a built-in primitive; \
                                 cannot determine conformance (this is bad input, not a negative result — \
                                 check the spelling and ensure the type is declared before use)",
                                name
                            )),
                        }
                    }
                }
            }
        }

        // ── Parametric arm ───────────────────────────────────────────────
        TypeExpr::Parametric { head, args } => {
            // Verify the value's collection classifier matches the head.
            let value_tag = value.type_name();
            let classifier_ok = match head.as_str() {
                "wat::core::Vector" => value_tag == "wat::core::Vector",
                "wat::core::List" => value_tag == "wat::core::List",
                "wat::core::HashSet" => value_tag == "wat::core::HashSet",
                "wat::core::HashMap" => value_tag == "wat::core::HashMap",
                // User parametric type — nominal head match only (full
                // parametric-instance introspection is arc 235 territory).
                other => value_tag == other,
            };
            if !classifier_ok {
                return Ok(false);
            }
            // Recurse element-wise for known collection classifiers.
            match head.as_str() {
                "wat::core::Vector" => {
                    if args.is_empty() {
                        return Ok(true);
                    }
                    let elem_type = &args[0];
                    if let Value::Vec(elems) = value {
                        for elem in elems.iter() {
                            if !conforms_check(elem, elem_type, types)? {
                                return Ok(false);
                            }
                        }
                    }
                    Ok(true)
                }
                "wat::core::List" => {
                    if args.is_empty() {
                        return Ok(true);
                    }
                    let elem_type = &args[0];
                    if let Value::wat__core__List(elems) = value {
                        for elem in elems.iter() {
                            if !conforms_check(elem, elem_type, types)? {
                                return Ok(false);
                            }
                        }
                    }
                    Ok(true)
                }
                "wat::core::HashSet" => {
                    if args.is_empty() {
                        return Ok(true);
                    }
                    let elem_type = &args[0];
                    if let Value::wat__std__HashSet(elems) = value {
                        for elem in elems.iter() {
                            if !conforms_check(elem, elem_type, types)? {
                                return Ok(false);
                            }
                        }
                    }
                    Ok(true)
                }
                "wat::core::HashMap" => {
                    if args.len() < 2 {
                        return Ok(true);
                    }
                    let key_type = &args[0];
                    let val_type = &args[1];
                    if let Value::wat__std__HashMap(map) = value {
                        for (k, v) in map.iter() {
                            if !conforms_check(k, key_type, types)? {
                                return Ok(false);
                            }
                            if !conforms_check(v, val_type, types)? {
                                return Ok(false);
                            }
                        }
                    }
                    Ok(true)
                }
                // User parametric — classifier matched; nominal head match is the honest check.
                _ => Ok(true),
            }
        }

        // ── Tuple arm ─────────────────────────────────────────────────────
        TypeExpr::Tuple(elems) => {
            if let Value::Tuple(items) = value {
                if items.len() != elems.len() {
                    return Ok(false);
                }
                for (item, elem_type) in items.iter().zip(elems.iter()) {
                    if !conforms_check(item, elem_type, types)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }

        // ── Fn arm — unsupported (affirmative scope cut; runtime limitation) ─
        TypeExpr::Fn { .. } => Err(
            "fn-type conformance is unsupported: runtime function values do not carry \
             a recoverable full arg/ret signature, so deep fn-conformance cannot be \
             honestly computed. This is an affirmative scope cut, not a deferral."
                .into(),
        ),

        // ── Var arm — synthetic; never appears in user-written :TypeExpr ─────
        TypeExpr::Var(id) => Err(format!(
            "TypeExpr::Var({}) is a synthetic unification variable and cannot appear \
             in a user-written type expression passed to conforms?",
            id
        )),
    }
}

/// Nominal identity check: does `value`'s declared FQDN match `path_with_colon`?
///
/// Routes through `Value::declared_type_name` — the ONE authority for value→type
/// identity (arc 237 Stone 237.5.fix-nominal-identity).  The previous inline
/// match (Record special-case + `other.type_name()` wildcard) is deleted; all
/// forms (HolonAST / Struct / Record / Enum / primitives) are handled by the
/// exhaustive `declared_type_name` method.
#[inline]
fn concrete_type_name_matches(value: &Value, path_with_colon: &str) -> bool {
    // Strip the leading ':' from the Path name; declared_type_name returns colon-free FQDN.
    let stripped = path_with_colon.strip_prefix(':').unwrap_or(path_with_colon);
    value.declared_type_name() == stripped
}

/// Returns `true` if `name` (colon-free FQDN) is a built-in primitive type
/// recognized at the runtime level. These paths never appear in the TypeEnv
/// (they're substrate, not user-declared), but `conforms?` must handle them.
fn is_builtin_primitive(name: &str) -> bool {
    matches!(
        name,
        "wat::core::bool"
            | "wat::core::i64"
            | "wat::core::u8"
            | "wat::core::f64"
            | "wat::core::String"
            | "wat::core::keyword"
            | "wat::core::nil"
            | "wat::core::Uuid"
            | "wat::core::char"
            | "wat::core::rational"
            | "wat::core::bigint"
            | "wat::core::fn"
            | "wat::core::Tuple"
            | "wat::core::Vector"
            | "wat::core::List"
            | "wat::core::HashMap"
            | "wat::core::HashSet"
            | "wat::core::Option"
            | "wat::core::Result"
            | "wat::core::Record"
            | "wat::WatAST"
            | "wat::holon::HolonAST"
            | "wat::holon::Vector"
            | "wat::holon::OnlineSubspace"
            | "wat::holon::Reckoner"
            | "wat::holon::Engram"
            | "wat::holon::EngramLibrary"
            | "wat::holon::Hologram"
            | "wat::time::Instant"
            | "wat::time::Duration"
            | "wat::kernel::Sender"
            | "wat::kernel::Receiver"
            | "wat::kernel::ProgramHandle"
            | "wat::kernel::HandlePool"
            | "wat::kernel::ChildHandle"
            | "wat::io::IOReader"
            | "wat::io::IOWriter"
    )
}

// ─── end Stone 237.5 ─────────────────────────────────────────────────────────

// ─── Arc 237 Stone S-A — :wat::core::subtype? ────────────────────────────────

/// `(:wat::core::subtype? :ChildType :ParentType)` → `:wat::core::bool` — arc 237 Stone S-A.
///
/// Directional, transitive, reflexive predicate over the `typesub` child→parent
/// edge-registry on [`TypeEnv`]. Both arguments are **type-position keywords**
/// (taken literally — NOT evaluated as values). Mirrors `eval_conforms` in shape.
///
/// Error contract:
/// - Both args must be `WatAST::Keyword`; else `MalformedForm`.
/// - Both names must be known (in TypeEnv or is_builtin_primitive); else `MalformedForm`.
///   This keeps `false` honest (probe 10): an unknown name is bad input, not a negative result.
/// - Well-formed known pair → `Value::bool(is_subtype(a, b, types))`.
fn eval_subtype(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::subtype?";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // Both args are type-position keywords — extract paths literally (labels-are-ASTs).
    let a_kw = match &args[0] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "first arg must be a type keyword (e.g. :my::Child)".into(),
                },
            )
            .into())
        }
    };
    let b_kw = match &args[1] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "second arg must be a type keyword (e.g. :my::Parent)".into(),
                },
            )
            .into())
        }
    };
    // Acquire the runtime TypeEnv.
    let types = sym.types().ok_or_else(|| {
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: "subtype? requires the type registry, but the SymbolTable has no TypeEnv attached \
                 (programmer error: this build path didn't go through startup_from_source / freeze)"
            .into()
    })
    })?;
    // Validate both names are known (in TypeEnv OR a built-in primitive).
    // This keeps `false` honest: an unknown name is bad input, not a negative result.
    let a_known = {
        let stripped = a_kw.strip_prefix(':').unwrap_or(&a_kw);
        types.get(&a_kw).is_some() || is_builtin_primitive(stripped)
    };
    let b_known = {
        let stripped = b_kw.strip_prefix(':').unwrap_or(&b_kw);
        types.get(&b_kw).is_some() || is_builtin_primitive(stripped)
    };
    if !a_known {
        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "unknown type name '{}' is not registered in the TypeEnv and is not a built-in primitive; \
                 cannot determine subtype relationship (this is bad input, not a negative result — \
                 check the spelling and ensure the type is declared before use)",
                a_kw
            )
        }).into());
    }
    if !b_known {
        return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "unknown type name '{}' is not registered in the TypeEnv and is not a built-in primitive; \
                 cannot determine subtype relationship (this is bad input, not a negative result — \
                 check the spelling and ensure the type is declared before use)",
                b_kw
            )
        }).into());
    }
    Ok(Value::bool(crate::types::is_subtype(&a_kw, &b_kw, types)))
}

// ─── end Stone S-A ───────────────────────────────────────────────────────────

// ─── Arc 294.c.2a — aggregate-new + build_holon_hologram ─────────────────────


// Arc 109 Stone — the record home — `eval_aggregate_new` moved to `src/record/construct.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `construct_aggregate` moved to `src/record/construct.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_kwargs_construct` moved to `src/record/construct.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// ─── End Arc 294.c.2a ─────────────────────────────────────────────────────────

// ─── Arc 293 K3 — three projection verbs ──────────────────────────────────────

// Arc 109 Stone — the record home — `project_surface_attrs` moved to `src/record/project.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `parse_projection_args` moved to `src/record/project.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// RETIRED 293 K3-revise — `fn eval_to_struct` (`:wat::core::to-struct x :S` → `:S$struct`).
// Projection is ONE-WAY UP (AGGREGATE-MODEL.md § to-record, settled 2026-06-29): you never
// project down to a struct; `$struct` is the impure tier; you already hold the struct in locus.
// Retirement entry lives in `src/remedy/retirement.rs`.

// Arc 109 Stone — the record home — `eval_to_core_record` moved to `src/record/project.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.


// ─── End Arc 293 K3 ───────────────────────────────────────────────────────────

// Arc 109 Stone — the record home — `eval_record_field_at` moved to `src/record/access.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_record_q` moved to `src/record/access.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_list_q` moved to `src/record/access.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `record_field_map` moved to `src/record/update.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_record_to_map` moved to `src/record/update.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_record_same_data` moved to `src/record/update.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `record_assoc_inner` moved to `src/record/update.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — the record home — `eval_record_assoc` moved to `src/record/update.rs`
// (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.


// ─── Algebra-core UpperCall runtime construction ────────────────────────


// ─── Arc 074 — Substrate floor accessors ────────────────────────────


// ─── Arc 076 — therm-routed Hologram + filtered-argmax ─────────────


// ─── Arc 228 — Pascal-Case collection classifier-wrap constructors ────────────
//
// Each constructor takes a Vec<HolonAST> and wraps it in
// Bind(Atom("ClassName"), Bundle(items)) per the typed-entities doctrine.
// The outer Bind carries the classifier; the inner Bundle carries the data.
// Type recovery: extract the classifier-atom from the outer Bind.


// ─── Arc 226 Stone 226.1 — Type predicates (classifier-name match) ───────────
//
// Type checking emerges from VSA similarity — per [[typed-entities-doctrine]]:
//   (is-X? value) ≡ similarity(value's class atom vector, prototype-of-X vector)
//
// Stone 226.1 ships v1: EXACT STRUCTURAL MATCH on classifier name.
// The classifier name IS a perfect VSA similarity probe in the degenerate
// (exact-match) case — two identical atom strings produce identical vectors,
// cosine = 1.0. Future stones 226.2+ add threshold-tunable continuous scoring.
//
// All functions share the same shape:
//   1. Evaluate the HolonAST argument.
//   2. Call `extract_classifier` (arc 228 helper) to recover the classifier name.
//   3. Compare against the expected class name (or check `is_nil()` for Nil).
//   4. Return `Value::bool(matches)`.
//
// Non-HolonAST values (bare i64, String, etc.) are accepted but return false —
// the absence of a classifier is an honest "not this type" signal.


// Arc 109 Stone — holon into parity — `PairedVectors` moved to
// `src/holon/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `pair_values_to_vectors` moved to
// `src/holon/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Arc 296 G′ — empty `EnumValue.names`, shared so unit-variant (and
/// zero-field tagged-variant) constructors don't each allocate their own.
/// Correct for ANY variant with zero fields: there is nothing to name, so
/// there is no ambiguity to resolve via a registry lookup.
pub(crate) fn no_field_names() -> Arc<Vec<String>> {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| Arc::new(Vec::new())).clone()
}

// Arc 296 G′ — `:wat::spawn::ServiceEvent`'s tagged-variant field names, read straight
// from its `defenum` in `wat/spawn.wat` at COMPILE TIME via `wat_enum_field_names_from!`
// — never a runtime `TypeEnv` lookup.
//
// Why not the obvious "look it up in the registry" move `builtin_enum_variant_names`
// below uses for the Rust-registered outcome enums: `ServiceEvent` is declared via
// `defenum` in the `.wat` stdlib, not as a `types.rs::register_builtin_types` literal, so
// it is ABSENT from the cheap `TypeEnv::with_builtins()`. The first fix for that gap
// called `crate::freeze::env::build_env(vec![])` (the full baked-stdlib pipeline) lazily
// behind a `OnceLock`. Measured (a `#[test]`, isolated, `--test-threads=1`,
// `/proc/<pid>/stat` showing zero CPU time accruing, every thread parked in
// `futex_do_wait`): it DEADLOCKS — `OnceLock::get_or_init`'s closure runs stdlib
// macro-expansion, which re-enters a call needing the SAME lock before the first call
// returns, and `OnceLock` treats reentrant `get_or_init` as a hang, not an error. Reading
// the `.wat` source TEXT at compile time — what `wat_field_names_from!` already does for
// records — has no such hazard: there is no runtime environment to build, so nothing to
// re-enter. See `wat_enum_field_names_from!`'s doc (`wat-source-derive/src/lib.rs`) for
// the full account.
::wat_source_derive::wat_enum_field_names_from!(
    SERVICE_EVENT_ADMIN_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::ServiceEvent",
    "Admin"
);
::wat_source_derive::wat_enum_field_names_from!(
    SERVICE_EVENT_CONNECTION_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::ServiceEvent",
    "Connection"
);
::wat_source_derive::wat_enum_field_names_from!(
    SERVICE_EVENT_MESSAGE_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::ServiceEvent",
    "Message"
);
::wat_source_derive::wat_enum_field_names_from!(
    SERVICE_EVENT_CLOSED_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::ServiceEvent",
    "Closed"
);
::wat_source_derive::wat_enum_field_names_from!(
    SERVICE_EVENT_LOST_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::ServiceEvent",
    "Lost"
);
::wat_source_derive::wat_enum_field_names_from!(
    SERVICE_EVENT_MALFORMED_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::ServiceEvent",
    "Malformed"
);
::wat_source_derive::wat_enum_field_names_from!(
    SERVICE_EVENT_REJECTED_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::ServiceEvent",
    "Rejected"
);

// Arc 296 G′ — `:wat::sqlite::Cell`'s tagged-variant field names, same reasoning, read from
// `wat/sqlite.wat`. `Nil` needs no const: `parse_defenum`'s one-token lookahead makes even
// `:Nil []` a `Tagged{fields: []}` (never `Unit`), so its names are provably `no_field_names()`
// — zero fields, nothing to name, not a guess.
::wat_source_derive::wat_enum_field_names_from!(
    CELL_I64_FIELDS,
    "wat/sqlite.wat",
    ":wat::sqlite::Cell",
    "I64"
);
::wat_source_derive::wat_enum_field_names_from!(
    CELL_F64_FIELDS,
    "wat/sqlite.wat",
    ":wat::sqlite::Cell",
    "F64"
);
::wat_source_derive::wat_enum_field_names_from!(
    CELL_STR_FIELDS,
    "wat/sqlite.wat",
    ":wat::sqlite::Cell",
    "Str"
);

/// Arc 296 G′ — field names for a BUILTIN enum's tagged variant, for the many
/// constructors in this file (and its sibling modules) that build a value of a
/// statically-known type with no `TypeEnv` in scope (a `CosineOutcome`, a
/// `LociDiedError`, a `ServiceEvent`, a `:wat::sqlite::Cell`, …).
///
/// Two DIFFERENT kinds of "builtin" back these types, and this is the ONE door over both:
/// - Registered directly in `types.rs::register_builtin_types` as a literal `EnumDef`
///   (`CosineOutcome`, `LociDiedError`, `RunResult`, …) — present in the cheap
///   `TypeEnv::with_builtins()`, looked up there below.
/// - Declared via `(:wat::core::defenum …)` in a `.wat` stdlib file (`ServiceEvent` —
///   `wat/spawn.wat`; `:wat::sqlite::Cell` — `wat/sqlite.wat`) — ABSENT from
///   `with_builtins()`; sourced from the compile-time `wat_enum_field_names_from!` consts
///   just above instead (see that block's doc for why NOT a runtime registry).
///
/// Panics on an unknown type/variant — a programmer error (the constructor and the
/// registration disagree), not a user-facing fault; see STOP-2 in DESIGN-STONE-G-prime —
/// raise, never fall back to positional.
pub(crate) fn builtin_enum_variant_names(type_path: &str, variant: &str) -> Arc<Vec<String>> {
    // The `.wat`-declared exceptions first — `with_builtins()` doesn't carry them.
    match (type_path, variant) {
        (":wat::spawn::ServiceEvent", "Admin") => {
            return crate::value::value::names_arc_from_static(SERVICE_EVENT_ADMIN_FIELDS)
        }
        (":wat::spawn::ServiceEvent", "Connection") => {
            return crate::value::value::names_arc_from_static(SERVICE_EVENT_CONNECTION_FIELDS)
        }
        (":wat::spawn::ServiceEvent", "Message") => {
            return crate::value::value::names_arc_from_static(SERVICE_EVENT_MESSAGE_FIELDS)
        }
        (":wat::spawn::ServiceEvent", "Closed") => {
            return crate::value::value::names_arc_from_static(SERVICE_EVENT_CLOSED_FIELDS)
        }
        (":wat::spawn::ServiceEvent", "Lost") => {
            return crate::value::value::names_arc_from_static(SERVICE_EVENT_LOST_FIELDS)
        }
        (":wat::spawn::ServiceEvent", "Malformed") => {
            return crate::value::value::names_arc_from_static(SERVICE_EVENT_MALFORMED_FIELDS)
        }
        (":wat::spawn::ServiceEvent", "Rejected") => {
            return crate::value::value::names_arc_from_static(SERVICE_EVENT_REJECTED_FIELDS)
        }
        (":wat::sqlite::Cell", "I64") => {
            return crate::value::value::names_arc_from_static(CELL_I64_FIELDS)
        }
        (":wat::sqlite::Cell", "F64") => {
            return crate::value::value::names_arc_from_static(CELL_F64_FIELDS)
        }
        (":wat::sqlite::Cell", "Str") => {
            return crate::value::value::names_arc_from_static(CELL_STR_FIELDS)
        }
        (":wat::sqlite::Cell", "Nil") => return no_field_names(),
        _ => {}
    }
    static ENV: OnceLock<crate::types::TypeEnv> = OnceLock::new();
    let env = ENV.get_or_init(crate::types::TypeEnv::with_builtins);
    match env.get(type_path) {
        Some(crate::types::TypeDef::Enum(e)) => e.variant_names_arc(variant).unwrap_or_else(|| {
            panic!(
                "builtin_enum_variant_names: `{type_path}` has no TAGGED variant `{variant}` in \
                 its registration — the constructor and the registry disagree"
            )
        }),
        _ => panic!("builtin_enum_variant_names: `{type_path}` is not a registered builtin enum"),
    }
}


// Arc 109 Stone — holon into parity — `cosine_outcome_from_values` moved to
// `src/holon/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `presence_q_from_values` moved to
// `src/holon/coincident.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `coincident_q_from_values` moved to
// `src/holon/coincident.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `run_ast_arg_for_eval_coincident` moved to
// `src/holon/coincident.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `coincident_of_two_values` moved to
// `src/holon/coincident.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `eval_form_digest_coincident_shared` moved to
// `src/holon/coincident.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `eval_form_signed_coincident_shared` moved to
// `src/holon/coincident.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `FallbackVerdict` moved to
// `src/holon/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `classify_fallback_outcome` moved to
// `src/holon/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone — holon into parity — `dot_outcome_from_values` moved to
// `src/holon/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// ─── Vector portability (arc 061) — vector-bytes / bytes-vector ──────
//
// Wire format for transmission between users:
//
//   bytes 0..4   : dim as u32 little-endian  (validation header)
//   bytes 4..end : packed 2-bit cells, 4 cells per byte, LSB-first
//
// Each ternary cell encodes one i8 in {-1, 0, +1}:
//
//   0b00 →  0
//   0b01 → +1
//   0b10 → -1
//   0b11 →  reserved (rejected on decode as corrupt input)
//
// The substrate's encoding produces ternary vectors (deterministic
// rng % 3 in `holon-rs::deterministic_vector_from_seed`; bundle
// ties produce 0); 1-bit-per-dim packing would lose information.
// Total size at d=10000: 4-byte header + 2500 data bytes = 2504
// bytes. The dim header lets the receiver validate "wrong universe
// shape" cleanly (returns :None on dim mismatch with ambient
// encoder).
//
// No universe metadata in the bytes — per DESIGN Q5: the seed is
// the receiver's responsibility to know. V + K + F three-factor
// verification UX.


// ─── Bytes ↔ hex (arc 063) ──────────────────────────────────────────
//
// Carved to `src/intrinsic/bytes.rs` (arc 255.1b-ii) — the two handlers
// (`Bytes::to-hex` / `from-hex`) now wear `#[wat_intrinsic("<fqdn>")]`
// and self-register via inventory. See that module for the impls + the
// text-bridge rationale.

// ─── show — polymorphic value rendering (arc 064) ───────────────────
//
// `:wat::core::show<T>` renders any wat Value to a debug-friendly
// String. Used internally by `assert-eq` to populate the failure
// payload's actual/expected fields; exposed publicly so test code
// and future assertions (assert-not-eq, assert-true, etc.) can reuse.
//
// Per-variant rendering follows wat surface conventions where they
// exist (literal forms for primitives; (Some x) / (Ok x) / (Err x)
// for Option/Result; (vec :T x ...) shape via `[...]` shorthand for
// Vec; quoted-string semantics matching Rust's {:?} for String).
// Compound substrate values (Struct, Enum, HolonAST, Vector,
// channels, ProgramHandle) render as angle-bracketed summaries
// naming the type and key dimensions — full structural dumps are
// out of scope (the cost of pretty-printing a 4096-element ternary
// vector inline is not worth it for diagnostics).
//
// Pretty-print depth is bounded at ~1KB per render via truncation
// guards in the recursive helper; deeply nested structures collapse
// to a `…` marker rather than blowing past a sensible limit.

/// `(:wat::core::show v)` → `:String` (arc 064). Polymorphic
/// renderer; per-variant dispatch via [`render_value`].
///
/// **Purity ground —** the sole arg is evaluated by ordinary call-by-value (`eval_inner`, not
/// itself an effect); past that the body only calls `render_value`, a pure recursive
/// structural formatter that reads the already-evaluated `Value` and writes a `String` — no
/// I/O, no ambient state, no `eval_inner`/`apply_function` on caller-supplied code.
///
/// **Totality ground —** `render_value`'s match is exhaustive over every `Value` variant —
/// primitives render literally, compound substrate values fall to the angle-bracketed
/// summary arm — and its depth/length truncation guards (`SHOW_MAX_DEPTH`/`SHOW_MAX_LEN`,
/// this module) collapse runaway recursion to a `…` marker rather than raising. No variant
/// lacks an arm and no failure path exists.
///
/// **Expand-time ground —** on `macros/eval.rs`'s `is_expand_time_legal` residue list today
/// (the "value/control-flow ops" group names `show` explicitly), so it is legal inside a
/// macro body today; registering it here REPLACES that residue entry, so it must declare the
/// SAME verdict — `Legal` — or the registration silently revokes today's legality (arc 255
/// the `fn` lesson). This is also the ledger the DESIGN predicts: `show` sits in
/// `rete/purity.rs`'s `KNOWN_UNREVIEWED` today, and registering it here gives
/// `intrinsic_meta` a classification, which should trip that gate to demand the line's
/// deletion.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     args :T the value to render
/// @ret     :wat::core::String a debug-friendly rendering of `args` — quoted for `String`, literal for other primitives, a bounded angle-bracketed summary for compound values
/// @example (:wat::core::show 42) #=> "42"
#[wat_intrinsic(":wat::core::show")]
fn eval_show(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::show";
    if args.len() != 1 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // ⛔ `show` is a SUMMARIZER, not a renderer — do NOT route it through the EDN
    // encoder. It annotates with the type and ELIDES the payload: `<Vector dim=1024>`
    // (never 1024 floats), `<Duration 86400000000000ns>`, `<HolonAST>`, `<WatAST>`.
    // That bound is why `ValueSnapshot::of` shares it — an error message must not
    // inline a whole VSA vector. Stone 279.2 tried the swap on the theory that `show`
    // was `str`-with-quotes; the floor refuted it with 27 failures naming the elisions
    // (`show must render a compact dim summary, not raw values`). `str` is the FULL
    // rendering; `show` is the BOUNDED one. They are two jobs, not one with a flag.
    Ok(Value::String(Arc::new(
        crate::value::observe::render_value(&v, 0),
    )))
}

// ─── str — unquoted display (arc 279) ─────────────────────────────────────
//
// `:wat::core::str<T>` renders any primitive value to a String UNQUOTED:
//   String  → the string itself (no surrounding `"..."` — unlike `show`)
//   i64     → decimal digits
//   f64     → decimal representation
//   bool    → `true` / `false`
//
// Used by the `format` macro (arc 279): the macro emits
//   `(:wat::core::str <val>)` for each `{name}` placeholder so the
// substituted value fills as itself, not as its EDN representation.
// "Does a macro need it?" → YES: `format` needs a polymorphic unquoted
// renderer at runtime to display the substituted value. Added as an
// intrinsic (not a defn) because the macro emits it directly.

/// `(:wat::core::str v)` → `:String` (arc 279). Unquoted polymorphic
/// renderer: String→itself, i64→digits, f64→decimal, bool→true/false.
/// Distinct from `show` which wraps strings in `"..."`.
fn eval_str(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::str";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // 296 / 279.2 fix — pass the registry so a record renders by NAME. Before this,
    // `(str <record>)` answered `#user/Pt {:field-0 1 :field-1 2}` while `println` of
    // the same value answered `#user/Pt {:x 1 :y 2}`: one value, two faces. Now shared
    // via `render_str_total` (279.3) so `join`'s per-element render cannot drift from
    // this one.
    let s = crate::string::render_str_total(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(s)))
}


// ─── Function application ───────────────────────────────────────────────

/// Arc 233 Stone 233.2.k — apply a TrackedValue callee, preserving provenance
/// in NotCallable errors. Used by eval_list Symbol + List head paths where the
/// callee is looked up or evaluated as a TrackedValue (so producer info flows
/// to the error site intact via ValueSnapshot::of_tracked).
fn apply_tracked_callee(
    callee_tv: TrackedValue,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    let func = match callee_tv.value() {
        Value::wat__core__fn(f) => f.clone(),
        _ => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::NotCallable {
                    got: Box::new(ValueSnapshot::of_tracked(&callee_tv)),
                },
            )
            .into())
        }
    };
    let vals = args
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    apply_function(func, vals, sym, crate::rust_caller_span!())
        .map(TrackedValue::from)
        .map_err(EvalBreak::from)
}

fn apply_value(
    callee: &Value,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let func = match callee {
        Value::wat__core__fn(f) => f.clone(),
        other => {
            // arc 138: no span — apply_value receives a Value not a WatAST; callee span not in scope
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::NotCallable {
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let vals = args
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    apply_function(func, vals, sym, crate::rust_caller_span!()).map_err(EvalBreak::from)
}

/// Apply a function to a list of argument values, evaluated under the
/// given symbol table. Arity must match the function's declared
/// parameters; mismatch returns [`RuntimeError::ArityMismatch`].
///
/// Public so the freeze module's `:user::main` invocation and
/// constrained-eval paths can apply pre-registered functions from a
/// frozen world without duplicating the param-binding logic.
///
/// ## Tail-call trampoline (TCO, Stage 1 — named defines)
///
/// The body runs inside a loop that catches
/// [`EvalSignal::TailCall`] (wrapped in [`EvalBreak::Signal`]). When `eval_tail` recognizes a
/// user-defined function call in tail position it emits `TailCall`
/// carrying the next function and its already-evaluated args; this
/// loop reassigns `cur_func`/`cur_args` and re-iterates without
/// recursing. Rust stack stays constant across arbitrary
/// tail-recursion depth (`Console/loop`, `Cache/loop-step`, any
/// `gen_server`-shaped driver). See
/// `docs/arc/2026/04/003-tail-call-optimization/DESIGN.md` for the
/// full treatment.
///
/// Fn self-tail-calls still consume stack in Stage 1 — the
/// evaluator's user-function-call detection keys on
/// `sym.functions`, which holds named defines only. A fn body
/// that tail-calls a *named* define IS covered: the signal fires
/// at the named call, this loop catches it exactly as it does for
/// a define calling itself. Stage 2 extends detection to
/// fn-valued calls.
pub fn apply_function(
    func: Arc<Function>,
    args: Vec<Value>,
    sym: &SymbolTable,
    call_span: Span,
) -> Result<Value, RuntimeError> {
    let mut cur_func = func;
    let mut cur_args = args;
    let mut cur_span = call_span;

    // Arc 016 slice 2: push a frame onto the wat call stack for this
    // invocation. The guard pops on drop — any exit path (Ok, Err,
    // panic) cleans up the frame. Tail calls REPLACE the top frame
    // in place (the current call is substituted by the next callee
    // at the same stack depth), matching what a user reads as
    // "recursion without stack growth."
    // Stone 255.1a — helper: display name for a function without a registered name.
    // Wat fns use their body span; Native builtins should never reach fn-apply
    // without a name (they are always named keywords in sym.functions).
    let fn_display_name = |f: &Function| -> String {
        match &f.body {
            // Arc 109 — an anonymous fn's identity is the structured
            // ANON_FN_SYMBOL marker (becomes the Frame's `symbol`), NOT a
            // `<fn@span>` stringy costume; the location travels structurally via
            // the Frame's file/line (the call_span) / `:at`, not the name.
            FunctionBody::Wat(_) => crate::value::ANON_FN_SYMBOL.to_string(),
            FunctionBody::Native => "<native>".to_string(),
        }
    };
    let callee_name_initial = match cur_func.name.clone() {
        Some(name) => name,
        None => fn_display_name(&cur_func),
    };
    let _frame_guard = FrameGuard::push(callee_name_initial, cur_span.clone());

    loop {
        let fixed_arity = cur_func.params.len();
        let actual_arity = cur_args.len();
        // Arc 150 — variadic arity: when the callee has a rest-param,
        // accept `actual_arity >= fixed_arity`. The first N args bind
        // positionally; the remainder collect into a Value::Vec bound
        // to `rest_param`. Strict-arity behavior is preserved when
        // `rest_param.is_none()`. Mirrors `expand_macro_call`'s
        // arity check (`src/macros.rs:558-580`).
        match cur_func.rest_param {
            None => {
                if actual_arity != fixed_arity {
                    return Err(RuntimeError::new(
                        cur_span.clone(),
                        RuntimeErrorKind::ArityMismatch {
                            op: match cur_func.name.clone() {
                                Some(name) => name,
                                None => fn_display_name(&cur_func),
                            },
                            expected: fixed_arity,
                            got: actual_arity,
                        },
                    ));
                }
            }
            Some(_) => {
                if actual_arity < fixed_arity {
                    return Err(RuntimeError::new(
                        cur_span.clone(),
                        RuntimeErrorKind::ArityMismatch {
                            op: match cur_func.name.clone() {
                                Some(name) => name,
                                None => fn_display_name(&cur_func),
                            },
                            expected: fixed_arity,
                            got: actual_arity,
                        },
                    ));
                }
            }
        }
        // Build the call env: parent is the closed env (fn) or a
        // fresh root (define — the body resolves global names via sym).
        let parent = cur_func.closed_env.clone().unwrap_or_default();
        let mut builder = parent.child();
        // Bind the first `fixed_arity` args positionally to `params`.
        // `cur_args.drain(..fixed_arity)` empties the front of the Vec
        // and leaves the rest-args (if any) at indices 0..N for the
        // rest-binding pass below.
        let mut drained = cur_args.drain(..);
        for name in cur_func.params.iter() {
            let value = drained.next().expect("arity checked above");
            builder = builder.bind_unknown_span(
                crate::scope::env_key(name).into_owned(),
                TrackedValue::from(value),
            );
        }
        // Arc 150 — collect the remaining args (post-fixed) into a
        // Value::Vec and bind to the rest-name. For zero rest-args the
        // binding is an empty Vec; tests rely on this for the
        // zero-rest-args coverage row.
        if let Some(rest_name) = &cur_func.rest_param {
            let rest: Vec<Value> = drained.collect();
            builder = builder.bind_unknown_span(
                rest_name.clone(),
                TrackedValue::from(Value::Vec(Arc::new(rest))),
            );
        } else {
            // Drop the iterator so cur_args is fully drained even on
            // the strict-arity path. (drain runs to completion on
            // drop; explicit drop here makes the lifecycle obvious.)
            drop(drained);
        }
        let call_env = builder.build();
        // Evaluate the body in tail position. `eval_tail` is the
        // tail-aware sibling of `eval`; it emits `EvalBreak::Signal(EvalSignal::TailCall)`
        // when it meets a user-defined function call at the tail — the
        // match below converts that signal into loop continuation.
        //
        // `TryPropagate` keeps its legacy behavior: wrap in the
        // function's own `Err(e)` return. The type checker guarantees
        // this function's declared return type is `(Result :- [_ E])`
        // whenever its body contains a `try`, so the wrap is
        // type-correct by construction.
        // Stone 255.1a — Native builtins are intercepted by the runtime dispatch match
        // before reaching apply_function; if a Native body somehow arrives here it is a bug.
        let body_ast = match &cur_func.body {
            FunctionBody::Wat(ast) => ast,
            FunctionBody::Native => unreachable!(
                "native builtin fn-applied — dispatched via the runtime match, not fn-apply"
            ),
        };
        match eval_tail(body_ast, &call_env, sym) {
            Ok(v) => return Ok(v),
            Err(EvalBreak::Signal(EvalSignal::TailCall {
                func: next,
                args: next_args,
                call_span: next_span,
            })) => {
                cur_func = next;
                cur_args = next_args;
                cur_span = next_span;
                // Replace the top frame with the new callee's info —
                // tail calls don't deepen the stack; they substitute.
                let next_name = match cur_func.name.clone() {
                    Some(name) => name,
                    None => fn_display_name(&cur_func),
                };
                replace_top_frame(next_name, cur_span.clone());
                continue;
            }
            Err(EvalBreak::Signal(EvalSignal::TryPropagate(e))) => {
                return Ok(Value::Result(Arc::new(Err(*e))));
            }
            Err(EvalBreak::Signal(EvalSignal::OptionPropagate)) => {
                return Ok(Value::Option(Arc::new(None)));
            }
            Err(EvalBreak::Diagnostic(other)) => return Err(*other),
        }
    }
}

// ─── Wat call-stack for failure-diagnosis (arc 016 slice 2) ─────────
//
// Moved to crate::value::frame (Stone 251.2a). Re-exported below for
// in-module use.

use crate::value::{replace_top_frame, FrameGuard};
use crate::value::FrameInfo;

// ─── Seven eval forms ────────────────────────────────────────────────────
//
// Mirror of the six load forms, with one extra on the AST side. Arc 028
// dropped the `:wat::eval::<iface>` interface keyword — each form takes
// its source (AST, inline string, or path) directly:
//
//   (:wat::eval-ast!  <Value::Ast>)
//   (:wat::eval-edn!  <source>)
//   (:wat::eval-file! <path>)
//   (:wat::eval-digest!        <path>   :wat::verify::digest-<algo>
//                                       :wat::verify::<iface> <payload>)
//   (:wat::eval-digest-string! <source> :wat::verify::digest-<algo>
//                                       :wat::verify::<iface> <payload>)
//   (:wat::eval-signed!        <path>   :wat::verify::signed-<algo>
//                                       :wat::verify::<iface> <sig>
//                                       :wat::verify::<iface> <pubkey>)
//   (:wat::eval-signed-string! <source> :wat::verify::signed-<algo>
//                                       :wat::verify::<iface> <sig>
//                                       :wat::verify::<iface> <pubkey>)
//
// `eval-ast!` takes a value that IS a parsed AST (already past any trust
// boundary); the others take EDN source text (inline or via path) with
// optional byte-level (digest) or meaning-level (signed) verification.
//
// The mutation-form refusal (FOUNDATION line 663) runs inside every
// path: an AST that contains a `define` / `defmacro` / `struct` / etc.
// is rejected before anything executes.

// ─── Kernel primitives: stopped / send / recv ───────────────────────────

// Arc 109 Stone B — the seven kernel sub-modules — `eval_kernel_stopped` moved to
// `src/kernel/ambient.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_kernel_call_site` moved to
// `src/kernel/source.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_kernel_macro_call_site` moved to
// `src/kernel/source.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Returns the host parallelism as `i64` — `std::thread::available_parallelism()`,
/// falling back to 1 if the OS refuses to report.
///
/// Single source of truth used by:
/// - `(:wat::program::cpu-count)` — the live nullary verb (Arc 259 S3.2b-i)
/// - `src/freeze.rs` — the program-env seam constructor
/// - `src/kernel/spawn.rs` — the thread-peer env constructor
pub(crate) fn host_cpu_count() -> i64 {
    std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1)
}

// `eval_program_env` that used to live here (`:wat::program::env`) moved to
// `src/intrinsic/program.rs` — arc 255 Stone P6-c-W2, the P6-c campaign's second wave. It
// was declaring a variadic `&[WatAST]` used only to reject (a hand-rolled length check);
// homing it meant declaring the real arity (0) so `#[wat_intrinsic]`'s generated shim owns
// the check.

/// `(:wat::program::self-peer :S :R)` — returns the calling thread's self-peer
/// (the spawned process child's owner-link as a unified `(Peer' :- [S R])`).
///
/// Arc 209 C0b.3a-0 / C0b.2e-i-b. The self-peer is installed into the `SELF_PEER`
/// thread-local by `install_self_peer` at the child-only seam
/// `run_forms_as_server_child` (process/verbs.rs), before `:user::main` runs.
/// Root never calls that seam → root gets a clean MalformedForm error.
///
/// The two type-keyword args (:S :R) are checker-only; they are validated to
/// be keywords but not evaluated.  The runtime value is a boxed `Peer` (socket
/// tier) under `PEER_TYPE_PATH`.
fn eval_program_self_peer(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::program::self-peer";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    for (i, a) in args.iter().enumerate() {
        if !is_type_arg_shaped(a) {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "argument {} must be a type keyword (e.g. :wat::core::i64)",
                        i
                    ),
                },
            )
            .into());
        }
    }
    crate::services::current_self_peer().ok_or_else(|| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "no self-peer — (:wat::program::self-peer) is only valid inside a spawned \
                     process service; root has no owner-link"
                    .into(),
            },
        )
        .into()
    })
}

/// `(:wat::program::cpu-count)` — nullary; returns the host parallelism as
/// `:wat::core::i64` via [`host_cpu_count`].
///
/// Arc 259 S3.2b-i. Unlike the stamped `cpu-count` env field (reachable only
/// via `(:wat::program::env)` when a program env is installed), this verb answers
/// `std::thread::available_parallelism()` directly — no installed program env
/// required. Mirrors `(:wat::time::now)`: a live host fact available in ANY eval
/// context. Used by the brackets pool to size its default runner count.
///
/// Homed to `#[wat_intrinsic]` arc 255 Stone P6-c-1 (the second proof verb — the SUBSET
/// shape: this handler declares ONLY `list_span` in its context tail, no `env`/`sym`).
/// The leading `args: &[WatAST]` param predates the later true-nullary convention
/// (contrast `:wat::time::now`'s zero-param shape) — kept exactly as declared (this
/// stone's whole claim is that homing a verb never edits its parameter list), so the
/// shim still forwards the whole slice and this fn keeps its own arity check below.
/// `check.rs` already carries a hand-registered generic `TypeScheme` for this FQDN
/// (`(:wat::program::cpu-count) -> :wat::core::i64`, near line 18703) — untouched by
/// this move, same as every other pre-existing intrinsic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Ambient
/// @arg     args… :wat::core::Value must be empty — this verb takes no wat-level arguments
/// @ret     :wat::core::i64 the host's available parallelism (`std::thread::available_parallelism()`), sampled at call time
/// @example-norun (:wat::program::cpu-count) #=> 8
#[wat_intrinsic(":wat::program::cpu-count")]
fn eval_program_cpu_count(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::program::cpu-count";
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    Ok(Value::i64(host_cpu_count()))
}

/// `(:wat::runtime::argv)` — nullary; returns the process-wide argv ambient as `(:wat::core::Vector
/// :- [:wat::core::String])`.
///
/// Arc 170 slice 1e (REALIZATIONS pass 7). The four-arg `:user::main` shape
/// (stdin/stdout/stderr/argv) retires; argv moves to ambient. Wat-cli (or any embedder) calls
/// [`crate::runtime::set_argv`] before `:user::main` runs, committing `ARGV` (a `OnceLock`) once;
/// every subsequent read, from any depth in the program, for the rest of THIS run returns the
/// identical `Vec` — the same install-once/read-many shape `:wat::program::env`'s ambient
/// thread-local has, not a live per-call OS query (contrast `:wat::runtime::current-thread`,
/// below, which IS a live per-call query). Empty Vec when no embedder set it (in-process tests,
/// library bridges that bypass wat-cli) — the ambient is "always available."
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Ambient
/// @ret     (:wat::core::Vector :- [:wat::core::String]) the process argv, fixed for this run's duration (empty if never set)
/// @example (:wat::core::= (:wat::runtime::argv) (:wat::runtime::argv)) #=> true
/// @see     :wat::program::env
#[wat_intrinsic(":wat::runtime::argv")]
fn eval_runtime_argv() -> Result<Value, EvalBreak> {
    let argv = argv();
    let values: Vec<Value> = argv
        .iter()
        .map(|s| Value::String(Arc::new(s.clone())))
        .collect();
    Ok(Value::Vec(Arc::new(values)))
}

/// `(:wat::runtime::current-thread)` — nullary; returns the calling thread's id as
/// `:wat::core::String`.
///
/// Arc 170 slice 1e (REALIZATIONS pass 7). For slice 1e this is the main thread's representation;
/// slice 1g extends to spawned threads via thread-locals populated at spawn-time. Implemented
/// against `std::thread::current().id()` directly — a LIVE query, not read from an install-once
/// cell — which is meaningful on every thread the substrate creates today.
///
/// ★ Purity grounding (arc 255 Stone P6-c-W3, per the brief's explicit call-out): unlike the eight
/// symbol-table/AST readers in this wave, this verb takes no wat-level args and answers
/// `std::thread::current().id()` fresh on every call — the SAME live-host-fact shape
/// `:wat::program::cpu-count` was ruled on (`@Determinism Nondeterministic`, "a live host fact...
/// not a committed value"), for the identical reason: the answer depends on WHICH thread is
/// calling, an ambient fact no argument captures, not on a value fixed once and read many times
/// (contrast `argv`, above, which IS install-once via a `OnceLock`). No side effect is possible
/// (`@Purity Pure`), but the result is not a deterministic function of the call's arguments.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Ambient
/// @ret     :wat::core::String the calling thread's id, `{:?}`-formatted
/// @example-norun (:wat::runtime::current-thread) #=> "ThreadId(1)"
/// @see     :wat::program::cpu-count
#[wat_intrinsic(":wat::runtime::current-thread")]
fn eval_runtime_current_thread() -> Result<Value, EvalBreak> {
    let id = std::thread::current().id();
    Ok(Value::String(Arc::new(format!("{:?}", id))))
}

// Arc 109 Stone B — the seven kernel sub-modules — `eval_user_signal_query` moved to
// `src/kernel/ambient.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_user_signal_reset` moved to
// `src/kernel/ambient.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// ─── Config accessors ─────────────────────────────────────────────────
//
// Every setter in `:wat::config::set-*!` commits exactly once during
// the startup's config pass. After freeze, the committed value is read
// by its nullary accessor. These have the same discipline as other
// substrate constants — no arguments, deterministic, safe to call from
// any context as long as the SymbolTable carries an EncodingCtx (which
// it does after freeze).

pub(crate) fn require_encoding_ctx<'a>(
    op: &'static str,
    sym: &'a SymbolTable,
    list_span: &Span,
) -> Result<&'a EncodingCtx, EvalBreak> {
    sym.encoding_ctx().map(|arc| arc.as_ref()).ok_or_else(|| {
        EvalBreak::from(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::NoEncodingCtx { op: op.into() },
        ))
    })
}

/// Arc 077: the program runs at one d. Read it from the ambient
/// `EncodingCtx`. Returns `NoEncodingCtx` if no ctx is attached
/// (test harnesses that bypass freeze).
pub(crate) fn program_dim(op: &'static str, sym: &SymbolTable, list_span: &Span) -> Result<usize, EvalBreak> {
    let ctx = require_encoding_ctx(op, sym, list_span)?;
    Ok(ctx.dim_count)
}

// `check_nullary` and the four `eval_config_*` handlers that used to live here
// (`:wat::config::dim-count`/`dim-capacity`/`global-seed`/`noise-floor`) moved to
// `src/intrinsic/config.rs` — arc 255 Stone P6-c-W1, the P6-c campaign's first wave.
// All four were declaring a variadic `&[WatAST]` they used only to reject (a
// hand-rolled `check_nullary` arity guard); homing them meant DELETING that
// fiction and declaring the real arity (0) so `#[wat_intrinsic]`'s generated shim
// owns the check and `metadata-of` reports the true arity instead of a lie.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_listener_prime` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

::wat_source_derive::wat_field_names_from!(
    BOUND_FIELDS,
    "wat/spawn.wat",
    ":wat::spawn::Bound"
);
// Arc 109 Stone B — the seven kernel sub-modules — `bound_names` moved to
// `src/kernel/source.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_connect_prime` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `wrap_connect_request` moved to
// `src/kernel/message.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_accept_prime` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_allow_prime` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_deny_prime` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Shared implementation for the unary stdlib math calls —
/// `:wat::math::ln`, `sin`, `cos`, `exp`, `sqrt` (arc 255 Stone HOME-9 — moved off the dead
/// `:wat::std::` namespace; `log` was deleted here rather than moved, see the dispatch arm's
/// comment). Arity 1. Argument must
/// evaluate to `:f64` (or `:i64` auto-promoted). `op_name` is the
/// wat-facing short name for error messages.
pub(crate) fn eval_math_unary(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    op_name: &str,
    f: fn(f64) -> f64,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: format!(":wat::math::{}", op_name),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let x = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::f64(x) => x,
        Value::i64(n) => n as f64,
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: format!(":wat::math::{}", op_name),
                    expected: "f64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::f64(f(x)))
}

/// `(:wat::math::pi)` — the mathematical constant π as `:f64` (arc 255 Stone HOME-9 — moved
/// off the dead `:wat::std::` namespace).
/// Nullary. Backing: `std::f64::consts::PI`.
pub(crate) fn eval_math_pi(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::math::pi".into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    Ok(Value::f64(std::f64::consts::PI))
}

/// `(:wat::stat::mean (:wat::core::Vector :- [f64])) -> (:wat::core::Option :- [f64])`. Population
/// mean. None on empty input — matches `f64::min-of`/`max-of`'s
/// reduction-empty convention. Arc 255 Stone HOME-9 — moved off the dead `:wat::std::`
/// namespace.
///
/// Surfaced by holon-lab-trading arc 026 slice 9 (Hurst's R/S
/// analysis) and slice 4 (Bollinger's RollingStddev). Universal
/// enough to live in core stdlib.
pub(crate) fn eval_stat_mean(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::stat::mean";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let xs = require_vec(OP, eval_inner(&args[0], env, sym)?.value_owned())?;
    if xs.is_empty() {
        return Ok(Value::Option(Arc::new(None)));
    }
    let mut sum = 0.0;
    for v in xs.iter() {
        match v {
            Value::f64(x) => sum += x,
            other => {
                return Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "(Vector :- [f64])",
                        got: Box::new(ValueSnapshot::of(other)),
                        // arc 138: no — iterating over Vec<Value>; no per-element AST span
                    },
                )
                .into());
            }
        }
    }
    Ok(Value::Option(Arc::new(Some(Value::f64(
        sum / xs.len() as f64,
    )))))
}

/// `(:wat::stat::variance (:wat::core::Vector :- [f64])) -> (:wat::core::Option :- [f64])`. Population
/// variance (divides by n). Matches numpy default `ddof=0`. None on
/// empty input. Single-point input returns `Some(0.0)` (no spread).
/// Arc 255 Stone HOME-9 — moved off the dead `:wat::std::` namespace.
pub(crate) fn eval_stat_variance(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::stat::variance";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let xs = require_vec(OP, eval_inner(&args[0], env, sym)?.value_owned())?;
    if xs.is_empty() {
        return Ok(Value::Option(Arc::new(None)));
    }
    let n = xs.len() as f64;
    let mut sum = 0.0;
    for v in xs.iter() {
        match v {
            Value::f64(x) => sum += x,
            other => {
                return Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "(Vector :- [f64])",
                        got: Box::new(ValueSnapshot::of(other)),
                        // arc 138: no — iterating over Vec<Value>; no per-element AST span
                    },
                )
                .into());
            }
        }
    }
    let mean = sum / n;
    let mut sq = 0.0;
    for v in xs.iter() {
        if let Value::f64(x) = v {
            let dx = x - mean;
            sq += dx * dx;
        }
    }
    Ok(Value::Option(Arc::new(Some(Value::f64(sq / n)))))
}

/// `(:wat::stat::stddev (:wat::core::Vector :- [f64])) -> (:wat::core::Option :- [f64])`. Square
/// root of population variance. Arc 255 Stone HOME-9 — moved off the dead `:wat::std::`
/// namespace.
pub(crate) fn eval_stat_stddev(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::stat::stddev";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    match eval_stat_variance(args, env, sym, list_span)? {
        Value::Option(opt) => match &*opt {
            Some(Value::f64(var)) => Ok(Value::Option(Arc::new(Some(Value::f64(var.sqrt()))))),
            Some(_) => unreachable!("variance returned a non-f64 inside Option"),
            None => Ok(Value::Option(Arc::new(None))),
        },
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Option :- [f64]) from inner variance",
                got: Box::new(ValueSnapshot::of(&other)),
                // arc 138: no — internal variance re-wrap; no originating AST element
            },
        )
        .into()),
    }
}

// Arc 109 Stone B — the seven kernel sub-modules — `eval_handle_pool_new` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_handle_pool_pop` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_handle_pool_finish` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// :wat::kernel::spawn retired in arc 114. The arc-060 SpawnOutcome
// channel pattern lived on inside the arc-114 in-thread satisfier
// (`:wat::kernel::spawn-thread`, itself retired — non-prime IPC
// de-prime, this pass) and `spawn_with_world_into_result` (the
// (Process :- [I O]) in-thread driver from arc 103a). The retired bare-
// spawn impl is gone; the type-checker poisons every call site
// pre-runtime so this layer is unreachable. See arc 114
// INSCRIPTION for the contract retirement. Arc 278's vacate-spawn-
// outcome strike purged the SpawnOutcome/ProgramHandleInner types
// themselves: a locus (`:user::main`) has no meaningful return
// value — it communicates only by channel — so the "spawn a fn,
// get a Value back" chain those types existed to carry had zero
// producers left; its death-reason job was already carried
// structurally by `recv'` -> `Lost[LociDiedError]`.

/// Coerce a `catch_unwind` panic payload to a printable String —
/// same shape Rust's default panic hook does, plus an
/// `AssertionPayload` arm so substrate-raised assertion failures
/// (the structured panic shape `:wat::kernel::assertion-failed!`
/// uses) surface their full message instead of falling through to
/// the generic non-string marker.
///
/// Order tried: `&str` (literal `panic!("...")`); `String`
/// (formatted `panic!("{}", ...)`); `AssertionPayload` (substrate's
/// structured assertion shape); fallback marker.
///
/// Borrows the payload — caller retains ownership for further
/// inspection. `extract_panic_payload` (below) is the
/// owning sibling that takes the box and recovers the
/// AssertionPayload's structured fields when present.
pub(crate) fn format_panic_payload(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        return (*s).to_string();
    }
    if let Some(s) = payload.downcast_ref::<String>() {
        return s.clone();
    }
    if let Some(p) = payload.downcast_ref::<crate::assertion::AssertionPayload>() {
        return p.message.clone();
    }
    "panic with non-string payload".to_string()
}

// Arc 109 Stone B — the seven kernel sub-modules — `extract_panic_payload` moved to
// the existing `src/kernel/spawn.rs` (its sole caller in the tree; no new module
// for it) (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// :wat::kernel::join and :wat::kernel::join-result retired in arc
// 114. Thread/join-result is the canonical replacement;
// Process/join-result handles the forked-program case. The bare
// verbs assumed an :R return through the spawn channel — arc 114
// retires that contract; programs deliver values only through
// their output pipe. Type-checker poison ensures no caller reaches
// this layer.

// Arc 109 Stone 4a — the kernel error vocabulary — `thread_died_error_panic` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Convert an [`AssertionPayload`] into a `:wat::kernel::Failure`
/// `Value::Aggregate(Record)`. Field order mirrors the type registration:
/// `(error, frames, actual, expected)`.
/// Arc 293.W.2b — Failure is now Nature::Record (pure EDN data; all fields pure).
///
/// Arc 278 the string-wrap annihilation — the mandatory `error` field carries
/// the raised `:wat::core::Error` STRUCTURALLY. When the payload came from
/// `raise!` (`raised_error = Some(e)`), that error value rides directly. Every
/// other panic (assert-* failures, `expect`, plain panics) has no structured
/// error, so a `:wat::core::Fault` is SYNTHESIZED from the payload's `message`
/// + `location` — honest (a panic IS an error with that message), not fabrication.
pub(crate) fn failure_value_from_assertion_payload(p: crate::assertion::AssertionPayload) -> Value {
    let crate::assertion::AssertionPayload {
        message,
        actual,
        expected,
        location,
        frames,
        // Arc 113 — chain rides on the panic payload but doesn't
        // become part of the Failure record (Failure pre-dates the
        // chain; cascade reconstruction lives one layer up, in
        // join-result).
        upstream_chain: _,
        // Arc 138 F-NAMES-1d — thread_name is for the panic hook render
        // only; it doesn't map to a Failure record field.
        thread_name: _,
        raised_error,
    } = p;
    // The mandatory structured cause: the raised Error verbatim, or a Fault
    // synthesized from the bare message + location.
    let error_field = match raised_error {
        Some(e) => e,
        None => fault_value(message, location),
    };
    let frames_field = Value::Vec(Arc::new(
        frames
            .into_iter()
            .map(value_from_frame_info)
            .collect::<Vec<_>>(),
    ));
    let actual_field = match actual {
        Some(s) => Value::Option(Arc::new(Some(Value::String(Arc::new(s))))),
        None => Value::Option(Arc::new(None)),
    };
    let expected_field = match expected {
        Some(s) => Value::Option(Arc::new(Some(Value::String(Arc::new(s))))),
        None => Value::Option(Arc::new(None)),
    };
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::Failure".into(),
        failure_names(),
        Arc::new(vec![
            error_field,
            frames_field,
            actual_field,
            expected_field,
        ]),
    )))
}

::wat_source_derive::wat_field_names_from!(
    FAILURE_FIELDS,
    "wat/kernel/diagnostics.wat",
    ":wat::kernel::Failure"
);
pub(crate) fn failure_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(FAILURE_FIELDS))
        .clone()
}

/// Arc 278 — build a `:wat::core::Fault` `Value::Aggregate(Record)` from a
/// human message + an optional source location. Field order matches the
/// `:wat::core::Fault` registration (core.wat): `(message, location, causes)`.
/// `location` is a MANDATORY `:wat::kernel::Location` (not `Option`); when the
/// panic carried no span (transport/synthetic failures — disconnected, shutdown,
/// service crash), a synthetic `<runtime>` location marks it honestly. `causes`
/// is an empty `(Vector :- [Error])`. This is the canonical synthesizer for every
/// death that is a bare message rather than a structured `raise!`.
fn fault_value(message: String, location: Option<crate::span::Span>) -> Value {
    let location_value = match location {
        Some(span) => value_from_span(span),
        None => value_from_span(crate::span::Span::new(
            Arc::new("<runtime>".to_string()),
            0,
            0,
        )),
    };
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::core::Fault".into(),
        fault_names(),
        Arc::new(vec![
            Value::String(Arc::new(message)),
            location_value,
            Value::Vec(Arc::new(Vec::new())), // causes: empty Vector<Error>
        ]),
    )))
}

/// Arc 278 — read a named field off a record `Value` using the `TypeEnv` to
/// resolve the field's positional index (records store fields positionally;
/// names live only in the registered `AggregateDef`). Returns `None` when the
/// value is not an aggregate, its type is unregistered, or the field is absent.
/// Backs the DERIVED `:wat::kernel::Failure/message` / `Failure/location`
/// accessors (read `error.message` / `error.location`) and `raise!`'s human-message
/// extraction — the same field-by-name lookup `keyword_accessor_record` performs.
pub(crate) fn record_field_by_name(
    v: &Value,
    field: &str,
    types: Option<&crate::types::TypeEnv>,
) -> Option<Value> {
    let Value::Aggregate(agg) = v else {
        return None;
    };
    let types = types?;
    let type_key = format!(":{}", agg.class);
    match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) => {
            let idx = a.field_names().position(|n| n == field)?;
            agg.fields.get(idx).cloned()
        }
        _ => None,
    }
}

/// Convert a `Span` into a `:wat::kernel::Location` `Value::Aggregate(Record)`.
/// Field order: `(file, line, col)`.
/// Arc 293.W.2b — Location is now Nature::Record (pure EDN data).
pub(crate) fn value_from_span(span: crate::span::Span) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::Location".into(),
        location_names(),
        Arc::new(vec![
            Value::String(Arc::new((*span.file).clone())),
            Value::i64(span.line),
            Value::i64(span.col),
        ]),
    )))
}

::wat_source_derive::wat_field_names_from!(
    LOCATION_FIELDS,
    "wat/core.wat",
    ":wat::kernel::Location"
);
pub(crate) fn location_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(LOCATION_FIELDS))
        .clone()
}

/// Convert a `FrameInfo` (wat call-stack frame from the trampoline)
/// into a `:wat::kernel::Frame` `Value::Aggregate(Record)`. Field order matches
/// the arc 016 type registration: `(file, line, symbol)`. The
/// callee path becomes the `symbol` field.
/// Arc 293.W.2b — Frame is now Nature::Record (pure EDN data).
/// Arc 109 — Frame's fields are concrete (non-`Option`): a `FrameInfo` always
/// carries a real span (file/line) and a real callee path (symbol).
pub(crate) fn value_from_frame_info(frame: FrameInfo) -> Value {
    let FrameInfo {
        callee_path,
        call_span,
    } = frame;
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::Frame".into(),
        frame_names(),
        Arc::new(vec![
            Value::String(Arc::new((*call_span.file).clone())),
            Value::i64(call_span.line),
            Value::String(Arc::new(callee_path)),
        ]),
    )))
}

::wat_source_derive::wat_field_names_from!(
    FRAME_FIELDS,
    "wat/kernel/diagnostics.wat",
    ":wat::kernel::Frame"
);
fn frame_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(FRAME_FIELDS))
        .clone()
}

// Arc 109 Stone 4a — the kernel error vocabulary — `thread_died_error_runtime` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `thread_died_error_shutdown` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `eval_failure_message` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `eval_failure_location` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `failure_error_field` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `single_died_chain` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `thread_crash_panic_edn` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `thread_crash_runtime_edn` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `conj_died_chain` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `conj_died_chain_value` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_panic` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_panic_value` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_runtime` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_runtime_value` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_main_signature` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_main_signature_value`
// moved to `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_bad_return` moved to
// `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4b — the process died-error vocabulary — `process_died_error_bad_return_value`
// moved to `src/process/died.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `died_error_payload_message` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `eval_died_error_message` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 170 CULMINATION (arc 278 IPC de-prime) — `eval_kernel_extract_panics`
// (the wat verb `:wat::kernel::extract-panics`) ANNIHILATED with the
// run-sandboxed family. It walked a manual sandbox driver's captured
// stderr lines to recover the LociDiedError chain; the primed peer wire
// delivers that chain directly via recv' Lost, so the stderr-scrape
// reader is dead. The `edn_is_loci_died_chain` helper below survives
// (still used by the recv' Lost EDN decoder).

// Arc 109 Stone 4a — the kernel error vocabulary — `edn_is_loci_died_chain` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `eval_died_error_to_failure` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Build a `:wat::kernel::Failure` `Value::Aggregate(Record)` carrying a
/// SYNTHESIZED `:wat::core::Fault` (from `message`, `<runtime>` location, empty
/// causes) as its mandatory `error` field; actual / expected are `:wat::core::None`,
/// frames is empty `Vec<Frame>`.
/// Arc 293.W.2b — Failure is now Nature::Record (pure EDN data).
/// Arc 278 the string-wrap annihilation — the death carries its cause STRUCTURALLY
/// (a Fault), not a bare String; `(:wat::kernel::Failure/message f)` derives back
/// to `fault.message`. Mirrors the wat-side `:wat::kernel::message-only-failure`.
pub(crate) fn message_only_failure(message: String) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::Failure".into(),
        failure_names(),
        Arc::new(vec![
            fault_value(message, None),       // error (synthesized Fault)
            Value::Vec(Arc::new(Vec::new())), // frames
            Value::Option(Arc::new(None)),    // actual
            Value::Option(Arc::new(None)),    // expected
        ]),
    )))
}

// Arc 109 Stone A — the kernel outcome vocabulary — `RECV_OUTCOME_TYPE` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `recv_outcome_message` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `recv_outcome_closed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `recv_outcome_lost` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `recv_outcome_shutdown` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `loci_died_error_from_reason` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `recv_outcome_from_decoded` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `SEND_OUTCOME_TYPE` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `send_outcome_sent` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `send_outcome_closed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `loci_died_disconnected` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone 4a — the kernel error vocabulary — `loci_died_from_send_error` moved to
// `src/kernel/error.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `send_outcome_stopped` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `send_outcome_from_error` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `send_outcome_lost` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `TRY_SEND_OUTCOME_TYPE` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `try_send_outcome_sent` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `try_send_outcome_would_block` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `try_send_outcome_closed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `try_send_outcome_lost` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `CLOSE_OUTCOME_TYPE` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `close_outcome_closed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `close_outcome_signaled` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `close_outcome_failed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `SIGNAL_TYPE` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `SIGNAL_OUTCOME_TYPE` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `signal_outcome_delivered` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `signal_outcome_failed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `ACCEPT_OUTCOME_TYPE` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `accept_outcome_accepted` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `accept_outcome_closed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `accept_outcome_failed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `CONNECT_OUTCOME_TYPE` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `connect_outcome_connected` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `connect_outcome_refused` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `connect_outcome_rejected` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone A — the kernel outcome vocabulary — `connect_outcome_failed` moved to
// `src/kernel/outcome.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// Map a [`RuntimeError`] to an [`EvalError`] struct value — the
/// Err payload returned by the eval-family forms on any failure
/// that isn't a control-flow signal.
///
/// Matches struct-field order `(kind, message)` from
/// [`crate::types::TypeEnv::with_builtins`]'s registration of
/// `:wat::core::EvalError`.
fn runtime_error_to_eval_error_value(err: &RuntimeError) -> Value {
    let (kind, message): (&'static str, String) = match err.kind() {
        RuntimeErrorKind::EvalVerificationFailed { err } => {
            ("verification-failed", format!("{}", err))
        }
        RuntimeErrorKind::EvalForbidsMutationForm { head, .. } => (
            "mutation-form-refused",
            format!("eval refused mutation form: {}", head),
        ),
        RuntimeErrorKind::UnknownFunction(path) => {
            ("unknown-function", format!("unknown function: {}", path))
        }
        RuntimeErrorKind::UnboundSymbol(name) => {
            ("unbound-symbol", format!("unbound symbol: {}", name))
        }
        RuntimeErrorKind::TypeMismatch {
            op, expected, got, ..
        } => (
            "type-mismatch",
            format!("{}: expected {}, got {}", op, expected, got),
        ),
        RuntimeErrorKind::ArityMismatch {
            op, expected, got, ..
        } => (
            "arity-mismatch",
            format!("{}: expected {} args, got {}", op, expected, got),
        ),
        RuntimeErrorKind::ChannelDisconnected { op, .. } => (
            "channel-disconnected",
            format!("{}: channel disconnected", op),
        ),
        RuntimeErrorKind::BadCondition { got, .. } => (
            "bad-condition",
            format!("if/when condition not :bool; got {}", got),
        ),
        RuntimeErrorKind::DivisionByZero => ("division-by-zero", "division by zero".into()),
        RuntimeErrorKind::PatternMatchFailed { value_type, .. } => (
            "pattern-match-failed",
            format!("no match arm fired for {} scrutinee", value_type),
        ),
        RuntimeErrorKind::EffectfulInStep { op, .. } => (
            "effectful-in-step",
            format!("eval-step! refuses effectful op: {}", op),
        ),
        RuntimeErrorKind::NoStepRule { op, .. } => (
            "no-step-rule",
            format!("eval-step! has no rule for op: {}", op),
        ),
        RuntimeErrorKind::MalformedForm { head, reason, .. } => {
            ("malformed-form", format!("{}: {}", head, reason))
        }
        RuntimeErrorKind::NotCallable { got, .. } => {
            ("not-callable", format!("not callable: {}", got))
        }
        // Arc 255 Stone O-iv-a. This arm is NOT cosmetic: without it the variant falls to the
        // wildcard below, whose `format!("{}", err)` renders `RuntimeError`'s Display — the full
        // EDN WIRE FORM, not the prose. So `EvalError/message` would hand a wat program a nested
        // blob for the one diagnostic this stone exists to make READABLE, while every other error
        // on the same path returns a sentence. The rider that struck O-iv-a stayed inside its
        // measured blast radius and reported this rather than widening scope — correctly; the
        // orchestrator then ruled it part of the deliverable, because a stone about an honest
        // message that ships an unreadable one has not shipped.
        RuntimeErrorKind::NotValueDispatchable { name, .. } => (
            "not-value-dispatchable",
            format!(
                "{} is registered, but no handler taking EVALUATED arguments is registered under \
                 that name, and apply dispatches with evaluated arguments. Call it directly.",
                name
            ),
        ),
        // Fallback for variants that don't deserve a dedicated kind.
        _ => ("runtime-error", format!("{}", err)),
    };
    Value::Aggregate(Arc::new(AggregateValue::struct_(
        "wat::core::EvalError".into(),
        eval_error_names(),
        vec![
            Value::String(Arc::new(kind.into())),
            Value::String(Arc::new(message)),
        ],
    )))
}

::wat_source_derive::wat_field_names_from!(
    EVAL_ERROR_FIELDS,
    "wat/core.wat",
    ":wat::core::EvalError"
);
fn eval_error_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(EVAL_ERROR_FIELDS))
        .clone()
}

/// Wrap an inner evaluation's `Result<Value, EvalBreak>` as the
/// `Value::Result` — a `(Result :- [V EvalError])` — the eval-family forms return.
///
/// Preserves `EvalBreak::Signal(TryPropagate)` / `EvalBreak::Signal(OptionPropagate)`
/// so `:wat::core::Result/try` and `:wat::core::Option/try` inside eval'd
/// code still propagate to the calling function. Every diagnostic break
/// becomes `Err(EvalError{...})` as a value. TailCall signals pass through
/// (they originate from within an apply_function and will be caught there).
pub(crate) fn wrap_as_eval_result(inner: Result<Value, EvalBreak>) -> Result<Value, EvalBreak> {
    match inner {
        Ok(v) => Ok(Value::Result(Arc::new(Ok(v)))),
        Err(EvalBreak::Signal(_)) => inner, // pass through all signals
        Err(EvalBreak::Diagnostic(e)) => {
            let err_struct = runtime_error_to_eval_error_value(&e);
            Ok(Value::Result(Arc::new(Err(err_struct))))
        }
    }
}

fn eval_form_ast(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    // Structural pre-check — NOT wrapped as EvalError. This is the
    // caller's syntactic shape; the type checker should have caught
    // it at startup. If it fires at runtime, it's a checker gap or
    // eval-ast! reached from a path that skipped the check (unlikely
    // but possible).
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::eval-ast!".into(),
                reason: format!(
                    "(:wat::eval-ast! <ast-value>) takes exactly 1 argument; got {}",
                    args.len()
                ),
            },
        )
        .into());
    }
    // From here, any RuntimeError (except TryPropagate) becomes an
    // `EvalError` in the Err slot of the returned Value::Result. The
    // value-extraction, mutation-form refusal, the inner eval, and
    // the post-eval HolonAST wrap (arc 066) are all "dynamic
    // evaluation" concerns.
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let value = eval_inner(&args[0], env, sym)?.value_owned();
        let ast = match value {
            Value::wat__WatAST(a) => a,
            other => {
                return Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::eval-ast!".into(),
                        expected: "Ast",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        };
        // Arc 102 — return the bare Value (revert arc 066's
        // `value_to_holon` wrap). The static scheme is now
        // `Result<:T, :EvalError>` polymorphic; T unifies with
        // whatever the caller binds. Callers who want the
        // HolonAST shape annotate `T = :wat::holon::HolonAST`
        // and the runtime's inner Value has to be a HolonAST
        // already (the substrate's static-T trust-the-caller
        // discipline that `:wat::edn::read` / `:wat::eval-edn!`
        // already use).
        run_constrained(&ast, env, sym)
    })())
}

/// Arc 170 — the type path of `(:wat::eval::FormOutcome :- [T])` (registered in `types.rs`).
const FORM_OUTCOME_TYPE: &str = ":wat::eval::FormOutcome";

fn form_outcome(variant: &str, fields: Vec<Value>) -> Value {
    // Arc 296 G′ — `Declared` is a Unit variant (no fields to name); every other
    // `FormOutcome` variant is Tagged with exactly one field, looked up by name.
    let names = if fields.is_empty() {
        no_field_names()
    } else {
        builtin_enum_variant_names(FORM_OUTCOME_TYPE, variant)
    };
    Value::Enum(Arc::new(EnumValue {
        type_path: FORM_OUTCOME_TYPE.into(),
        variant_name: variant.into(),
        names,
        fields,
    }))
}

/// `FormOutcome::CheckFailed [cause <- :wat::core::Error]` — the form did not survive
/// the freeze. A STATIC failure: nothing ran.
///
/// The cause is the freeze error's own `error_edn()` floor record, STRICT-decoded back
/// to a typed value — a navigable `#wat.check/CheckErrors {…}` /
/// `#wat.resolve/UnresolvedReferences {…}` tree with its `:location` and `:causes`
/// intact, NOT that tree `to_string()`'d into a String slot.
///
/// That distinction is the whole point and it has bitten this codebase repeatedly. The
/// FIRST draft of this function did exactly the wrong thing — `e.to_string()` into
/// `:wat::kernel::StartupError`, whose registered shape is a single `message <- String`
/// (`types.rs`) — which re-created the mask `startup_error_chain_edn`
/// (`process/verbs.rs`) had already been fixed to remove, one function away, under a
/// test named `cache_probe_startup_error_is_navigable_edn_not_string`. The builder
/// caught it on sight: *"why is message wrapping a structured edn form?"*
///
/// The root was never the call site — it is that a one-`String` carrier leaves a
/// producer NO honest option, so every new producer re-creates the mask. Arc 296's
/// `DESIGN-296-typed-causes.md` states the cure as a field-TYPE change ("a typed error
/// nested as a typed CAUSE — never `format!`'d away"), and affirmatively defers the
/// same change on ALREADY-REGISTERED wat types (S3/S4) as a breaking change needing its
/// own decision. This variant is newly minted, so it takes the honest carrier from birth
/// and inherits none of that debt.
fn check_failed_cause(e: &crate::freeze::StartupError, sym: &SymbolTable) -> Value {
    use crate::edn::contract::WatError;
    let cause_edn = wat_edn::write(&e.error_edn());
    let types = sym.types().map(|t| &**t);
    let ctx = sym.encoding_ctx().map(|c| &**c);

    // Two decodes, in order of how much the substrate can promise about the result:
    //
    //   1. STRICT — a fully TYPED record, when the diagnostic's tag is registered.
    //   2. FOREIGN — arc 278 Stone A's data mode: an unregistered tag reconstructs as a
    //      self-describing dynamic value instead of raising, recursively, all the way
    //      down. Most freeze diagnostics land here TODAY (`#wat.resolve/…`,
    //      `#wat.check/…` are not registered wat types yet — that is arc 296.3's derive
    //      sweep, `NOTE-pre-world-decode-is-hand-written.md`). The tree is fully
    //      navigable either way; strict just adds nominal typing, so when 296.3 lands
    //      these silently upgrade from (2) to (1) with no change here.
    //
    // The nested diagnostic rides as a CAUSE under a real `Fault`, rather than BEING the
    // returned value, so `:CheckFailed`'s declared `:wat::core::Error` is always
    // satisfied by a genuinely typed record — the dynamic part is contained in the
    // causes chain, which is exactly what a causes chain is for.
    let nested = crate::edn::render::decode_trusted_wire(&cause_edn, types, ctx).or_else(|_| {
        wat_edn::parse_owned(&cause_edn)
            .map_err(|_| ())
            .and_then(|owned| {
                crate::edn::render::edn_to_value_foreign(&owned, types, ctx).map_err(|_| ())
            })
    });

    match nested {
        Ok(inner) => fault_with_cause(
            e.message(),
            crate::span::Span::new(Arc::new("<runtime>".to_string()), 0, 0),
            inner,
        ),
        // A diagnostic whose own EDN neither strict- nor foreign-decodes is itself a
        // defect. Report the headline honestly rather than smuggling the tree back in as
        // prose — a degraded TRUE record beats a complete LYING one.
        Err(()) => fault_value(e.message(), None),
    }
}

fn form_outcome_check_failed(e: &crate::freeze::StartupError, sym: &SymbolTable) -> Value {
    form_outcome("CheckFailed", vec![check_failed_cause(e, sym)])
}

/// A `:wat::core::Fault` carrying one nested structured cause — the shape for "here is
/// what I can say about this failure, and here is the real diagnostic underneath",
/// keeping the nested error walkable instead of folding it into the sentence.
///
/// Arc 109 — `pub(crate)` and location-taking. THE one door for "a decoded diagnostic
/// becomes an `:wat::core::Error`". Three sites run the strict→foreign decode ladder
/// (`check_failed_cause` here, `read_outcome_malformed` and `read_json_outcome_malformed`
/// in `edn/render.rs`); each feeds an enum variant whose cause field is DECLARED
/// `:wat::core::Error`, and the ladder's FOREIGN arm yields a `Value::ForeignRecord` —
/// a dynamic bag that satisfies that surface NOWHERE. Two of the three used to return it
/// directly, making the declared type a lie at the boundary. They route through here now,
/// so the ladder and its disposal cannot drift apart again.
pub(crate) fn fault_with_cause(
    message: String,
    location: crate::span::Span,
    cause: Value,
) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::core::Fault".into(),
        fault_names(),
        Arc::new(vec![
            Value::String(Arc::new(message)),
            value_from_span(location),
            Value::Vec(Arc::new(vec![cause])),
        ]),
    )))
}

/// Arc 170 — `:wat::eval-with-defs!`: evaluate ONE form against a world built from a
/// SUPPLIED definition set, rather than against the caller's ambient symbol table.
///
/// This is the whole of what stood between the substrate and a REPL. `eval-ast!`
/// (above) reaches `run_constrained(ast, env, sym)` where `sym` is `&SymbolTable` —
/// immutable, and not a parameter the caller may supply — so a wat program could hold
/// an accumulated definition set and had no way to run anything IN it. The RED gate is
/// `wat-scripts/scratch-pad/probe-repl-eval-in-gap.wat`.
///
/// # Why it is deliberately SLOW
///
/// It re-derives the ENTIRE world on every call. That is not an oversight — it is the
/// R1/R9 dual-impl discipline: this is the correct-but-slow ORACLE, and a fast
/// incremental data plane (registering one declaration without re-freezing) gets built
/// later, behind a differential against this. Obvious correctness first, because this
/// path IS the ordinary program pipeline: a REPL built on it is exactly as strongly
/// typed as a compiled program, by construction rather than by care.
///
/// # The classification, and why it works this way
///
/// The caller cannot pre-classify the line, and neither can an error: `defn` and
/// `defrecord` fail eval with `unknown-function`, byte-identical to a TYPO (measured,
/// `wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat`). Both are macros with
/// no runtime verb to find. So classification happens on the POST-EXPANSION residue the
/// freeze produces, against `is_runtime_declaration_head` — which means a user's OWN
/// macro that expands to a `def` classifies correctly with no special knowledge of it.
///
/// Two freezes, not one: the baseline (`defs` alone) is what tells us which residue
/// forms THIS line contributed. Halving that is a data-plane concern, not an oracle's.
///
/// The live `Environment` is threaded through UNCHANGED. That is the whole reason an
/// impure binding — a bound service peer — survives a re-freeze: `run_constrained`
/// already takes `env` separately from `sym`, and a peer value holds no reference back
/// into the symbol table. The durable half is a function of the forms; the ephemeral
/// half is simply never rebuilt.
fn eval_form_with_defs(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    // Structural pre-check — NOT a FormOutcome. Same discipline as eval-ast!: the
    // caller's syntactic shape is the type checker's business, not a runtime outcome.
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::eval-with-defs!".into(),
                reason: format!(
                    "(:wat::eval-with-defs! <form> <defs>) takes exactly 2 arguments; got {}",
                    args.len()
                ),
            },
        )
        .into());
    }

    let form = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::wat__WatAST(a) => a,
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::eval-with-defs!".into(),
                    expected: "Ast",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let defs: Vec<WatAST> = match eval_inner(&args[1], env, sym)?.value_owned() {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(a) => out.push((**a).clone()),
                    other => {
                        return Err(RuntimeError::new(
                            args[1].span().clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: ":wat::eval-with-defs!".into(),
                                expected: "Vector of Ast",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into());
                    }
                }
            }
            out
        }
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::eval-with-defs!".into(),
                    expected: "Vector of Ast",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    Ok(eval_form_against_defs(&form, defs, env, sym)?.0)
}

/// The turn itself, with the wat-verb's argument handling stripped off — see
/// [`eval_form_with_defs`] above for the full rationale, which applies unchanged.
///
/// It is split out because the verb is not the only caller. `wat --mcp` drives the same
/// read/eval/print/loop from Rust (`distribution/mcp.rs`): a JSON-RPC frame is not EDN, so
/// the loop cannot live on wat's stdio channels (R51 — a wat `println` EDN-encodes what it
/// is handed, which would deliver an escaped string literal to the harness instead of a
/// JSON object). The SEMANTICS must not fork with the transport, so both modes call this
/// one function and the classification lives in exactly one place.
///
/// The outcome comes back as the wat `FormOutcome` enum value rather than a Rust enum. That
/// is deliberate for now: the wat value IS the shipped contract, four arms that `wat --repl`
/// already gates, and a parallel Rust enum would be a second definition of the same thing
/// free to drift from it. A Rust caller reads `EnumValue::variant_name`.
/// Second return is the freeze's symbol table when the freeze succeeded.
/// The session TCO's it into the next turn so `runtime_def_values` live
/// until `reset`. CheckFailed-on-freeze yields `None`.
pub(crate) fn eval_form_against_defs(
    form: &WatAST,
    defs: Vec<WatAST>,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<(Value, Option<SymbolTable>), EvalBreak> {
    // The session's Config rides through, so a caller is not made to re-declare
    // `set-dims!` on every line. Same source the spawn path uses (kernel/spawn.rs).
    let inherit: Option<crate::config::Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

    // The error stays a typed StartupError all the way to the carrier — the moment it
    // becomes a String, its structure is gone and the mask is back.
    //
    // The live session's `runtime_def_values` are seeded into the freeze
    // so a `def` of a handle / uuid / peer is not re-run. That is the
    // TCO of the ephemeral half (`repl.wat`: bound service survives
    // re-freeze for free).
    let freeze_forms =
        |program: Vec<WatAST>| -> Result<crate::freeze::FrozenWorld, crate::freeze::StartupError> {
            let loader: std::sync::Arc<dyn crate::load::loader::SourceLoader> =
                std::sync::Arc::new(crate::load::loader::InMemoryLoader::new());
            match &inherit {
                Some(cfg) => {
                    crate::freeze::startup_from_forms_with_session(program, None, loader, cfg, sym)
                }
                None => crate::freeze::startup_from_forms(program, None, loader),
            }
        };

    // The BASELINE — the session as it stands. Its residue length is the only way to
    // know which of the full world's residue forms the new line contributed.
    let baseline_residue_len = match freeze_forms(defs.clone()) {
        Ok(world) => world.program.len(),
        // The accumulated defs no longer freeze on their own. That is not this line's
        // fault, and saying so is the honest report — but the real diagnostic is still
        // the freeze's own structured error, so it rides as a nested CAUSE rather than
        // being folded into the sentence.
        Err(e) => {
            return Ok((
                form_outcome(
                    "CheckFailed",
                    vec![fault_with_cause(
                        "the accumulated definition set no longer freezes on its own".to_string(),
                        crate::span::Span::new(Arc::new("<runtime>".to_string()), 0, 0),
                        check_failed_cause(&e, sym),
                    )],
                ),
                None,
            ));
        }
    };

    let mut program = defs;
    program.push(form.clone());
    let world = match freeze_forms(program) {
        Ok(w) => w,
        Err(e) => return Ok((form_outcome_check_failed(&e, sym), None)),
    };

    // What this line contributed, post-expansion.
    let contributed: Vec<WatAST> = world
        .program
        .iter()
        .skip(baseline_residue_len)
        .cloned()
        .collect();

    // Consumed by the freeze itself (a TYPE declaration — defrecord / defenum /
    // defstruct / typealias), so it left no residue at all.
    if contributed.is_empty() {
        return Ok((form_outcome("Declared", vec![]), Some(world.symbols)));
    }

    // Register the runtime declarations this line brought, into a symbol table that
    // carries the session's world. The registration must happen before evaluation so a
    // line that both declares and computes sees its own declaration.
    let mut session_sym = world.symbols.clone();
    let head_of = |f: &WatAST| -> Option<String> {
        match f {
            WatAST::List(items, _) => match items.first() {
                Some(WatAST::Keyword(k, _)) => Some(k.clone()),
                _ => None,
            },
            _ => None,
        }
    };
    // `is_declaration_form`, NOT `is_runtime_declaration_head`. A `do` of
    // only declarations (defservice's expansion) IS a declaration: every
    // child is a def / extend-type / derive. A `do` that yields a value
    // is still an expression. `let` is never a declaration.
    let all_declarations = contributed.iter().all(is_declaration_form);

    register_runtime_defs(
        &world.program,
        env,
        &mut session_sym,
        &world.declared_rete_defns,
    )?;

    if all_declarations {
        return Ok((form_outcome("Declared", vec![]), Some(session_sym)));
    }

    // An expression (possibly alongside declarations — a `do` can carry both). The
    // LAST contributed non-declaration form is the line's value, mirroring an
    // implicit-do: earlier forms run for effect, the last one answers.
    //
    // A `do`/`let` is RUN here (it is an expression), and its nested defs were already
    // registered by `register_runtime_defs` above — registration precedes evaluation
    // precisely so a form that both declares and computes sees its own declaration.
    let mut result = Value::Unit;
    for f in &contributed {
        if head_of(f).map(|h| is_declaration_head(&h)).unwrap_or(false) {
            continue;
        }
        match run_constrained(f, env, &session_sym) {
            Ok(v) => result = v,
            Err(EvalBreak::Signal(s)) => return Err(EvalBreak::Signal(s)),
            Err(EvalBreak::Diagnostic(e)) => {
                return Ok((
                    form_outcome("Raised", vec![runtime_error_to_eval_error_value(&e)]),
                    None,
                ));
            }
        }
    }
    Ok((form_outcome("Evaluated", vec![result]), Some(session_sym)))
}


// ─── Incremental evaluator (arc 068) — :wat::eval-step! ─────────────
//
// `:wat::eval-step!` performs ONE call-by-value reduction at the
// leftmost-outermost redex. Returns:
//
//   Ok(StepNext form)      — one rewrite happened; `form` is the next
//                            WatAST to feed back in.
//   Ok(StepTerminal value) — the form had no redex; `value` is its
//                            HolonAST representation.
//   Err(EvalError)         — malformed form, effectful op in step
//                            mode, or a shape with no step rule yet.
//
// The substrate already has `:wat::eval-ast!` (full evaluation in
// one shot). Step mode exists for the BOOK Chapter 59 dual-LRU
// coordinate cache: every intermediate form is its own coordinate,
// its own potential cache hit, its own potential short-circuit for
// a parallel walker. Without per-step observation, the cache can't
// be built cleanly in user-level wat code.
//
// Strategy: textual substitution (Plotkin small-step) on the WatAST.
// Wat is hygienic; identifier matching uses (name, scope set) so
// distinct bindings of the same name never alias. Effectful ops are
// rejected (consumer falls back to eval-ast! for those sub-forms);
// non-HolonAST-expressible terminals also go through the EvalError
// path (consumer falls back).

/// Internal step result — translated to `Value::Enum` at the
/// `:wat::eval::StepResult` boundary.
///
/// `AlreadyTerminal` is arc 070's "no work happened — input was
/// already a value-shape" variant. Distinct from `Terminal`, which
/// says "this step reduced a redex to a value." A walker/tracer
/// distinguishing chain-length 0 from ≥ 1 reads the variant.
#[derive(Debug)]
enum StepValue {
    Next(WatAST),
    Terminal(HolonAST),
    AlreadyTerminal(HolonAST),
}

/// `(:wat::eval-step! <wat-ast>)` dispatch entry. Mirrors arc 066's
/// `eval_form_ast` Result-wrap shape — every RuntimeError except
/// the control-flow signals becomes an `EvalError` in the Err arm
/// of the returned Value::Result.
fn eval_form_step(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::eval-step!".into(),
                reason: format!(
                    "(:wat::eval-step! <ast-value>) takes exactly 1 argument; got {}",
                    args.len()
                ),
            },
        )
        .into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let value = eval_inner(&args[0], env, sym)?.value_owned();
        let ast = match value {
            Value::wat__WatAST(a) => a,
            other => {
                return Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::eval-step!".into(),
                        expected: "wat::WatAST",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        };
        let stepped = step_form(&ast, env, sym)?;
        Ok(step_value_to_enum(stepped))
    })())
}

/// Arc 070 — `:wat::eval::walk` fold over the eval-step! chain.
///
/// `(:wat::eval::walk form init visit) -> Result<(HolonAST, A),
/// EvalError>`. Lifts the walker pattern that proofs 015/016/017/018
/// each reimplemented into a single substrate primitive. The
/// visitor is called once per coordinate with `(acc, current-form,
/// step-result)` and returns a `(WalkStep :- [A])`:
///
///   - `Continue(acc')` — keep walking. On `StepNext` the loop
///     recurses on the next form; on either terminal flavor the
///     loop returns `(terminal, acc')`.
///   - `Skip(terminal, acc')` — caller has its own answer (cache
///     hit, etc.). Loop stops; returns `(terminal, acc')`.
///
/// `Err(EvalError)` from the inner `eval-step!` propagates as the
/// outer `Result::Err`. The visitor never sees it — if a consumer
/// wants to recover, they wrap walk and match on the outer Result.
///
/// Iterative loop, not recursion — avoids unbounded stack growth on
/// long chains. Walks until: (a) visitor returns Skip, (b) step-
/// result is StepTerminal/AlreadyTerminal, or (c) eval-step! errors.
fn eval_walk(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::eval::walk";
    if args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "(:wat::eval::walk form init visit) takes exactly 3 args; got {}",
                    args.len()
                ),
            },
        )
        .into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let form_value = eval_inner(&args[0], env, sym)?.value_owned();
        let mut current_form: Arc<WatAST> = match form_value {
            Value::wat__WatAST(a) => a,
            other => {
                return Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "wat::WatAST",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        };
        let mut acc = eval_inner(&args[1], env, sym)?.value_owned();
        let visit_value = eval_inner(&args[2], env, sym)?.value_owned();
        let visit_func = match visit_value {
            Value::wat__core__fn(f) => f,
            other => {
                return Err(RuntimeError::new(
                    args[2].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "wat::core::fn — visitor (acc, form, step) → (wat::eval::WalkStep :- [A])",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        };
        loop {
            let stepped = step_form(&current_form, env, sym)?;
            // Cache the structural shape before we hand it to
            // visit so we can decide what to do post-visit
            // (recurse on next form, or return on terminal).
            let (next_form_opt, terminal_opt) = match &stepped {
                StepValue::Next(form) => (Some(form.clone()), None),
                StepValue::Terminal(h) => (None, Some(h.clone())),
                StepValue::AlreadyTerminal(h) => (None, Some(h.clone())),
            };
            let step_value = step_value_to_enum(stepped);
            let walkstep_value = apply_function(
                visit_func.clone(),
                vec![acc, Value::wat__WatAST(current_form.clone()), step_value],
                sym,
                list_span.clone(),
            )?;
            // Visitor must return (:wat::eval::WalkStep :- [A]) as a
            // tagged-enum value. Read the variant + fields.
            let (variant_name, fields) = match walkstep_value {
                Value::Enum(ev) => {
                    if ev.type_path != ":wat::eval::WalkStep" {
                        return Err(RuntimeError::new(
                            args[2].span().clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: OP.into(),
                                expected: "(wat::eval::WalkStep :- [A])",
                                got: Box::new(ValueSnapshot::unavailable("different enum")),
                                // arc 138: no — Value::Enum result from visitor; no originating AST
                            },
                        )
                        .into());
                    }
                    let ev = (*ev).clone();
                    (ev.variant_name, ev.fields)
                }
                other => {
                    return Err(RuntimeError::new(
                        args[2].span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "(wat::eval::WalkStep :- [A])",
                            got: Box::new(ValueSnapshot::of(&other)),
                            // arc 138: no — visitor return value; no originating AST
                        },
                    )
                    .into());
                }
            };
            match variant_name.as_str() {
                "Continue" => {
                    if fields.len() != 1 {
                        return Err(RuntimeError::new(
                            args[2].span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: OP.into(),
                                reason: format!(
                                    "WalkStep::Continue takes exactly 1 field (acc); got {}",
                                    fields.len()
                                ),
                                // arc 138: no — WalkStep field count from visitor return; no AST
                            },
                        )
                        .into());
                    }
                    let mut iter = fields.into_iter();
                    acc = iter.next().expect("length checked");
                    if let Some(next_form) = next_form_opt {
                        current_form = Arc::new(next_form);
                        continue;
                    }
                    // Terminal reached — return (terminal, acc).
                    let terminal = terminal_opt.expect("terminal_opt set when next_form_opt None");
                    return Ok(Value::Tuple(Arc::new(vec![
                        Value::holon__HolonAST(Arc::new(terminal)),
                        acc,
                    ])));
                }
                "Skip" => {
                    if fields.len() != 2 {
                        return Err(RuntimeError::new(
                            args[2].span().clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: OP.into(),
                                reason: format!(
                                    "WalkStep::Skip takes exactly 2 fields (terminal, acc); got {}",
                                    fields.len()
                                ),
                                // arc 138: no — WalkStep field count from visitor return; no AST
                            },
                        )
                        .into());
                    }
                    let mut iter = fields.into_iter();
                    let terminal_v = iter.next().expect("length checked");
                    let new_acc = iter.next().expect("length checked");
                    let terminal_h = match terminal_v {
                        Value::holon__HolonAST(h) => h,
                        other => {
                            return Err(RuntimeError::new(
                                args[2].span().clone(),
                                RuntimeErrorKind::TypeMismatch {
                                    op: OP.into(),
                                    expected: "wat::holon::HolonAST (Skip's terminal field)",
                                    got: Box::new(ValueSnapshot::of(&other)),
                                    // arc 138: no — visitor return value field; no AST
                                },
                            )
                            .into());
                        }
                    };
                    return Ok(Value::Tuple(Arc::new(vec![
                        Value::holon__HolonAST(terminal_h),
                        new_acc,
                    ])));
                }
                other => {
                    return Err(RuntimeError::new(
                        args[2].span().clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!(
                                "WalkStep variant must be Continue or Skip; got {}",
                                other
                            ),
                            // arc 138: no — visitor return value variant; no AST
                        },
                    )
                    .into());
                }
            }
        }
    })())
}

/// Construct the `:wat::eval::StepResult` enum value from an
/// internal `StepValue`. Mirrors arc 060's `thread_died_error_*`
/// helper shape.
fn step_value_to_enum(sv: StepValue) -> Value {
    match sv {
        StepValue::Next(form) => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::eval::StepResult".into(),
            variant_name: "StepNext".into(),
            names: builtin_enum_variant_names(":wat::eval::StepResult", "StepNext"),
            fields: vec![Value::wat__WatAST(Arc::new(form))],
        })),
        StepValue::Terminal(holon) => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::eval::StepResult".into(),
            variant_name: "StepTerminal".into(),
            names: builtin_enum_variant_names(":wat::eval::StepResult", "StepTerminal"),
            fields: vec![Value::holon__HolonAST(Arc::new(holon))],
        })),
        StepValue::AlreadyTerminal(holon) => Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::eval::StepResult".into(),
            variant_name: "AlreadyTerminal".into(),
            names: builtin_enum_variant_names(":wat::eval::StepResult", "AlreadyTerminal"),
            fields: vec![Value::holon__HolonAST(Arc::new(holon))],
        })),
    }
}

/// Step a wat form one rewrite. Outer-driver for the per-shape step
/// rules. Arc 068 covered literal/arithmetic/control flow/let/match/
/// holon-ctor/user-fn rules. Arc 070 prepends a structural-already-
/// terminal check: if the form's WatAST recognizes as a value-shape
/// (literal leaves, holon-constructor lists with all-value args,
/// bare-list Bundle lifts), short-circuit to `AlreadyTerminal` —
/// signaling "no work happened; this IS the value" rather than the
/// current behavior where literals returned `Terminal` and lifted
/// Bundles returned `Err(NoStepRule)`.
fn step_form(form: &WatAST, env: &Environment, sym: &SymbolTable) -> Result<StepValue, EvalBreak> {
    // Arc 070 — try value-shape recognition first. Covers everything
    // a `to-wat(holon)` round-trip can produce, plus primitive
    // literals. Reduction-shape forms (arithmetic, comparison,
    // user-function calls, special forms) fall through.
    if let Some(holon) = try_recognize_holon_value(form) {
        return Ok(StepValue::AlreadyTerminal(holon));
    }
    match form {
        // Literal arms reach here only if `try_recognize_holon_value`
        // somehow misses (it shouldn't — these are the canonical
        // cases). Defense in depth.
        WatAST::IntLit(n, _) => Ok(StepValue::Terminal(HolonAST::i64(*n))),
        WatAST::FloatLit(x, _) => Ok(StepValue::Terminal(HolonAST::f64(*x))),
        // Arc 300 stone B — SURPRISE (see `watast_to_holon`'s note): holon-rs
        // has no native rational leaf; lower to its canonical rendered string.
        WatAST::RationalLit(r, _) => Ok(StepValue::Terminal(HolonAST::string(format!(
            "{}/{}",
            r.numer(),
            r.denom()
        )))),
        // Arc 300 stone C1 — same SURPRISE as Rational immediately above.
        WatAST::BigIntLit(n, _) => Ok(StepValue::Terminal(HolonAST::string(format!("{}N", n)))),
        // Arc 300 stone D — native holon-rs Char leaf (see `watast_to_holon`'s
        // note); no lossy string rendering needed.
        WatAST::CharLit(c, _) => Ok(StepValue::Terminal(HolonAST::char_(*c))),
        WatAST::BoolLit(b, _) => Ok(StepValue::Terminal(HolonAST::bool_(*b))),
        WatAST::StringLit(s, _) => Ok(StepValue::Terminal(HolonAST::string(s.as_str()))),
        // Arc 244 — NilLit terminal step → HolonAST::symbol("nil") (nil HolonAST representation).
        WatAST::NilLit(_) => Ok(StepValue::Terminal(HolonAST::symbol("nil"))),
        // Arc 221 Stone 221.4b — Keyword literal terminal → HolonAST::Keyword leaf.
        // Pre-arc-221 used HolonAST::symbol(k.as_str()); retired per arc 221 doctrine.
        WatAST::Keyword(k, _) => Ok(StepValue::Terminal(HolonAST::keyword(k.as_str()))),
        // A bare symbol that survived to step time means substitution
        // didn't reach it — an unbound free variable. Surface as
        // NoStepRule so the consumer falls back to eval-ast! (which
        // would have raised UnboundSymbol there too).
        WatAST::Symbol(ident, sym_span) => Err(RuntimeError::new(
            sym_span.clone(),
            RuntimeErrorKind::NoStepRule {
                op: format!("symbol-ref:{}", ident.as_str()),
            },
        )
        .into()),
        WatAST::List(items, span) => step_list(items, span, env, sym),
        // Arc 167 slice 1 — vector literals reaching the stepper
        // means a binding-position consumer hasn't intercepted
        // them. Surface as NoStepRule so the consumer falls back
        // to eval, which raises the canonical "vector literals at
        // value position" error.
        WatAST::Vector(_, vec_span) => Err(RuntimeError::new(
            vec_span.clone(),
            RuntimeErrorKind::NoStepRule {
                op: "<vector literal>".into(),
            },
        )
        .into()),
        // Arc 257 slice 1 — Map/Set literals reaching the stepper
        // fall through to eval via NoStepRule.
        WatAST::Map(_, map_span) => Err(RuntimeError::new(
            map_span.clone(),
            RuntimeErrorKind::NoStepRule {
                op: "<map literal>".into(),
            },
        )
        .into()),
        WatAST::Set(_, set_span) => Err(RuntimeError::new(
            set_span.clone(),
            RuntimeErrorKind::NoStepRule {
                op: "<set literal>".into(),
            },
        )
        .into()),
    }
}


/// Dispatcher for a `List` form. Recognizes the head keyword and
/// chooses the matching rule: special forms (if / let / match) get
/// dedicated rewrites; pure ops descend leftmost-non-canonical and
/// fire-via-eval; user-defined functions descend args then β-reduce
/// by substitution; effectful prefixes refuse with `EffectfulInStep`;
/// anything else surfaces `NoStepRule` for the consumer's fallback.
fn step_list(
    items: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<StepValue, EvalBreak> {
    let head = match items.first() {
        Some(h) => h,
        None => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::NoStepRule { op: "()".into() },
            )
            .into())
        }
    };
    let head_kw = match head {
        WatAST::Keyword(k, _) => k.clone(),
        WatAST::Symbol(ident, sym_span) => {
            // Bare-symbol heads (inline fn call sites, let-bound
            // function values) need a higher-order step rule that
            // hasn't shipped yet. Phase 3 territory.
            return Err(RuntimeError::new(
                sym_span.clone(),
                RuntimeErrorKind::NoStepRule {
                    op: format!("symbol-head:{}", ident.as_str()),
                },
            )
            .into());
        }
        _ => {
            return Err(RuntimeError::new(
                head.span().clone(),
                RuntimeErrorKind::NoStepRule {
                    op: "<non-keyword-head>".into(),
                },
            )
            .into());
        }
    };

    if crate::rete::purity::is_effectful_op(&head_kw) {
        return Err(RuntimeError::new(
            head.span().clone(),
            RuntimeErrorKind::EffectfulInStep { op: head_kw },
        )
        .into());
    }

    let args = &items[1..];
    match head_kw.as_str() {
        ":wat::core::if" => step_if(args, list_span, env, sym),
        // Arc 154 — sequential semantics under `:wat::core::let`.
        ":wat::core::let" => step_let(args, list_span, env, sym),
        ":wat::core::do" => step_do(args, list_span, env, sym),
        ":wat::core::match" => step_match(args, list_span, env, sym),
        // Pure operations whose redex fires when all args are
        // primitive-canonical. We delegate the actual computation to
        // `eval` once that condition holds — eval gives the right
        // semantics for free, including i64/f64 promotion, division-
        // by-zero, comparison ordering, etc.
        ":wat::core::+"
        | ":wat::core::-"
        | ":wat::core::*"
        | ":wat::core::/"
        | ":wat::core::="
        | ":wat::core::not="
        | ":wat::core::<"
        | ":wat::core::>"
        | ":wat::core::<="
        | ":wat::core::>="
        | ":wat::core::not"
        | ":wat::core::and"
        | ":wat::core::or"
        // Stone 237.8b — drop '2 suffix from per-Type binary primitives.
        // Arc 255 Stone C — spelling updated from `:wat::core::{i64,f64}::*` to
        // the surviving `:wat::{i64,f64}::*` (the old spelling retired).
        | ":wat::i64::+"
        | ":wat::i64::-"
        | ":wat::i64::*"
        | ":wat::i64::/"
        | ":wat::i64::to-string"
        | ":wat::i64::to-f64"
        | ":wat::f64::+"
        | ":wat::f64::-"
        | ":wat::f64::*"
        | ":wat::f64::/"
        | ":wat::f64::abs"
        | ":wat::f64::max"
        | ":wat::f64::min"
        // arc 237 Stone 237.8a — +'i64'f64 / +'f64'i64 etc. mixed-type
        // canonical entries DELETED under THE DECISION.
        | ":wat::core::u8" => step_descend_then_fire(items, list_span, env, sym),
        // Holon constructors — pure ops over the closed algebra (arc 057).
        // They use a holon-canonical fire condition: a list whose head is
        // itself a holon constructor with recursively-canonical args
        // counts as a single holon "value." Lifting an intermediate
        // typed-leaf back to a primitive WatAST would lose the
        // HolonAST-typed distinction the next constructor expects, so
        // the whole holon tree fires in one step instead of piecemeal.
        // Arc 225 Stone 225.1 — `to-holon` added (new polymorphic UP verb;
        // always returns HolonAST). `Atom` remains (narrow HolonAST→Atom wrap).
        ":wat::holon::Atom"
        | ":wat::holon::to-holon"
        | ":wat::holon::leaf"
        | ":wat::holon::Bind"
        | ":wat::holon::Bundle"
        | ":wat::holon::Permute"
        | ":wat::holon::Thermometer"
        | ":wat::holon::Blend" => {
            step_holon_descend_then_fire(items, list_span, env, sym)
        }
        // Bare fn terminal — Q6 of arc 068 DESIGN. A `(fn ...)`
        // form is its own canonical-form holon: no captures (a closure-
        // bearing fn would have already produced a Function value
        // with closed_env, not a literal `(fn ...)` form). Wrap as
        // an opaque-identity Atom of the structural lowering so cosine /
        // hash / cache keys see it as a single coordinate.
        // Arc 155 slice 2 — lambda dispatch arm retired; only
        // canonical `:wat::core::fn` recognized.
        ":wat::core::fn" => {
            let form = WatAST::List(items.to_vec(), list_span.clone());
            let h = watast_to_holon(&form);
            Ok(StepValue::Terminal(HolonAST::Atom(Arc::new(h))))
        }
        _ => {
            // User-defined function looked up by full keyword path.
            // Top-level defines have closed_env=None; closures (from
            // fn) have it Some — we refuse those for now (Phase 3).
            if sym.has_function(&head_kw) {
                step_user_call(items, list_span, env, sym, &head_kw)
            } else {
                Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::NoStepRule {
                    op: head_kw
                }).into())
            }
        }
    }
}

// Arc 109 Stone — the last two map items — `effectful_by_prefix` moved to
// `src/rete/purity.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged;
// moves together with `is_effectful_op` (the two-tier classifier), never
// split — see that file for the doc this comment used to carry.

// Arc 109 Stone — the last two map items — `is_effectful_op` moved to
// `src/rete/purity.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// True iff `form` is a primitive literal — Phase 2's notion of
/// canonicity for arithmetic/comparison/logical fire conditions.
/// Lists and symbols are non-canonical.
fn is_step_canonical(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::IntLit(_, _)
            | WatAST::FloatLit(_, _)
            | WatAST::BoolLit(_, _)
            | WatAST::StringLit(_, _)
            | WatAST::Keyword(_, _)
    )
}

/// Step `form` and lift the result back into a `WatAST` so callers
/// rebuilding an outer form have something to splice in. If the
/// inner step terminated, `holon_to_watast` provides the lift; if it
/// produced a Next form, that form is the WatAST directly.
fn step_to_watast(
    form: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<WatAST, EvalBreak> {
    match step_form(form, env, sym)? {
        StepValue::Next(w) => Ok(w),
        // Both terminal flavors lift the same way for descend-rule
        // rebuilds — the caller wants a WatAST to splice into an
        // outer form. AlreadyTerminal differs from Terminal only in
        // signaling chain length to the consumer; descent doesn't
        // care.
        StepValue::Terminal(h) | StepValue::AlreadyTerminal(h) => Ok(holon_to_watast(&h)),
    }
}

/// Generic descend-then-fire for pure ops. If any arg is non-
/// canonical, recursively step the leftmost non-canonical one,
/// rebuild the outer form, return Next. If all args are canonical,
/// call `eval` — args are values, so eval reduces only the top-level
/// redex — convert the result via `value_to_holon`, return Terminal.
fn step_descend_then_fire(
    items: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<StepValue, EvalBreak> {
    for (idx, arg) in items.iter().enumerate().skip(1) {
        if !is_step_canonical(arg) {
            let new_arg = step_to_watast(arg, env, sym)?;
            let mut new_items: Vec<WatAST> = items.to_vec();
            new_items[idx] = new_arg;
            return Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())));
        }
    }
    // All args canonical — fire.
    let form = WatAST::List(items.to_vec(), list_span.clone());
    let v = eval_inner(&form, env, sym)?.value_owned();
    let h_val = value_to_holon(":wat::eval-step!", v)?;
    let h = match h_val {
        Value::holon__HolonAST(h) => (*h).clone(),
        // value_to_holon Ok-arm only ever returns Value::holon__HolonAST.
        _ => unreachable!("value_to_holon returns Value::holon__HolonAST on Ok"),
    };
    Ok(StepValue::Terminal(h))
}

/// Holon-constructor variant of descend-then-fire. Same shape as
/// `step_descend_then_fire`, but uses `is_holon_arg_canonical` so a
/// nested holon-constructor list (its inner args canonical) counts
/// as a single value for the parent — the entire holon tree fires
/// in one rewrite. This is the honest answer to the type-loss
/// problem: `Atom("k")` produces a typed `HolonAST::String` leaf
/// (per arc 057's polymorphic Atom), and lifting that back to a
/// bare WatAST `StringLit` would make the next `Bind` step fail
/// `require_holon` — so we don't lift; we recognize the structural
/// holon shape and let `eval` reduce the whole tree.
fn step_holon_descend_then_fire(
    items: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<StepValue, EvalBreak> {
    for (idx, arg) in items.iter().enumerate().skip(1) {
        if !is_holon_arg_canonical(arg) {
            let new_arg = step_to_watast(arg, env, sym)?;
            let mut new_items: Vec<WatAST> = items.to_vec();
            new_items[idx] = new_arg;
            return Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())));
        }
    }
    // Fire. Bundle's signature is `(:Result :- [HolonAST CapacityExceeded])`
    // — a wat-side Result wrap orthogonal to the EvalError wrap that
    // eval-step!'s caller sees. Other holon constructors return a
    // bare HolonAST. Peel the inner Result if present so the
    // user-visible step terminal is uniformly a HolonAST: Ok cases
    // unwrap to the inner; Err cases lift to TypeMismatch so
    // wrap_as_eval_result surfaces the capacity overflow as the
    // outer EvalError. (Q9 of arc 068 DESIGN.)
    let form = WatAST::List(items.to_vec(), list_span.clone());
    let v = eval_inner(&form, env, sym)?.value_owned();
    let v = match v {
        Value::Result(r) => match Arc::try_unwrap(r).unwrap_or_else(|a| (*a).clone()) {
            Ok(inner) => inner,
            Err(err_val) => {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::eval-step!".into(),
                        expected: "successful holon construction",
                        got: Box::new(ValueSnapshot::of(&err_val)),
                    },
                )
                .into());
            }
        },
        other => other,
    };
    let h_val = value_to_holon(":wat::eval-step!", v)?;
    let h = match h_val {
        Value::holon__HolonAST(h) => (*h).clone(),
        _ => unreachable!("value_to_holon returns Value::holon__HolonAST on Ok"),
    };
    Ok(StepValue::Terminal(h))
}


/// `(:wat::core::if cond then else)` — five-arg shape per
/// arc 023. If `cond` is a canonical `BoolLit`, project to the chosen
/// branch as the next form; otherwise descend the cond. The `-> :T`
/// annotation is preserved verbatim across rewrites.
fn step_if(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<StepValue, EvalBreak> {
    if args.len() == 3 {
        // Arc 258.1 — bare `(if cond then else)`: args = [cond, then, else].
        let cond = &args[0];
        return match cond {
            WatAST::BoolLit(true, _) => Ok(StepValue::Next(args[1].clone())),
            WatAST::BoolLit(false, _) => Ok(StepValue::Next(args[2].clone())),
            _ => {
                let new_cond = step_to_watast(cond, env, sym)?;
                let new_items = vec![
                    WatAST::Keyword(":wat::core::if".into(), list_span.clone()),
                    new_cond,
                    args[1].clone(),
                    args[2].clone(),
                ];
                Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())))
            }
        };
    }
    // Arc 258.4 — the `-> :T` ascription is retired; a stray `->` (the old 5-arg form)
    // is the retired shape; refuse it with a migration hint.
    if args.len() >= 2 && matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::if".into(),
            reason: "`:wat::core::if` no longer takes `-> :T`; the result type is inferred by unifying the branches. Write (:wat::core::if cond then else)".into()
        }).into());
    }
    Err(RuntimeError::new(
        list_span.clone(),
        RuntimeErrorKind::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond then else) — 3 args; got {}",
                args.len()
            ),
        },
    )
    .into())
}

/// `(:wat::core::let [n1 e1 n2 e2 ...] body1 body2 ... bodyN)` —
/// peel one binding per step. If the head binding's RHS is
/// non-canonical, descend it and rebuild. If canonical, substitute
/// name → RHS into remaining bindings and body, drop the now-bound
/// first pair, return Next of the smaller form. Arc 168 — flat-vector
/// outer + implicit-do body.
///
/// Non-Vector outer (e.g. legacy `((n e) ...)` nested-pair list)
/// produces a clean `MalformedForm`. Arc 168 slice 3 retired the
/// outer-List arm.
///
/// Empty bindings + body → see `synthesize_let_body` for the
/// implicit-do collapse rule.
///
/// Pre-arc-168 `step_let_star` renamed to canonical `step_let` after
/// arc 154 retired `let*` into `let` (single-letform vocabulary).
fn step_let(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<StepValue, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "expected (:wat::core::let bindings body ...); got {} args",
                    args.len()
                ),
            },
        )
        .into());
    }

    // Outer shape is the canonical flat-Vector (arc 168). Desugar into
    // a uniform `Vec<(binder, rhs)>` pair-list for the stepper.
    let bindings_form_span = args[0].span().clone();
    let pairs: Vec<(WatAST, WatAST)> = match &args[0] {
        WatAST::Vector(items, _) => {
            if items.len() % 2 != 0 {
                return Err(RuntimeError::new(
                    bindings_form_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::let".into(),
                        reason: format!(
                            "let bindings vector must have an even number of elements; got {}",
                            items.len()
                        ),
                    },
                )
                .into());
            }
            items
                .chunks_exact(2)
                .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
                .collect()
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::let".into(),
                    reason: "let bindings must be a flat vector `[name expr ...]`".into(),
                },
            )
            .into());
        }
    };

    let body_forms = &args[1..];

    if pairs.is_empty() {
        // No more bindings — collapse the let into the body.
        return Ok(StepValue::Next(synthesize_let_body(body_forms, list_span)));
    }

    // Inspect first pair. The stepper handles single-Symbol-binder
    // chunks (canonical post-arc-168 shape). Vector destructure
    // binders fall through to the eval path (single-step semantics
    // matches one binding peel; destructure is multi-bind atomic).
    let (binder, rhs) = (&pairs[0].0, &pairs[0].1);

    let name_ident: crate::scope::Identifier = match binder {
        WatAST::Symbol(ident, _) => ident.clone(),
        _ => {
            return Err(RuntimeError::new(
                binder.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::let".into(),
                    reason:
                        "step_let only handles Symbol binders; destructure goes through eval_let"
                            .into(),
                },
            )
            .into());
        }
    };

    if !is_step_canonical(rhs) {
        // Step the RHS one rewrite, rebuild the let with the stepped
        // RHS in place. Outer is always Vector post-arc-168.
        let new_rhs = step_to_watast(rhs, env, sym)?;
        let new_args = rebuild_let_with_first_rhs(&args[0], &pairs, &new_rhs)?;
        let mut new_items: Vec<WatAST> = vec![
            WatAST::Keyword(":wat::core::let".into(), list_span.clone()),
            new_args,
        ];
        new_items.extend(body_forms.iter().cloned());
        return Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())));
    }

    // RHS canonical — peel: substitute name → rhs through the
    // remaining bindings + body forms.
    let rest_pairs: Vec<(WatAST, WatAST)> = pairs[1..]
        .iter()
        .map(|(b, r)| (b.clone(), substitute(r, &name_ident, rhs)))
        .collect();
    let new_body_forms: Vec<WatAST> = body_forms
        .iter()
        .map(|f| substitute(f, &name_ident, rhs))
        .collect();

    if rest_pairs.is_empty() {
        return Ok(StepValue::Next(synthesize_let_body(
            &new_body_forms,
            list_span,
        )));
    }

    // Rebuild the bindings carrier as canonical flat-Vector.
    let mut flat: Vec<WatAST> = Vec::with_capacity(rest_pairs.len() * 2);
    for (b, r) in &rest_pairs {
        flat.push(b.clone());
        flat.push(r.clone());
    }
    let rebuilt_bindings = WatAST::Vector(flat, bindings_form_span.clone());
    let mut new_items: Vec<WatAST> = vec![
        WatAST::Keyword(":wat::core::let".into(), list_span.clone()),
        rebuilt_bindings,
    ];
    new_items.extend(new_body_forms);
    Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())))
}

/// Collapse an implicit-do body of N forms into a single AST. Arc 168
/// — implicit-do is purely additive: no body forms means nil; one
/// form means the form itself; many forms get wrapped in
/// `(:wat::core::do f1 f2 ... fN)` for uniform downstream handling.
/// Arc 244: empty body was the nil-type Keyword heresy;
/// now `NilLit` (canonical nil value literal).
/// Used by step_let and step_let_drop_canonical when they peel away
/// all bindings.
fn synthesize_let_body(forms: &[WatAST], outer_span: &Span) -> WatAST {
    if forms.is_empty() {
        // Arc 244 — canonical nil value literal (not the type keyword).
        return WatAST::NilLit(outer_span.clone());
    }
    if forms.len() == 1 {
        return forms[0].clone();
    }
    let mut do_items: Vec<WatAST> = Vec::with_capacity(forms.len() + 1);
    do_items.push(WatAST::Keyword(":wat::core::do".into(), outer_span.clone()));
    do_items.extend(forms.iter().cloned());
    WatAST::List(do_items, outer_span.clone())
}

/// Rebuild a let bindings carrier with a new RHS in the first
/// position. Outer is always canonical flat-Vector post-arc-168.
fn rebuild_let_with_first_rhs(
    bindings_form: &WatAST,
    pairs: &[(WatAST, WatAST)],
    new_first_rhs: &WatAST,
) -> Result<WatAST, EvalBreak> {
    let outer_span = bindings_form.span().clone();
    match bindings_form {
        WatAST::Vector(_, _) => {
            let mut flat: Vec<WatAST> = Vec::with_capacity(pairs.len() * 2);
            // First pair gets the new RHS.
            flat.push(pairs[0].0.clone());
            flat.push(new_first_rhs.clone());
            for (b, r) in &pairs[1..] {
                flat.push(b.clone());
                flat.push(r.clone());
            }
            Ok(WatAST::Vector(flat, outer_span))
        }
        other => Err(RuntimeError::new(
            other.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::let".into(),
                reason: "let bindings carrier must be a vector `[name expr ...]`".into(),
            },
        )
        .into()),
    }
}

/// `(:wat::core::do f1 f2 ... fN)` — Clojure-faithful sequential
/// evaluation form. Arc 136 slice 1a.
///
/// Single-step semantics for the eval-step! interpreter:
/// - Empty arg list → MalformedForm (mirrors `eval_do`).
/// - One remaining arg → `Next(arg)`; the do form is "transparent" once
///   only the final form is left (matches Clojure's `(do x) ≡ x`).
/// - Head non-canonical → step the head one rewrite, rebuild the do
///   form with the stepped head in front, return `Next`.
/// - Head canonical (and there's more than one arg) → drop the head
///   (its result is discarded per do semantics) and return `Next` with
///   the do form re-headed by the second arg.
fn step_do(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<StepValue, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::do".into(),
                reason: "do form requires at least one form; got zero".into(),
            },
        )
        .into());
    }
    if args.len() == 1 {
        return Ok(StepValue::Next(args[0].clone()));
    }
    let head = &args[0];
    if !is_step_canonical(head) {
        let new_head = step_to_watast(head, env, sym)?;
        let mut new_items: Vec<WatAST> = vec![
            WatAST::Keyword(":wat::core::do".into(), list_span.clone()),
            new_head,
        ];
        new_items.extend(args[1..].iter().cloned());
        return Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())));
    }
    // Head canonical — discard it; rebuild do form starting at args[1].
    let mut new_items: Vec<WatAST> =
        vec![WatAST::Keyword(":wat::core::do".into(), list_span.clone())];
    new_items.extend(args[1..].iter().cloned());
    Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())))
}

/// `(:wat::core::match scrut arm1 arm2 ...)` — descend the
/// scrutinee until match-canonical, then pick the first arm whose
/// pattern matches structurally and substitute pattern bindings into
/// that arm's body. Single rewrite per step: arm selection + binder
/// substitution happen together.
fn step_match(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<StepValue, EvalBreak> {
    if args.len() < 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!(
                    "expected (:wat::core::match scrut arm1 ...); got {} args",
                    args.len()
                ),
            },
        )
        .into());
    }
    let scrut = &args[0];
    if !is_match_canonical(scrut) {
        let new_scrut = step_to_watast(scrut, env, sym)?;
        let mut new_items: Vec<WatAST> = vec![
            WatAST::Keyword(":wat::core::match".into(), list_span.clone()),
            new_scrut,
        ];
        new_items.extend(args[1..].iter().cloned());
        return Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())));
    }
    for arm in &args[1..] {
        let arm_items = match arm {
            WatAST::List(p, _) if p.len() == 2 => p,
            _ => {
                return Err(RuntimeError::new(
                    arm.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: "arm shape must be (pattern body)".into(),
                    },
                )
                .into());
            }
        };
        let pattern = &arm_items[0];
        let body = &arm_items[1];
        if let Some(binds) = try_match_pattern_ast(pattern, scrut)? {
            let new_body = substitute_many(body, &binds);
            return Ok(StepValue::Next(new_body));
        }
    }
    Err(RuntimeError::new(
        scrut.span().clone(),
        RuntimeErrorKind::PatternMatchFailed {
            value_type: scrut.variant_name(),
        },
    )
    .into())
}

/// Match canonicity — Phase 2 admits primitive literals, keyword
/// tokens, and constructor-form lists (`Some` / `Ok` / `Err`) whose
/// fields are recursively match-canonical. Anything else must descend.
///
/// Arc 109 slice 1h+1i — also accept FQDN keyword heads
/// (`:wat::core::Some` / `:wat::core::Ok` / `:wat::core::Err`)
/// as canonical constructors.
fn is_match_canonical(form: &WatAST) -> bool {
    match form {
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::Keyword(_, _) => true,
        WatAST::List(items, _) => {
            // One arm since the bare-symbol arm's removal (below): only the canonical
            // FQDN keyword head names a matchable constructor form.
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                // THE THIRD DOOR of the bare-symbol shorthand, closed 2026-08-30.
                //
                // A `Some(WatAST::Symbol(..))` arm used to bless a bare-symbol constructor form
                // (`(Some 5)`) as canonical matchable data here, which is what kept the retired
                // shorthand evaluable through the CEK stepper (`:wat::eval::walk` /
                // `:wat::eval-step!`) long after arc 109 slice 1h closed the constructor door and
                // this session closed the pattern door. Measured live before removal:
                // `(:wat::eval::walk '(match (Some 5) ((Some n) n) …) …)` returned `Ok [[5 2]]`.
                //
                // It was the SHORTHAND, not generic structural matching — discriminated with a
                // made-up head, which returned `no-step-rule for op: symbol-head:Zorble` while
                // `Some` evaluated. With the arm gone, `Some` behaves exactly like `Zorble`.
                //
                // The Keyword arm below carries the canonical FQDN spelling and is untouched.
                // `try_match_pattern_ast`'s head comparison stays generic (Symbol-vs-Symbol by
                // name) — it was never the heresy; it only ever reached these forms because THIS
                // arm blessed them as scrutinees.
                let s = k.as_str();
                if matches!(s, ":wat::core::Some" | ":wat::core::Ok" | ":wat::core::Err")
                    && items.len() >= 2
                {
                    return items[1..].iter().all(is_match_canonical);
                }
            }
            false
        }
        _ => false,
    }
}

/// WatAST-level pattern matcher mirroring `try_match_pattern`'s
/// dispatch but operating entirely on parse-tree shape. Returns the
/// list of `(binder, replacement-form)` pairs to substitute into the
/// arm body, `None` if the pattern doesn't match this scrutinee, or
/// `Err` for malformed patterns.
fn try_match_pattern_ast(
    pattern: &WatAST,
    scrutinee: &WatAST,
) -> Result<Option<Vec<(crate::scope::Identifier, WatAST)>>, EvalBreak> {
    match pattern {
        WatAST::Symbol(ident, _) if ident.as_str() == "_" => Ok(Some(Vec::new())),
        WatAST::Symbol(ident, _) => Ok(Some(vec![(ident.clone(), scrutinee.clone())])),
        WatAST::IntLit(n, _) => Ok(match scrutinee {
            WatAST::IntLit(s, _) if s == n => Some(Vec::new()),
            _ => None,
        }),
        WatAST::FloatLit(f, _) => Ok(match scrutinee {
            WatAST::FloatLit(s, _) if s == f => Some(Vec::new()),
            _ => None,
        }),
        // Arc 300 stone B — rational literal pattern (parse-tree level).
        WatAST::RationalLit(r, _) => Ok(match scrutinee {
            WatAST::RationalLit(s, _) if s == r => Some(Vec::new()),
            _ => None,
        }),
        // Arc 300 stone C1 — bigint literal pattern (parse-tree level; mirrors
        // the Rational arm immediately above, one type over).
        WatAST::BigIntLit(n, _) => Ok(match scrutinee {
            WatAST::BigIntLit(s, _) if s == n => Some(Vec::new()),
            _ => None,
        }),
        // Arc 300 stone D — char literal pattern (parse-tree level; mirrors
        // the BigInt/Rational arms immediately above).
        WatAST::CharLit(c, _) => Ok(match scrutinee {
            WatAST::CharLit(s, _) if s == c => Some(Vec::new()),
            _ => None,
        }),
        WatAST::BoolLit(b, _) => Ok(match scrutinee {
            WatAST::BoolLit(s, _) if s == b => Some(Vec::new()),
            _ => None,
        }),
        WatAST::StringLit(s, _) => Ok(match scrutinee {
            WatAST::StringLit(v, _) if v == s => Some(Vec::new()),
            _ => None,
        }),
        WatAST::Keyword(k, _) => Ok(match scrutinee {
            WatAST::Keyword(s, _) if s == k => Some(Vec::new()),
            _ => None,
        }),
        WatAST::List(p_items, _) => {
            let s_items = match scrutinee {
                WatAST::List(s, _) => s,
                _ => return Ok(None),
            };
            if p_items.is_empty() || p_items.len() != s_items.len() {
                return Ok(None);
            }
            // Constructor heads (Some / Ok / Err / a registered
            // keyword variant) must compare literally — "Some" the
            // pattern head names the constructor, not a binder.
            let head_match = match (&p_items[0], &s_items[0]) {
                (WatAST::Symbol(p, _), WatAST::Symbol(s, _)) => p.as_str() == s.as_str(),
                (WatAST::Keyword(p, _), WatAST::Keyword(s, _)) => p == s,
                _ => false,
            };
            if !head_match {
                return Ok(None);
            }
            let mut binds: Vec<(crate::scope::Identifier, WatAST)> = Vec::new();
            for (p, s) in p_items.iter().skip(1).zip(s_items.iter().skip(1)) {
                match try_match_pattern_ast(p, s)? {
                    Some(b) => binds.extend(b),
                    None => return Ok(None),
                }
            }
            Ok(Some(binds))
        }
        // Arc 167 slice 1 — vector sub-patterns are not admitted
        // in arc 167. Slice 2 wires fn / defn signature consumers;
        // pattern positions remain illegal.
        WatAST::Vector(_, _) => Err(RuntimeError::new(
            pattern.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "vector sub-patterns are not supported in arc 167".into(),
            },
        )
        .into()),
        // Arc 244 — NilLit pattern at AST level: matches another NilLit.
        WatAST::NilLit(_) => Ok(match scrutinee {
            WatAST::NilLit(_) => Some(Vec::new()),
            _ => None,
        }),
        // Arc 257.2 — Map in AST-level match pattern.
        // Hash-destructure form `{var :field ...}`: the AST-level mirror
        // cannot evaluate hash-destructure patterns because receiver-dispatch
        // requires runtime Value types. Return Ok(None) so the arm does not
        // match at AST level (quasiquote/macro paths fall to the next arm).
        // Keys-destructure and plain map literals are not match sub-patterns.
        WatAST::Map(map_pairs, span) => {
            let md = WatAST::classify_map_destructure(map_pairs);
            match &md {
                Some(m) if m.kind == crate::ast::MapDestructureKind::Hash => {
                    // Hash-destructure: AST-level match cannot dispatch on runtime type.
                    Ok(None)
                }
                _ => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason:
                            "map in match-arm position at AST level must be a hash-destructure \
                            ({var :field ...}); keys-destructure and plain map literals are not \
                            valid match sub-patterns"
                                .into(),
                    },
                )
                .into()),
            }
        }
        // Set literals are not match sub-patterns at AST level.
        WatAST::Set(_, span) => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "set literal is not a valid match sub-pattern".into(),
            },
        )
        .into()),
    }
}

/// Capture-free textual substitution. Replace every `Symbol(ident)`
/// equal to `target` with `replacement`. Wat's hygiene model means
/// `Identifier` equality already covers (name, scope-set) — distinct
/// bindings of the same name carry distinct scope sets and never
/// alias accidentally. No α-renaming required.
fn substitute(form: &WatAST, target: &crate::scope::Identifier, replacement: &WatAST) -> WatAST {
    match form {
        WatAST::Symbol(ident, _) if ident == target => replacement.clone(),
        WatAST::List(items, span) => WatAST::List(
            items
                .iter()
                .map(|i| substitute(i, target, replacement))
                .collect(),
            span.clone(),
        ),
        // Arc 167 slice 1 — recurse into vector children so
        // textual substitution reaches binder targets buried in
        // a fn-sig vector (slice 2 territory).
        WatAST::Vector(items, span) => WatAST::Vector(
            items
                .iter()
                .map(|i| substitute(i, target, replacement))
                .collect(),
            span.clone(),
        ),
        // Arc 257 slice 1 — recurse into Map/Set children so substitution
        // reaches any symbol references inside map keys, values, or set elements.
        WatAST::Map(pairs, span) => WatAST::Map(
            pairs
                .iter()
                .map(|(k, v)| {
                    (
                        substitute(k, target, replacement),
                        substitute(v, target, replacement),
                    )
                })
                .collect(),
            span.clone(),
        ),
        WatAST::Set(items, span) => WatAST::Set(
            items
                .iter()
                .map(|i| substitute(i, target, replacement))
                .collect(),
            span.clone(),
        ),
        other => other.clone(),
    }
}

/// Fold-style multi-binder substitution. Used by match arm rewrite
/// where the matcher returns several binder→replacement pairs at once.
fn substitute_many(form: &WatAST, binds: &[(crate::scope::Identifier, WatAST)]) -> WatAST {
    binds
        .iter()
        .fold(form.clone(), |acc, (k, v)| substitute(&acc, k, v))
}

/// β-reduction step for user-defined functions registered at full
/// keyword path. Args descend leftmost-non-canonical until all are
/// canonical, then params get substituted by argument forms in the
/// body and the substituted body becomes the next form. Closures
/// (functions with `closed_env = Some`) need a different rule
/// (Phase 3) — they carry environment that textual substitution
/// can't reproduce.
fn step_user_call(
    items: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    head_kw: &str,
) -> Result<StepValue, EvalBreak> {
    let func = match sym.get(head_kw) {
        Some(f) => f.clone(),
        None => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::UnknownFunction(head_kw.to_string()),
            )
            .into())
        }
    };
    if func.closed_env.is_some() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::NoStepRule {
                op: format!("{} (closure-bearing — Phase 3)", head_kw),
            },
        )
        .into());
    }
    let args = &items[1..];
    if args.len() != func.params.len() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: head_kw.into(),
                expected: func.params.len(),
                got: args.len(),
            },
        )
        .into());
    }
    for (idx, arg) in args.iter().enumerate() {
        if !is_step_canonical(arg) {
            let new_arg = step_to_watast(arg, env, sym)?;
            let mut new_items: Vec<WatAST> = items.to_vec();
            new_items[idx + 1] = new_arg;
            return Ok(StepValue::Next(WatAST::List(new_items, list_span.clone())));
        }
    }
    // All canonical — substitute params for args in body.
    // Stone 255.1a — Native builtins have no wat body; they are never step-reduced.
    let mut new_body: WatAST = match &func.body {
        FunctionBody::Wat(ast) => (**ast).clone(),
        FunctionBody::Native => unreachable!(
            "native builtin fn-applied — dispatched via the runtime match, not fn-apply"
        ),
    };
    for (param, arg) in func.params.iter().zip(args.iter()) {
        // Arc 170 — substitute against the binder ITSELF, scopes included.
        new_body = substitute(&new_body, param, arg);
    }
    Ok(StepValue::Next(new_body))
}

// Arc 028 slice 3 — eval family iface drop + split eval-edn into
// eval-edn (string) and eval-file (path). First arg is now the
// source or the path directly; no :wat::eval::<iface> keyword.

/// `(:wat::eval-edn! <source>)` — parse + evaluate an inline
/// EDN source string at runtime. One arg.
fn eval_form_edn(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::eval-edn!".into(),
                reason: format!(
                    "(:wat::eval-edn! <source>) takes exactly 1 argument; got {}",
                    args.len()
                ),
            },
        )
        .into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let source = expect_string_value(":wat::eval-edn!", &args[0], env, sym)?;
        parse_and_run(&source, env, sym)
    })())
}

/// `(:wat::eval-file! <path>)` — read a file via the outer
/// loader, parse, evaluate at runtime. One arg. Separated from
/// eval-edn! so each form has one source shape (matching the
/// load! / load-string! split).
fn eval_form_file(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::eval-file!".into(),
                reason: format!(
                    "(:wat::eval-file! <path>) takes exactly 1 argument; got {}",
                    args.len()
                ),
            },
        )
        .into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let source = read_source_via_loader(":wat::eval-file!", &args[0], env, sym)?;
        parse_and_run(&source, env, sym)
    })())
}

/// `(:wat::eval-digest! <path>
///                             :wat::verify::digest-<algo>
///                             :wat::verify::<iface> <hex>)`
/// — verify SHA-256 (or sibling algo) of file bytes BEFORE parse,
/// then parse + evaluate. Four args.
fn eval_form_digest(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_digest_shared(args, env, sym, list_span, /*is_string*/ false)
}

/// `(:wat::eval-digest-string! <source>
///                                    :wat::verify::digest-<algo>
///                                    :wat::verify::<iface> <hex>)`
/// — same verification as `eval-digest!` but the source is inline.
/// No loader access needed. Four args.
fn eval_form_digest_string(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_digest_shared(args, env, sym, list_span, /*is_string*/ true)
}

fn eval_form_digest_shared(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
    is_string: bool,
) -> Result<Value, EvalBreak> {
    let op: &'static str = if is_string {
        ":wat::eval-digest-string!"
    } else {
        ":wat::eval-digest!"
    };
    if args.len() != 4 {
        let shape = if is_string { "<source>" } else { "<path>" };
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!(
                "({} {} :wat::verify::digest-<algo> :wat::verify::<iface> <hex>) takes exactly 4 arguments; got {}",
                op, shape, args.len()
            )
        }).into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let source = if is_string {
            expect_string_value(op, &args[0], env, sym)?
        } else {
            read_source_via_loader(op, &args[0], env, sym)?
        };
        let algo = parse_verify_algo_keyword(&args[1], "digest-", op)?;
        let hex = resolve_verify_payload(&args[2], &args[3], env, sym)?;
        crate::hash::verify_source_hash(source.as_bytes(), &algo, hex.trim()).map_err(|err| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::EvalVerificationFailed { err },
            )
        })?;
        parse_and_run(&source, env, sym)
    })())
}

/// `(:wat::eval-signed! <path>
///                             :wat::verify::signed-<algo>
///                             :wat::verify::<iface> <sig>
///                             :wat::verify::<iface> <pubkey>)`
/// — verify Ed25519 (or sibling algo) over canonical-EDN AFTER parse,
/// then run. Six args.
fn eval_form_signed(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_signed_shared(args, env, sym, list_span, /*is_string*/ false)
}

/// `(:wat::eval-signed-string! <source>
///                                    :wat::verify::signed-<algo>
///                                    :wat::verify::<iface> <sig>
///                                    :wat::verify::<iface> <pubkey>)`
/// — same verification as `eval-signed!` but the source is inline.
/// Six args.
fn eval_form_signed_string(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_signed_shared(args, env, sym, list_span, /*is_string*/ true)
}

fn eval_form_signed_shared(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
    is_string: bool,
) -> Result<Value, EvalBreak> {
    let op: &'static str = if is_string {
        ":wat::eval-signed-string!"
    } else {
        ":wat::eval-signed!"
    };
    if args.len() != 6 {
        let shape = if is_string { "<source>" } else { "<path>" };
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!(
                "({} {} :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pubkey>) takes exactly 6 arguments; got {}",
                op, shape, args.len()
            )
        }).into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let source = if is_string {
            expect_string_value(op, &args[0], env, sym)?
        } else {
            read_source_via_loader(op, &args[0], env, sym)?
        };
        let algo = parse_verify_algo_keyword(&args[1], "signed-", op)?;
        let sig_b64 = resolve_verify_payload(&args[2], &args[3], env, sym)?;
        let pk_b64 = resolve_verify_payload(&args[4], &args[5], env, sym)?;
        let ast = parse_program(&source, op)?;
        crate::hash::verify_program_signature(&ast, &algo, sig_b64.trim(), pk_b64.trim()).map_err(
            |err| {
                RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::EvalVerificationFailed { err },
                )
            },
        )?;
        run_program(&ast, env, sym)
    })())
}

/// Evaluate a string-literal or string-expression arg and return
/// its :String value. Shared helper for eval-edn! and similar
/// forms that take an inline source / string payload directly.
pub(crate) fn expect_string_value(
    op: &'static str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, EvalBreak> {
    match eval_inner(arg, env, sym)?.value_owned() {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(
            arg.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Evaluate a path arg and read the file's source via the outer
/// SymbolTable's loader. Shared helper for eval-file!, eval-digest!,
/// eval-signed! — each takes its path directly as the first arg.
pub(crate) fn read_source_via_loader(
    op: &'static str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, EvalBreak> {
    let path = expect_string_value(op, arg, env, sym)?;
    let loader = sym.source_loader().ok_or_else(|| {
        RuntimeError::new(
            arg.span().clone(),
            RuntimeErrorKind::NoSourceLoader { op: op.into() },
        )
    })?;
    loader
        .fetch_source_file(&path, None)
        .map(|loaded| loaded.source)
        .map_err(|e| {
            EvalBreak::from(RuntimeError::new(
                arg.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: op.into(),
                    reason: format!("read {:?}: {:?}", path, e),
                },
            ))
        })
}

// Arc 028 slice 3 — resolve_eval_source retired alongside the
// :wat::eval::* keyword namespace. Each eval form now takes its
// source directly: eval-edn! a string, eval-file!/digest/signed
// a path (read via the outer loader by read_source_via_loader).

/// Resolve a `:wat::verify::<iface> <locator>` pair to a payload string.
/// Verify payloads retain the two-shape keyword dispatch because the
/// verification location can be inline (`:wat::verify::string`) or a
/// sidecar file (`:wat::verify::file-path`).
pub(crate) fn resolve_verify_payload(
    iface_ast: &WatAST,
    locator_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, EvalBreak> {
    let iface = match iface_ast {
        WatAST::Keyword(k, _) => k.as_str(),
        other => {
            return Err(RuntimeError::new(
                iface_ast.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::verify::<iface>".into(),
                    reason: format!(
                        "verify payload interface must be a :wat::verify::<iface> keyword; got {}",
                        other.variant_name()
                    ),
                },
            )
            .into());
        }
    };
    match iface {
        ":wat::verify::string" => match eval_inner(locator_ast, env, sym)?.value_owned() {
            Value::String(s) => Ok((*s).clone()),
            other => Err(RuntimeError::new(locator_ast.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::verify::string".into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other))
            }).into()),
        },
        ":wat::verify::file-path" => match eval_inner(locator_ast, env, sym)?.value_owned() {
            Value::String(s) => {
                let loader = sym.source_loader().ok_or_else(|| {
                    RuntimeError::new(locator_ast.span().clone(), RuntimeErrorKind::NoSourceLoader {
                        op: ":wat::verify::file-path".into()
                    })
                })?;
                loader.fetch_payload_file(&s, None)
                    .map_err(|e| EvalBreak::from(RuntimeError::new(locator_ast.span().clone(), RuntimeErrorKind::MalformedForm {
                        head: ":wat::verify::file-path".into(),
                        reason: format!("read {:?}: {:?}", s, e)
                    })))
            }
            other => Err(RuntimeError::new(locator_ast.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::verify::file-path".into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other))
            }).into()),
        },
        ":wat::verify::http-path" | ":wat::verify::s3-path" => {
            Err(RuntimeError::new(iface_ast.span().clone(), RuntimeErrorKind::MalformedForm {
                head: iface.to_string(),
                reason: format!(
                    "verify payload interface {} is reserved but not implemented in this build",
                    iface
                )
            }).into())
        }
        other => Err(RuntimeError::new(iface_ast.span().clone(), RuntimeErrorKind::MalformedForm {
            head: iface.to_string(),
            reason: format!(
                "unknown verify payload interface {}; expected :wat::verify::string or :wat::verify::file-path",
                other
            )
        }).into()),
    }
}

/// Parse a `:wat::verify::<kind>-<algo>` keyword and extract the algo.
/// `expected_kind` is `"digest-"` or `"signed-"` depending on which
/// form called this.
pub(crate) fn parse_verify_algo_keyword(
    ast: &WatAST,
    expected_kind: &str,
    form: &str,
) -> Result<String, EvalBreak> {
    let kw = match ast {
        WatAST::Keyword(k, _) => k.as_str(),
        other => {
            return Err(RuntimeError::new(
                ast.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: form.into(),
                    reason: format!(
                    "verification algorithm must be a :wat::verify::<kind>-<algo> keyword; got {}",
                    other.variant_name()
                ),
                },
            )
            .into());
        }
    };
    let stripped = kw.strip_prefix(":wat::verify::").ok_or_else(|| {
        RuntimeError::new(
            ast.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: form.into(),
                reason: format!(
                    "verification algorithm keyword must start with :wat::verify::; got {}",
                    kw
                ),
            },
        )
    })?;
    let algo = stripped.strip_prefix(expected_kind).ok_or_else(|| {
        RuntimeError::new(
            ast.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: form.into(),
                reason: format!(
                    "this form expects a :wat::verify::{}<algo> keyword; got {}",
                    expected_kind, kw
                ),
            },
        )
    })?;
    if algo.is_empty() {
        return Err(RuntimeError::new(
            ast.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: form.into(),
                reason: format!("no algorithm named after {}", expected_kind),
            },
        )
        .into());
    }
    Ok(algo.to_string())
}

/// Parse a source string into one or more top-level forms.
pub(crate) fn parse_program(source: &str, form: &str) -> Result<Vec<WatAST>, EvalBreak> {
    crate::parser::parse_all_with_file(source, "<runtime-eval>").map_err(|e| {
        EvalBreak::from(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: form.into(),
                reason: format!("parse error: {}", e),
                // arc 138: no — parsing a raw string; no WatAST call-site in scope
            },
        ))
    })
}

/// Parse a source string and evaluate all forms in sequence under the
/// constrained-eval discipline. Returns the value of the last form
/// (or Unit if the program was empty).
pub(crate) fn parse_and_run(source: &str, env: &Environment, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let forms = parse_program(source, ":wat::eval-edn!")?;
    run_program(&forms, env, sym)
}

/// Run a sequence of pre-parsed forms under the constrained-eval
/// discipline: each form has mutation heads refused before execution.
pub(crate) fn run_program(forms: &[WatAST], env: &Environment, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let mut last = Value::Unit;
    for form in forms {
        last = run_constrained(form, env, sym)?;
    }
    Ok(last)
}

/// Refuse mutation forms in the given AST, then delegate to the
/// normal `eval` dispatcher against the (frozen) symbol table.
pub(crate) fn run_constrained(ast: &WatAST, env: &Environment, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    refuse_mutation_forms_in(ast)?;
    eval_inner(ast, env, sym).map(|tv| tv.value_owned())
}

fn refuse_mutation_forms_in(ast: &WatAST) -> Result<(), EvalBreak> {
    if let WatAST::List(items, list_span) = ast {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_mutation_head(head) {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::EvalForbidsMutationForm { head: head.clone() },
                )
                .into());
            }
        }
        for child in items {
            refuse_mutation_forms_in(child)?;
        }
    }
    Ok(())
}

fn is_mutation_head(head: &str) -> bool {
    matches!(
        head,
        // Stone 241.16 — `:wat::core::define` arm DELETED. HARD CUT is total;
        // define is no longer a recognized mutation head at eval time.
        ":wat::core::defmacro"
            // Stone 241.8 — defstruct replaces struct (HARD CUT).
            //
            // Stone 255.1a-β-i-b — KEPT. `defstruct` is a stdlib `defmacro` that `expand_all`
            // rewrites to `structtype`, but this function guards `eval-ast!`
            // (`refuse_mutation_forms_in` → `run_constrained` → `eval_form_ast`), which evaluates
            // user-supplied AST that was never macro-expanded. Measured: `(:wat::eval-ast!
            // '(:wat::core::defstruct …))` still answers "eval refused mutation form:
            // :wat::core::defstruct" — the literal head reaches this guard, so the arm is
            // load-bearing. Do not remove it in a future "finish the defstruct sweep" pass.
            | ":wat::core::defstruct"
            // Arc 293.2-parity — structtype is the low-level primitive defstruct (macro) expands to.
            | ":wat::core::structtype"
            // Stone 241.9 — defenum replaces enum (HARD CUT).
            | ":wat::core::defenum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            | ":wat::load-file!"
            | ":wat::digest-load!"
            | ":wat::signed-load!"
    ) || head.starts_with(":wat::config::set-")
}

// ─── Arc 214 Stone 4.6a-ii — peer verb eval arms ─────────────────────────────
//
// PARTITION — CLAUSE vs INTRINSIC (see docs/DISPATCH.md + check.rs ~4814+):
// All four are intrinsic; the eval dispatch here is a Rust match on the
// `RustOpaque.type_path` sentinel — that is fine (the rubric governs the
// *type-check* mechanism; intrinsics are custom Rust by definition).
//
// Pattern: eval args[0] → try downcast as Thread' first → else try Process' →
// else TypeMismatch. Thread' passes Value through; Process' bridges via EDN.
// The Option wrap added in Stone 4.6a-ii lets close' consume the peer and
// lets send'/recv' detect use-after-close (None → RuntimeError).

// §7 wire-wall (OUTBOUND): a bare `Nature::Struct` value must not be WRITTEN to a
// ── RETIRED arc 293.W.2a (deleted by arc 293.W.2d) ───────────────────────────
// `reject_non_portable_on_wire` — deleted. The §7 runtime send-side guard that
// refused a bare struct at the wire-serialize step is superseded by the
// compile-time purity wall at wire-peer PRODUCERS (peer-pair',
// connect', accept', program-self-peer'). A struct can no longer be typed into
// a wire peer at CHECK time; the runtime path is no longer reachable. The two
// call sites (PROCESS branch and socket-tier PEER' branch of eval_peer_send_prime)
// were removed. Symmetric to the decode backstop retirement in edn/render.rs.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_send_prime` moved to
// `src/kernel/message.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_try_send_prime` moved to
// `src/kernel/message.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_pid` moved to
// `src/kernel/identity.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

/// `(:wat::kernel::recv peer)` — Stone 4.6a-ii / arc 258.5b.
///
/// Thread': `peer.recv()` → Value.
/// Process': `peer.recv()` → decode EDN String → Value via the self-describing wire.
/// RecvError (peer closed / child gone) → RuntimeError.
/// Use-after-close (Option is None) → RuntimeError.
///
/// The `-> :T` ascription form is KILLED (arc 258.5b). `recv'` is 1-arg only.
/// The EDN wire is self-describing (post-234.7: tagged records/structs/enums + typed
/// scalars) so `decode_trusted_wire(edn, sym.types())` reconstructs the exact value
/// with no declared target type. `-> :T` in a non-return position is illegal.
/// Arc 278 no-hidden-failures — detect the reserved PROTOCOL-TIER failure reply.
///
/// `synthesize_surface_protocol` (types.rs) mints a reserved variant
/// `<S>::Reply::Failed [cause <- :wat::kernel::Failure]` on every serviceable surface's
/// `Reply` enum — the floor BELOW the per-op `<Op>Response` outcome enums: a client
/// message that never hydrates to ANY op cannot be carried by an op's response, so the
/// serve loop replies `Reply::Failed[cause]` to that client and keeps serving. `recv'`
/// surfaces it HERE as a catchable raise carrying the cause's rich reason (`unknown tag
/// #probe/Note … no matching struct or enum …`), so the caller is NEVER left blind. This
/// is the ONE uniform surfacing point — it covers both the Path-B intrinsic peer-method
/// dispatch and the defservice-generated client methods (both round-trip through `recv'`),
/// and it is CATCHABLE (a wat-level `assertion-failed!` in a client method would be an
/// uncatchable `panic_any`). Returns the reason when `v` IS a `*::Reply::Failed`, else None.
pub(crate) fn reply_failed_reason(v: &Value) -> Option<String> {
    let Value::Enum(e) = v else { return None };
    if !(e.type_path.ends_with("::Reply") && e.variant_name == "Failed") {
        return None;
    }
    // field[0] is the `:wat::kernel::Failure` record; ITS field[0] is the mandatory
    // `error` (a `:wat::core::Error`, canonically a Fault) whose OWN field[0] is the
    // `message` String (arc 278 — Failure carries the error structurally; the Fault's
    // fields are [message, location, causes]).
    match e.fields.first() {
        Some(Value::Aggregate(a)) if a.class.as_ref() == "wat::kernel::Failure" => match a.fields.first() {
            // The `error` field: read its `message` (Fault field[0]).
            Some(Value::Aggregate(err)) => match err.fields.first() {
                Some(Value::String(s)) => Some((**s).clone()),
                _ => Some(
                    "service replied Reply::Failed (protocol-tier decode failure) with an \
                     unreadable cause"
                        .to_string(),
                ),
            },
            _ => Some(
                "service replied Reply::Failed (protocol-tier decode failure) with an \
                 unreadable cause"
                    .to_string(),
            ),
        },
        Some(other) => Some(format!(
            "service replied Reply::Failed (protocol-tier decode failure); cause: {:?}",
            other
        )),
        None => Some(
            "service replied Reply::Failed (protocol-tier decode failure) with no cause"
                .to_string(),
        ),
    }
}

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_recv_prime` moved to
// `src/kernel/message.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_close_prime` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_signal` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_process` moved to
// `src/kernel/identity.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_wire` moved to
// `src/kernel/identity.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_address_wire` moved to
// `src/kernel/identity.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_require_wire_address` moved to
// `src/kernel/identity.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_peer_select_prime` moved to
// `src/kernel/message.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// `(:wat::kernel::poll self-peer listener peers)` — Arc 209 Stone C0b.1b / C0b.2e-i-c.
//
// 3-arg service-multiplexer form: multiplexes THREE inputs — the **self-peer**
// (owner/supervisor link → `:Shutdown`), the **listener** (new connections),
// and the **connected client `Peer'`s** (requests) — returning a `(ServiceEvent :- [I O])`.
//
// Registration order (= Select index):
//   0 = self-peer `.rx`  (= `input_rx`; wakes when owner drops the handle via RAII drain)
//   1 = listener receiver (new connections)
//   2..=N+1 = client peers[0..N-1] `.rx`
//
// Select outcome mapping:
//   `Recv { index: 0, .. }`       → `ServiceEvent::Shutdown`  (owner dropped; RAII drain fired)
//   `Recv { index: 1, result }`   → unpack + wrap → `ServiceEvent::Connection { peer }`
//   `Recv { index: k, result }`, k≥2:
//     `Ok(msg)`  → `ServiceEvent::Message { idx: k-2, msg }`
//     `Err(_)`   → `ServiceEvent::Closed  { idx: k-2 }`
//
// Thread tier only. Uses existing `comms::thread::Select` (no `comms/thread.rs` change).
// `wrap_connect_request` is reused from `accept'` — ONE helper, THREE callers.

// ─── after — one-shot timer peer ─────────────────────────────────────────────

// Arc 109 Stone B — the seven kernel sub-modules — `eval_kernel_after` moved to
// `src/kernel/resource.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_kernel_serve_dispatch_op_tail` moved to
// `src/kernel/serve.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

// Arc 255.1c-kernel-remainder (home #8) — the `eval_inner`-based non-tail companion
// (`eval_kernel_serve_dispatch_op`) that used to live HERE is DELETED, not merely
// unregistered. It was "defensive parity… reached only if `serve-dispatch-op'` is ever
// evaluated outside serve's tail position… the codegen never places it anywhere else" per
// its own doc — already dead in practice. With both literal match arms gone (the tail arm
// in `eval_tail` and the non-tail arm above), `:wat::kernel::serve-dispatch-op` has exactly
// ONE dispatch path left: the intrinsic registry, which registers
// `eval_kernel_serve_dispatch_op_tail` (unchanged, still above this comment) for the FQDN —
// there is no second call shape remaining for a "non-tail companion" to be parity FOR.
// Keeping it would have been unreachable duplicate code. See
// `src/intrinsic/kernel/serve.rs`'s doc for the full derivation, including why the tail
// delegate is the correct (and only safe) choice to preserve `serve`'s TCO when reached
// through the registry's generic fallback path.

// Arc 109 Stone B — the seven kernel sub-modules — `eval_poll_prime` moved to
// `src/kernel/message.rs` (docs/arc/2026/04/109-kill-std/). Behaviour unchanged.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::snapshot_call_stack;
    use crate::config::Config;
    use std::sync::OnceLock;

    /// The stdlib is the standard library — always available, without
    /// ceremony. Test harnesses load it once per process via
    /// `OnceLock`, then clone the resulting SymbolTable / MacroRegistry
    /// / TypeEnv per test. This mirrors what `startup_from_source` does
    /// at the stdlib phase, minus the user-source passes.
    ///
    /// Without this, `run` and `eval_expr` would hand back bare
    /// `SymbolTable::new()` values where `:wat::std::*` names resolve
    /// to `UnknownFunction` — dishonest framing of what "standard
    /// library" means.
    ///
    /// Delegates to the canonical [`crate::freeze::env::build_env`]
    /// pipeline so the test environment CANNOT drift from production.
    /// The 13 check::tests:: failures that fired before arc-293's
    /// extirpare were caused by the old copy discarding the stdlib
    /// residue and therefore skipping `preregister_stdlib_defclause_stub`
    /// + `register_stdlib_runtime_defs`. One pipeline, no drift.
    fn stdlib_loaded() -> &'static (
        SymbolTable,
        crate::macros::MacroRegistry,
        crate::types::TypeEnv,
    ) {
        static LOADED: OnceLock<(
            SymbolTable,
            crate::macros::MacroRegistry,
            crate::types::TypeEnv,
        )> = OnceLock::new();
        LOADED.get_or_init(|| {
            let b = crate::freeze::env::build_env(vec![]).expect("stdlib env builds");
            (b.symbols, b.macros, b.types)
        })
    }

    fn run(src: &str) -> Result<Value, EvalBreak> {
        let (stdlib_sym, stdlib_macros, stdlib_types) = stdlib_loaded();
        let mut macros = stdlib_macros.clone();
        let forms = crate::parse_all!(src).expect("parse ok");
        // Expand any stdlib-macro calls in the user source before
        // registering defines and evaluating.
        // LOAD-BEARING ORDER: expand_all must run before user-defn registration — see src/macros/eval.rs module doc + freeze.rs expand_runs_before_register_defines_phase_order
        let expanded =
            crate::macros::expand_all(forms, &mut macros, &Environment::new(), stdlib_sym)
                .expect("macro expansion");
        let mut sym = stdlib_sym.clone();
        let rest = register_defines(expanded, &mut sym)?;
        // Arc 071 follow-up — type-check the program before
        // evaluating. Mirrors what freeze.rs:580 (the lab harness's
        // path) does. Any type-system bug visible at a use site
        // surfaces here instead of escaping to a downstream
        // consumer. Uses stdlib's TypeEnv (no user-source type
        // declarations are honored — `run` deliberately doesn't
        // accept those).
        if let Err(errors) = crate::check::check_program(&rest, &sym, stdlib_types) {
            panic!("type-check errors in test wat:\n{}", errors);
        }
        let env = Environment::new();
        let mut last = Value::Unit;
        for form in &rest {
            // Stone 241.11 — `defn` macro-expands to `(:wat::core::def ...)` which
            // `register_defines` pre-registers into `sym` and leaves in `rest`
            // for the freeze path's `register_runtime_defs` step. In this unit-test
            // `run()` helper we evaluate forms directly (no freeze path), so skip
            // declaration forms that are already pre-registered — evaluating them
            // again would hit `DeclarationInExpressionPosition`.
            // Stone 241.14 — def-restricted removed from this guard (HARD CUT).
            if let WatAST::List(items, _) = form {
                if let Some(WatAST::Keyword(head, _)) = items.first() {
                    if matches!(head.as_str(), ":wat::core::def") {
                        continue;
                    }
                }
            }
            last = eval_inner(form, &env, &sym)?.value_owned();
        }
        Ok(last)
    }

    fn eval_expr(src: &str) -> Result<Value, EvalBreak> {
        let (stdlib_sym, stdlib_macros, _) = stdlib_loaded();
        let mut macros = stdlib_macros.clone();
        let ast = crate::parse_one!(src).expect("parse ok");
        // LOAD-BEARING ORDER: expand_all must run before user-defn registration — see src/macros/eval.rs module doc + freeze.rs expand_runs_before_register_defines_phase_order
        let expanded =
            crate::macros::expand_all(vec![ast], &mut macros, &Environment::new(), stdlib_sym)
                .expect("macro expansion");
        let ast = expanded
            .into_iter()
            .next()
            .expect("one form in, one form out");
        eval_inner(&ast, &Environment::new(), stdlib_sym).map(|tv| tv.value_owned())
    }

    /// Same as [`eval_expr`] but clones the shared stdlib SymbolTable
    /// and attaches a real filesystem loader. Tests that exercise
    /// `:wat::eval-file!` or the file-path variants of the verified
    /// eval/load forms (or `:wat::verify::file-path` payloads) need
    /// the capability explicitly — arc 007 closed the direct-fs bypass,
    /// so the loader must be announced per call site.
    fn eval_expr_with_fs(src: &str) -> Result<Value, EvalBreak> {
        let (stdlib_sym, stdlib_macros, _) = stdlib_loaded();
        let mut macros = stdlib_macros.clone();
        let mut sym = stdlib_sym.clone();
        sym.set_source_loader(std::sync::Arc::new(crate::load::loader::FsLoader));
        let ast = crate::parse_one!(src).expect("parse ok");
        // LOAD-BEARING ORDER: expand_all must run before user-defn registration — see src/macros/eval.rs module doc + freeze.rs expand_runs_before_register_defines_phase_order
        let expanded = crate::macros::expand_all(vec![ast], &mut macros, &Environment::new(), &sym)
            .expect("macro expansion");
        let ast = expanded
            .into_iter()
            .next()
            .expect("one form in, one form out");
        eval_inner(&ast, &Environment::new(), &sym).map(|tv| tv.value_owned())
    }

    // ─── Arc 278 "errors first-class EDN" (stone 1) — the acceptance gate ──
    //
    // The cache-probe RED gate: a process-tier startup failure reproducing the
    // cache probe (a startup-time unknown-function call → `RuntimeError::
    // UnknownFunction` wrapped in `StartupError`). The emitted
    // `LociDiedError/StartupError` chain MUST be a fully-structured, navigable
    // EDN tree — the cause a real `#wat.runtime/UnknownFunction` RECORD (typed,
    // STRICT-decoded) with its own `:message` headline, a real `:location`, its
    // `:path` coordinate, and `:causes` — ZERO escaped-EDN-inside-a-String.
    //
    // RED before the fix: `startup_error_chain_edn` string-wrapped the cause via
    // `to_wire_edn`, so the chain rendered
    // `[#wat.kernel.LociDiedError/StartupError ["#wat.runtime/UnknownFunction {…}"]]`
    // and the cause STRICT-decoded to a `Value::String` (the mask). GREEN after:
    // the cause is emitted as the `error_edn()` record and decodes to a typed
    // `Value::Aggregate`.
    #[test]
    fn cache_probe_startup_error_is_navigable_edn_not_string() {
        use crate::freeze::StartupError;

        // The cache-probe failure: a startup-time unknown-function call.
        let re = RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::UnknownFunction(":wat::kernel::typo".into()),
        );
        let e = StartupError::Runtime(Box::new(re));

        // What the dying process child writes on fd 2 (captured without a fork).
        let chain_edn = crate::process::verbs::startup_error_chain_edn(&e);
        let line = wat_edn::write(&chain_edn);

        // The owner's recv' Lost decoder STRICT-decodes the chain.
        let types = crate::types::TypeEnv::with_builtins();
        let parsed = wat_edn::parse_owned(&line).expect("emitted chain must parse");
        let decoded = crate::edn::render::edn_to_value(&parsed, Some(&types), None).unwrap_or_else(|err| {
            panic!("the emitted StartupError chain must STRICT-decode to typed records; got {err:?}\n  line: {line}")
        });

        // chain = (Vector :- [LociDiedError]); head = StartupError.
        let chain = match &decoded {
            Value::Vec(items) => items,
            other => panic!("chain must be a (Vector :- [LociDiedError]); got {other:?}"),
        };
        assert_eq!(chain.len(), 1, "one death in the chain; line: {line}");
        let ev = match &chain[0] {
            Value::Enum(ev) => ev,
            other => panic!("chain head must be a LociDiedError enum; got {other:?}"),
        };
        assert_eq!(
            ev.type_path, ":wat::kernel::LociDiedError",
            "head is a LociDiedError"
        );
        assert_eq!(ev.variant_name, "StartupError");

        // THE GATE: the cause is a fully-structured, navigable
        // #wat.runtime/UnknownFunction RECORD — NOT an escaped-EDN String.
        let cause = &ev.fields[0];
        let agg = match cause {
            Value::Aggregate(a) => a,
            Value::String(s) => panic!(
                "MASK: StartupError cause is a string-wrapped blob, not a typed record: {s:?}"
            ),
            other => panic!("StartupError cause must be a typed record; got {other:?}"),
        };
        assert_eq!(
            agg.class.as_ref(), "wat::runtime::UnknownFunction",
            "cause is the typed RuntimeError record"
        );

        // Floor + coordinate fields, in declaration order [message, location, causes, path].
        let field = |i: usize| {
            agg.fields
                .get(i)
                .unwrap_or_else(|| panic!("cause missing field {i}"))
        };
        // :message — a one-line headline (no file:line prefix, no embedded newline).
        match field(0) {
            Value::String(s) => assert_eq!(&**s, "unknown function: :wat::kernel::typo"),
            other => panic!(":message must be a String headline; got {other:?}"),
        }
        // :location — a REAL located #wat.core/Span record, never nil.
        match field(1) {
            Value::Aggregate(loc) => {
                assert_eq!(loc.class.as_ref(), "wat::core::Span", ":location is a typed Span")
            }
            other => panic!(":location must be a typed Span record (never nil); got {other:?}"),
        }
        // :causes — an empty Vector (this is a leaf error).
        match field(2) {
            Value::Vec(c) => assert!(c.is_empty(), ":causes is empty for a leaf error"),
            other => panic!(":causes must be a Vector; got {other:?}"),
        }
        // :path — the unknown-function coordinate, PRESERVED (not dropped by a
        // floor-only shortcut).
        match field(3) {
            Value::String(s) => {
                assert_eq!(&**s, ":wat::kernel::typo", ":path carries the unknown name")
            }
            other => panic!(":path must be a String; got {other:?}"),
        }
    }

    // ─── Literals ───────────────────────────────────────────────────────

    #[test]
    fn int_literal() {
        assert!(matches!(eval_expr("42").unwrap(), Value::i64(42)));
    }

    // ─── Arc 016 slice 2: call-stack population ─────────────────────────

    /// When `assertion-failed!` fires inside a user-defined function,
    /// the `AssertionPayload` carries the call's source span + the
    /// stack of enclosing user-function frames. This is the mechanism
    /// a later slice's panic hook uses to render Rust-style failure
    /// output pointing at the user's `.wat` source.
    #[test]
    fn call_stack_populates_on_assertion() {
        use crate::assertion::AssertionPayload;
        // Install the wat panic hook so the panic writes Rust-style
        // failure output to stderr (harmlessly captured by cargo
        // test) rather than Rust's default "thread X panicked" line.
        crate::panic_hook::install();

        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :my::app::failing-fn [] -> :() (:wat::kernel::assertion-failed! "stack test" :wat::core::None :wat::core::None))
        "#;
        let (stdlib_sym, stdlib_macros, _) = stdlib_loaded();
        let mut macros = stdlib_macros.clone();
        let forms = crate::parse_all!(src).expect("parse");
        // LOAD-BEARING ORDER: expand_all must run before user-defn registration — see src/macros/eval.rs module doc + freeze.rs expand_runs_before_register_defines_phase_order
        let expanded =
            crate::macros::expand_all(forms, &mut macros, &Environment::new(), stdlib_sym)
                .expect("expand");
        let mut sym = stdlib_sym.clone();
        let _ = register_defines(expanded, &mut sym).expect("register");
        let func = sym.get(":my::app::failing-fn").expect("defined").clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply_function(func, Vec::new(), &sym, crate::rust_caller_span!())
        }));

        let payload = match result {
            Ok(_) => panic!("expected panic, got Ok"),
            Err(p) => p,
        };
        let boxed = match payload.downcast::<AssertionPayload>() {
            Ok(b) => *b,
            Err(_) => panic!("expected AssertionPayload"),
        };

        // Location must be populated — the span of the call site
        // that invoked failing-fn.
        assert!(
            boxed.location.is_some(),
            "expected location to be populated; got None"
        );
        // Frames must contain at least one entry for failing-fn.
        assert!(!boxed.frames.is_empty(), "expected at least one frame");
        assert_eq!(
            boxed.frames[0].callee_path, ":my::app::failing-fn",
            "top frame should be the user-defined function"
        );
    }

    /// Call stack must unwind cleanly on every exit path. After
    /// `apply_function` returns, the stack should be empty. Tests the
    /// FrameGuard's Drop behavior.
    #[test]
    fn call_stack_unwinds_on_ok() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :my::app::plain-fn [] -> :i64 42)
        "#;
        let (stdlib_sym, stdlib_macros, _) = stdlib_loaded();
        let mut macros = stdlib_macros.clone();
        let forms = crate::parse_all!(src).expect("parse");
        // LOAD-BEARING ORDER: expand_all must run before user-defn registration — see src/macros/eval.rs module doc + freeze.rs expand_runs_before_register_defines_phase_order
        let expanded =
            crate::macros::expand_all(forms, &mut macros, &Environment::new(), stdlib_sym)
                .expect("expand");
        let mut sym = stdlib_sym.clone();
        let _ = register_defines(expanded, &mut sym).expect("register");
        let func = sym.get(":my::app::plain-fn").expect("defined").clone();

        assert_eq!(snapshot_call_stack().len(), 0, "stack must start empty");
        let v = apply_function(func, Vec::new(), &sym, crate::rust_caller_span!()).expect("call");
        assert!(matches!(v, Value::i64(42)));
        assert_eq!(
            snapshot_call_stack().len(),
            0,
            "stack must unwind cleanly after Ok return"
        );
    }

    #[test]
    fn float_literal() {
        match eval_expr("2.5").unwrap() {
            Value::f64(x) => assert_eq!(x, 2.5),
            v => panic!("expected float, got {:?}", v),
        }
    }

    #[test]
    fn bool_literals() {
        assert!(matches!(eval_expr("true").unwrap(), Value::bool(true)));
        assert!(matches!(eval_expr("false").unwrap(), Value::bool(false)));
    }

    #[test]
    fn string_literal() {
        match eval_expr(r#""hello""#).unwrap() {
            Value::String(s) => assert_eq!(&*s, "hello"),
            v => panic!("expected string, got {:?}", v),
        }
    }

    // ─── Arithmetic ─────────────────────────────────────────────────────

    #[test]
    fn add_ints() {
        assert!(matches!(
            eval_expr("(:wat::i64::+ 2 3)").unwrap(),
            Value::i64(5)
        ));
    }

    #[test]
    fn subtract_ints() {
        assert!(matches!(
            eval_expr("(:wat::i64::- 10 4)").unwrap(),
            Value::i64(6)
        ));
    }

    #[test]
    fn i64_mul_refuses_f64_arg() {
        // Post-split (2026-04-19): arith is strictly typed. the i64
        // namespace ops refuse any f64 argument — no silent promotion.
        // Users commit to the numeric tier at the call site; users who
        // want float math reach for the :wat::core::f64 namespace ops.
        let err = eval_expr("(:wat::i64::* 3 2.0)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn f64_mul_refuses_i64_arg() {
        let err = eval_expr("(:wat::f64::* 3.0 2)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn f64_mul_float_times_float() {
        match eval_expr("(:wat::f64::* 3.0 2.0)").unwrap() {
            Value::f64(x) => assert_eq!(x, 6.0),
            v => panic!("expected float, got {:?}", v),
        }
    }

    #[test]
    fn divide_by_zero_errors() {
        assert!(matches!(
            eval_expr("(:wat::i64::/ 5 0)"),
            Err(EvalBreak::Diagnostic(e)) if matches!(e.kind(), RuntimeErrorKind::DivisionByZero)
        ));
    }

    // ─── Scalar conversions (arc 014) ───────────────────────────────────

    fn expect_string(v: Value) -> String {
        match v {
            Value::String(s) => (*s).clone(),
            other => panic!("expected String, got {:?}", other),
        }
    }

    fn expect_i64(v: Value) -> i64 {
        match v {
            Value::i64(n) => n,
            other => panic!("expected i64, got {:?}", other),
        }
    }

    fn expect_f64(v: Value) -> f64 {
        match v {
            Value::f64(x) => x,
            other => panic!("expected f64, got {:?}", other),
        }
    }

    fn expect_some(v: Value) -> Value {
        match v {
            Value::Option(inner) => match &*inner {
                Some(x) => x.clone(),
                None => panic!("expected Some(_), got None"),
            },
            other => panic!("expected Option, got {:?}", other),
        }
    }

    fn expect_none(v: Value) {
        match v {
            Value::Option(inner) => match &*inner {
                None => {}
                Some(x) => panic!("expected None, got Some({:?})", x),
            },
            other => panic!("expected Option, got {:?}", other),
        }
    }

    #[test]
    fn i64_to_string_renders_value() {
        assert_eq!(
            expect_string(eval_expr("(:wat::i64::to-string 42)").unwrap()),
            "42"
        );
        assert_eq!(
            expect_string(eval_expr("(:wat::i64::to-string -7)").unwrap()),
            "-7"
        );
        assert_eq!(
            expect_string(eval_expr("(:wat::i64::to-string 0)").unwrap()),
            "0"
        );
    }

    #[test]
    fn i64_to_f64_widens_infallibly() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::i64::to-f64 42)").unwrap()),
            42.0
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::i64::to-f64 -3)").unwrap()),
            -3.0
        );
    }

    #[test]
    fn f64_to_string_renders_value() {
        assert_eq!(
            expect_string(eval_expr("(:wat::f64::to-string 2.5)").unwrap()),
            "2.5"
        );
        assert_eq!(
            expect_string(eval_expr("(:wat::f64::to-string -0.125)").unwrap()),
            "-0.125"
        );
    }

    #[test]
    fn f64_to_i64_truncates_in_range() {
        let some = expect_some(eval_expr("(:wat::f64::to-i64 3.75)").unwrap());
        assert_eq!(expect_i64(some), 3);
        let some = expect_some(eval_expr("(:wat::f64::to-i64 -2.5)").unwrap());
        assert_eq!(expect_i64(some), -2);
    }

    #[test]
    fn f64_to_i64_rejects_nan() {
        // Stone 237.8b — f64::/ now follows IEEE 754 (0.0/0.0 → NaN, no error).
        // Use i64::MAX overflow to test to-i64 range rejection instead.
        // i64::MAX ≈ 9.22e18; 1e19 is safely past.
        expect_none(eval_expr("(:wat::f64::to-i64 1e19)").unwrap());
        // And past i64::MIN on the negative side.
        expect_none(eval_expr("(:wat::f64::to-i64 -1e19)").unwrap());
    }

    // ─── f64::round (arc 019) ─────────────────────────────────────────────

    #[test]
    fn f64_round_to_zero_digits() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::round 1.00001 0)").unwrap()),
            1.0
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::round 1.5 0)").unwrap()),
            2.0
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::round -1.5 0)").unwrap()),
            -2.0
        );
    }

    #[test]
    fn f64_round_to_three_digits() {
        let v = expect_f64(eval_expr("(:wat::f64::round 12.1234 3)").unwrap());
        assert!((v - 12.123).abs() < 1e-12, "got {}", v);
    }

    #[test]
    fn f64_round_to_two_digits() {
        let v = expect_f64(eval_expr("(:wat::f64::round 4.5678 2)").unwrap());
        assert!((v - 4.57).abs() < 1e-12, "got {}", v);
    }

    #[test]
    fn f64_round_rejects_negative_digits() {
        let err = eval_expr("(:wat::f64::round 15.0 -1)").unwrap_err();
        match err {
            EvalBreak::Diagnostic(e) => match e.kind() {
                RuntimeErrorKind::MalformedForm { head, reason, .. } => {
                    assert_eq!(head, ":wat::f64::round");
                    assert_eq!(
                        reason,
                        "`digits` must be non-negative; got -1. Negative digits (round to nearest 10 / 100 / ...) has no load-bearing use case today"
                    );
                }
                other => panic!("expected MalformedForm, got {:?}", other),
            },
            other => panic!("expected MalformedForm, got {:?}", other),
        }
    }

    #[test]
    fn f64_round_arity_mismatch() {
        let err = eval_expr("(:wat::f64::round 1.0)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── f64::max / min / abs / clamp + math::exp (arc 046) ───────────────

    #[test]
    fn f64_max_picks_larger() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::max 1.0 2.0)").unwrap()),
            2.0
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::max -3.0 -5.0)").unwrap()),
            -3.0
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::max 4.2 4.2)").unwrap()),
            4.2
        );
    }

    #[test]
    fn f64_min_picks_smaller() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::min 1.0 2.0)").unwrap()),
            1.0
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::min -3.0 -5.0)").unwrap()),
            -5.0
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::min 4.2 4.2)").unwrap()),
            4.2
        );
    }

    #[test]
    fn f64_abs_handles_sign_and_zero() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::abs 3.5)").unwrap()),
            3.5
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::abs -3.5)").unwrap()),
            3.5
        );
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::abs 0.0)").unwrap()),
            0.0
        );
    }

    #[test]
    fn f64_abs_rejects_i64() {
        let err = eval_expr("(:wat::f64::abs 5)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn f64_clamp_in_range_unchanged() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::clamp 0.5 -1.0 1.0)").unwrap()),
            0.5
        );
    }

    #[test]
    fn f64_clamp_below_lo_lifts() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::clamp -5.0 -1.0 1.0)").unwrap()),
            -1.0
        );
    }

    #[test]
    fn f64_clamp_above_hi_caps() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::clamp 5.0 -1.0 1.0)").unwrap()),
            1.0
        );
    }

    #[test]
    fn f64_clamp_lo_equals_hi_pins() {
        assert_eq!(
            expect_f64(eval_expr("(:wat::f64::clamp 5.0 2.0 2.0)").unwrap()),
            2.0
        );
    }

    #[test]
    fn f64_clamp_rejects_lo_greater_than_hi() {
        let err = eval_expr("(:wat::f64::clamp 0.0 1.0 -1.0)").unwrap_err();
        match err {
            EvalBreak::Diagnostic(e) => match e.kind() {
                RuntimeErrorKind::MalformedForm { head, reason, .. } => {
                    assert_eq!(head, ":wat::f64::clamp");
                    assert_eq!(
                        reason,
                        "lo must be ≤ hi and neither may be NaN; got lo=1, hi=-1"
                    );
                }
                other => panic!("expected MalformedForm, got {:?}", other),
            },
            other => panic!("expected MalformedForm, got {:?}", other),
        }
    }

    #[test]
    fn f64_clamp_arity_mismatch() {
        let err = eval_expr("(:wat::f64::clamp 1.0 0.0)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn math_exp_round_trips_with_ln() {
        // exp(0) == 1.0 exactly.
        assert_eq!(
            expect_f64(eval_expr("(:wat::math::exp 0.0)").unwrap()),
            1.0
        );
        // exp(1) ≈ e.
        let v = expect_f64(eval_expr("(:wat::math::exp 1.0)").unwrap());
        assert!((v - std::f64::consts::E).abs() < 1e-12, "got {}", v);
        // exp(-1) ≈ 1/e.
        let v = expect_f64(eval_expr("(:wat::math::exp -1.0)").unwrap());
        assert!((v - (1.0 / std::f64::consts::E)).abs() < 1e-12, "got {}", v);
    }

    #[test]
    fn math_exp_accepts_i64_promotion() {
        // :wat::math:: permits i64 → f64 promotion (matches
        // ln/sin/cos siblings); :wat::core::f64 namespace does not.
        assert_eq!(
            expect_f64(eval_expr("(:wat::math::exp 0)").unwrap()),
            1.0
        );
    }

    #[test]
    fn string_to_i64_parses_valid_input() {
        let some = expect_some(eval_expr(r#"(:wat::string::to-i64 "42")"#).unwrap());
        assert_eq!(expect_i64(some), 42);
        let some = expect_some(eval_expr(r#"(:wat::string::to-i64 "-7")"#).unwrap());
        assert_eq!(expect_i64(some), -7);
    }

    #[test]
    fn string_to_i64_returns_none_for_unparseable() {
        expect_none(eval_expr(r#"(:wat::string::to-i64 "abc")"#).unwrap());
        expect_none(eval_expr(r#"(:wat::string::to-i64 "")"#).unwrap());
        expect_none(eval_expr(r#"(:wat::string::to-i64 " 42 ")"#).unwrap());
    }

    #[test]
    fn string_to_f64_parses_valid_input() {
        let some = expect_some(eval_expr(r#"(:wat::string::to-f64 "2.5")"#).unwrap());
        assert_eq!(expect_f64(some), 2.5);
    }

    #[test]
    fn string_to_f64_returns_none_for_unparseable() {
        expect_none(eval_expr(r#"(:wat::string::to-f64 "abc")"#).unwrap());
    }

    #[test]
    fn bool_to_string_renders_true_false() {
        assert_eq!(
            expect_string(eval_expr("(:wat::core::bool::to-string true)").unwrap()),
            "true"
        );
        assert_eq!(
            expect_string(eval_expr("(:wat::core::bool::to-string false)").unwrap()),
            "false"
        );
    }

    #[test]
    fn string_to_bool_parses_valid_input() {
        let some = expect_some(eval_expr(r#"(:wat::string::to-bool "true")"#).unwrap());
        assert!(matches!(some, Value::bool(true)));
        let some = expect_some(eval_expr(r#"(:wat::string::to-bool "false")"#).unwrap());
        assert!(matches!(some, Value::bool(false)));
    }

    #[test]
    fn string_to_bool_returns_none_for_unparseable() {
        expect_none(eval_expr(r#"(:wat::string::to-bool "True")"#).unwrap());
        expect_none(eval_expr(r#"(:wat::string::to-bool "1")"#).unwrap());
        expect_none(eval_expr(r#"(:wat::string::to-bool "")"#).unwrap());
    }

    #[test]
    fn i64_string_roundtrip() {
        let s = eval_expr("(:wat::i64::to-string 12345)").unwrap();
        let s_lit = match s {
            Value::String(s) => format!("\"{}\"", s),
            _ => panic!("expected String"),
        };
        let round =
            expect_some(eval_expr(&format!("(:wat::string::to-i64 {})", s_lit)).unwrap());
        assert_eq!(expect_i64(round), 12345);
    }

    #[test]
    fn conversions_reject_wrong_input_type() {
        // Type checker catches these at startup — but the runtime
        // handlers also reject wrong-type inputs defensively. Call
        // through the raw dispatch to bypass check.
        let err = eval_expr("(:wat::i64::to-string 2.5)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
        let err = eval_expr(r#"(:wat::f64::to-string "abc")"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    // ─── Comparison ─────────────────────────────────────────────────────

    #[test]
    fn equality() {
        assert!(matches!(
            eval_expr("(:wat::core::= 3 3)").unwrap(),
            Value::bool(true)
        ));
        assert!(matches!(
            eval_expr("(:wat::core::= 3 4)").unwrap(),
            Value::bool(false)
        ));
    }

    #[test]
    fn less_than() {
        // Stone 237.8b — polymorphic `<` is now a defclause; use per-Type primitive in unit test.
        assert!(matches!(
            eval_expr("(:wat::i64::< 2 3)").unwrap(),
            Value::bool(true)
        ));
        assert!(matches!(
            eval_expr("(:wat::i64::< 3 2)").unwrap(),
            Value::bool(false)
        ));
    }

    // ─── Boolean ────────────────────────────────────────────────────────

    #[test]
    fn and_short_circuits() {
        assert!(matches!(
            eval_expr("(:wat::core::and true false true)").unwrap(),
            Value::bool(false)
        ));
    }

    #[test]
    fn or_short_circuits() {
        assert!(matches!(
            eval_expr("(:wat::core::or false false true false)").unwrap(),
            Value::bool(true)
        ));
    }

    #[test]
    fn not_bool() {
        assert!(matches!(
            eval_expr("(:wat::core::not true)").unwrap(),
            Value::bool(false)
        ));
    }

    // ─── Control flow ───────────────────────────────────────────────────

    #[test]
    fn if_true_branch() {
        assert!(matches!(
            eval_expr("(:wat::core::if true 1 2)").unwrap(),
            Value::i64(1)
        ));
    }

    #[test]
    fn if_false_branch() {
        assert!(matches!(
            eval_expr("(:wat::core::if false 1 2)").unwrap(),
            Value::i64(2)
        ));
    }

    #[test]
    fn if_non_bool_rejected() {
        assert!(matches!(
            eval_expr("(:wat::core::if 42 1 2)"),
            Err(EvalBreak::Diagnostic(e)) if matches!(e.kind(), RuntimeErrorKind::BadCondition { .. })
        ));
    }

    #[test]
    fn let_binds_parallel() {
        assert!(matches!(
            eval_expr(r#"(:wat::core::let [x 2 y 3] (:wat::i64::+ x y))"#).unwrap(),
            Value::i64(5)
        ));
    }

    #[test]
    fn let_shadows_outer() {
        // Inner let shadows the outer x.
        assert!(matches!(
            eval_expr(r#"(:wat::core::let [x 1] (:wat::core::let [x 100] x))"#).unwrap(),
            Value::i64(100)
        ));
    }

    // ─── Define + function call ─────────────────────────────────────────

    #[test]
    fn define_and_call() {
        let result = run(
            r#"
            (:wat::core::defn :my::app::inc [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1))
            (:my::app::inc 41)
            "#,
        )
        .unwrap();
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn define_recursive_factorial() {
        let result = run(r#"
            (:wat::core::defn :my::app::fact [n <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::if (:wat::core::= n 0)
                                1
                                (:wat::i64::* n (:my::app::fact (:wat::i64::- n 1)))))
            (:my::app::fact 5)
            "#)
        .unwrap();
        assert!(matches!(result, Value::i64(120)));
    }

    #[test]
    fn reserved_prefix_define_rejected() {
        // Stone 241.11 — use FQDN types (:wat::core::i64) since defn
        // goes through check_program which rejects bare :i64.
        let err = run(
            r#"(:wat::core::defn :wat::holon::Bogus [x <- :wat::core::i64] -> :wat::core::i64 x)"#,
        )
        .unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ReservedPrefix(_)))
        );
    }

    #[test]
    fn duplicate_define_rejected() {
        // Stone 241.11 — defn macro-expands to def, which is left in `rest`
        // and goes through check_program. Duplicate def now surfaces as
        // DefRedefForbidden (check error) not DuplicateDefine (runtime error).
        // The run() helper panics on check errors, so we test that startup fails.
        // Use FQDN types (:wat::core::i64) since defn goes through check_program.
        use crate::freeze::startup_from_source;
        use crate::load::loader::InMemoryLoader;
        use std::sync::Arc;
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :foo [x <- :wat::core::i64] -> :wat::core::i64 x)
            (:wat::core::defn :foo [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1))
            (:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)
        "#;
        let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
        assert!(result.is_err(), "duplicate defn must be rejected");
    }

    #[test]
    fn undefined_function_errors() {
        assert!(matches!(
            eval_expr("(:my::app::missing 1)"),
            Err(EvalBreak::Diagnostic(e)) if matches!(e.kind(), RuntimeErrorKind::UnknownFunction(_))
        ));
    }

    // ─── Fn + closures ──────────────────────────────────────────────────

    #[test]
    fn fn_as_value() {
        // The fn produces a callable; invoking it inline.
        let result = eval_expr(
            r#"((:wat::core::fn [x <- :i64 y <- :i64] -> :i64
                  (:wat::i64::+ x y))
                3 4)"#,
        )
        .unwrap();
        assert!(matches!(result, Value::i64(7)));
    }

    #[test]
    fn closure_captures_let_binding() {
        let result = eval_expr(
            r#"(:wat::core::let
                 [adder
                   (:wat::core::fn [x <- :i64] -> :i64
                     (:wat::i64::+ x 10))]
                 (adder 5))"#,
        )
        .unwrap();
        assert!(matches!(result, Value::i64(15)));
    }

    #[test]
    fn closure_captures_enclosing_variable() {
        // The fn captures `n` from the outer let; even when invoked
        // from a deeper scope, it sees the captured value.
        let result = eval_expr(
            r#"(:wat::core::let [n 100]
                 (:wat::core::let [f
                                  (:wat::core::fn [x <- :i64] -> :i64
                                    (:wat::i64::+ x n))]
                   (:wat::core::let [n 999]
                     (f 1))))"#,
        )
        .unwrap();
        // Expected: f captured n=100, so f(1) = 1 + 100 = 101 regardless of inner rebind.
        assert!(matches!(result, Value::i64(101)));
    }

    // ─── Algebra-core runtime construction ──────────────────────────────

    #[test]
    fn algebra_atom_from_literal() {
        // Arc 225 Stone 225.1 — Atom is now narrow (HolonAST→Atom); use to-holon for primitives.
        let v = eval_expr(r#"(:wat::holon::to-holon "role")"#).unwrap();
        assert!(matches!(v, Value::holon__HolonAST(_)));
    }

    #[test]
    fn algebra_atom_from_bound_variable() {
        // Arc 225 Stone 225.1 — to-holon lifts bound integer → HolonAST leaf.
        let v = eval_expr(r#"(:wat::core::let [x 42] (:wat::holon::to-holon x))"#).unwrap();
        match v {
            Value::holon__HolonAST(h) => {
                assert_eq!(h.as_i64(), Some(42));
            }
            other => panic!("expected Holon, got {:?}", other),
        }
    }

    #[test]
    fn algebra_bind_composes_holons() {
        // Arc 225 Stone 225.1 — to-holon lifts string primitives into HolonAST.
        let v = eval_expr(
            r#"(:wat::holon::Bind
                 (:wat::holon::to-holon "role")
                 (:wat::holon::to-holon "filler"))"#,
        )
        .unwrap();
        assert!(matches!(v, Value::holon__HolonAST(_)));
    }

    #[test]
    fn algebra_bundle_via_list_ctor() {
        // Bundle now returns (Result :- [wat::holon::HolonAST CapacityExceeded])
        // under every mode — end-to-end tests in `tests/wat_bundle_*`
        // exercise the four capacity-mode paths. This unit test
        // confirms the Ok wrap happens at cost <= budget (at d=1024,
        // budget=32 and we Bundle 3 atoms).
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let v = eval_with_ctx(
            r#"(:wat::holon::Bundle
                 (:wat::core::Vector :- [:wat::holon::HolonAST]
                   (:wat::holon::to-holon "a")
                   (:wat::holon::to-holon "b")
                   (:wat::holon::to-holon "c")))"#,
            1024,
        )
        .unwrap();
        match v {
            Value::Result(r) => match &*r {
                Ok(Value::holon__HolonAST(_)) => {}
                other => panic!("expected Ok(wat::holon::HolonAST); got {:?}", other),
            },
            other => panic!("expected Value::Result; got {:?}", other),
        }
    }

    #[test]
    fn algebra_blend_with_runtime_weight() {
        // Weight computed at runtime via arithmetic.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let v = eval_expr(
            r#"(:wat::holon::Blend
                 (:wat::holon::to-holon "x")
                 (:wat::holon::to-holon "y")
                 1
                 (:wat::i64::- 0 1))"#,
        )
        .unwrap();
        assert!(matches!(v, Value::holon__HolonAST(_)));
    }

    #[test]
    fn algebra_bundle_non_list_rejected() {
        // Arc 225 Stone 225.1 — to-holon; Bundle still refuses non-list.
        let err = eval_expr(r#"(:wat::holon::Bundle (:wat::holon::to-holon "a"))"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    // ─── Program-level integration ──────────────────────────────────────

    #[test]
    fn program_with_defines_and_algebra() {
        // A small program that defines a helper and uses it to build a Holon.
        // Arc 225 Stone 225.1 — to-holon lifts string parameters.
        let result = run(
            r#"
            (:wat::core::defn :my::app::encode-pair [a <- :wat::core::String b <- :wat::core::String] -> :wat::holon::HolonAST
              (:wat::holon::Bind
                              (:wat::holon::to-holon a)
                              (:wat::holon::to-holon b)))
            (:my::app::encode-pair "role" "filler")
            "#,
        )
        .unwrap();
        assert!(matches!(result, Value::holon__HolonAST(_)));
    }

    // ─── Four eval forms (wat-source callable) ──────────────────────────
    //
    // Per 2026-04-20 INSCRIPTION: eval-ast! / eval-edn! / eval-digest! /
    // eval-signed! all return (:Result :- [wat::holon::HolonAST :wat::core::EvalError])
    // now. Test helpers below unwrap the Result wrap so the assertions
    // against Ok values and Err-kind strings stay concise.

    /// Helper: run a program with a pre-bound `program` local holding
    /// a `Value::Ast` — simulates a caller that parsed or extracted
    /// the AST before passing it to `eval-ast!`.
    fn run_with_ast_local(body: &str, ast_to_bind: WatAST) -> Result<Value, EvalBreak> {
        let form = crate::parse_one!(body).expect("parse body");
        let env = Environment::new()
            .child()
            .bind_unknown_span(
                "program",
                TrackedValue::from(Value::wat__WatAST(Arc::new(ast_to_bind))),
            )
            .build();
        eval_inner(&form, &env, &SymbolTable::new()).map(|tv| tv.value_owned())
    }

    /// Unwrap the outer `Value::Result(Ok(v))` from an eval-family
    /// call's return; panics with diagnostic if the value isn't a
    /// Result, or if the Result is Err.
    fn eval_ok_inner(v: Value) -> Value {
        match v {
            Value::Result(r) => match &*r {
                Ok(inner) => inner.clone(),
                Err(err) => panic!("expected Ok from eval-family; got Err({:?})", err),
            },
            other => panic!("expected Value::Result from eval-family; got {:?}", other),
        }
    }

    /// Unwrap an eval-family Err and return its (kind, message) as
    /// strings. Panics if the value isn't a Result or isn't Err or
    /// isn't a Struct with the expected EvalError field shape.
    fn eval_err_kind_and_message(v: Value) -> (String, String) {
        match v {
            Value::Result(r) => match &*r {
                Err(err) => match err {
                    Value::Aggregate(sv)
                        if sv.nature == Nature::Struct && sv.class.as_ref() == "wat::core::EvalError" =>
                    {
                        let kind = match &sv.fields[0] {
                            Value::String(s) => (**s).clone(),
                            _ => panic!("EvalError.kind not String"),
                        };
                        let msg = match &sv.fields[1] {
                            Value::String(s) => (**s).clone(),
                            _ => panic!("EvalError.message not String"),
                        };
                        (kind, msg)
                    }
                    other => panic!("expected Aggregate(EvalError); got {:?}", other),
                },
                Ok(inner) => panic!("expected Err from eval-family; got Ok({:?})", inner),
            },
            other => panic!("expected Value::Result from eval-family; got {:?}", other),
        }
    }

    #[test]
    fn eval_ast_bang_runs_a_parsed_program() {
        // Arc 102 — eval-ast! returns the form's terminal value
        // bare (reverts arc 066's value_to_holon wrap). (40 + 2)
        // → Value::i64(42); the polymorphic Result<:T, :EvalError>
        // scheme has T = i64 here. Caller match-arm gets the
        // bare i64 directly.
        let program = crate::parse_one!("(:wat::i64::+ 40 2)").unwrap();
        let result = run_with_ast_local("(:wat::eval-ast! program)", program).unwrap();
        let inner = eval_ok_inner(result);
        match inner {
            Value::i64(42) => {}
            other => panic!("expected i64(42), got {:?}", other),
        }
    }

    #[test]
    fn eval_ast_bang_refuses_mutation_form() {
        // Stone 241.16 — migrated from `:wat::core::define` (HARD CUT; no longer in
        // is_mutation_head) to `:wat::core::defstruct` (still a recognized mutation form).
        // parse_one! bypasses macro expansion so defstruct head is preserved as-is.
        // Mechanism under test: eval-ast! refuses ANY mutation-headed form.
        let program =
            crate::parse_one!(r#"(:wat::core::defstruct :evil::T [x <- :wat::core::i64])"#)
                .unwrap();
        let result = run_with_ast_local("(:wat::eval-ast! program)", program).unwrap();
        let (kind, _msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "mutation-form-refused");
    }

    #[test]
    fn eval_ast_bang_rejects_non_ast_value() {
        // Binding a string as program; eval-ast! refuses because it
        // only accepts Value::wat__WatAST (not Value::String).
        // The refusal lands as Err(EvalError{kind="type-mismatch"}),
        // NOT a RuntimeError unwind — the eval-family Result-wrap
        // per the 2026-04-20 INSCRIPTION.
        let form = crate::parse_one!(r#"(:wat::eval-ast! "oops")"#).unwrap();
        let result = eval_inner(&form, &Environment::new(), &SymbolTable::new())
            .unwrap()
            .value_owned();
        let (kind, msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "type-mismatch");
        assert_eq!(
            msg,
            r#":wat::eval-ast!: expected Ast, got wat::core::String `"oops"`"#
        );
    }

    // ─── Programs-as-atoms roundtrip ────────────────────────────────────
    //
    // quote + Atom + atom-value + Bind self-inverse — the substrate
    // claim made executable. A wat program is captured as data, atomized,
    // passed through Bind/unbind, extracted, and evaluated.

    #[test]
    fn quote_captures_unevaluated_ast() {
        // (quote (+ 1 2)) returns a WatAST; does NOT evaluate the +.
        let result = eval_expr("(:wat::core::quote (:wat::i64::+ 1 2))").unwrap();
        match result {
            Value::wat__WatAST(ast) => {
                // The captured AST should be a List whose head is :wat::i64::+
                match &*ast {
                    WatAST::List(items, _) => {
                        assert!(matches!(
                            items.first(),
                            Some(WatAST::Keyword(k, _)) if k == ":wat::i64::+"
                        ));
                    }
                    other => panic!("expected List AST, got {:?}", other),
                }
            }
            other => panic!("expected Value::wat__WatAST, got {:?}", other),
        }
    }

    #[test]
    fn quote_arity_mismatch() {
        let err = eval_expr("(:wat::core::quote 1 2)").unwrap_err();
        assert!(matches!(
            err,
            EvalBreak::Diagnostic(e)
                if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { op, expected: 1, got: 2, .. } if op == ":wat::core::quote")
        ));
    }

    #[test]
    fn atom_wraps_quoted_program() {
        // Arc 225 Stone 225.1 — from-wat lowers quoted WatAST form to HolonAST.
        // Old: (Atom (quote form)) — Atom accepted WatAST (polymorphic, now retired).
        // New: (from-wat (quote form)) — the honest directional verb.
        let result =
            eval_expr("(:wat::holon::from-wat (:wat::core::quote (:wat::i64::+ 1 2)))")
                .unwrap();
        assert!(matches!(result, Value::holon__HolonAST(_)));
    }

    #[test]
    fn atom_value_recovers_string() {
        // Arc 225 Stone 225.1 — from-holon replaces atom-value; to-holon replaces polymorphic Atom.
        let result =
            eval_expr(r#"(:wat::holon::from-holon (:wat::holon::to-holon "hello"))"#).unwrap();
        match result {
            Value::String(s) => assert_eq!(s.as_str(), "hello"),
            other => panic!("expected Value::String, got {:?}", other),
        }
    }

    #[test]
    fn atom_lowers_quoted_list_to_bundle() {
        // Per arc 057's quote-all-the-way-down framing: a quoted list
        // form lowers structurally to a Bundle of its lowered children.
        // The form's identity participates in the algebra; this is the
        // substrate-side prerequisite for the cache-as-coordinate-tree
        // and for Reckoner labels on intermediary forms.
        //
        // Arc 221 Stone 221.4b cascade — `(:wat::core::quote (:wat::i64::+ 40 2))`
        // produces `WatAST::List([WatAST::Keyword(":wat::i64::+"), ...])`.
        // `watast_to_holon` at Stone 221.4b now maps `WatAST::Keyword(k) →
        // HolonAST::keyword(k.as_str())` → `HolonAST::Keyword("wat::i64::+")`
        // (leading colon stripped). Assertions flipped from as_symbol() to as_keyword().
        //
        // Arc 225 Stone 225.1 — from-wat replaces Atom for WatAST inputs.
        let v = eval_expr("(:wat::holon::from-wat (:wat::core::quote (:wat::i64::+ 40 2)))")
            .unwrap();
        let h = match v {
            Value::holon__HolonAST(h) => h,
            other => panic!("expected Holon, got {:?}", other),
        };
        match &*h {
            HolonAST::Bundle(items) => {
                assert_eq!(
                    items.len(),
                    3,
                    "expected 3-item Bundle, got {}",
                    items.len()
                );
                // Stone 221.4b: WatAST::Keyword → HolonAST::Keyword; content without leading colon.
                assert_eq!(items[0].as_keyword(), Some("wat::i64::+"));
                assert_eq!(
                    items[0].as_symbol(),
                    None,
                    "must NOT be Symbol after arc 221"
                );
                assert_eq!(items[1].as_i64(), Some(40));
                assert_eq!(items[2].as_i64(), Some(2));
            }
            other => panic!("expected Bundle, got {:?}", other),
        }
    }

    #[test]
    fn atom_value_recovers_quoted_keyword() {
        // Atomic literals inside quote DO survive the trip — they lower to
        // their matching primitive leaf via from-wat, and from-holon
        // reads them back as the corresponding wat Value.
        //
        // Arc 221 Stone 221.4b cascade — `(:wat::core::quote :outcome)` produces
        // `WatAST::Keyword(":outcome")` → `HolonAST::Keyword("outcome")` (no colon).
        // `eval_holon_from_holon` (renamed from eval_atom_value) has a `HolonAST::Keyword(s)`
        // arm that returns `Value::keyword(":outcome")` (restores leading colon).
        //
        // Arc 225 Stone 225.1 — from-holon replaces atom-value; from-wat replaces WatAST arm of Atom.
        let result = eval_expr(
            "(:wat::holon::from-holon (:wat::holon::from-wat (:wat::core::quote :outcome)))",
        )
        .unwrap();
        match result {
            Value::wat__core__keyword(k) => assert_eq!(k.as_str(), ":outcome"),
            other => panic!("expected keyword, got {:?}", other),
        }
    }

    #[test]
    fn atom_value_refuses_non_atom_holon() {
        // Bind(to-holon, to-holon) is a Bind — from-holon refuses (not a primitive leaf or Atom).
        // Arc 225 Stone 225.1 — from-holon replaces atom-value; to-holon replaces Atom for primitives.
        let err = eval_expr(
            r#"(:wat::holon::from-holon
                 (:wat::holon::Bind
                   (:wat::holon::to-holon "a")
                   (:wat::holon::to-holon "b")))"#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { op, .. } if op == ":wat::holon::from-holon")
        ));
    }

    #[test]
    fn bind_always_constructs_tree() {
        // Bind never reduces at the AST level — even when the pattern would
        // be self-inverse at the vector level. The structure stays; the
        // vector is where the self-inverse shows up via cosine. Per 058-024
        // rejection text + FOUNDATION 1718 (presence is measurement).
        // Arc 225 Stone 225.1 — to-holon replaces Atom for string primitives.
        let result = eval_expr(
            r#"(:wat::holon::Bind
                 (:wat::holon::Bind
                   (:wat::holon::to-holon "key")
                   (:wat::holon::to-holon "program"))
                 (:wat::holon::to-holon "key"))"#,
        )
        .unwrap();
        match result {
            Value::holon__HolonAST(h) => {
                // Must be a Bind tree, NOT reduced to the "program" atom.
                assert!(matches!(&*h, HolonAST::Bind(_, _)));
            }
            other => panic!("expected Bind holon, got {:?}", other),
        }
    }

    #[test]
    fn programs_as_atoms_structural_lowering() {
        // Per arc 057's quote-all-the-way-down: a quoted form lowers
        // structurally to a HolonAST tree (List → Bundle, leaves →
        // primitive leaves). The form is now a coordinate in the
        // algebra; cosine, Hash, and Eq all see its structure.
        //
        // The pre-arc-057 lossless round-trip is intentionally gone —
        // the substrate holds coordinates, not runnable programs. Consumers
        // who want the value walk the form themselves (or use a future cache
        // layer that records the form → value edge).
        //
        // Arc 221 Stone 221.4b cascade — `(:wat::core::quote (:wat::i64::+ 40 2))`
        // produces `WatAST::List([WatAST::Keyword(":wat::i64::+"), ...])`.
        // `watast_to_holon` at Stone 221.4b maps `WatAST::Keyword(k) →
        // HolonAST::keyword(k.as_str())` → `HolonAST::Keyword("wat::i64::+")`
        // (leading colon stripped). Assertion flipped from as_symbol() to as_keyword().
        //
        // Arc 225 Stone 225.1 — from-wat replaces Atom for WatAST (quoted form) inputs.
        let v = eval_expr("(:wat::holon::from-wat (:wat::core::quote (:wat::i64::+ 40 2)))")
            .unwrap();
        let h = match v {
            Value::holon__HolonAST(h) => h,
            other => panic!("expected Holon, got {:?}", other),
        };
        match &*h {
            HolonAST::Bundle(items) => {
                assert_eq!(items.len(), 3);
                // Stone 221.4b: WatAST::Keyword → HolonAST::Keyword; content without leading colon.
                assert_eq!(items[0].as_keyword(), Some("wat::i64::+"));
                assert_eq!(
                    items[0].as_symbol(),
                    None,
                    "must NOT be Symbol after arc 221"
                );
                assert_eq!(items[1].as_i64(), Some(40));
                assert_eq!(items[2].as_i64(), Some(2));
            }
            other => panic!("expected Bundle, got {:?}", other),
        }
    }

    // ─── Presence measurement (FOUNDATION 1718) ─────────────────────────
    //
    // The vector-level proof that `Bind(k, p)` obscures `p` in the
    // composite vector, and that the self-inverse Bind-on-Bind recovers
    // it. The algebra's retrieval primitive: cosine between encoded
    // holons, scalar output, caller binarizes.

    /// Build a SymbolTable with an EncodingCtx attached — mirrors what
    /// `FrozenWorld::freeze` does. Needed for tests exercising presence
    /// or config accessors without running the full startup pipeline.
    fn test_sym_with_ctx(dim_count: usize) -> SymbolTable {
        let cfg = Config {
            capacity_mode: crate::config::CapacityMode::Error,
            global_seed: 42,
            dim_count,
            presence_sigma_ast: None,
            coincident_sigma_ast: None,
            redef_allowed: false,
            eval_redef_allowed: false,
        };
        let mut sym = SymbolTable::new();
        sym.set_encoding_ctx(Arc::new(EncodingCtx::from_config(&cfg)));
        // Arc 077: dim router retired; program-d lives in EncodingCtx.
        // Tests still install the default sigma fns to mirror freeze.
        sym.set_presence_sigma_fn(Arc::new(crate::holon::sigma::DefaultPresenceSigma));
        sym.set_coincident_sigma_fn(Arc::new(crate::holon::sigma::DefaultCoincidentSigma));
        sym
    }

    fn eval_with_ctx(src: &str, dims: usize) -> Result<Value, EvalBreak> {
        let ast = crate::parse_one!(src).expect("parse ok");
        let sym = test_sym_with_ctx(dims);
        eval_inner(&ast, &Environment::new(), &sym).map(|tv| tv.value_owned())
    }

    /// Arc 278 the cosine outcome wall — `:wat::holon::cosine` returns
    /// `:wat::holon::CosineOutcome`, not a bare f64. These tests all exercise
    /// the well-defined, same-dimension, non-zero-magnitude path (they measure
    /// a real substrate property, not the wall itself — the wall's own faces
    /// are covered by `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat`
    /// and the dimension-mismatch match sites), so `Degenerate`/`DimensionMismatch`
    /// here are test bugs, not cases to weaken the assertion for.
    fn expect_cosine_similarity(v: Value) -> f64 {
        match v {
            Value::Enum(ev) if ev.type_path == ":wat::holon::CosineOutcome" => {
                match (ev.variant_name.as_str(), ev.fields.as_slice()) {
                    ("Similarity", [Value::f64(s)]) => *s,
                    other => panic!("expected CosineOutcome::Similarity[f64], got {:?}", other),
                }
            }
            other => panic!("expected CosineOutcome, got {:?}", other),
        }
    }

    /// Arc 278 the cosine outcome wall — `:wat::holon::dot`'s test-side twin
    /// of [`expect_cosine_similarity`].
    fn expect_dot_computed(v: Value) -> f64 {
        match v {
            Value::Enum(ev) if ev.type_path == ":wat::holon::DotOutcome" => {
                match (ev.variant_name.as_str(), ev.fields.as_slice()) {
                    ("Computed", [Value::f64(p)]) => *p,
                    other => panic!("expected DotOutcome::Computed[f64], got {:?}", other),
                }
            }
            other => panic!("expected DotOutcome, got {:?}", other),
        }
    }

    #[test]
    fn presence_of_atom_in_itself_is_one() {
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::cosine
                 (:wat::holon::to-holon "hello")
                 (:wat::holon::to-holon "hello"))"#,
            1024,
        )
        .unwrap();
        let x = expect_cosine_similarity(result);
        assert!((x - 1.0).abs() < 1e-9, "expected ≈1.0, got {}", x);
    }

    #[test]
    fn dot_of_atom_with_itself_is_large_positive() {
        // dot(v, v) = |v|² — positive and equal to the number of
        // non-zero dimensions in v's encoding. The exact count
        // depends on the substrate's ternary content; we just
        // assert it's well above sqrt(d) (the noise scale).
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::dot
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "alice"))"#,
            1024,
        )
        .unwrap();
        let x = expect_dot_computed(result);
        // Expect |v|² > 5*sqrt(d) (~160 at d=1024).
        assert!(x > 5.0 * (1024f64).sqrt(), "got {}", x);
    }

    #[test]
    fn dot_of_unrelated_atoms_vs_self_orders_correctly() {
        // dot(a, a) >> dot(a, b) for independent atoms. The exact
        // magnitudes are substrate-dependent; the ordering is the
        // load-bearing invariant for Gram-Schmidt (Reject / Project).
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let self_dot = expect_dot_computed(
            eval_with_ctx(
                r#"(:wat::holon::dot
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "alice"))"#,
                1024,
            )
            .unwrap(),
        );
        let cross_dot = expect_dot_computed(
            eval_with_ctx(
                r#"(:wat::holon::dot
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "charlie"))"#,
                1024,
            )
            .unwrap(),
        );
        assert!(
            self_dot > cross_dot.abs() * 3.0,
            "self dot {} should dwarf cross dot {}",
            self_dot,
            cross_dot
        );
    }

    #[test]
    fn dot_wrong_arity() {
        // Arc 225 Stone 225.1 — to-holon lifts string primitive.
        let ast = crate::parse_one!(r#"(:wat::holon::dot (:wat::holon::to-holon "a"))"#).unwrap();
        let err = eval_inner(&ast, &Environment::new(), &test_sym_with_ctx(1024)).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // Arc 294.a — UPDATED: dot now accepts any EdnRepresentable value by lifting via
    // to_holon_inner. i64 is EDN-representable; the old TypeMismatch rejection was the
    // inversion 294.a annihilates. The test is renamed to document the new behavior.
    #[test]
    fn dot_accepts_edn_i64() {
        // i64 is lifted via to_holon_inner; dot returns a scalar (may be any f64).
        let result = eval_with_ctx(r#"(:wat::holon::dot 1 2)"#, 1024);
        assert!(
            result.is_ok(),
            "dot on i64 args must now succeed (EDN-representable); got: {:?}",
            result
        );
    }

    #[test]
    fn presence_q_true_for_self() {
        // presence? is the boolean verdict — cosine > noise floor.
        // An atom against itself: cosine = 1.0, well above the floor.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::presence?
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "alice"))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(true)));
    }

    #[test]
    fn presence_q_false_for_unrelated() {
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::presence?
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "charlie"))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(false)));
    }

    // --- coincident? — arc 023 --------------------------------------------

    #[test]
    fn coincident_q_true_for_self() {
        // Atom vs itself: cosine = 1.0, (1 - cosine) = 0 < noise-floor.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident?
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "alice"))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(true)));
    }

    #[test]
    fn coincident_q_true_for_structurally_same() {
        // Two hand-built identical-structure holons: same Bind shape
        // with same atom children.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident?
                 (:wat::holon::Bind (:wat::holon::to-holon "k") (:wat::holon::to-holon "v"))
                 (:wat::holon::Bind (:wat::holon::to-holon "k") (:wat::holon::to-holon "v")))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(true)));
    }

    #[test]
    fn coincident_q_false_for_unrelated() {
        // Two orthogonal atoms: cosine ≈ 0, (1 - cosine) ≈ 1 > noise-floor.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident?
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "charlie"))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(false)));
    }

    /// Empirical sweep: find the actual coincidence window around a
    /// thermometer-encoded center value at d=1024 with default
    /// coincident_sigma=1.
    ///
    /// For thermometer encoding on [low, high] at dims d:
    ///   - A value v lights up approximately (v - low)/(high - low) * d bits.
    ///   - Moving from v1 to v2 flips approximately |v2 - v1|/(high - low) * d bits.
    ///   - coincident? at 1σ needs bit-flip fraction < 1/(2*sqrt(d)).
    ///   - At d=1024: fraction < 1/64 = 1.5625% of range.
    ///
    /// This test: center = 4.0 on range [0, 10], so range_width = 10.
    /// Predicted coincidence window: 4 ± 0.15625 = [3.84, 4.16].
    /// Values well inside should coincide; values well outside should not.
    #[test]
    fn coincident_q_window_around_4_on_range_0_10() {
        // Inside the predicted window — all should coincide with 4.0.
        for &delta in &[0.0, 0.05, 0.10, 0.14] {
            let below = 4.0 - delta;
            let above = 4.0 + delta;
            for v in &[below, above] {
                let src = format!(
                    r#"(:wat::holon::coincident?
                         (:wat::holon::Thermometer 4.0 0.0 10.0)
                         (:wat::holon::Thermometer {v} 0.0 10.0))"#
                );
                let result = eval_with_ctx(&src, 1024).unwrap();
                assert!(
                    matches!(result, Value::bool(true)),
                    "expected v={} to coincide with 4.0 (inside window)",
                    v
                );
            }
        }

        // Outside the predicted window — should NOT coincide.
        for &delta in &[0.25, 0.50, 1.00, 2.00] {
            for v in &[4.0 - delta, 4.0 + delta] {
                let src = format!(
                    r#"(:wat::holon::coincident?
                         (:wat::holon::Thermometer 4.0 0.0 10.0)
                         (:wat::holon::Thermometer {v} 0.0 10.0))"#
                );
                let result = eval_with_ctx(&src, 1024).unwrap();
                assert!(
                    matches!(result, Value::bool(false)),
                    "expected v={} to NOT coincide with 4.0 (outside window)",
                    v
                );
            }
        }
    }

    #[test]
    fn coincident_q_true_for_close_thermometer_values() {
        // Structural coincident? on two Thermometer holons whose
        // values sit close inside their range. Models percentages
        // on [0, 1] — 3.9% vs 4.1% as fractions (0.039 vs 0.041,
        // difference 0.002 = 0.2% of range). The thermometer-
        // gradient bits agree almost everywhere; cosine lands
        // inside the coincident_floor window at d=1024.
        //
        // The archive's (Linear v scale) maps to
        // (Thermometer v -scale scale) per 058-008; for percentage
        // domains [0, 1] is the honest range, no negative half.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident?
                 (:wat::holon::Thermometer 0.039 0.0 1.0)
                 (:wat::holon::Thermometer 0.041 0.0 1.0))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(true)));
    }

    #[test]
    fn coincident_q_stricter_than_presence_q() {
        // Construct a case where presence? passes but coincident? fails.
        // Bind(k, v1) vs Bind(k, v1) -- identical, both true.
        // Bind(k, v1) vs Bind(k, v2) -- different filler, cosine is
        // close to 0 (different atoms orthogonalize). Both false.
        // For the stricter-than check: Atom("x") vs itself is coincident
        // (cosine=1), and is also present. Flip the bound: the
        // interesting asymmetry is the THRESHOLD level. At d=1024 the
        // noise floor is ~0.156, so presence? fires at any cosine > 0.156.
        // Coincident? only fires at cosine > 0.844. A structural-mismatch
        // of Bind-shape-vs-Atom-shape gives cosine well below 0.844 but
        // testing that reliably needs a constructed pair with known
        // overlap — skip that combinatorial test here and lock the
        // threshold-level invariant at the wat-test tier where we have
        // concrete numeric probes.
        //
        // What this test asserts: presence? can be true while coincident?
        // is false for the CAS where cosine is between floor and 1-floor.
        // Easy case: a=Atom("a"), b=Bundle([Atom("a"), Atom("b"), Atom("c")]).
        // The Bundle contains Atom("a") — so presence? is true — but
        // the Bundle is NOT the same as the single atom.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let bundle_src = r#"(:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST]
                               (:wat::holon::to-holon "a")
                               (:wat::holon::to-holon "b")
                               (:wat::holon::to-holon "c")))"#;
        let present = eval_with_ctx(
            &format!(
                r#"(:wat::core::match {bundle}
                     ((:wat::core::Ok h) (:wat::holon::presence? (:wat::holon::to-holon "a") h))
                     ((:wat::core::Err _) false))"#,
                bundle = bundle_src
            ),
            1024,
        )
        .unwrap();
        assert!(
            matches!(present, Value::bool(true)),
            "presence? should fire for Atom in Bundle"
        );
        let coincident = eval_with_ctx(
            &format!(
                r#"(:wat::core::match {bundle}
                     ((:wat::core::Ok h) (:wat::holon::coincident? (:wat::holon::to-holon "a") h))
                     ((:wat::core::Err _) false))"#,
                bundle = bundle_src
            ),
            1024,
        )
        .unwrap();
        assert!(
            matches!(coincident, Value::bool(false)),
            "coincident? must NOT fire — the bundle is not the atom"
        );
    }

    // --- coincident-explain — arc 069 ---------------------------------------
    //
    // Diagnostic primitive bundling the cosine, the floor, the dim,
    // the sigma, the predicate result, and the smallest sigma at
    // which the pair would coincide.

    fn explain_fields(v: &Value) -> &[Value] {
        match v {
            Value::Aggregate(sv)
                if sv.nature == Nature::Struct
                    && sv.class.as_ref() == "wat::holon::CoincidentExplanation" =>
            {
                assert_eq!(sv.fields.len(), 6);
                sv.fields.as_slice()
            }
            other => panic!("expected CoincidentExplanation struct, got {:?}", other),
        }
    }

    fn explain_cosine(v: &Value) -> f64 {
        match &explain_fields(v)[0] {
            Value::f64(x) => *x,
            other => panic!("expected f64 cosine, got {:?}", other),
        }
    }

    fn explain_coincident(v: &Value) -> bool {
        match &explain_fields(v)[4] {
            Value::bool(b) => *b,
            other => panic!("expected bool coincident, got {:?}", other),
        }
    }

    fn explain_min_sigma(v: &Value) -> i64 {
        match &explain_fields(v)[5] {
            Value::i64(n) => *n,
            other => panic!("expected i64 min-sigma-to-pass, got {:?}", other),
        }
    }

    fn explain_dim(v: &Value) -> i64 {
        match &explain_fields(v)[2] {
            Value::i64(n) => *n,
            other => panic!("expected i64 dim, got {:?}", other),
        }
    }

    fn explain_sigma(v: &Value) -> i64 {
        match &explain_fields(v)[3] {
            Value::i64(n) => *n,
            other => panic!("expected i64 sigma, got {:?}", other),
        }
    }

    #[test]
    fn coincident_explain_byte_identical() {
        // Same holon against itself: cosine = 1.0, coincident at sigma=1.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident-explain
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "alice"))"#,
            1024,
        )
        .unwrap();
        assert!((explain_cosine(&result) - 1.0).abs() < 1e-9);
        assert!(explain_coincident(&result));
        assert_eq!(explain_min_sigma(&result), 1);
        assert_eq!(explain_dim(&result), 1024);
        assert_eq!(explain_sigma(&result), 1);
    }

    #[test]
    fn coincident_explain_near_coincident() {
        // Thermometer values inside the predicted window at d=1024
        // (window ≈ ±0.15625 on range [0, 10]; we pick 4.0 vs 4.05).
        // Cosine should be very close to 1, coincident=true,
        // min-sigma-to-pass=1.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident-explain
                 (:wat::holon::Thermometer 4.0  0.0 10.0)
                 (:wat::holon::Thermometer 4.05 0.0 10.0))"#,
            1024,
        )
        .unwrap();
        assert!(explain_coincident(&result));
        let cos = explain_cosine(&result);
        assert!(cos > 0.99, "expected cos > 0.99, got {}", cos);
        assert_eq!(explain_min_sigma(&result), 1);
    }

    #[test]
    fn coincident_explain_just_below_threshold() {
        // Thermometer values outside the d=1024 window. Coincident=false;
        // min-sigma-to-pass > 1 — the diagnostic literally tells the
        // caller how much wider to set sigma to make the pair pass.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident-explain
                 (:wat::holon::Thermometer 4.0  0.0 10.0)
                 (:wat::holon::Thermometer 4.50 0.0 10.0))"#,
            1024,
        )
        .unwrap();
        assert!(!explain_coincident(&result));
        let min_sigma = explain_min_sigma(&result);
        assert!(min_sigma > 1, "expected > 1, got {}", min_sigma);
    }

    #[test]
    fn coincident_explain_distant() {
        // Two unrelated atoms: cosine ≈ 0; (1 - cos) * sqrt(d) is
        // honestly large. The diagnostic surfaces the structural
        // distance — caller reads cosine directly to see "not near-
        // coincident, structurally distant."
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident-explain
                 (:wat::holon::to-holon "alice")
                 (:wat::holon::to-holon "charlie"))"#,
            1024,
        )
        .unwrap();
        assert!(!explain_coincident(&result));
        let cos = explain_cosine(&result);
        assert!(cos.abs() < 0.5, "expected near 0, got {}", cos);
        let min_sigma = explain_min_sigma(&result);
        // (1 - 0) * sqrt(1024) = 32; allow a band around it.
        assert!(min_sigma >= 16, "expected >= 16, got {}", min_sigma);
    }

    #[test]
    fn coincident_explain_polymorphic_holon_vector() {
        // One side AST, the other side a pre-encoded Vector. Same
        // input shape `coincident?` accepts post arc 061.
        // Arc 225 Stone 225.1 — to-holon lifts string primitive.
        let result = eval_with_ctx(
            r#"(:wat::core::let
                 [a (:wat::holon::to-holon "x")
                  va (:wat::holon::encode a)]
                 (:wat::holon::coincident-explain a va))"#,
            1024,
        )
        .unwrap();
        assert_eq!(explain_dim(&result), 1024);
        assert!(explain_coincident(&result));
    }

    #[test]
    fn coincident_explain_dim_reflects_router_choice() {
        // Build with d=512; the diagnostic's `dim` field reports
        // the actual encoding d, not a hard-coded constant.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::coincident-explain
                 (:wat::holon::to-holon "a")
                 (:wat::holon::to-holon "a"))"#,
            512,
        )
        .unwrap();
        assert_eq!(explain_dim(&result), 512);
    }

    #[test]
    fn coincident_explain_arity_mismatch() {
        // Arc 225 Stone 225.1 — to-holon lifts string primitive.
        let err = eval_with_ctx(
            r#"(:wat::holon::coincident-explain (:wat::holon::to-holon "x"))"#,
            1024,
        )
        .unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn coincident_explain_agrees_with_coincident_q() {
        // The struct's `coincident` field returns the same value as
        // `:wat::holon::coincident?` for the same inputs. This is
        // the "the diagnostic doesn't lie" invariant.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let cases = [
            (
                r#"(:wat::holon::to-holon "a")"#,
                r#"(:wat::holon::to-holon "a")"#,
            ),
            (
                r#"(:wat::holon::to-holon "a")"#,
                r#"(:wat::holon::to-holon "b")"#,
            ),
            (
                r#"(:wat::holon::Thermometer 4.0 0.0 10.0)"#,
                r#"(:wat::holon::Thermometer 4.05 0.0 10.0)"#,
            ),
            (
                r#"(:wat::holon::Thermometer 4.0 0.0 10.0)"#,
                r#"(:wat::holon::Thermometer 6.0 0.0 10.0)"#,
            ),
        ];
        for (a, b) in cases {
            // Field index 4 is `coincident` per the struct's field
            // declaration order. Tests run with a bare SymbolTable
            // (no `register_struct_methods`), so we access by
            // position rather than the auto-generated `/coincident`
            // helper.
            let probe = format!(
                r#"(:wat::core::let
                     [aa {a}
                      bb {b}
                      p (:wat::holon::coincident? aa bb)
                      expl
                        (:wat::holon::coincident-explain aa bb)]
                     (:wat::core::Tuple p
                       (:wat::core::struct-field expl 4)))"#
            );
            let result = eval_with_ctx(&probe, 1024).unwrap();
            match result {
                Value::Tuple(t) => {
                    let elems = (*t).clone();
                    let p = match &elems[0] {
                        Value::bool(b) => *b,
                        other => panic!("expected bool, got {:?}", other),
                    };
                    let q = match &elems[1] {
                        Value::bool(b) => *b,
                        other => panic!("expected bool, got {:?}", other),
                    };
                    assert_eq!(
                        p, q,
                        "predicate vs explanation.coincident disagree on ({}, {})",
                        a, b
                    );
                }
                other => panic!("expected tuple, got {:?}", other),
            }
        }
    }

    // --- eval-coincident? — arc 026 slice 1 -------------------------------
    //
    // Two forms, each quoted as an AST; each reduces under
    // run_constrained; each result atomizes via value_to_atom; the
    // two Atoms compare with the same coincident_floor test
    // structural coincident? uses. Return is eval-family-shaped
    // (Result :- [bool EvalError]).

    #[test]
    fn eval_coincident_q_true_for_equivalent_arithmetic() {
        // The book's Chapter 28 retort: two distinct expressions that
        // reduce to the same :i64 4 → same Atom(4) → same vector →
        // coincident? fires true.
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-coincident?
                 (:wat::core::quote (:wat::i64::+ 2 2))
                 (:wat::core::quote (:wat::i64::* 1 4)))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(true)));
    }

    #[test]
    fn eval_coincident_q_true_for_same_string() {
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-coincident?
                 (:wat::core::quote "rsi")
                 (:wat::core::quote "rsi"))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(true)));
    }

    #[test]
    fn eval_coincident_q_false_for_different_scalars() {
        // 4 vs 5: distinct Atom hashes → orthogonal vectors → (1 - cos)
        // well above coincident_floor.
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-coincident?
                 (:wat::core::quote 4)
                 (:wat::core::quote 5))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(false)));
    }

    #[test]
    fn eval_coincident_q_true_for_structurally_same_holon() {
        // Two structurally-identical constructions share a hash →
        // same vector → coincident? fires.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives inside quoted forms.
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-coincident?
                 (:wat::core::quote
                   (:wat::holon::Bind (:wat::holon::to-holon "k") (:wat::holon::to-holon "v")))
                 (:wat::core::quote
                   (:wat::holon::Bind (:wat::holon::to-holon "k") (:wat::holon::to-holon "v"))))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(true)));
    }

    #[test]
    fn eval_coincident_q_accepts_mixed_types() {
        // Side A reduces to :i64 4 → to_holon_inner(i64 4) → I64 leaf.
        // Side B reduces to an already-built HolonAST I64(4) →
        // to_holon_inner(HolonAST) wraps it as Atom(HolonAST::I64(4)) —
        // different shape from the bare I64 leaf. This test locks that behavior.
        //
        // Arc 225 Stone 225.1 — to-holon replaces polymorphic Atom.
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-coincident?
                 (:wat::core::quote 4)
                 (:wat::core::quote (:wat::holon::to-holon 4)))"#,
            1024,
        )
        .unwrap();
        // Expect false — side A is I64 leaf, side B is Atom(HolonAST::I64(4)).
        // Different payloads; the primitive is "coincidence of lifted results."
        assert!(matches!(eval_ok_inner(result), Value::bool(false)));
    }

    #[test]
    fn eval_coincident_q_err_on_non_ast_arg() {
        // Passing a non-WatAST value (e.g., a string literal directly,
        // not quoted) yields EvalError{kind="type-mismatch"}. Mirrors
        // eval-ast!'s rejection of non-AST input.
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-coincident? "not-ast" "also-not-ast")"#,
            1024,
        )
        .unwrap();
        let (kind, msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "type-mismatch");
        assert_eq!(
            msg,
            r#":wat::holon::eval-coincident?: expected Ast, got wat::core::String `"not-ast"`"#
        );
    }

    // --- eval-edn-coincident? — arc 026 slice 2 ---------------------------

    #[test]
    fn eval_edn_coincident_q_true_for_equivalent_sources() {
        // Same shape as slice 1's arithmetic-equivalence test, but
        // each side is an inline EDN source string rather than a
        // quoted form. Both parse, both evaluate to :i64 4, both
        // Atom-lift identically → coincident? fires.
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-edn-coincident?
 "(:wat::i64::+ 2 2)"
 "(:wat::i64::* 1 4)")"#,
            1024,
        )
        .unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(true)));
    }

    #[test]
    fn eval_edn_coincident_q_false_for_different_sources() {
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-edn-coincident?
 "(:wat::i64::+ 2 2)"
 "(:wat::i64::+ 2 3)")"#,
            1024,
        )
        .unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(false)));
    }

    #[test]
    fn eval_edn_coincident_q_err_on_parse_failure() {
        // Side B has an unclosed paren — parse fails → EvalError with
        // kind="malformed-form" propagates.
        let result = eval_with_ctx(
            r#"(:wat::holon::eval-edn-coincident?
 "(:wat::i64::+ 2 2)"
 "(:wat::i64::+ 2")"#,
            1024,
        )
        .unwrap();
        let (kind, _msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "malformed-form");
    }

    // --- eval-digest-coincident? — arc 026 slice 3 ------------------------
    //
    // Uses real SHA-256 digests computed inline. Same helper pattern
    // as load.rs's digest-load tests.

    fn sha256_hex(source: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    #[test]
    fn eval_digest_coincident_q_true_for_equivalent_verified_sources() {
        let src_a = "(:wat::i64::+ 2 2)";
        let src_b = "(:wat::i64::* 1 4)";
        let h_a = sha256_hex(src_a);
        let h_b = sha256_hex(src_b);
        let program = format!(
            r#"(:wat::holon::eval-digest-string-coincident?
 "{src_a}"
                 :wat::verify::digest-sha256
                 :wat::verify::string "{h_a}"
 "{src_b}"
                 :wat::verify::digest-sha256
                 :wat::verify::string "{h_b}")"#
        );
        let result = eval_with_ctx(&program, 1024).unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(true)));
    }

    #[test]
    fn eval_digest_coincident_q_err_on_bad_digest() {
        // Side A digest is wrong → verification fails before parse;
        // EvalError with kind="verification-failed" propagates.
        let src_a = "(:wat::i64::+ 2 2)";
        let src_b = "(:wat::i64::* 1 4)";
        let h_b = sha256_hex(src_b);
        let bogus = "0".repeat(64);
        let program = format!(
            r#"(:wat::holon::eval-digest-string-coincident?
 "{src_a}"
                 :wat::verify::digest-sha256
                 :wat::verify::string "{bogus}"
 "{src_b}"
                 :wat::verify::digest-sha256
                 :wat::verify::string "{h_b}")"#
        );
        let result = eval_with_ctx(&program, 1024).unwrap();
        let (kind, _msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "verification-failed");
    }

    // --- eval-signed-coincident? — arc 026 slice 4 ------------------------

    fn sign_src_ed25519(source: &str) -> (String, String) {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        use ed25519_dalek::Signer;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let forms = crate::parse_all!(source).expect("source parses");
        let hash = crate::hash::hash_canonical_program(&forms);
        let sig = signing_key.sign(&hash);
        let sig_b64 = B64.encode(sig.to_bytes());
        let pk_b64 = B64.encode(signing_key.verifying_key().as_bytes());
        (sig_b64, pk_b64)
    }

    /// Guard test: the Ed25519 signatures embedded in
    /// `wat-tests/holon/eval-coincident.wat` must still verify against
    /// the source strings they sign. If a source string is edited
    /// without regenerating its sig, this test fails with the
    /// mismatch. Prevents silent drift between the unit-test sources
    /// and the wat-level sandbox tests that hard-code their sigs.
    ///
    /// To regenerate the embedded values when a source changes:
    /// - Temporarily add `eprintln!("sig = {}", sig_b64)` to
    ///   `sign_src_ed25519`, run this test, copy the new values into
    ///   the corresponding `wat-tests/` file, remove the eprintln.
    /// - OR use a scratch `src/bin/` binary that calls
    ///   `sign_src_ed25519` and prints.
    ///
    /// The signing key is fixed at `[7u8; 32]`, so the pubkey is
    /// deterministic across runs — same discipline as `load.rs`'s
    /// `fixed_signing_key` helper.
    #[test]
    fn wat_test_embedded_signatures_verify() {
        // The two sources used by wat-tests/holon/eval-coincident.wat's
        // signed variants (slices in that file's test-signed-*
        // deftests). If these source strings diverge from what's in
        // the .wat file, the sig constants below will not match — fix
        // by regenerating both.
        const SRC_A: &str = "(:wat::i64::+ 2 2)";
        const SRC_B: &str = "(:wat::i64::* 1 4)";

        // Embedded constants — if a wat-tests/ file changes a source,
        // update these AND the string literals in that file together.
        // Arc 255 Stone C — regenerated after the `:wat::core::i64::+`/`*` ->
        // `:wat::i64::+`/`*` rename in source strings (the old spelling retired).
        const EXPECTED_SRC_A_SIG: &str = "LyePIYwXIW1CYKuv7BQeDMs0hV7+89uBVmbUCjTiLkZ9KKrcXkVVANj2BdX6bMUb4CwkwNMoGBZAWG/zI0rAAQ==";
        // Arc 255 Stone C — regenerated after the `:wat::core::i64::+`/`*` ->
        // `:wat::i64::+`/`*` rename in source strings (the old spelling retired).
        const EXPECTED_SRC_B_SIG: &str = "dT8mJMDyrhLj2GSQtwP6ptpP9USXuLPGyH6t5H47Xb9zkej3J7qSyEdl0SU2frQRSd4sySA4/Ogt2RnapekpAQ==";
        const EXPECTED_PK: &str = "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=";

        let (sig_a, pk_a) = sign_src_ed25519(SRC_A);
        let (sig_b, pk_b) = sign_src_ed25519(SRC_B);

        assert_eq!(
            pk_a, EXPECTED_PK,
            "public key drifted; update wat-tests/holon/eval-coincident.wat"
        );
        assert_eq!(
            pk_a, pk_b,
            "same signing key produces same pubkey for both sources"
        );
        assert_eq!(
            sig_a, EXPECTED_SRC_A_SIG,
            "SRC_A signature drifted; source changed? regenerate and update wat-tests/holon/eval-coincident.wat"
        );
        assert_eq!(
            sig_b, EXPECTED_SRC_B_SIG,
            "SRC_B signature drifted; source changed? regenerate and update wat-tests/holon/eval-coincident.wat"
        );
    }

    #[test]
    fn eval_signed_coincident_q_true_for_equivalent_verified_sources() {
        let src_a = "(:wat::i64::+ 2 2)";
        let src_b = "(:wat::i64::* 1 4)";
        let (sig_a, pk_a) = sign_src_ed25519(src_a);
        let (sig_b, pk_b) = sign_src_ed25519(src_b);
        let program = format!(
            r#"(:wat::holon::eval-signed-string-coincident?
 "{src_a}"
                 :wat::verify::signed-ed25519
                 :wat::verify::string "{sig_a}"
                 :wat::verify::string "{pk_a}"
 "{src_b}"
                 :wat::verify::signed-ed25519
                 :wat::verify::string "{sig_b}"
                 :wat::verify::string "{pk_b}")"#
        );
        let result = eval_with_ctx(&program, 1024).unwrap();
        assert!(matches!(eval_ok_inner(result), Value::bool(true)));
    }

    #[test]
    fn eval_signed_coincident_q_err_on_bad_signature() {
        let src_a = "(:wat::i64::+ 2 2)";
        let src_b = "(:wat::i64::* 1 4)";
        let (_sig_a, pk_a) = sign_src_ed25519(src_a);
        let (sig_b, pk_b) = sign_src_ed25519(src_b);
        // Side A carries a signature for a DIFFERENT source (src_b's
        // sig over src_a) → verification fails → EvalError
        // kind="verification-failed".
        let wrong_sig = sig_b.clone();
        let program = format!(
            r#"(:wat::holon::eval-signed-string-coincident?
 "{src_a}"
                 :wat::verify::signed-ed25519
                 :wat::verify::string "{wrong_sig}"
                 :wat::verify::string "{pk_a}"
 "{src_b}"
                 :wat::verify::signed-ed25519
                 :wat::verify::string "{sig_b}"
                 :wat::verify::string "{pk_b}")"#
        );
        let result = eval_with_ctx(&program, 1024).unwrap();
        let (kind, _msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "verification-failed");
    }

    #[test]
    fn cosine_of_atom_with_itself_is_one() {
        // The renamed primitive (algebra::cosine) returns the same
        // scalar the old :wat::core::presence did.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::holon::cosine
                 (:wat::holon::to-holon "self")
                 (:wat::holon::to-holon "self"))"#,
            1024,
        )
        .unwrap();
        let x = expect_cosine_similarity(result);
        assert!((x - 1.0).abs() < 1e-9, "got {}", x);
    }

    #[test]
    fn stopped_q_reads_kernel_flag() {
        // The renamed primitive — stopped? per the `?` convention.
        reset_kernel_stop();
        assert!(matches!(
            eval_expr("(:wat::kernel::stopped?)").unwrap(),
            Value::bool(false)
        ));
        request_kernel_stop();
        assert!(matches!(
            eval_expr("(:wat::kernel::stopped?)").unwrap(),
            Value::bool(true)
        ));
        reset_kernel_stop();
    }

    #[test]
    fn presence_requires_encoding_ctx() {
        // Without a frozen SymbolTable, presence must error — can't
        // reach into encoding machinery that doesn't exist.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let ast = crate::parse_one!(
            r#"(:wat::holon::cosine
                 (:wat::holon::to-holon "a")
                 (:wat::holon::to-holon "b"))"#
        )
        .unwrap();
        let err = eval_inner(&ast, &Environment::new(), &SymbolTable::new()).unwrap_err();
        assert!(matches!(
            err,
            EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::NoEncodingCtx { op, .. } if op == ":wat::holon::cosine")
        ));
    }

    #[test]
    fn bind_obscures_child_at_vector_level() {
        // Core claim: cosine(encode(p), encode(Bind(k, p))) is near zero —
        // MAP bind orthogonalizes. The presence of p in Bind(k,p) is
        // below the substrate's presence floor (15σ at d=1024).
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::core::let
                 [program (:wat::holon::to-holon "the-program")
                  key (:wat::holon::to-holon "the-key")
                  bound (:wat::holon::Bind key program)]
                 (:wat::holon::cosine program bound))"#,
            1024,
        )
        .unwrap();
        // Arc 024: presence_floor = 15 * (1/sqrt(1024)) = 15/32 ≈ 0.469.
        let presence_floor = 15.0 / (1024f64).sqrt();
        let x = expect_cosine_similarity(result);
        // Cosine is ternary-vector small, well below the presence floor.
        assert!(
            x < presence_floor,
            "expected cosine below presence floor {}, got {}",
            presence_floor,
            x
        );
    }

    #[test]
    fn bind_on_bind_recovers_child_at_vector_level() {
        // Self-inverse: cosine(encode(p), encode(Bind(Bind(k,p), k))) is
        // well above the presence floor. MAP's bind(bind(k,p), k) ≈ p on
        // non-zero positions of k.
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let result = eval_with_ctx(
            r#"(:wat::core::let
                 [program (:wat::holon::to-holon "the-program")
                  key (:wat::holon::to-holon "the-key")
                  bound (:wat::holon::Bind key program)
                  recovered (:wat::holon::Bind bound key)]
                 (:wat::holon::cosine program recovered))"#,
            1024,
        )
        .unwrap();
        let presence_floor = 15.0 / (1024f64).sqrt();
        let x = expect_cosine_similarity(result);
        assert!(
            x > presence_floor,
            "expected cosine above presence floor {}, got {}",
            presence_floor,
            x
        );
    }

    // Arc 037 slice 6: :wat::config::dims and :wat::config::noise-floor
    // accessors retired. dims is no longer a single value (router
    // picks per construction); noise-floor is per-d, computed on
    // Encoders via the ambient sigma-fn. The tests that verified
    // those accessors are retired alongside the accessors.

    #[test]
    fn eval_edn_bang_inline_string_runs() {
        let result = eval_expr(r#"(:wat::eval-edn! "(:wat::i64::+ 40 2)")"#).unwrap();
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
    }

    // Arc 028 slice 3 retired `eval_edn_bang_unknown_iface_refused`
    // and `eval_edn_bang_reserved_unimplemented_iface_refused` —
    // both asserted that unknown / reserved iface keywords were
    // rejected. After the iface-drop, those keywords have no meaning
    // in the grammar; the arity check fires instead and the tests
    // stopped describing a real behavior.
    #[test]
    fn eval_edn_bang_wrong_arity_rejected() {
        // Arity fires BEFORE the EvalError wrap — structural /
        // caller-syntactic error, not a runtime evaluation failure.
        let err = eval_expr(r#"(:wat::eval-edn! "foo" "bar")"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn eval_edn_bang_refuses_mutation_inside_string() {
        // The parsed AST from the string still walks through the
        // mutation-form guard — now surfaced as EvalError data.
        // Stone 241.16 — migrated from `:wat::core::define` (HARD CUT; no longer in
        // is_mutation_head) to `:wat::core::defstruct` (still a recognized mutation form).
        // eval-edn! does not expand macros before checking; defstruct head preserved as-is.
        // Mechanism under test: eval-edn! refuses ANY mutation-headed form inside the string.
        let result =
            eval_expr(r#"(:wat::eval-edn! "(:wat::core::defstruct :evil::T [x <- :i64])")"#)
                .unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "mutation-form-refused");
    }

    #[test]
    fn eval_digest_bang_valid_hex_runs() {
        use sha2::Digest as _;
        let source = r#"(:wat::i64::+ 1 1)"#;
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let form = format!(
            r#"(:wat::eval-digest-string!
 "{}"
                :wat::verify::digest-sha256
                :wat::verify::string "{}")"#,
            source, hex
        );
        let result = eval_expr(&form).unwrap();
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(2)));
    }

    #[test]
    fn eval_digest_bang_mismatch_refused() {
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        let form = format!(
            r#"(:wat::eval-digest-string!
 "(:wat::i64::+ 1 1)"
                :wat::verify::digest-sha256
                :wat::verify::string "{}")"#,
            wrong
        );
        let result = eval_expr(&form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "verification-failed");
    }

    #[test]
    fn eval_digest_bang_unknown_algo_refused() {
        let form = r#"(:wat::eval-digest-string!
 "(:wat::i64::+ 1 1)"
            :wat::verify::signed-ed25519
            :wat::verify::string "abc")"#;
        let result = eval_expr(form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        // signed-ed25519 in a digest slot is a grammar error surfaced
        // as malformed-form inside the wrap.
        assert_eq!(kind, "malformed-form");
    }

    #[test]
    fn eval_signed_bang_valid_sig_runs() {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        use ed25519_dalek::{Signer, SigningKey};
        let source = r#"(:wat::i64::+ 20 22)"#;
        let sk = SigningKey::from_bytes(&[17u8; 32]);
        let forms = crate::parse_all!(source).unwrap();
        let hash = crate::hash::hash_canonical_program(&forms);
        let sig = sk.sign(&hash);
        let sig_b64 = B64.encode(sig.to_bytes());
        let pk_b64 = B64.encode(sk.verifying_key().as_bytes());
        let form = format!(
            r#"(:wat::eval-signed-string!
 "{}"
                :wat::verify::signed-ed25519
                :wat::verify::string "{}"
                :wat::verify::string "{}")"#,
            source, sig_b64, pk_b64
        );
        let result = eval_expr(&form).unwrap();
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
    }

    #[test]
    fn eval_signed_bang_tampered_source_refused() {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        use ed25519_dalek::{Signer, SigningKey};
        let signed_source = r#"(:wat::i64::+ 20 22)"#;
        let tampered_source = r#"(:wat::i64::+ 99 99)"#;
        let sk = SigningKey::from_bytes(&[17u8; 32]);
        let forms = crate::parse_all!(signed_source).unwrap();
        let hash = crate::hash::hash_canonical_program(&forms);
        let sig = sk.sign(&hash);
        let sig_b64 = B64.encode(sig.to_bytes());
        let pk_b64 = B64.encode(sk.verifying_key().as_bytes());
        let form = format!(
            r#"(:wat::eval-signed-string!
 "{}"
                :wat::verify::signed-ed25519
                :wat::verify::string "{}"
                :wat::verify::string "{}")"#,
            tampered_source, sig_b64, pk_b64
        );
        let result = eval_expr(&form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "verification-failed");
    }

    #[test]
    fn eval_signed_bang_wrong_algo_kind_refused() {
        // digest-sha256 in a signed slot is a grammar error.
        let form = r#"(:wat::eval-signed-string!
 "(:wat::i64::+ 1 1)"
            :wat::verify::digest-sha256
            :wat::verify::string "sig"
            :wat::verify::string "pk")"#;
        let result = eval_expr(form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "malformed-form");
    }

    // ─── File-path interface (real runtime I/O) ─────────────────────────

    fn write_temp(contents: &str, suffix: &str) -> std::path::PathBuf {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "wat-eval-test-{}-{}.{}",
            std::process::id(),
            // Unique per test via a nanos timestamp.
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            suffix
        ));
        let mut f = std::fs::File::create(&path).expect("create temp");
        f.write_all(contents.as_bytes()).expect("write");
        path
    }

    #[test]
    fn eval_file_bang_runs() {
        let path = write_temp("(:wat::i64::+ 10 11)", "wat");
        let form = format!(r#"(:wat::eval-file! "{}")"#, path.display());
        let result = eval_expr_with_fs(&form).expect("eval");
        let _ = std::fs::remove_file(&path);
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(21)));
    }

    #[test]
    fn eval_file_bang_missing_path_errors() {
        let form = r#"(:wat::eval-file! "/nonexistent/path/abc.xyz")"#;
        let result = eval_expr_with_fs(form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "malformed-form");
    }

    #[test]
    fn eval_digest_bang_sidecar_file_runs() {
        use sha2::Digest as _;
        let source = "(:wat::i64::* 6 7)";
        let source_path = write_temp(source, "wat");
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let digest_path = write_temp(&hex, "sha256");
        let form = format!(
            r#"(:wat::eval-digest!
 "{}"
                :wat::verify::digest-sha256
                :wat::verify::file-path "{}")"#,
            source_path.display(),
            digest_path.display()
        );
        let result = eval_expr_with_fs(&form).expect("eval");
        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&digest_path);
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
    }

    // ─── User signals — kernel measures, userland owns transitions ─────
    //
    // The three user-signal flags are process-lifetime AtomicBool statics
    // (KERNEL_SIGUSR1 / SIGUSR2 / SIGHUP in this file). P3
    // (DESIGN-STONE-process-signal-owner-to-child.md) retired the three
    // flag-state tests that used to live here: each called a setter and
    // then a getter inside the harness's OWN process — no signal was ever
    // delivered, no handler ever ran, and the comment this block replaces
    // explained away their races as a runner-dependent accident rather than
    // naming the deeper defect. Real process measurements now live as
    // deftests in `wat-tests/process/` (`signal-user1-delivers-child-
    // observes.wat`, `signal-user2-and-hangup-independent.wat`,
    // `signal-reset-sigusr1-is-a-transition.wat`) — each spawns a real
    // child, signals it, and asserts on what the CHILD reports observing.
    //
    // The two tests below survive because neither one actually asserts a
    // flag: `reset_sighup_returns_unit` asserts the verb's return SHAPE
    // (`Value::Unit`), independent of any prior flag state, and
    // `user_signal_predicates_refuse_arguments` asserts `ArityMismatch`
    // shape only. They no longer touch the process-global statics at all.

    #[test]
    fn reset_sighup_returns_unit() {
        let v = eval_expr("(:wat::kernel::reset-sighup!)").expect("reset");
        assert!(matches!(v, Value::Unit));
    }

    #[test]
    fn user_signal_predicates_refuse_arguments() {
        assert!(matches!(
            eval_expr("(:wat::kernel::sigusr1? 1)"),
            Err(EvalBreak::Diagnostic(e)) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. })
        ));
        assert!(matches!(
            eval_expr("(:wat::kernel::reset-sigusr1! true)"),
            Err(EvalBreak::Diagnostic(e)) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. })
        ));
    }

    // ─── Tuples + destructure + first/second ───────────────────────────

    /// Helper: evaluate `src` in an env pre-bound with `name -> value`.
    fn eval_with_binding(src: &str, name: &str, value: Value) -> Result<Value, EvalBreak> {
        let ast = crate::parse_one!(src).expect("parse ok");
        let env = Environment::new()
            .child()
            .bind_unknown_span(name, TrackedValue::from(value))
            .build();
        eval_inner(&ast, &env, &SymbolTable::new()).map(|tv| tv.value_owned())
    }

    fn pair(a: Value, b: Value) -> Value {
        Value::Tuple(Arc::new(vec![a, b]))
    }

    #[test]
    fn first_extracts_zeroth_element() {
        let p = pair(Value::i64(10), Value::i64(20));
        match eval_with_binding("(:wat::core::first pair)", "pair", p).unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10, got {:?}", v),
        }
    }

    #[test]
    fn second_extracts_first_element() {
        let p = pair(Value::i64(10), Value::i64(20));
        match eval_with_binding("(:wat::core::second pair)", "pair", p).unwrap() {
            Value::i64(20) => {}
            v => panic!("expected 20, got {:?}", v),
        }
    }

    #[test]
    fn first_refuses_non_tuple() {
        let err = eval_with_binding("(:wat::core::first v)", "v", Value::i64(42)).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn first_index_out_of_range_on_empty_tuple() {
        let t = Value::Tuple(Arc::new(vec![]));
        let err = eval_with_binding("(:wat::core::first t)", "t", t).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn let_destructures_a_pair() {
        let src = r#"
            (:wat::core::let [[a b] p] (:wat::i64::+ a b))
        "#;
        let p = pair(Value::i64(3), Value::i64(4));
        match eval_with_binding(src, "p", p).unwrap() {
            Value::i64(7) => {}
            v => panic!("expected 7, got {:?}", v),
        }
    }

    #[test]
    fn let_destructure_arity_mismatch_errors() {
        let src = r#"
            (:wat::core::let [[a b c] p] a)
        "#;
        let p = pair(Value::i64(1), Value::i64(2));
        let err = eval_with_binding(src, "p", p).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn let_destructure_requires_tuple() {
        let src = r#"
            (:wat::core::let [[a b] v] a)
        "#;
        let err = eval_with_binding(src, "v", Value::i64(42)).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    // ─── Vector primitives (Round 4a) ───────────────────────────────

    #[test]
    fn vector_constructor_produces_vec() {
        // Arc 163 slice 3d — :wat::core::Vector is the canonical constructor;
        // :wat::core::vec and :wat::core::list are retired.
        let v = eval_expr("(:wat::core::Vector :- [:i64] 1 2 3)").unwrap();
        match v {
            Value::Vec(a) => {
                assert_eq!(a.len(), 3);
                match (&a[0], &a[1], &a[2]) {
                    (Value::i64(1), Value::i64(2), Value::i64(3)) => {}
                    _ => panic!("expected [1, 2, 3]"),
                }
            }
            _ => panic!("expected Vec value"),
        }
    }

    #[test]
    fn length_of_three_element_vec() {
        match eval_expr("(:wat::core::length (:wat::core::Vector :- [:i64] 1 2 3))").unwrap() {
            Value::i64(3) => {}
            v => panic!("expected 3, got {:?}", v),
        }
    }

    #[test]
    fn empty_true_on_empty_vec() {
        match eval_expr("(:wat::core::empty? (:wat::core::Vector :- [:i64]))").unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn empty_false_on_nonempty_vec() {
        match eval_expr("(:wat::core::empty? (:wat::core::Vector :- [:i64] 1))").unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false, got {:?}", v),
        }
    }

    #[test]
    fn reverse_flips_order() {
        match eval_expr("(:wat::core::reverse (:wat::core::Vector :- [:i64] 1 2 3))").unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                assert_eq!(ns, vec![3, 2, 1]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn range_start_end() {
        match eval_expr("(:wat::core::range 0 4)").unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                assert_eq!(ns, vec![0, 1, 2, 3]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn range_start_geq_end_is_empty() {
        match eval_expr("(:wat::core::range 5 5)").unwrap() {
            Value::Vec(items) => assert!(items.is_empty()),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    // Arc 118.2a — `take`/`drop`/`map` flipped LAZY (return `Value::wat__stream__Stream`, not
    // `Value::Vec`, directly). These tests still exercise the SAME op + assert the SAME
    // resulting values; the source string now materializes via `(:wat::core::into [] …)` so
    // the Rust-side assertion (unchanged: still expects `Value::Vec`) stays meaningful without
    // needing to hand-walk a `Stream` from this test module.
    #[test]
    fn take_first_n() {
        match eval_expr(
            "(:wat::core::into [] (:wat::core::take (:wat::core::Vector :- [:i64] 1 2 3 4 5) 3))",
        )
        .unwrap()
        {
            Value::Vec(items) => assert_eq!(items.len(), 3),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn take_more_than_length_returns_full_vec() {
        match eval_expr("(:wat::core::into [] (:wat::core::take (:wat::core::Vector :- [:i64] 1 2) 99))")
            .unwrap()
        {
            Value::Vec(items) => assert_eq!(items.len(), 2),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn drop_skips_first_n() {
        match eval_expr(
            "(:wat::core::into [] (:wat::core::drop (:wat::core::Vector :- [:i64] 1 2 3 4 5) 2))",
        )
        .unwrap()
        {
            Value::Vec(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    Value::i64(3) => {}
                    v => panic!("expected 3, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn map_doubles_every_element() {
        let src = r#"
            (:wat::core::into []
              (:wat::core::map
                (:wat::core::fn [x <- :i64] -> :i64 (:wat::i64::* x 2))
                (:wat::core::Vector :- [:i64] 1 2 3)))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                assert_eq!(ns, vec![2, 4, 6]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn foldl_sums_with_init() {
        let src = r#"
            (:wat::core::foldl
              (:wat::core::fn [acc <- :i64 x <- :i64] -> :i64
                (:wat::i64::+ acc x))
              10
              (:wat::core::Vector :- [:i64] 1 2 3 4))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(20) => {}
            v => panic!("expected 20, got {:?}", v),
        }
    }

    #[test]
    fn list_window_builds_sliding_windows() {
        let src = r#"
            (:wat::seq::window (:wat::core::Vector :- [:i64] 1 2 3 4) 2)
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(outer) => {
                // Expect 3 windows of size 2.
                assert_eq!(outer.len(), 3);
                // First window = [1, 2].
                match &outer[0] {
                    Value::Vec(w) => {
                        assert_eq!(w.len(), 2);
                        match (&w[0], &w[1]) {
                            (Value::i64(1), Value::i64(2)) => {}
                            other => panic!("expected [1,2], got {:?}", other),
                        }
                    }
                    v => panic!("expected Vec window, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn seq_window_accepts_list_too() {
        // Arc 255 Stone HOME-9, acceptance row 2 — the SEQABLE PROOF: `window` used to call
        // `require_vec` and REJECT a `List` outright; it accepts one now, same result shape as
        // the Vector case immediately above.
        let src = r#"
            (:wat::seq::window (:wat::core::List 1 2 3 4) 2)
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(outer) => {
                assert_eq!(outer.len(), 3);
                match &outer[0] {
                    Value::Vec(w) => {
                        assert_eq!(w.len(), 2);
                        match (&w[0], &w[1]) {
                            (Value::i64(1), Value::i64(2)) => {}
                            other => panic!("expected [1,2], got {:?}", other),
                        }
                    }
                    v => panic!("expected Vec window, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn first_polymorphic_on_vec() {
        // Arc 047 — first on Vec now returns bare T (raises on out-of-range).
        let v = eval_expr("(:wat::core::first (:wat::core::Vector :- [:i64] 10 20 30))").unwrap();
        assert_eq!(expect_i64(v), 10);
    }

    #[test]
    fn first_on_empty_vec_returns_none() {
        // Arc 047 — empty-range access uses get (safe Option path).
        expect_none(eval_expr("(:wat::core::get (:wat::core::Vector :- [:i64]) 0)").unwrap());
    }

    #[test]
    fn second_polymorphic_on_vec() {
        let v = eval_expr("(:wat::core::second (:wat::core::Vector :- [:i64] 10 20 30))").unwrap();
        assert_eq!(expect_i64(v), 20);
    }

    #[test]
    fn third_on_vec() {
        let v = eval_expr("(:wat::core::third (:wat::core::Vector :- [:i64] 10 20 30))").unwrap();
        assert_eq!(expect_i64(v), 30);
    }

    #[test]
    fn third_on_short_vec_returns_none() {
        // Arc 047 — out-of-range access uses get (safe Option path).
        expect_none(eval_expr("(:wat::core::get (:wat::core::Vector :- [:i64] 10 20) 2)").unwrap());
    }

    // ─── last + find-last-index + f64::max-of/min-of (arc 047) ────────────

    #[test]
    fn last_returns_some_for_non_empty() {
        let v = expect_some(
            eval_expr("(:wat::core::last (:wat::core::Vector :- [:i64] 1 2 3 99))").unwrap(),
        );
        assert_eq!(expect_i64(v), 99);
    }

    #[test]
    fn last_returns_none_for_empty() {
        expect_none(eval_expr("(:wat::core::last (:wat::core::Vector :- [:i64]))").unwrap());
    }

    #[test]
    fn find_last_index_returns_rightmost_match() {
        let src = r#"
            (:wat::core::find-last-index
              (:wat::core::Vector :- [:i64] 5 12 3 18 7)
              (:wat::core::fn [x <- :i64] -> :bool (:wat::i64::> x 10)))
        "#;
        let v = expect_some(eval_expr(src).unwrap());
        assert_eq!(expect_i64(v), 3); // index of 18 (last x > 10)
    }

    #[test]
    fn find_last_index_returns_none_for_no_match() {
        let src = r#"
            (:wat::core::find-last-index
              (:wat::core::Vector :- [:i64] 1 2 3)
              (:wat::core::fn [x <- :i64] -> :bool (:wat::i64::> x 99)))
        "#;
        expect_none(eval_expr(src).unwrap());
    }

    #[test]
    fn find_last_index_returns_none_for_empty() {
        let src = r#"
            (:wat::core::find-last-index
              (:wat::core::Vector :- [:i64])
              (:wat::core::fn [x <- :i64] -> :bool (:wat::i64::> x 0)))
        "#;
        expect_none(eval_expr(src).unwrap());
    }

    // Arc 255 Stone C — `:wat::f64::max-of` / `min-of` are VARIADIC (bare args), a
    // genuinely different calling convention from the retired
    // `:wat::core::f64::max-of` / `min-of` (single `Vector` arg) — see
    // `src/intrinsic/f64.rs`'s module header.

    #[test]
    fn f64_max_of_picks_largest() {
        let v = expect_some(
            eval_expr("(:wat::f64::max-of -1.5 4.2 2.0 4.2 0.0)")
                .unwrap(),
        );
        assert_eq!(expect_f64(v), 4.2);
    }

    #[test]
    fn f64_min_of_picks_smallest() {
        let v = expect_some(
            eval_expr("(:wat::f64::min-of -1.5 4.2 2.0 -1.5 0.0)")
                .unwrap(),
        );
        assert_eq!(expect_f64(v), -1.5);
    }

    #[test]
    fn f64_max_of_singleton_returns_single() {
        let v = expect_some(
            eval_expr("(:wat::f64::max-of 7.5)").unwrap(),
        );
        assert_eq!(expect_f64(v), 7.5);
    }

    #[test]
    fn f64_max_of_empty_returns_none() {
        expect_none(eval_expr("(:wat::f64::max-of)").unwrap());
    }

    #[test]
    fn f64_min_of_empty_returns_none() {
        expect_none(eval_expr("(:wat::f64::min-of)").unwrap());
    }

    #[test]
    fn rest_drops_first() {
        match eval_expr("(:wat::core::rest (:wat::core::Vector :- [:i64] 1 2 3))").unwrap() {
            Value::Vec(items) => {
                assert_eq!(items.len(), 2);
                match (&items[0], &items[1]) {
                    (Value::i64(2), Value::i64(3)) => {}
                    other => panic!("expected [2,3]; got {:?}", other),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn rest_of_empty_errors() {
        let err = eval_expr("(:wat::core::rest (:wat::core::Vector :- [:i64]))").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn map_indexed_attaches_positions_replacing_deleted_map_with_index() {
        // Arc 255 Stone HOME-9 — `:wat::std::list::map-with-index` is DELETED;
        // `:wat::core::map-indexed` is its replacement, migrated deliberately (NOT a drop-in):
        // the argument order flips ((Vector,fn) -> (fn,coll)), the closure's own params flip
        // too ((item,index) -> (index,item)), and the result is a lazy Stream, not an eager
        // Vector — `into []` forces it back to the same shape this test used to assert on.
        // Same VALUES as the deleted verb's test: 10+0, 20+1, 30+2.
        let src = r#"
            (:wat::core::into []
              (:wat::core::map-indexed
                (:wat::core::fn [i <- :i64 x <- :i64] -> :i64
                  (:wat::i64::+ x i))
                (:wat::core::Vector :- [:i64] 10 20 30)))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                // 10+0, 20+1, 30+2
                assert_eq!(ns, vec![10, 21, 32]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    // ─── HashMap ───────────────────────────────────────────────────────

    #[test]
    fn hashmap_constructor_even_arity() {
        let v = eval_expr(r#"(:wat::core::HashMap :- [:String :i64] "a" 1 "b" 2)"#).unwrap();
        match v {
            Value::wat__std__HashMap(m) => {
                assert_eq!(m.len(), 2);
            }
            v => panic!("expected HashMap, got {:?}", v),
        }
    }

    #[test]
    fn hashmap_constructor_odd_arity_errors() {
        let err = eval_expr(r#"(:wat::core::HashMap :- [:String :i64] "a" 1 "b")"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn hashmap_get_hit_returns_some() {
        let src = r#"
            (:wat::core::let
              [m (:wat::core::HashMap :- [:String :i64] "a" 10 "b" 20)]
              (:wat::core::match (:wat::core::get m "a")
                ((:wat::core::Some n) n)
                (:wat::core::None 0)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10, got {:?}", v),
        }
    }

    #[test]
    fn hashmap_get_miss_returns_none() {
        let src = r#"
            (:wat::core::let
              [m (:wat::core::HashMap :- [:String :i64] "a" 10)]
              (:wat::core::match (:wat::core::get m "missing")
                ((:wat::core::Some n) n)
                (:wat::core::None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(-1) => {}
            v => panic!("expected -1 (miss path), got {:?}", v),
        }
    }

    #[test]
    fn hashmap_contains_tracks_membership() {
        let src = r#"
            (:wat::core::let
              [m (:wat::core::HashMap :- [:String :i64] "a" 10)]
              (:wat::core::contains? m "a"))
        "#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(true)));
        let src_missing = r#"
            (:wat::core::let
              [m (:wat::core::HashMap :- [:String :i64] "a" 10)]
              (:wat::core::contains? m "b"))
        "#;
        assert!(matches!(
            eval_expr(src_missing).unwrap(),
            Value::bool(false)
        ));
    }

    #[test]
    fn hashmap_int_and_string_keys_dont_collide() {
        // "42" (String) and 42 (i64) should be distinct keys — type-tag
        // prefix in the canonical key string prevents collision.
        let src = r#"
            (:wat::core::let
              [m
                (:wat::core::HashMap :- [:String :i64] "42" 100)]
              (:wat::core::contains? m 42))
        "#;
        // Map has one entry under String "42". Contains? with i64 key 42
        // would stringify to "I:42" — different from "S:42" — no match.
        match eval_expr(src).unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false (no collision), got {:?}", v),
        }
    }

    #[test]
    fn hashmap_accepts_composite_key() {
        // Arc 216.5a + 216.5c: Value: Hash + Eq is canonical; HashMap
        // storage is Arc<HashMap<Value, Value>>. Composite keys (Vec,
        // HashSet, Tuple, etc.) are accepted natively — the
        // "primitives-only" restriction of the pre-antidote substrate is
        // gone.
        let result = eval_expr(
            r#"(:wat::core::HashMap :- [(:wat::core::Vector :- [:i64]) :String] (:wat::core::Vector :- [:i64] 1 2) "x")"#,
        );
        assert!(
            result.is_ok(),
            "composite key should construct HashMap; got {:?}",
            result
        );
    }

    #[test]
    fn hashmap_get_requires_hashmap_arg() {
        // Arc 237 Stone 237.7b-iv — `:wat::core::get` is now a Rust ∀T intrinsic
        // (eval_get). Calling it on a non-collection Value produces TypeMismatch
        // (teaching error from eval_get's else-arm), superseding the old
        // MalformedForm (no-arm-match from define-dispatch, arc 146 slice 3).
        let err = eval_expr(r#"(:wat::core::get 42 "k")"#).unwrap_err();
        assert!(
            matches!(&err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. })),
            "expected TypeMismatch (eval_get else-arm); got {:?}",
            err
        );
    }

    // ─── :wat::core::assoc (arc 020) ───────────────────────────────────

    #[test]
    fn assoc_adds_entry_returning_new_map() {
        let src = r#"
            (:wat::core::let
              [m0
                (:wat::core::HashMap :- [:String :i64])
               m1
                (:wat::core::assoc m0 "count" 1)]
              (:wat::core::match (:wat::core::get m1 "count")
                ((:wat::core::Some n) n)
                (:wat::core::None 0)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(1) => {}
            v => panic!("expected 1, got {:?}", v),
        }
    }

    #[test]
    fn assoc_overwrites_existing_key() {
        let src = r#"
            (:wat::core::let
              [m0
                (:wat::core::HashMap :- [:String :i64] "count" 1)
               m1
                (:wat::core::assoc m0 "count" 2)]
              (:wat::core::match (:wat::core::get m1 "count")
                ((:wat::core::Some n) n)
                (:wat::core::None 0)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(2) => {}
            v => panic!("expected 2 (overwrite), got {:?}", v),
        }
    }

    #[test]
    fn assoc_preserves_original_map() {
        // Values-up: the input map is unchanged after assoc returns.
        let src = r#"
            (:wat::core::let
              [m0
                (:wat::core::HashMap :- [:String :i64] "a" 10)
               m1
                (:wat::core::assoc m0 "b" 20)]
              (:wat::core::match (:wat::core::get m0 "b")
                ((:wat::core::Some n) n)
                (:wat::core::None -1)))
        "#;
        // Original m0 doesn't have "b" — assoc returned a new map,
        // m0 stays as {a: 10}.
        match eval_expr(src).unwrap() {
            Value::i64(-1) => {}
            v => panic!("expected -1 (m0 unchanged), got {:?}", v),
        }
    }

    #[test]
    fn assoc_requires_hashmap_arg() {
        let err = eval_expr(r#"(:wat::core::assoc 42 "k" 1)"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn assoc_arity_mismatch() {
        let err =
            eval_expr(r#"(:wat::core::assoc (:wat::core::HashMap :- [:String :i64]) "k")"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── Vec concat (arc 059, arc 146 slice 4) ───────────────────────────
    //
    // Arc 146 slice 4 — variadic concat retired; honest binary shape.
    // `:wat::core::concat` is now an alias for `:wat::core::Vector/concat`
    // (HashMap and other containers excluded per DESIGN audit table).
    // Callers nest for >2 args (or use foldl over a Vec of Vecs).

    #[test]
    fn concat_two_arg_basic() {
        let src = r#"
            (:wat::core::length
              (:wat::core::concat
                (:wat::core::Vector :- [:i64] 1 2)
                (:wat::core::Vector :- [:i64] 3 4)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(4) => {}
            v => panic!("expected 4, got {:?}", v),
        }
    }

    #[test]
    fn concat_nested_for_more_than_two() {
        // Sum the result to verify all elements made it through in order.
        // Variadic 4-arg form is no longer supported; nest two binary
        // concats instead.
        let src = r#"
            (:wat::core::foldl
              (:wat::core::fn [acc <- :i64 n <- :i64] -> :i64
                (:wat::i64::+ acc n))
              0
              (:wat::core::concat
                (:wat::core::concat
                  (:wat::core::Vector :- [:i64] 1)
                  (:wat::core::Vector :- [:i64] 2))
                (:wat::core::concat
                  (:wat::core::Vector :- [:i64] 3)
                  (:wat::core::Vector :- [:i64] 4))))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10, got {:?}", v),
        }
    }

    #[test]
    fn concat_empty_vecs() {
        let src = r#"
            (:wat::core::length
              (:wat::core::concat
                (:wat::core::Vector :- [:i64])
                (:wat::core::Vector :- [:i64] 1 2)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(2) => {}
            v => panic!("expected 2, got {:?}", v),
        }

        let all_empty = r#"
            (:wat::core::length
              (:wat::core::concat
                (:wat::core::Vector :- [:i64])
                (:wat::core::Vector :- [:i64])))
        "#;
        match eval_expr(all_empty).unwrap() {
            Value::i64(0) => {}
            v => panic!("expected 0 for all-empty concat, got {:?}", v),
        }
    }

    #[test]
    fn concat_preserves_left_to_right_order() {
        // First element of (concat [10] (concat [20] [30])) must be 10.
        let src = r#"
            (:wat::core::match
              (:wat::core::get
                (:wat::core::concat
                  (:wat::core::Vector :- [:i64] 10)
                  (:wat::core::concat
                    (:wat::core::Vector :- [:i64] 20)
                    (:wat::core::Vector :- [:i64] 30)))
                0)
              ((:wat::core::Some n) n)
              (:wat::core::None -1))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10 (first element), got {:?}", v),
        }
    }

    #[test]
    fn concat_non_vec_arg_rejected() {
        let err = eval_expr(r#"(:wat::core::concat (:wat::core::Vector :- [:i64] 1) 42)"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn concat_zero_arg_rejected() {
        // Post-slice-4 the alias has arity 2; zero-arg → ArityMismatch.
        let err = eval_expr(r#"(:wat::core::concat)"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── HashMap completion (arc 058) — dissoc / keys / values ───────────

    #[test]
    fn dissoc_removes_existing_key() {
        let src = r#"
            (:wat::core::let
              [m0
                (:wat::core::HashMap :- [:String :i64] "a" 1 "b" 2)
               m1
                (:wat::core::dissoc m0 "a")]
              (:wat::core::match (:wat::core::get m1 "a")
                ((:wat::core::Some n) n)
                (:wat::core::None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(-1) => {}
            v => panic!("expected -1 (key removed), got {:?}", v),
        }
    }

    #[test]
    fn dissoc_missing_key_is_no_op() {
        let src = r#"
            (:wat::core::let
              [m0
                (:wat::core::HashMap :- [:String :i64] "a" 1)
               m1
                (:wat::core::dissoc m0 "missing")]
              (:wat::core::match (:wat::core::get m1 "a")
                ((:wat::core::Some n) n)
                (:wat::core::None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(1) => {}
            v => panic!("expected 1 (no-op preserves entries), got {:?}", v),
        }
    }

    #[test]
    fn dissoc_preserves_original_map() {
        // Values-up: input map still has the key after dissoc returns.
        let src = r#"
            (:wat::core::let
              [m0
                (:wat::core::HashMap :- [:String :i64] "a" 1 "b" 2)
               _m1
                (:wat::core::dissoc m0 "a")]
              (:wat::core::match (:wat::core::get m0 "a")
                ((:wat::core::Some n) n)
                (:wat::core::None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(1) => {}
            v => panic!("expected 1 (m0 unchanged), got {:?}", v),
        }
    }

    #[test]
    fn dissoc_requires_hashmap_arg() {
        let err = eval_expr(r#"(:wat::core::dissoc 42 "k")"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn dissoc_arity_mismatch() {
        let err =
            eval_expr(r#"(:wat::core::dissoc (:wat::core::HashMap :- [:String :i64]))"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn keys_returns_vec_of_correct_length() {
        let src = r#"
            (:wat::core::length
              (:wat::core::keys
                (:wat::core::HashMap :- [:String :i64] "a" 1 "b" 2 "c" 3)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(3) => {}
            v => panic!("expected 3, got {:?}", v),
        }
    }

    #[test]
    fn keys_empty_map_returns_empty_vec() {
        let src = r#"
            (:wat::core::length
              (:wat::core::keys
                (:wat::core::HashMap :- [:String :i64])))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(0) => {}
            v => panic!("expected 0, got {:?}", v),
        }
    }

    #[test]
    fn keys_contents_match_map() {
        // Order is unspecified — check membership via contains?.
        // Arc 146 slice 3: contains? now tests ELEMENT membership
        // (was: Vec×i64 valid-index). The honest check is "does the
        // returned keys Vec contain each known key string?"
        let src = r#"
            (:wat::core::let
              [ks
                (:wat::core::keys
                  (:wat::core::HashMap :- [:String :i64] "alpha" 1 "beta" 2))]
              (:wat::core::and
                (:wat::core::contains? ks "alpha")
                (:wat::core::contains? ks "beta")))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn keys_requires_hashmap_arg() {
        let err = eval_expr(r#"(:wat::core::keys 42)"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn keys_arity_mismatch() {
        let err = eval_expr(r#"(:wat::core::keys (:wat::core::HashMap :- [:String :i64]) "extra")"#)
            .unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn values_returns_vec_of_correct_length() {
        let src = r#"
            (:wat::core::length
              (:wat::core::values
                (:wat::core::HashMap :- [:String :i64] "a" 1 "b" 2 "c" 3)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(3) => {}
            v => panic!("expected 3, got {:?}", v),
        }
    }

    #[test]
    fn values_empty_map_returns_empty_vec() {
        let src = r#"
            (:wat::core::length
              (:wat::core::values
                (:wat::core::HashMap :- [:String :i64])))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(0) => {}
            v => panic!("expected 0, got {:?}", v),
        }
    }

    #[test]
    fn values_sum_matches_map_values() {
        // Order-agnostic — sum of values is a stable invariant.
        let src = r#"
            (:wat::core::foldl
              (:wat::core::fn [acc <- :i64 v <- :i64] -> :i64
                (:wat::i64::+ acc v))
              0
              (:wat::core::values
                (:wat::core::HashMap :- [:String :i64] "a" 10 "b" 20 "c" 30)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(60) => {}
            v => panic!("expected 60, got {:?}", v),
        }
    }

    #[test]
    fn values_requires_hashmap_arg() {
        let err = eval_expr(r#"(:wat::core::values 42)"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn values_arity_mismatch() {
        let err = eval_expr(r#"(:wat::core::values (:wat::core::HashMap :- [:String :i64]) "extra")"#)
            .unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── empty? polymorphism extension (arc 058) ─────────────────────────

    #[test]
    fn empty_q_hashmap_true_when_empty() {
        let src = r#"
            (:wat::core::empty? (:wat::core::HashMap :- [:String :i64]))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn empty_q_hashmap_false_when_populated() {
        let src = r#"
            (:wat::core::empty? (:wat::core::HashMap :- [:String :i64] "a" 1))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false, got {:?}", v),
        }
    }

    #[test]
    fn empty_q_hashset_polymorphism() {
        let src_empty = r#"(:wat::core::empty? (:wat::core::HashSet :- [:String]))"#;
        match eval_expr(src_empty).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true on empty HashSet, got {:?}", v),
        }
        let src_full = r#"(:wat::core::empty? (:wat::core::HashSet :- [:String] "x"))"#;
        match eval_expr(src_full).unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false on populated HashSet, got {:?}", v),
        }
    }

    // ─── HashSet ───────────────────────────────────────────────────────

    #[test]
    fn hashset_constructor() {
        let v = eval_expr(r#"(:wat::core::HashSet :- [:String] "a" "b" "c")"#).unwrap();
        match v {
            Value::wat__std__HashSet(s) => assert_eq!(s.len(), 3),
            v => panic!("expected HashSet, got {:?}", v),
        }
    }

    #[test]
    fn hashset_collapses_duplicates() {
        let v = eval_expr(r#"(:wat::core::HashSet :- [:String] "a" "a" "b")"#).unwrap();
        match v {
            Value::wat__std__HashSet(s) => assert_eq!(s.len(), 2),
            v => panic!("expected HashSet, got {:?}", v),
        }
    }

    #[test]
    fn hashset_member_present_and_absent() {
        let present = r#"(:wat::core::let
            [s (:wat::core::HashSet :- [:String] "a" "b")]
            (:wat::core::contains? s "a"))"#;
        assert!(matches!(eval_expr(present).unwrap(), Value::bool(true)));
        let absent = r#"(:wat::core::let
            [s (:wat::core::HashSet :- [:String] "a" "b")]
            (:wat::core::contains? s "z"))"#;
        assert!(matches!(eval_expr(absent).unwrap(), Value::bool(false)));
    }

    // ─── Arc 025: polymorphic get / assoc / conj / contains? ─────────

    #[test]
    fn vec_get_hit_returns_some_at_valid_index() {
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::match (:wat::core::get xs 1)
              ((:wat::core::Some v) v)
              (:wat::core::None    -1)))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::i64(20)));
    }

    #[test]
    fn vec_get_out_of_range_returns_none() {
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::match (:wat::core::get xs 5)
              ((:wat::core::Some _) false)
              (:wat::core::None    true)))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(true)));
    }

    #[test]
    fn vec_get_negative_index_returns_none() {
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::match (:wat::core::get xs -1)
              ((:wat::core::Some _) false)
              (:wat::core::None    true)))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(true)));
    }

    // Arc 237 Stone 237.7c — `assoc` is now a ∀T intrinsic (eval_assoc) spanning
    // HashMap + Record. The pre-arc-146 Vec branch was a Vec-as-HashMap anachronism
    // (arc 025); Vec/set is the honest verb for "replace at index" and lives
    // independently. The test below asserts the post-7c honest behaviour:
    // the intrinsic's else-arm returns a TypeMismatch with op ":wat::core::assoc"
    // (mechanism-swap from the old alias's ":wat::core::HashMap/assoc"; correct
    // mechanical correction per arc 237 Stone 237.7c).

    #[test]
    fn assoc_on_vec_rejects_post_slice4() {
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::assoc xs 1 99))"#;
        let err = eval_expr(src).unwrap_err();
        assert!(
            matches!(&err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { op, .. } if op == ":wat::core::assoc")),
            "expected :wat::core::assoc TypeMismatch on Vec; got {:?}",
            err
        );
    }

    #[test]
    fn hashset_conj_adds_element() {
        let src = r#"(:wat::core::let
            [s0 (:wat::core::HashSet :- [:String] "a" "b")
             s1 (:wat::core::conj s0 "c")]
            (:wat::core::contains? s1 "c"))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(true)));
    }

    #[test]
    fn hashset_conj_values_up_preserves_input() {
        let src = r#"(:wat::core::let
            [s0 (:wat::core::HashSet :- [:String] "a" "b")
             _ (:wat::core::conj s0 "c")]
            (:wat::core::contains? s0 "c"))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(false)));
    }

    // Arc 146 slice 3 — Vector/contains? now tests ELEMENT membership
    // (matching HashSet semantics), not valid-index. The pre-arc-146
    // Vec×i64-as-index check was inconsistent with `contains?` across
    // HashSet (which always tested element equality); the dispatch
    // promotion regularises it. Index-validity callers should use
    // `(< i (length xs))` directly.
    #[test]
    fn vec_contains_existing_element_returns_true() {
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::contains? xs 20))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(true)));
    }

    #[test]
    fn vec_contains_missing_element_returns_false() {
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::contains? xs 99))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(false)));
    }

    #[test]
    fn vec_contains_negative_missing_element_returns_false() {
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::contains? xs -1))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(false)));
    }

    // Arc 146 slice 3 — HashSet/get retired (per arc 146 DESIGN audit
    // table: "HashSet's 'get-by-equality' IS just contains?"). The
    // dispatch arms for `:get` now cover only Vector + HashMap; HashSet
    // membership is expressed via `:contains?`. These two tests
    // restructured to assert the contains? equivalent.
    #[test]
    fn hashset_contains_existing_element_returns_true() {
        let src = r#"
            (:wat::core::let
              [s (:wat::core::HashSet :- [:String] "apple" "banana")]
              (:wat::core::contains? s "apple"))
        "#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(true)));
    }

    #[test]
    fn hashset_contains_missing_element_returns_false() {
        let src = r#"
            (:wat::core::let
              [s (:wat::core::HashSet :- [:String] "apple")]
              (:wat::core::contains? s "banana"))
        "#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(false)));
    }

    #[test]
    fn hashset_accepts_composite_element() {
        // Arc 216.5a + 216.5b: Value: Hash + Eq is canonical; HashSet
        // storage is Arc<HashSet<Value>>. Composite elements are accepted
        // natively — the pre-antidote "primitives-only" restriction is gone.
        let result = eval_expr(r#"(:wat::core::HashSet :- [(:wat::core::Vector :- [:i64])] (:wat::core::Vector :- [:i64] 1 2))"#);
        assert!(
            result.is_ok(),
            "composite element should construct HashSet; got {:?}",
            result
        );
    }

    // LocalCache runtime tests retired in arc 013 slice 4b — the
    // wat-lru sibling crate owns that surface now. End-to-end
    // coverage lives in crates/wat-lru/tests/.

    #[test]
    fn thread_owned_cell_crossing_thread_boundary_errors() {
        // The generic scope guard. Same shape as the old LruCacheCell
        // test — post-#195 (macro regeneration) the lru shim uses
        // ThreadOwnedCell<WatLruCache>, so this test is now scoped to
        // the generic guard itself.
        use crate::rust_deps::ThreadOwnedCell;
        let cell: Arc<ThreadOwnedCell<i64>> = Arc::new(ThreadOwnedCell::new(1));
        cell.with_mut(":test::put", crate::rust_caller_span!(), |n| {
            *n = 42;
        })
        .unwrap();

        let cell_clone = Arc::clone(&cell);
        let handle = std::thread::Builder::new()
            .name("wat-thread::thread_owned_cell_crossing_thread_boundary_errors".to_string())
            .spawn(move || cell_clone.with_mut(":test::get", crate::rust_caller_span!(), |n| *n))
            .expect("Thread::Builder::spawn failed");
        let child_result = handle.join().unwrap();
        assert!(
            matches!(&child_result, Err(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. })),
            "expected cross-thread access to error, got {:?}",
            child_result
        );
        let parent_result = cell
            .with_mut(":test::get", crate::rust_caller_span!(), |n| *n)
            .unwrap();
        assert_eq!(parent_result, 42);
    }

    // ─── reduce-over-reverse / filter / zip ────────────────────────────

    #[test]
    fn reduce_over_reverse_is_right_associative() {
        // Arc 118.B6b — `foldr` retired: it was `reverse` + `foldl` wearing a name borrowed
        // from Haskell, where the verb is distinct only because it is LAZY (a property strict
        // wat cannot have). This is the replacement spelling: `(reduce f init (reverse xs))`.
        // f(x0, f(x1, f(x2, init))) = 1-(2-(3-0)) = 2 — the same right-associative answer
        // `foldr` gave. A left fold over the un-reversed input gives -6 (see the sibling test
        // below), so this assertion still discriminates a right fold from a left one.
        let src = r#"
            (:wat::core::reduce
              (:wat::core::fn [acc <- :i64 x <- :i64] -> :i64
                (:wat::i64::- x acc))
              0
              (:wat::core::reverse (:wat::core::Vector :- [:i64] 1 2 3)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(2) => {}
            v => panic!("expected 2, got {:?}", v),
        }
    }

    #[test]
    fn foldl_vs_reduce_over_reverse_differ_on_nonassoc_op() {
        // Arc 118.B6b — renamed from `foldl_vs_foldr_differ_on_nonassoc_op`: this body only
        // ever called `foldl` (the `foldr` half lived in the name and this comment, never in a
        // call) — it is the negative control that keeps a left fold discriminated from the
        // `(reduce f init (reverse xs))` right fold above, so it stays.
        // (foldl f init xs) where f = - : ((0 - 1) - 2) - 3 = -6
        let src_l = r#"
            (:wat::core::foldl
              (:wat::core::fn [acc <- :i64 x <- :i64] -> :i64
                (:wat::i64::- acc x))
              0
              (:wat::core::Vector :- [:i64] 1 2 3))
        "#;
        match eval_expr(src_l).unwrap() {
            Value::i64(-6) => {}
            v => panic!("expected -6, got {:?}", v),
        }
    }

    // Arc 118.2a — `filter` flipped LAZY (returns a `Stream`, built without validating
    // pred/coll shape until forced). `filter_keeps_true_predicates` materializes via
    // `(:wat::core::into [] …)` to get back a `Value::Vec` the same way as before.
    #[test]
    fn filter_keeps_true_predicates() {
        // Stone 237.8b — polymorphic `>` is now a defclause; use per-Type primitive in unit test.
        let src = r#"
            (:wat::core::into []
              (:wat::core::filter
                (:wat::core::fn [x <- :i64] -> :bool
                  (:wat::i64::> x 2))
                (:wat::core::Vector :- [:i64] 1 2 3 4 5)))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                assert_eq!(ns, vec![3, 4, 5]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn filter_refuses_non_bool_predicate() {
        // Arc 118.2a — `filter` is lazy; a bad-shaped call (coll-first order here, predating
        // the fn-first flip) no longer errors at CONSTRUCTION time (runtime defclause
        // dispatch is permissive on `Fn`-typed params — nothing to check until forced), so
        // force via `(:wat::core::into [] …)` to make the shape mismatch surface.
        let src = r#"
            (:wat::core::into []
              (:wat::core::filter
                (:wat::core::Vector :- [:i64] 1 2 3)
                (:wat::core::fn [x <- :i64] -> :i64 x)))
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn zip_pairs_shorter_length() {
        let src = r#"
            (:wat::seq::zip
              (:wat::core::Vector :- [:i64] 1 2 3)
              (:wat::core::Vector :- [:String] "a" "b"))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    Value::Tuple(t) => {
                        assert_eq!(t.len(), 2);
                        match (&t[0], &t[1]) {
                            (Value::i64(1), Value::String(s)) => assert_eq!(&**s, "a"),
                            other => panic!("expected (1,\"a\"); got {:?}", other),
                        }
                    }
                    v => panic!("expected Tuple, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn zip_empty_with_nonempty_is_empty() {
        let src = r#"
            (:wat::seq::zip
              (:wat::core::Vector :- [:i64])
              (:wat::core::Vector :- [:i64] 1 2 3))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => assert!(items.is_empty()),
            v => panic!("expected empty Vec, got {:?}", v),
        }
    }

    #[test]
    fn seq_zip_accepts_list_too() {
        // Arc 255 Stone HOME-9, acceptance row 2 — the SEQABLE PROOF: `zip` used to call
        // `require_vec` on BOTH inputs and REJECT a `List`; each side accepts one
        // independently now. Same result shape as `zip_pairs_shorter_length` above, mixed
        // List + Vector inputs.
        let src = r#"
            (:wat::seq::zip
              (:wat::core::List 1 2 3)
              (:wat::core::Vector :- [:String] "a" "b"))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    Value::Tuple(t) => {
                        assert_eq!(t.len(), 2);
                        match (&t[0], &t[1]) {
                            (Value::i64(1), Value::String(s)) => assert_eq!(&**s, "a"),
                            other => panic!("expected (1,\"a\"); got {:?}", other),
                        }
                    }
                    v => panic!("expected Tuple, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn hashset_int_and_string_keys_distinct() {
        // A HashSet carrying only the String "42" shouldn't report
        // membership for the i64 42 (type-tagged canonical key).
        let src = r#"
            (:wat::core::let
              [s (:wat::core::HashSet :- [:String] "42")]
              (:wat::core::contains? s 42))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false (no collision), got {:?}", v),
        }
    }

    #[test]
    fn list_window_bigger_than_length_is_empty() {
        match eval_expr("(:wat::seq::window (:wat::core::Vector :- [:i64] 1 2) 5)").unwrap() {
            Value::Vec(items) => assert!(items.is_empty()),
            v => panic!("expected empty Vec, got {:?}", v),
        }
    }

    #[test]
    fn seq_remove_at_on_vector_drops_the_index() {
        match eval_expr("(:wat::seq::remove-at (:wat::core::Vector :- [:i64] 10 20 30) 1)").unwrap() {
            Value::Vec(items) => {
                let ns: Vec<i64> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        other => panic!("expected i64, got {:?}", other),
                    })
                    .collect();
                assert_eq!(ns, vec![10, 30]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn seq_remove_at_accepts_list_too() {
        // Arc 255 Stone HOME-9, acceptance row 2 — the SEQABLE PROOF: `remove-at` used to call
        // `require_vec` and REJECT a `List`; it accepts one now, same result as the Vector case
        // immediately above.
        match eval_expr("(:wat::seq::remove-at (:wat::core::List 10 20 30) 1)").unwrap() {
            Value::Vec(items) => {
                let ns: Vec<i64> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        other => panic!("expected i64, got {:?}", other),
                    })
                    .collect();
                assert_eq!(ns, vec![10, 30]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    // ── Arc 035 — :wat::core::length polymorphism ────────────

    #[test]
    fn hashmap_length_returns_entry_count() {
        let src = r#"(:wat::core::let
            [m
              (:wat::core::HashMap :- [:String :i64] "a" 1 "b" 2 "c" 3)]
            (:wat::core::length m))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::i64(3)));
    }

    #[test]
    fn hashmap_length_empty_returns_zero() {
        let src = r#"(:wat::core::let
            [m
              (:wat::core::HashMap :- [:String :i64])]
            (:wat::core::length m))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::i64(0)));
    }

    #[test]
    fn hashset_length_returns_element_count() {
        let src = r#"(:wat::core::let
            [s
              (:wat::core::HashSet :- [:String] "a" "b" "c")]
            (:wat::core::length s))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::i64(3)));
    }

    #[test]
    fn hashset_length_empty_returns_zero() {
        let src = r#"(:wat::core::let
            [s
              (:wat::core::HashSet :- [:String])]
            (:wat::core::length s))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::i64(0)));
    }

    #[test]
    fn vec_length_still_works_after_polymorphism() {
        // Sanity — the existing Vec arm is preserved.
        let src = r#"(:wat::core::let
            [xs (:wat::core::Vector :- [:i64] 10 20 30)]
            (:wat::core::length xs))"#;
        assert!(matches!(eval_expr(src).unwrap(), Value::i64(3)));
    }

    // ─── drop — RETIRED 2026-08-19 (255.1c-retire-kernel-drop) ─────────
    //
    // `drop_refuses_non_handle` lived here and asserted that
    // `(:wat::kernel::drop 42)` raises TypeMismatch. It is deleted WITH the verb,
    // not to make a deletion compile: its entire subject was retired, because
    // `:wat::kernel::drop` proved UNREACHABLE from wat — its only accepted
    // arguments were `Sender`/`Receiver`, and nothing in the corpus constructs
    // either (`:wat::kernel::Channel :- [T]` is a typealias, not a verb).
    //
    // The distinction matters and is the rule: deleting a test whose subject
    // still LIVES, to make an unrelated change pass, is the forbidden move. A
    // test for a deleted verb has no subject left to guard. The rider correctly
    // hit STOP-3 and left this for a ruling rather than deleting it itself.

    // ─── spawn + join + join-result deleted in arc 114 ─────────────────
    //
    // Bare-spawn lib tests retired alongside the verbs they tested.
    // Mini-TCP is the contract: programs deliver values via output
    // pipes, not via join. spawn-thread + Thread/join-result coverage
    // lives in tests/wat_spawn_fn.rs (mini-TCP roundtrip on a
    // typed input/output channel pair). Process/join-result coverage
    // lives in tests/wat_arc103_spawn_program.rs.

    // ─── Vector portability (arc 061) ──────────────────────────────────

    #[test]
    fn vector_bytes_round_trip_recovers_vector() {
        // Encode an AST → vector → bytes → vector, then check the
        // recovered vector cosines == 1.0 with the original
        // (byte-perfect round-trip).
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let src = r#"
            (:wat::core::let
              [v
                (:wat::holon::encode (:wat::holon::to-holon "round-trip-test"))
               bs (:wat::holon::vector-bytes v)
               decode-outcome
                (:wat::holon::bytes-vector bs)
               v2
                (:wat::core::match decode-outcome
                  ((:wat::holon::VectorDecodeOutcome::Decoded v2) v2)
                  ((:wat::holon::VectorDecodeOutcome::DimensionMismatch _e _g)
                    (:wat::holon::encode (:wat::holon::to-holon "decode-failed-sentinel")))
                  ((:wat::holon::VectorDecodeOutcome::TruncatedHeader _g)
                    (:wat::holon::encode (:wat::holon::to-holon "decode-failed-sentinel")))
                  ((:wat::holon::VectorDecodeOutcome::LengthMismatch _e _g)
                    (:wat::holon::encode (:wat::holon::to-holon "decode-failed-sentinel")))
                  ((:wat::holon::VectorDecodeOutcome::InvalidCell _at)
                    (:wat::holon::encode (:wat::holon::to-holon "decode-failed-sentinel"))))]
              (:wat::holon::cosine v v2))
        "#;
        let c = expect_cosine_similarity(eval_with_ctx(src, 1024).unwrap());
        assert!(
            (c - 1.0).abs() < 1e-9,
            "expected cosine == 1.0 (byte-perfect round-trip), got {}",
            c
        );
    }

    #[test]
    fn vector_bytes_deterministic() {
        // Same Vector → same bytes (substrate-level determinism;
        // arc 061 Q7).
        // Arc 225 Stone 225.1 — to-holon lifts string primitives.
        let src = r#"
            (:wat::core::let
              [v1
                (:wat::holon::encode (:wat::holon::to-holon "deterministic"))
               v2
                (:wat::holon::encode (:wat::holon::to-holon "deterministic"))
               b1 (:wat::holon::vector-bytes v1)
               b2 (:wat::holon::vector-bytes v2)]
              (:wat::core::= b1 b2))
        "#;
        match eval_with_ctx(src, 1024).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn bytes_vector_rejects_short_input() {
        // Three bytes — not enough for the 4-byte dim header. Arc 278 the
        // dimension-heresy strike: expect the named `TruncatedHeader`
        // variant, not a reason-free `:None`.
        // Integer literals default to :i64; cast each through
        // :wat::core::u8 so the Vec stores u8 elements.
        let src = r#"
            (:wat::core::match
              (:wat::holon::bytes-vector
                (:wat::core::Vector :- [:u8]
                  (:wat::core::u8 0)
                  (:wat::core::u8 0)
                  (:wat::core::u8 0)))
              ((:wat::holon::VectorDecodeOutcome::Decoded _v) false)
              ((:wat::holon::VectorDecodeOutcome::DimensionMismatch _e _g) false)
              ((:wat::holon::VectorDecodeOutcome::TruncatedHeader _g) true)
              ((:wat::holon::VectorDecodeOutcome::LengthMismatch _e _g) false)
              ((:wat::holon::VectorDecodeOutcome::InvalidCell _at) false))
        "#;
        match eval_with_ctx(src, 1024).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected TruncatedHeader on short input, got {:?}", v),
        }
    }

    #[test]
    fn bytes_vector_rejects_truncated_data() {
        // 4-byte header claiming dim=10000 followed by zero data
        // bytes — data length doesn't match expected. Arc 278 the
        // dimension-heresy strike: expect the named `LengthMismatch`
        // variant, not a reason-free `:None`.
        // dim=10000 little-endian u32 = 16 39 00 00.
        let src = r#"
            (:wat::core::match
              (:wat::holon::bytes-vector
                (:wat::core::Vector :- [:u8]
                  (:wat::core::u8 16)
                  (:wat::core::u8 39)
                  (:wat::core::u8 0)
                  (:wat::core::u8 0)))
              ((:wat::holon::VectorDecodeOutcome::Decoded _v) false)
              ((:wat::holon::VectorDecodeOutcome::DimensionMismatch _e _g) false)
              ((:wat::holon::VectorDecodeOutcome::TruncatedHeader _g) false)
              ((:wat::holon::VectorDecodeOutcome::LengthMismatch _e _g) true)
              ((:wat::holon::VectorDecodeOutcome::InvalidCell _at) false))
        "#;
        match eval_with_ctx(src, 1024).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected LengthMismatch on truncated input, got {:?}", v),
        }
    }

    #[test]
    fn coincident_q_polymorphic_accepts_vectors() {
        // Vector × Vector — same source AST encoded twice should
        // coincide (arc 061: coincident? widened from HolonAST-only
        // to HolonAST | Vector).
        let src = r#"
            (:wat::core::let
              [v1
                (:wat::holon::encode (:wat::holon::to-holon "coincide-me"))
               v2
                (:wat::holon::encode (:wat::holon::to-holon "coincide-me"))]
              (:wat::holon::coincident? v1 v2))
        "#;
        match eval_with_ctx(src, 1024).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true (coincident vectors), got {:?}", v),
        }
    }

    #[test]
    fn coincident_q_polymorphic_accepts_mixed_vector_holon() {
        // Mixed (Vector, HolonAST) — pre-encoded vector vs. the
        // same AST should coincide (arc 061 polymorphism + arc 052's
        // mixed-input pair_values_to_vectors).
        let src = r#"
            (:wat::core::let
              [v
                (:wat::holon::encode (:wat::holon::to-holon "mixed-input"))]
              (:wat::holon::coincident? v (:wat::holon::to-holon "mixed-input")))
        "#;
        match eval_with_ctx(src, 1024).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true (mixed coincident), got {:?}", v),
        }
    }

    #[test]
    fn vector_bytes_arity_mismatch() {
        let err = eval_expr("(:wat::holon::vector-bytes)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn bytes_vector_arity_mismatch() {
        let err = eval_expr("(:wat::holon::bytes-vector)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── Bytes ↔ hex (arc 063) ──────────────────────────────────────────

    #[test]
    fn bytes_to_hex_emits_lowercase_no_separators() {
        // 0xde 0xad 0xbe 0xef → "deadbeef" (lowercase, no spaces).
        let src = r#"
            (:wat::core::Bytes::to-hex
              (:wat::core::Vector :- [:u8]
                (:wat::core::u8 222)   ;; 0xde
                (:wat::core::u8 173)   ;; 0xad
                (:wat::core::u8 190)   ;; 0xbe
                (:wat::core::u8 239))) ;; 0xef
        "#;
        match eval_expr(src).unwrap() {
            Value::String(s) => assert_eq!(&*s, "deadbeef"),
            v => panic!("expected String, got {:?}", v),
        }
    }

    #[test]
    fn bytes_from_hex_round_trip() {
        // hex → bytes → hex must reproduce the original.
        let src = r#"
            (:wat::core::let
              [bs1
                (:wat::core::Vector :- [:u8]
                  (:wat::core::u8 1)
                  (:wat::core::u8 2)
                  (:wat::core::u8 254)
                  (:wat::core::u8 255))
               hex (:wat::core::Bytes::to-hex bs1)
               maybe-bs2
                (:wat::core::Bytes::from-hex hex)
               bs2
                (:wat::core::match maybe-bs2
                  ((:wat::core::Some b) b)
                  (:wat::core::None
                    (:wat::core::Vector :- [:u8] (:wat::core::u8 0))))]
              (:wat::core::= bs1 bs2))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true (round-trip preserves bytes), got {:?}", v),
        }
    }

    #[test]
    fn bytes_from_hex_accepts_mixed_case() {
        // Mixed case "AbCd" → 0xab 0xcd; same as lowercase "abcd".
        let src = r#"
            (:wat::core::let
              [mixed
                (:wat::core::Bytes::from-hex "AbCd")
               lower
                (:wat::core::Bytes::from-hex "abcd")]
              (:wat::core::= mixed lower))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true (mixed case = lowercase), got {:?}", v),
        }
    }

    #[test]
    fn bytes_from_hex_empty_string_round_trips() {
        // "" → :Some(empty Bytes); to-hex of empty Bytes → "".
        let empty_decode = r#"
            (:wat::core::match (:wat::core::Bytes::from-hex "")
              ((:wat::core::Some b) (:wat::core::length b))
              (:wat::core::None -1))
        "#;
        match eval_expr(empty_decode).unwrap() {
            Value::i64(0) => {}
            v => panic!("expected 0 (empty Bytes), got {:?}", v),
        }
        let empty_encode = r#"
            (:wat::core::Bytes::to-hex (:wat::core::Vector :- [:u8]))
        "#;
        match eval_expr(empty_encode).unwrap() {
            Value::String(s) => assert_eq!(&*s, ""),
            v => panic!("expected empty String, got {:?}", v),
        }
    }

    #[test]
    fn bytes_from_hex_rejects_odd_length() {
        let src = r#"
            (:wat::core::match (:wat::core::Bytes::from-hex "abc")
              ((:wat::core::Some _) false)
              (:wat::core::None true))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected None on odd length, got {:?}", v),
        }
    }

    #[test]
    fn bytes_from_hex_rejects_non_hex_chars() {
        // "zz" — z is not a hex character.
        let src = r#"
            (:wat::core::match (:wat::core::Bytes::from-hex "zz")
              ((:wat::core::Some _) false)
              (:wat::core::None true))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected None on non-hex char, got {:?}", v),
        }
    }

    #[test]
    fn bytes_from_hex_rejects_0x_prefix() {
        // Per DESIGN Q6: no `0x` tolerance in v1.
        let src = r#"
            (:wat::core::match (:wat::core::Bytes::from-hex "0xdead")
              ((:wat::core::Some _) false)
              (:wat::core::None true))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected None on 0x prefix, got {:?}", v),
        }
    }

    #[test]
    fn bytes_to_hex_arity_mismatch() {
        let err = eval_expr("(:wat::core::Bytes::to-hex)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn bytes_from_hex_arity_mismatch() {
        let err = eval_expr("(:wat::core::Bytes::from-hex)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── show — polymorphic rendering (arc 064) ─────────────────────────

    fn show_str(src: &str) -> String {
        match eval_expr(src).unwrap() {
            Value::String(s) => (*s).clone(),
            other => panic!("expected String, got {:?}", other),
        }
    }

    #[test]
    fn show_renders_primitive_leaves() {
        assert_eq!(show_str("(:wat::core::show true)"), "true");
        assert_eq!(show_str("(:wat::core::show false)"), "false");
        assert_eq!(show_str("(:wat::core::show 42)"), "42");
        assert_eq!(show_str("(:wat::core::show -7)"), "-7");
        assert_eq!(show_str("(:wat::core::show 3.14)"), "3.14");
        assert_eq!(show_str(r#"(:wat::core::show "hello")"#), "\"hello\"");
        assert_eq!(show_str(r#"(:wat::core::show "")"#), "\"\"");
        // Quoted keyword evaluates to a Value::wat__WatAST(Keyword …);
        // the WatAST arm renders as a `<WatAST>` summary. Diagnostic
        // for assert-eq is via the keyword's atom-of form (see
        // show_renders_compound_summaries) or via primitive equality.
        assert_eq!(
            show_str("(:wat::core::show (:wat::core::quote :outcome))"),
            "<WatAST>"
        );
    }

    #[test]
    fn show_renders_option_and_result() {
        assert_eq!(
            show_str("(:wat::core::show (:wat::core::Some 1))"),
            "(Some 1)"
        );
        assert_eq!(show_str("(:wat::core::show :wat::core::None)"), ":None");
        assert_eq!(
            show_str(r#"(:wat::core::show (:wat::core::Ok "hi"))"#),
            "(Ok \"hi\")"
        );
        assert_eq!(
            show_str("(:wat::core::show (:wat::core::Err 42))"),
            "(Err 42)"
        );
    }

    #[test]
    fn show_renders_vec_with_brackets() {
        assert_eq!(
            show_str("(:wat::core::show (:wat::core::Vector :- [:i64] 1 2 3))"),
            "[1, 2, 3]"
        );
        assert_eq!(
            show_str("(:wat::core::show (:wat::core::Vector :- [:i64]))"),
            "[]"
        );
    }

    #[test]
    fn show_renders_compound_summaries() {
        // Vector → angle-bracketed dim summary.
        let s = match eval_with_ctx(
            r#"(:wat::core::show
                  (:wat::holon::encode (:wat::holon::to-holon "x")))"#,
            1024,
        )
        .unwrap()
        {
            Value::String(s) => (*s).clone(),
            other => panic!("expected String, got {:?}", other),
        };
        assert_eq!(
            s, "<Vector dim=1024>",
            "show must render a compact dim summary, not raw values"
        );
    }

    #[test]
    fn assert_eq_failure_renders_actual_and_expected() {
        // Arc 064 — a failed assert-eq should populate the
        // AssertionPayload's actual/expected slots with the rendered
        // values via show.
        //
        // Post-arc-170: run-sandboxed + run-ast are retired. The
        // property is verified directly: define a WAT fn that calls
        // assert-eq(1, 2), invoke it via apply_function, catch the
        // AssertionPayload panic, inspect actual/expected.
        use crate::assertion::AssertionPayload;
        let src = r#"
            (:wat::core::defn :my::test::assert-mismatched [] -> :() (:wat::test::assert-eq 1 2))
        "#;
        let (stdlib_sym, stdlib_macros, _) = stdlib_loaded();
        let mut macros = stdlib_macros.clone();
        let forms = crate::parse_all!(src).expect("parse");
        // LOAD-BEARING ORDER: expand_all must run before user-defn registration — see src/macros/eval.rs module doc + freeze.rs expand_runs_before_register_defines_phase_order
        let expanded =
            crate::macros::expand_all(forms, &mut macros, &Environment::new(), stdlib_sym)
                .expect("expand");
        let mut sym = stdlib_sym.clone();
        let _ = register_defines(expanded, &mut sym).expect("register");
        let func = sym
            .get(":my::test::assert-mismatched")
            .expect("defined")
            .clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply_function(func, Vec::new(), &sym, crate::rust_caller_span!())
        }));

        let payload = match result {
            Ok(_) => panic!("expected AssertionPayload panic; got Ok"),
            Err(p) => p,
        };
        let boxed = match payload.downcast::<AssertionPayload>() {
            Ok(b) => *b,
            Err(_) => panic!("expected AssertionPayload in panic payload"),
        };
        // actual and expected must be populated — assert-eq renders
        // each argument via show and stores them.
        let actual = boxed.actual.expect("actual should be Some");
        let expected = boxed.expected.expect("expected should be Some");
        assert_eq!(actual, "1");
        assert_eq!(expected, "2");
    }

    #[test]
    fn show_arity_mismatch() {
        let err = eval_expr("(:wat::core::show)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── leaf / from-wat (arc 065; renamed from-watast → from-wat at arc 225) ──────────────────────────────────

    #[test]
    fn leaf_lifts_primitive_to_holon_leaf() {
        // Each primitive Value variant should become its matching
        // HolonAST leaf. from-holon extracts the value back to verify.
        let cases = [
            (r#"(:wat::holon::from-holon (:wat::holon::leaf 42))"#, "42"),
            (
                r#"(:wat::holon::from-holon (:wat::holon::leaf 3.14))"#,
                "3.14",
            ),
            (
                r#"(:wat::holon::from-holon (:wat::holon::leaf true))"#,
                "true",
            ),
            (
                r#"(:wat::holon::from-holon (:wat::holon::leaf "hi"))"#,
                "\"hi\"",
            ),
        ];
        for (src, expected) in cases.iter() {
            // Wrap each in a show call to get a stable comparison
            // string regardless of which Value variant atom-value
            // returned.
            let wrapped = format!("(:wat::core::show {})", src);
            match eval_expr(&wrapped).unwrap() {
                Value::String(s) => assert_eq!(&*s, *expected, "for source {}", src),
                v => panic!("expected String, got {:?} for source {}", v, src),
            }
        }
    }

    #[test]
    fn leaf_rejects_non_primitive() {
        // HolonAST input is the wrong verb; the rejection should
        // hint at Atom (which IS the right verb for HolonAST → wrap).
        // Arc 225 Stone 225.1: use leaf(1) to produce a real HolonAST
        // to pass as input to leaf (the narrow Atom is no longer valid
        // as a primitive-string constructor; leaf(1) is the simplest
        // way to obtain a HolonAST to hand to leaf).
        let err = eval_expr("(:wat::holon::leaf (:wat::holon::leaf 1))").unwrap_err();
        match err {
            EvalBreak::Diagnostic(e) => match e.kind() {
                RuntimeErrorKind::TypeMismatch { op, expected, .. } => {
                    assert_eq!(op, ":wat::holon::leaf");
                    assert_eq!(
                        *expected,
                        "primitive (i64/f64/bool/String/keyword/nil); use :wat::holon::Atom to wrap a HolonAST, :wat::holon::from-wat to lower a quoted form, :wat::holon::to-holon for other types"
                    );
                }
                other => panic!("expected TypeMismatch, got {:?}", other),
            },
            other => panic!("expected TypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn from_watast_lowers_quoted_list_to_bundle() {
        // Quoted list form lowers structurally — the result is a
        // Bundle of Keyword / I64 leaves (mirrors arc 057's path-2
        // structural lowering; head keyword is now HolonAST::Keyword
        // per arc 221 Stone 221.4b).
        //
        // Arc 221 Stone 221.4b cascade — `(:wat::core::quote (:wat::i64::+ 40 2))`
        // produces `WatAST::List([WatAST::Keyword(":wat::i64::+"), ...])`.
        // `watast_to_holon` at Stone 221.4b maps `WatAST::Keyword(k) →
        // HolonAST::keyword(k.as_str())` → `HolonAST::Keyword("wat::i64::+")`
        // (leading colon stripped). Assertion flipped from as_symbol() to as_keyword().
        // Arc 225 Stone 225.1: from-watast renamed to from-wat.
        let src = r#"
            (:wat::holon::from-wat
              (:wat::core::quote (:wat::i64::+ 40 2)))
        "#;
        let v = eval_expr(src).unwrap();
        let h = match v {
            Value::holon__HolonAST(h) => h,
            other => panic!("expected Holon, got {:?}", other),
        };
        match &*h {
            HolonAST::Bundle(items) => {
                assert_eq!(items.len(), 3);
                // Stone 221.4b: WatAST::Keyword → HolonAST::Keyword; content without leading colon.
                assert_eq!(items[0].as_keyword(), Some("wat::i64::+"));
                assert_eq!(
                    items[0].as_symbol(),
                    None,
                    "must NOT be Symbol after arc 221"
                );
                assert_eq!(items[1].as_i64(), Some(40));
                assert_eq!(items[2].as_i64(), Some(2));
            }
            other => panic!("expected Bundle, got {:?}", other),
        }
    }

    #[test]
    fn from_watast_lowers_atomic_quote_to_leaf() {
        // Atomic literal inside quote — atomic shape stays as a
        // primitive leaf, NOT wrapped in a Bundle.
        //
        // Arc 221 Stone 221.4b cascade — `(:wat::core::quote :outcome)` produces
        // `WatAST::Keyword(":outcome")`; `watast_to_holon` at Stone 221.4b now maps
        // `WatAST::Keyword(k) → HolonAST::keyword(k.as_str())` which strips the
        // leading colon and produces `HolonAST::Keyword("outcome")`. Pre-Stone-221.4b
        // used `HolonAST::symbol(k.as_str())` → `HolonAST::Symbol(":outcome")`.
        // Assertion flipped from as_symbol() to as_keyword() per arc 221 doctrine.
        // Arc 225 Stone 225.1: from-watast renamed to from-wat.
        let src = r#"
            (:wat::holon::from-wat (:wat::core::quote :outcome))
        "#;
        let v = eval_expr(src).unwrap();
        let h = match v {
            Value::holon__HolonAST(h) => h,
            other => panic!("expected Holon, got {:?}", other),
        };
        // Stone 221.4b: WatAST::Keyword → HolonAST::Keyword (no leading colon stored).
        assert_eq!(h.as_keyword(), Some("outcome"));
        // Confirm it is NOT a Symbol (regression guard against reverting to old convention).
        assert_eq!(
            h.as_symbol(),
            None,
            "must NOT be Symbol after arc 221 Stone 221.4b"
        );
    }

    #[test]
    fn from_watast_rejects_non_watast() {
        // Primitive input is the wrong verb; should hint at leaf.
        // Arc 225 Stone 225.1: from-watast renamed to from-wat.
        let err = eval_expr("(:wat::holon::from-wat 42)").unwrap_err();
        match err {
            EvalBreak::Diagnostic(e) => match e.kind() {
                RuntimeErrorKind::TypeMismatch { op, expected, .. } => {
                    assert_eq!(op, ":wat::holon::from-wat");
                    assert_eq!(
                        *expected,
                        ":wat::WatAST (typically from :wat::core::quote); use :wat::holon::Atom for HolonAST inputs, :wat::holon::to-holon for other types, :wat::holon::leaf for primitives"
                    );
                }
                other => panic!("expected TypeMismatch, got {:?}", other),
            },
            other => panic!("expected TypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn watast_round_trip_preserves_bundle_shape() {
        // The (to-wat → from-wat) round-trip preserves a
        // structurally-lowered Bundle of primitives — `to-wat`
        // emits `(items…)` for a Bundle, and `from-wat` reads
        // that List back as a Bundle of leaves. Trees that started
        // as algebra ops (Bind / Permute / Thermometer / Blend)
        // lift as symbol-headed Lists at the source level; they
        // come back as Bundles structurally rather than the original
        // composite — that's the substrate distinguishing
        // "form on the algebra grid" from "form as source text",
        // and the round-trip is faithful to whichever side h
        // started on.
        let src = r#"
            (:wat::core::let
              [h1
                (:wat::core::match
                  (:wat::holon::Bundle
                    (:wat::core::Vector :- [:wat::holon::HolonAST]
                      (:wat::holon::leaf "role")
                      (:wat::holon::leaf "filler")))
                  ((:wat::core::Ok h) h)
                  ((:wat::core::Err _) (:wat::holon::leaf "unreachable")))
               ast (:wat::holon::to-wat h1)
               h2 (:wat::holon::from-wat ast)]
              (:wat::holon::cosine h1 h2))
        "#;
        let c = expect_cosine_similarity(eval_with_ctx(src, 1024).unwrap());
        assert!(
            (c - 1.0).abs() < 1e-9,
            "expected cosine ≈ 1.0 (Bundle round-trip), got {}",
            c
        );
    }

    #[test]
    fn leaf_arity_mismatch() {
        let err = eval_expr("(:wat::holon::leaf)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn from_watast_arity_mismatch() {
        // Arc 225 Stone 225.1: from-watast renamed to from-wat.
        let err = eval_expr("(:wat::holon::from-wat)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    // ─── eval-ast! returns bare Value (arc 102 — reverts arc 066's wrap) ──

    #[test]
    fn eval_ast_returns_bare_i64_result() {
        // Arc 102: scheme is `Result<:T, :EvalError>` polymorphic.
        // Caller binds T = :i64 and the (Ok n) arm gets the i64
        // directly — no atom-value extraction needed (arc 066's
        // value_to_holon wrap was reverted).
        let src = r#"
            (:wat::core::match
              (:wat::eval-ast!
                (:wat::core::quote (:wat::i64::+ 2 2)))
              ((:wat::core::Ok n) n)
              ((:wat::core::Err _) -1))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(4) => {}
            v => panic!("expected 4, got {:?}", v),
        }
    }

    #[test]
    fn eval_ast_returns_bare_bool_result() {
        // Stone 237.8b — polymorphic `>` is now a defclause; use per-Type primitive in unit test.
        let src = r#"
            (:wat::core::match
              (:wat::eval-ast!
                (:wat::core::quote (:wat::i64::> 5 3)))
              ((:wat::core::Ok b) b)
              ((:wat::core::Err _) false))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn eval_ast_returns_bare_string_result() {
        let src = r#"
            (:wat::core::match
              (:wat::eval-ast!
                (:wat::core::quote
                  (:wat::string::concat "hello, " "world")))
              ((:wat::core::Ok s) s)
              ((:wat::core::Err _) "fail"))
        "#;
        match eval_expr(src).unwrap() {
            Value::String(s) => assert_eq!(&*s, "hello, world"),
            v => panic!("expected String, got {:?}", v),
        }
    }

    #[test]
    fn eval_ast_passes_through_holon_result() {
        // When the form's result is itself a HolonAST, the caller
        // binds T = :wat::holon::HolonAST and (Ok h) gets the
        // HolonAST directly. from-holon still works on it because
        // the runtime IS returning a HolonAST in this case.
        // Arc 225 Stone 225.1: atom-value renamed to from-holon.
        let src = r#"
            (:wat::core::match
              (:wat::eval-ast!
                (:wat::core::quote
                  (:wat::holon::leaf 42)))
              ((:wat::core::Ok h) (:wat::holon::from-holon h))
              ((:wat::core::Err _) -1))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(42) => {}
            v => panic!("expected 42, got {:?}", v),
        }
    }

    #[test]
    fn eval_ast_passes_through_vec_result() {
        // Arc 102: pre-arc-102, a Vec result errored because
        // value_to_holon couldn't wrap it. With arc 102's revert,
        // Vec results pass through cleanly — caller binds
        // T = (:wat::core::Vector :- [i64]) and (Ok xs) gets the Vec directly.
        let src = r#"
            (:wat::core::match
              (:wat::eval-ast!
                (:wat::core::quote (:wat::core::Vector :- [:i64] 1 2 3)))
              ((:wat::core::Ok xs) (:wat::core::length xs))
              ((:wat::core::Err _) -1))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(3) => {}
            v => panic!("expected 3 (vec length), got {:?}", v),
        }
    }

    // ─── eval-step! (arc 068) ──────────────────────────────────────────

    /// Run an `(:wat::eval-step! <form>)` chain through `eval_expr`
    /// (no encoding ctx) and assert the result matches the expected
    /// shape via the rendered `show` of the inner StepResult.
    fn step_to_show(quoted_src: &str) -> String {
        let src = format!(
            "(:wat::core::match {} \
                ((:wat::core::Ok r) (:wat::core::show r)) \
                ((:wat::core::Err e) (:wat::core::show e)))",
            quoted_src
        );
        match eval_expr(&src).unwrap() {
            Value::String(s) => (*s).clone(),
            other => panic!("expected String, got {:?}", other),
        }
    }

    #[test]
    fn step_lit_i64_is_terminal() {
        let s = step_to_show("(:wat::eval-step! (:wat::core::quote 5))");
        // Arc 070 — primitive literals are value-shapes; `eval-step!`
        // recognizes them via try_recognize_holon_value and returns
        // AlreadyTerminal (no work happened). Pre-arc-070 returned
        // StepTerminal; arc 070 narrows that variant to "this step
        // reduced a redex" only.
        assert_eq!(s, "(:wat::eval::StepResult::AlreadyTerminal <HolonAST>)");
    }

    #[test]
    fn step_lit_bool_is_terminal() {
        let s = step_to_show("(:wat::eval-step! (:wat::core::quote true))");
        assert_eq!(s, "(:wat::eval::StepResult::AlreadyTerminal <HolonAST>)");
    }

    #[test]
    fn step_lit_string_is_terminal() {
        let s = step_to_show(r#"(:wat::eval-step! (:wat::core::quote "hi"))"#);
        assert_eq!(s, "(:wat::eval::StepResult::AlreadyTerminal <HolonAST>)");
    }

    #[test]
    fn step_lit_keyword_is_terminal() {
        let s = step_to_show("(:wat::eval-step! (:wat::core::quote :outcome))");
        assert_eq!(s, "(:wat::eval::StepResult::AlreadyTerminal <HolonAST>)");
    }

    // --- :wat::eval::walk — arc 070 phase 2 -------------------------------
    //
    // Fold over the eval-step! chain. The walker visits every
    // coordinate exactly once with `(acc, form, step-result)` and
    // dispatches based on the (WalkStep :- [A]) the visitor returns.

    /// Wat program prelude defining a `count-visits` visitor that
    /// always returns Continue and increments the i64 accumulator.
    /// Used to drive walks that should run to natural terminal.
    fn walk_count_prelude() -> &'static str {
        r#"
        (:wat::core::defn :my::test::count-visit [acc <- :wat::core::i64 form <- :wat::WatAST step <- :wat::eval::StepResult] -> (:wat::eval::WalkStep :- [:wat::core::i64]) (:wat::eval::WalkStep::Continue (:wat::i64::+ acc 1)))
        "#
    }

    #[test]
    fn walk_w1_chain_to_terminal() {
        // Fully-reducible chain `(+ (+ 1 2) 3)`. Walker visits every
        // coordinate; final terminal is HolonAST::I64(6); the
        // accumulator (visit count) is positive (chain has length
        // ≥ 1 — at least one StepNext + one StepTerminal).
        let src = format!(
            r#"
            {}
            (:wat::core::match
              (:wat::eval::walk
                (:wat::core::quote (:wat::i64::+ (:wat::i64::+ 1 2) 3))
                0
                :my::test::count-visit)
              ((:wat::core::Ok pair)
                (:wat::core::let
                  [terminal (:wat::core::first pair)
                   count (:wat::core::second pair)
                   value (:wat::holon::from-holon terminal)
                   ;; encode (value, count) as one i64: value * 1000 + count.
                   ;; sufficient for a chain of length < 1000.
                   packed
                    (:wat::i64::+
                      (:wat::i64::* value 1000)
                      count)]
                  packed))
              ((:wat::core::Err _) -1))
            "#,
            walk_count_prelude()
        );
        match run(&src).unwrap() {
            Value::i64(packed) => {
                let value = packed / 1000;
                let count = packed % 1000;
                assert_eq!(value, 6, "expected terminal value 6, got {}", value);
                assert!(count >= 1, "expected at least 1 visit, got {}", count);
            }
            other => panic!("expected i64, got {:?}", other),
        }
    }

    #[test]
    fn walk_w2_already_terminal_input() {
        // Input that's already a value-shape (`Bind(Atom, Therm)`'s
        // canonical form). Walker visits exactly once with
        // step-result = AlreadyTerminal; final return is the form
        // itself; chain length is 0 — the visit count after one
        // visit is 1.
        let src = format!(
            r#"
            {}
            (:wat::core::match
              (:wat::eval::walk
                (:wat::core::quote
                  (:wat::holon::Bind
                    (:wat::holon::to-holon "k")
                    (:wat::holon::to-holon "v")))
                0
                :my::test::count-visit)
              ((:wat::core::Ok pair)
                (:wat::core::second pair))
              ((:wat::core::Err _) -1))
            "#,
            walk_count_prelude()
        );
        match run(&src).unwrap() {
            Value::i64(count) => {
                assert_eq!(count, 1, "expected exactly 1 visit, got {}", count);
            }
            other => panic!("expected i64, got {:?}", other),
        }
    }

    #[test]
    fn walk_w3_skip_short_circuits() {
        // Visitor returns Skip on the FIRST coordinate with a
        // sentinel terminal HolonAST::I64(999). Walker stops; final
        // return is (sentinel, acc'). Even on a chain that would
        // naturally terminate at I64(6), Skip wins.
        let src = r#"
        (:wat::core::defn :my::test::skip-on-first [acc <- :wat::core::i64 form <- :wat::WatAST step <- :wat::eval::StepResult] -> (:wat::eval::WalkStep :- [:wat::core::i64])
          (:wat::eval::WalkStep::Skip
                      (:wat::holon::leaf 999)
                      (:wat::i64::+ acc 1)))
        (:wat::core::match
          (:wat::eval::walk
            (:wat::core::quote (:wat::i64::+ (:wat::i64::+ 1 2) 3))
            0
            :my::test::skip-on-first)
          ((:wat::core::Ok pair)
            (:wat::core::let
              [terminal (:wat::core::first pair)
               value (:wat::holon::from-holon terminal)]
              value))
          ((:wat::core::Err _) -1))
        "#;
        match run(src).unwrap() {
            Value::i64(value) => {
                assert_eq!(value, 999, "expected sentinel 999 from Skip, got {}", value);
            }
            other => panic!("expected i64, got {:?}", other),
        }
    }

    #[test]
    fn walk_w4_propagates_eval_step_err() {
        // Quote-form (`:wat::core::quote`) inside the chain has no
        // step rule — eval-step! returns Err(NoStepRule). walk
        // propagates as the outer Result::Err; the visitor never
        // sees the error.
        let src = format!(
            r#"
            {}
            (:wat::core::match
              (:wat::eval::walk
                (:wat::core::quote
                  (:wat::holon::from-wat
                    (:wat::core::quote 42)))
                0
                :my::test::count-visit)
              ((:wat::core::Ok _) -2)
              ((:wat::core::Err e)
                ;; struct-field 0 is the kind tag.
                (:wat::core::if
                  (:wat::core::= "no-step-rule"
                                 (:wat::core::struct-field e 0))
                  1
                  -3)))
            "#,
            walk_count_prelude()
        );
        match run(&src).unwrap() {
            Value::i64(1) => {}
            other => panic!("expected Err(no-step-rule), got {:?}", other),
        }
    }

    #[test]
    fn step_already_terminal_on_lifted_bundle() {
        // Arc 070 — `holon_to_watast(Bundle([...]))` produces a bare-
        // list WatAST (no keyword head). Pre-arc-070 this would
        // return Err(NoStepRule); arc 070 recognizes the structural
        // value-shape and returns AlreadyTerminal with the rebuilt
        // Bundle. The walker can now distinguish "input is already
        // a value" from "no rule applies."
        //
        // Arc 225 Stone 225.1: narrow Atom only accepts HolonAST input;
        // primitive-string Atom forms no longer recognized by
        // try_recognize_holon_value. Use leaf forms (the primitive-to-
        // leaf constructor) which ARE recognized as value-shapes.
        let s = step_to_show(
            r#"(:wat::eval-step!
                 (:wat::core::quote
                   ((:wat::holon::leaf "k")
                    (:wat::holon::leaf "v"))))"#,
        );
        assert_eq!(
            s, "(:wat::eval::StepResult::AlreadyTerminal <HolonAST>)",
            "expected AlreadyTerminal for bare-list Bundle lift"
        );
    }

    #[test]
    fn step_already_terminal_on_holon_constructor_call() {
        // `(:wat::holon::leaf "k")` is a value-shape per arc 057's
        // `holon_to_watast` (the source form an already-built holon
        // round-trips to). Returns AlreadyTerminal — the substrate
        // KNOWS this is a value, not a function call to compute one.
        //
        // Arc 225 Stone 225.1: the original test used
        // `(:wat::holon::Atom "k")` (old polymorphic Atom accepting a
        // primitive string). The narrow Atom now only accepts HolonAST
        // input, so primitive-string Atom forms are no longer recognized
        // as value-shapes by try_recognize_holon_value. Use
        // `(:wat::holon::leaf "k")` which is a genuine primitive-to-leaf
        // constructor and IS recognized as a value-shape.
        let s = step_to_show(
            r#"(:wat::eval-step!
                 (:wat::core::quote (:wat::holon::leaf "k")))"#,
        );
        assert_eq!(
            s, "(:wat::eval::StepResult::AlreadyTerminal <HolonAST>)",
            "expected AlreadyTerminal for holon-ctor value-shape"
        );
    }

    #[test]
    fn step_terminal_on_arithmetic_redex() {
        // Sanity: actual reductions (arithmetic firings) still
        // return StepTerminal, not AlreadyTerminal — the variant
        // distinction matters. `(+ 2 2)` fires a real reduction.
        let s = step_to_show("(:wat::eval-step! (:wat::core::quote (:wat::i64::+ 2 2)))");
        assert_eq!(
            s, "(:wat::eval::StepResult::StepTerminal <HolonAST>)",
            "arithmetic fire must return StepTerminal, not AlreadyTerminal"
        );
    }

    #[test]
    fn step_unknown_form_yields_no_step_rule_err() {
        // `:wat::holon::from-wat` consumes a quoted form (a
        // `:wat::WatAST` value) and `:wat::core::quote` is a special
        // form not in the step-rule table — quote produces a
        // wat__WatAST Value that has no HolonAST representation. Step
        // mode routes both to NoStepRule; consumers that hit them
        // fall back to `eval-ast!`. Picking from-wat as the test
        // case documents that boundary.
        // Arc 225 Stone 225.1: from-watast renamed to from-wat.
        let s = step_to_show("(:wat::eval-step! (:wat::core::quote (:wat::holon::from-wat x)))");
        assert_eq!(
            s,
            r#":wat::core::EvalError{#0: "no-step-rule", #1: "eval-step! has no rule for op: :wat::holon::from-wat"}"#
        );
    }

    #[test]
    fn step_arity_mismatch() {
        let err = eval_expr("(:wat::eval-step!)").unwrap_err();
        // arity is checked BEFORE the wrap_as_eval_result block, so
        // it surfaces as an EvalBreak::Diagnostic.
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn step_non_watast_arg_yields_eval_error() {
        // Arg evaluates to an i64, not a WatAST — caught inside the
        // wrap_as_eval_result block, surfaces as EvalError(type-mismatch).
        let s = step_to_show("(:wat::eval-step! 42)");
        assert_eq!(
            s,
            r#":wat::core::EvalError{#0: "type-mismatch", #1: ":wat::eval-step!: expected wat::WatAST, got wat::core::i64 `42`"}"#
        );
    }

    /// Wat program prelude that defines a recursive
    /// `:my::test::step-to-terminal` driver — calls `eval-step!`
    /// repeatedly until StepTerminal, returning the inner HolonAST.
    /// Phase 2 multi-step tests call this on a quoted form.
    fn step_to_terminal_prelude() -> &'static str {
        // Tagged-enum variant patterns use the fully-qualified keyword
        // path per arc 048 (see try_match_pattern's `WatAST::Keyword`
        // arm). Three arms now (arc 070): StepNext recurses, both
        // terminal flavors return the inner HolonAST. The Err arm
        // packs the EvalError's message string into the result holon
        // so failing tests can show it instead of a silent sentinel.
        r#"
        (:wat::core::defn :my::test::step-to-terminal [form <- :wat::WatAST] -> :wat::holon::HolonAST
          (:wat::core::match (:wat::eval-step! form)
                      ((:wat::core::Ok r)
                        (:wat::core::match r
                          ((:wat::eval::StepResult::StepNext next)
                            (:my::test::step-to-terminal next))
                          ((:wat::eval::StepResult::StepTerminal h) h)
                          ((:wat::eval::StepResult::AlreadyTerminal h) h)))
                      ((:wat::core::Err e) (:wat::holon::leaf (:wat::core::struct-field e 1)))))
        "#
    }

    /// Run the `step-to-terminal` driver on a quoted form; expect the
    /// result to be a `Value::holon__HolonAST` and return its inner.
    fn step_drive_to_terminal(form_src: &str) -> std::sync::Arc<HolonAST> {
        let src = format!(
            "{}\n(:my::test::step-to-terminal (:wat::core::quote {}))",
            step_to_terminal_prelude(),
            form_src
        );
        match run(&src).unwrap() {
            Value::holon__HolonAST(h) => h,
            other => panic!("expected HolonAST, got {:?}", other),
        }
    }

    /// `run` variant that attaches an EncodingCtx + dim router to the
    /// SymbolTable — matches what `FrozenWorld::freeze` does for a
    /// real program. Required for step rules over forms that touch
    /// the encoding pipeline (`:wat::holon::Bundle`, cosine, etc.).
    fn run_with_ctx(src: &str, dims: usize) -> Result<Value, EvalBreak> {
        let (stdlib_sym, stdlib_macros, _) = stdlib_loaded();
        let mut macros = stdlib_macros.clone();
        let forms = crate::parse_all!(src).expect("parse ok");
        // LOAD-BEARING ORDER: expand_all must run before user-defn registration — see src/macros/eval.rs module doc + freeze.rs expand_runs_before_register_defines_phase_order
        let expanded =
            crate::macros::expand_all(forms, &mut macros, &Environment::new(), stdlib_sym)
                .expect("macro expansion");
        let mut sym = stdlib_sym.clone();
        sym.set_encoding_ctx(Arc::new(EncodingCtx::from_config(&Config {
            capacity_mode: crate::config::CapacityMode::Error,
            global_seed: 42,
            dim_count: dims,
            presence_sigma_ast: None,
            coincident_sigma_ast: None,
            redef_allowed: false,
            eval_redef_allowed: false,
        })));
        sym.set_presence_sigma_fn(Arc::new(crate::holon::sigma::DefaultPresenceSigma));
        sym.set_coincident_sigma_fn(Arc::new(crate::holon::sigma::DefaultCoincidentSigma));
        let rest = register_defines(expanded, &mut sym)?;
        let env = Environment::new();
        let mut last = Value::Unit;
        for form in &rest {
            // Stone 241.11 — skip declaration forms (already pre-registered);
            // mirrors the same guard in `run()`.
            // Stone 241.14 — def-restricted removed from this guard (HARD CUT).
            if let WatAST::List(items, _) = form {
                if let Some(WatAST::Keyword(head, _)) = items.first() {
                    if matches!(head.as_str(), ":wat::core::def") {
                        continue;
                    }
                }
            }
            last = eval_inner(form, &env, &sym)?.value_owned();
        }
        Ok(last)
    }

    #[test]
    fn step_arith_single_redex() {
        // `(+ 2 2)` — args canonical, fire on first step.
        let s = step_to_show("(:wat::eval-step! (:wat::core::quote (:wat::i64::+ 2 2)))");
        assert_eq!(s, "(:wat::eval::StepResult::StepTerminal <HolonAST>)");
        // Drive to terminal: same form, full chain → HolonAST::I64(4).
        let h = step_drive_to_terminal("(:wat::i64::+ 2 2)");
        assert_eq!(h.as_i64(), Some(4));
    }

    #[test]
    fn step_arith_left_descent() {
        // `(+ (+ 1 2) 3)` — first step descends inner; second step fires outer.
        let h = step_drive_to_terminal("(:wat::i64::+ (:wat::i64::+ 1 2) 3)");
        assert_eq!(h.as_i64(), Some(6));
    }

    #[test]
    fn step_arith_right_descent() {
        // `(+ 5 (+ 1 2))` — left arg already canonical; descend right.
        let h = step_drive_to_terminal("(:wat::i64::+ 5 (:wat::i64::+ 1 2))");
        assert_eq!(h.as_i64(), Some(8));
    }

    #[test]
    fn step_let_substitute() {
        // `(let ((x 5)) (* x x))` — RHS canonical, peel,
        // substitute, then arithmetic fire.
        let h = step_drive_to_terminal("(:wat::core::let [x 5] (:wat::i64::* x x))");
        assert_eq!(h.as_i64(), Some(25));
    }

    #[test]
    fn step_let_peel_first() {
        // Multi-binding: `(let ((a (+ 1 1)) (b a)) b)`.
        // a's RHS is non-canonical → descend; then peel a; then peel
        // b; body alone reduces to terminal.
        let h = step_drive_to_terminal("(:wat::core::let [a (:wat::i64::+ 1 1) b a] b)");
        assert_eq!(h.as_i64(), Some(2));
    }

    #[test]
    fn step_if_branch_true() {
        // `(if true -> :wat::core::i64 1 0)` — cond canonical → project to then-branch.
        let h = step_drive_to_terminal("(:wat::core::if true 1 0)");
        assert_eq!(h.as_i64(), Some(1));
    }

    #[test]
    fn step_if_branch_false() {
        let h = step_drive_to_terminal("(:wat::core::if false 1 0)");
        assert_eq!(h.as_i64(), Some(0));
    }

    #[test]
    fn step_if_cond_reduces() {
        // `(if (= 1 1) -> :wat::core::i64 1 0)` — cond non-canonical, descend until
        // BoolLit, then project.
        let h = step_drive_to_terminal("(:wat::core::if (:wat::core::= 1 1) 1 0)");
        assert_eq!(h.as_i64(), Some(1));
    }

    #[test]
    fn step_match_canonical() {
        // `(match (Some 5) -> :wat::core::i64 ((Some n) n) (:None 0))` —
        // scrutinee match-canonical (Some + canonical inner); arm
        // selection binds n→5; substituted body reduces to terminal.
        let h = step_drive_to_terminal(
            "(:wat::core::match (:wat::core::Some 5) ((:wat::core::Some n) n) (:wat::core::None 0))",
        );
        assert_eq!(h.as_i64(), Some(5));
    }

    #[test]
    fn step_match_scrutinee_reduces() {
        // `(match (+ 1 1) -> :wat::core::i64 (n n))` — scrutinee is arithmetic,
        // descend until canonical, then arm selection.
        let h = step_drive_to_terminal("(:wat::core::match (:wat::i64::+ 1 1) (n n))");
        assert_eq!(h.as_i64(), Some(2));
    }

    #[test]
    fn step_user_function_call() {
        // User define `square` — β-reduction by substitution.
        let src = format!(
            r#"
            {}
            (:wat::core::defn :my::test::square [n <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* n n))
            (:my::test::step-to-terminal
              (:wat::core::quote (:my::test::square 3)))
            "#,
            step_to_terminal_prelude()
        );
        match run(&src).unwrap() {
            Value::holon__HolonAST(h) => assert_eq!(h.as_i64(), Some(9)),
            other => panic!("expected HolonAST, got {:?}", other),
        }
    }

    #[test]
    fn step_effectful_kernel_rejected() {
        // `:wat::kernel::*` ops are effectful; step-mode refuses with
        // EvalError kind="effectful-in-step". We pick `assertion-failed!`
        // because it doesn't need a channel/mailbox to be quoted.
        let s = step_to_show(
            r#"(:wat::eval-step!
                 (:wat::core::quote
                   (:wat::kernel::assertion-failed! "x" :wat::core::None :wat::core::None)))"#,
        );
        assert_eq!(
            s,
            r#":wat::core::EvalError{#0: "effectful-in-step", #1: "eval-step! refuses effectful op: :wat::kernel::assertion-failed!"}"#
        );
    }

    #[test]
    fn step_round_trip_agrees_with_eval_ast() {
        // Five forms: each driven to terminal via step, vs eval-ast!
        // result. Same HolonAST out either way (arc 066's wrap aligns
        // step's terminal with eval-ast!'s Ok-arm).
        let forms = [
            ("(:wat::i64::+ 2 2)", 4),
            ("(:wat::i64::* 3 7)", 21),
            ("(:wat::core::if true 10 20)", 10),
            ("(:wat::core::let [x 5] (:wat::i64::+ x 1))", 6),
            ("(:wat::core::match (:wat::core::Some 7) ((:wat::core::Some n) n) (:wat::core::None 0))", 7),
        ];
        for (form, expected) in forms {
            let h = step_drive_to_terminal(form);
            assert_eq!(
                h.as_i64(),
                Some(expected),
                "step-driven: form `{}` expected {}, got {:?}",
                form,
                expected,
                h
            );
            // eval-ast! agreement. Arc 102: bare i64 returned
            // directly via the polymorphic Result<:T, :EvalError>
            // scheme; no atom-value extraction needed.
            let eval_src = format!(
                "(:wat::core::match (:wat::eval-ast! (:wat::core::quote {})) \
                  ((:wat::core::Ok n) n) ((:wat::core::Err _) -1))",
                form
            );
            match eval_expr(&eval_src).unwrap() {
                Value::i64(n) => assert_eq!(
                    n, expected,
                    "eval-ast!: form `{}` expected {}, got {}",
                    form, expected, n
                ),
                other => panic!("expected i64, got {:?}", other),
            }
        }
    }

    #[test]
    fn step_tail_recursion_terminates_under_bound() {
        // `sum-to` recurses by tail call. Each β-reduction substitutes
        // the body in place — no stack growth — so a small `n` should
        // terminate well under a generous step bound. We count the
        // rewrites driven through `:wat::eval-step!` and assert the
        // total stays below the bound (mirrors arc 003's TCO claim
        // at the step level).
        let src = format!(
            r#"
            (:wat::core::defn :my::test::sum-to [n <- :wat::core::i64 acc <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::if (:wat::core::= n 0)
                              acc
                              (:my::test::sum-to (:wat::i64::- n 1)
                                                 (:wat::i64::+ acc n))))
            (:wat::core::defn :my::test::step-count [form <- :wat::WatAST n <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::match (:wat::eval-step! form)
                              ((:wat::core::Ok r)
                                (:wat::core::match r
                                  ((:wat::eval::StepResult::StepNext next)
                                    (:my::test::step-count next (:wat::i64::+ n 1)))
                                  ((:wat::eval::StepResult::StepTerminal h) n)
                                  ((:wat::eval::StepResult::AlreadyTerminal h) n)))
                              ((:wat::core::Err e) -1)))
            {}
            (:wat::core::let
              [sum
                (:my::test::step-to-terminal
                  (:wat::core::quote (:my::test::sum-to 3 0)))
               steps
                (:my::test::step-count
                  (:wat::core::quote (:my::test::sum-to 3 0)) 0)]
              (:wat::core::Tuple sum steps))
            "#,
            step_to_terminal_prelude()
        );
        match run(&src).unwrap() {
            Value::Tuple(t) => {
                let elems = (*t).clone();
                let h = match &elems[0] {
                    Value::holon__HolonAST(h) => h.clone(),
                    other => panic!("sum: expected HolonAST, got {:?}", other),
                };
                let steps = match &elems[1] {
                    Value::i64(n) => *n,
                    other => panic!("steps: expected i64, got {:?}", other),
                };
                assert_eq!(h.as_i64(), Some(6), "sum-to 3 0 should equal 6");
                assert!(steps > 0 && steps < 50, "steps out of bound: {}", steps);
            }
            other => panic!("expected tuple, got {:?}", other),
        }
    }

    #[test]
    fn step_holon_constructor_atom() {
        // `(:wat::holon::to-holon "k")` — primitive string input to the
        // polymorphic UP verb; fires in one step. The result is
        // `HolonAST::String("k")` — to-holon lifts primitive strings to
        // typed leaves (the Atom-wrap is reserved for HolonAST inputs).
        //
        // Arc 225 Stone 225.1: the original test used
        // `(:wat::holon::Atom "k")` (old polymorphic Atom). After the
        // narrow, Atom only accepts HolonAST; the polymorphic UP arm
        // moved to :wat::holon::to-holon.
        let h = step_drive_to_terminal(r#"(:wat::holon::to-holon "k")"#);
        match &*h {
            HolonAST::String(s) if &s[..] == "k" => {}
            other => panic!("expected HolonAST::String(\"k\"), got {:?}", other),
        }
    }

    #[test]
    fn step_holon_constructor_bind() {
        // `(:wat::holon::Bind (to-holon "k") (to-holon "v"))` — both args
        // are holon-canonical (constructor lists with primitive fields),
        // so the whole tree fires as one rewrite. The result is the
        // Bind tree over typed-leaf children. Verifies the Phase 3
        // type-loss workaround: lifting a typed leaf back to a bare
        // primitive WatAST would make the parent's require_holon
        // check fail, so the macro-step rule keeps the holon tree
        // intact through eval.
        //
        // Arc 225 Stone 225.1: Atom "k" → to-holon "k" (polymorphic
        // UP verb absorbs the old Atom primitive-lift arm).
        let h = step_drive_to_terminal(
            r#"(:wat::holon::Bind (:wat::holon::to-holon "k") (:wat::holon::to-holon "v"))"#,
        );
        match &*h {
            HolonAST::Bind(a, b) => {
                assert!(matches!(&**a, HolonAST::String(s) if &s[..] == "k"));
                assert!(matches!(&**b, HolonAST::String(s) if &s[..] == "v"));
            }
            other => panic!("expected HolonAST::Bind, got {:?}", other),
        }
    }

    #[test]
    fn step_holon_constructor_bundle() {
        // `(:wat::holon::Bundle (:wat::core::Vector :- [HolonAST] (Atom "a")
        //                                                  (Atom "b")))`
        // — the vec list's elements are themselves holon-canonical
        // (Atom forms with primitive args). Bundle's arg recognizes
        // the `(vec :- [T] <holons>...)` shape as canonical, so the entire
        // tree fires in one step. The result is a HolonAST::Bundle of
        // typed-leaf Strings.
        //
        // Arc 109 "THE LAST DOORS" door 3 — this test used to be pinned to
        // the BARE-KEYWORD-ONLY spelling, with a comment explaining that
        // converting it to `:- [...]` turned it red: `is_holon_arg_canonical`
        // (`src/holon/ast.rs`) and `lower_bundle` (`src/lower.rs`) both
        // required a bare type Keyword at `items[1]` and never learned the
        // `:-` marker, so the canonical spelling — the only one a user can
        // write once the checker walls the bare form out of source — could
        // never fire as a single step
        // (`NOTE-bundle-is-coupled-to-the-retired-spelling.md`). Both sites
        // now peel the param-spec via `peel_param_spec` instead of assuming
        // its absence, so this test now asserts the single-step path on a
        // form a user can actually write — which it had never done before.
        //
        // Bundle exercises the encoding pipeline (capacity guard +
        // dim router), so this test runs through `run_with_ctx`
        // instead of `run`.
        // Arc 225 Stone 225.1: Atom "a"/"b" → to-holon (polymorphic UP
        // verb absorbs old Atom primitive-lift arm).
        let src = format!(
            r#"
            {}
            (:my::test::step-to-terminal
              (:wat::core::quote
                (:wat::holon::Bundle
                  (:wat::core::Vector :- [:wat::holon::HolonAST]
                    (:wat::holon::to-holon "a")
                    (:wat::holon::to-holon "b")))))
            "#,
            step_to_terminal_prelude()
        );
        let v = run_with_ctx(&src, 1024).unwrap();
        let h = match v {
            Value::holon__HolonAST(h) => h,
            other => panic!("expected HolonAST, got {:?}", other),
        };
        match &*h {
            HolonAST::Bundle(items) => {
                assert_eq!(items.len(), 2, "expected 2 elements, got {}", items.len());
                assert!(matches!(&items[0], HolonAST::String(s) if &s[..] == "a"));
                assert!(matches!(&items[1], HolonAST::String(s) if &s[..] == "b"));
            }
            other => panic!("expected HolonAST::Bundle, got {:?}", other),
        }
    }

    #[test]
    fn step_holon_thermometer() {
        // `(:wat::holon::Thermometer 0.5 0.0 1.0)` — three primitive
        // f64 args, all canonical, fires in one step.
        let h = step_drive_to_terminal("(:wat::holon::Thermometer 0.5 0.0 1.0)");
        match &*h {
            HolonAST::Thermometer { value, min, max } => {
                assert_eq!(*value, 0.5);
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 1.0);
            }
            other => panic!("expected HolonAST::Thermometer, got {:?}", other),
        }
    }

    #[test]
    fn step_outer_form_span_survives_rewrite() {
        // Per DESIGN's Q7: the rewritten outer form preserves the
        // original outer span. We parse `(+ (+ 1 2) 3)`, take the
        // outer list's parsed span, run one step (which descends the
        // inner `(+ 1 2)`), and assert the rebuilt outer form carries
        // the same span. Direct Rust access — no eval-step! wrap.
        let src = "(:wat::i64::+ (:wat::i64::+ 1 2) 3)";
        let ast = crate::parse_one!(src).expect("parse");
        let outer_span = ast.span().clone();
        let (sym, _, _) = stdlib_loaded();
        let env = Environment::new();
        let stepped = step_form(&ast, &env, sym).expect("step");
        match stepped {
            StepValue::Next(WatAST::List(_, span)) => {
                assert_eq!(span, outer_span, "outer-form span should survive a rewrite");
            }
            other => panic!("expected StepNext(List), got {:?}", other),
        }
    }

    #[test]
    fn to_watast_eval_ast_round_trip_for_form() {
        // A wat form built via `from-wat` round-trips through
        // `to-wat` → `eval-ast!` to its terminal value (bare,
        // post-arc-102). The arc-057 round-trip claim, now with
        // T = :i64 since arc 102 makes eval-ast! polymorphic.
        // Arc 225 Stone 225.1: from-watast → from-wat, to-watast → to-wat.
        let src = r#"
            (:wat::core::let
              [form
                (:wat::holon::from-wat
                  (:wat::core::quote (:wat::i64::+ 40 2)))
               ast (:wat::holon::to-wat form)]
              (:wat::core::match (:wat::eval-ast! ast)
                ((:wat::core::Ok n) n)
                ((:wat::core::Err _) -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(42) => {}
            v => panic!("expected 42 (round-trip), got {:?}", v),
        }
    }

    #[test]
    fn core_bytes_alias_resolves_to_vec_u8() {
        // Arc 062 — :wat::core::Bytes is a structural alias for
        // :Vec<u8>. Both forms must work at let-binding sites; the
        // pipeline through arc 061's vector-bytes / bytes-vector
        // type-checks identically whichever annotation is used.
        let src = r#"
            (:wat::core::let
              [v
                (:wat::holon::encode (:wat::holon::to-holon "alias-test"))
               ;; Annotate with the alias on one binding...
               bs1
                (:wat::holon::vector-bytes v)
               ;; ...and the verbose form on the other.
               bs2
                (:wat::holon::vector-bytes v)
               ;; Both must round-trip cleanly through bytes-vector.
               maybe-v1
                (:wat::holon::bytes-vector bs1)
               maybe-v2
                (:wat::holon::bytes-vector bs2)]
              ;; Bytes are deterministic; so the two byte-buffers
              ;; produced from the same vector must be equal at the
              ;; structural level.
              (:wat::core::= bs1 bs2))
        "#;
        match eval_with_ctx(src, 1024).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true (alias resolves structurally), got {:?}", v),
        }
    }

    // ─── HandlePool ────────────────────────────────────────────────────

    #[test]
    fn handle_pool_pop_all_then_finish() {
        let src = r#"
            (:wat::core::let
              [pool
                (:wat::kernel::HandlePool::new "test" (:wat::core::Vector :- [:i64] 1 2 3))
               a (:wat::kernel::HandlePool::pop pool)
               b (:wat::kernel::HandlePool::pop pool)
               c (:wat::kernel::HandlePool::pop pool)
               _ (:wat::kernel::HandlePool::finish pool)]
              (:wat::i64::+ (:wat::i64::+ a b) c))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(6) => {}
            v => panic!("expected 6, got {:?}", v),
        }
    }

    #[test]
    fn handle_pool_pop_from_empty_errors() {
        let src = r#"
            (:wat::core::let
              ((pool
                (:wat::kernel::HandlePool::new "empty" (:wat::core::Vector :- [:i64])))
               (_ (:wat::kernel::HandlePool::pop pool)))
              0)
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn handle_pool_finish_with_orphans_errors() {
        let src = r#"
            (:wat::core::let
              ((pool
                (:wat::kernel::HandlePool::new "orphaned" (:wat::core::Vector :- [:i64] 1 2 3)))
               (_ (:wat::kernel::HandlePool::finish pool)))
              0)
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }))
        );
    }

    #[test]
    fn handle_pool_name_surfaces_in_error() {
        let src = r#"
            (:wat::core::let
              [pool
                (:wat::kernel::HandlePool::new "named-pool" (:wat::core::Vector :- [:i64]))
               _ (:wat::kernel::HandlePool::pop pool)]
              0)
        "#;
        let err = eval_expr(src).unwrap_err();
        let msg = format!("{}", err);
        // rune:lint(loose-assert) — Display embeds a Rust source file path/line/col prefix
        // (e.g. "src/runtime.rs:N:col:end_col:"); the line number shifts when lines are added
        // above the eval_expr call site, making full assert_eq! infeasible
        assert!(
            msg.contains("named-pool"),
            "error should name the pool; got: {}",
            msg
        );
    }

    // ─── Stdlib math ───────────────────────────────────────────────────

    #[test]
    fn math_ln_of_e_is_one() {
        // ln(e) = 1.
        let src = "(:wat::math::ln 2.718281828459045)";
        match eval_expr(src).unwrap() {
            Value::f64(x) => assert!((x - 1.0).abs() < 1e-10, "got {}", x),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    // Arc 255 Stone HOME-9 — `math_log_is_natural_log` (which asserted `log` and `ln` agree)
    // is DELETED along with `log` itself: that test was proving the level-1 lie
    // (`:wat::std::math::log` was wired to the SAME `f64::ln` as `ln`), not a real feature.
    // `log` had zero call sites in the corpus and is not carried forward under `:wat::math::`.

    #[test]
    fn math_sin_pi_is_zero() {
        let src = "(:wat::math::sin (:wat::math::pi))";
        match eval_expr(src).unwrap() {
            Value::f64(x) => assert!(x.abs() < 1e-10, "got {}", x),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_cos_zero_is_one() {
        match eval_expr("(:wat::math::cos 0.0)").unwrap() {
            Value::f64(x) => assert_eq!(x, 1.0),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_pi_is_std_const() {
        match eval_expr("(:wat::math::pi)").unwrap() {
            Value::f64(x) => assert_eq!(x, std::f64::consts::PI),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_ln_accepts_i64_promotion() {
        // Integer arg gets promoted to f64 before the call.
        match eval_expr("(:wat::math::ln 1)").unwrap() {
            Value::f64(x) => assert_eq!(x, 0.0),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_ln_wrong_arity() {
        let err = eval_expr("(:wat::math::ln 1.0 2.0)").unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::ArityMismatch { .. }))
        );
    }

    #[test]
    fn math_ln_refuses_non_number() {
        let err = eval_expr(r#"(:wat::math::ln "nope")"#).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    #[test]
    fn math_old_std_spelling_is_retired_not_silently_accepted() {
        // Arc 255 Stone HOME-9, acceptance row 3 — the OLD spelling names its replacement,
        // not a bare `unknown function`. `eval_expr` here bypasses `check.rs` (parse +
        // macro-expand + `eval_inner` only — see its own doc a few dozen lines up), so this
        // exercises the RUNTIME retirement consult (door 2, `src/value/signal.rs`); the
        // CHECK-TIME door (door 1) is what `tests/cli/retirement_table_reachable.rs` drives
        // end-to-end through the real binary for every `RETIREMENT_TABLE` row, this one
        // included.
        let err = eval_expr("(:wat::std::math::sqrt 16.0)").unwrap_err();
        // Arc 255 HOME-9 — assert the MECHANISM, not the rendering. The runtime door's
        // Display consults `remedies_for` (src/value/signal.rs:598); a `contains("is retired")`
        // check passes on any error whose text happens to hold that phrase and tripped
        // `no_loose_string_assert`. Match the discriminant, then ask the remedy table itself
        // for the replacement — byte-exact, and it fails if the retirement row is ever removed.
        let EvalBreak::Diagnostic(rt) = err else {
            panic!("expected a RuntimeError from the retired spelling; got a non-Diagnostic break")
        };
        let RuntimeErrorKind::UnknownFunction(path) = rt.kind() else {
            panic!("expected UnknownFunction; got {:?}", rt.kind())
        };
        assert_eq!(path.as_str(), ":wat::std::math::sqrt");
        let remedies = crate::remedy::remedies_for(path, std::iter::empty());
        assert_eq!(
            remedies.first().map(|r| r.form.as_str()),
            Some(":wat::math::sqrt"),
            "the retirement table must name the replacement for the retired spelling"
        );
    }

    #[test]
    fn handle_pool_refuses_non_string_name() {
        let src = r#"
            (:wat::kernel::HandlePool::new 42 (:wat::core::Vector :- [:i64]))
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(
            matches!(err, EvalBreak::Diagnostic(e) if matches!(e.kind(), RuntimeErrorKind::TypeMismatch { .. }))
        );
    }

    // queue roundtrip across threads — covered by tests/wat_spawn_fn.rs
    // (mini-TCP shape on spawn-thread + Thread/join-result).

    /// Arc 140 slice 1 — runtime sandbox-scope leak fires when an
    /// inner sub-program's call head misses the inner scope but
    /// resolves in the outer. The teaching diagnostic carries both
    /// spans (offending invocation + outer-scope define) so users
    /// (and agents) navigate without grepping. Constructs the
    /// scenario directly: outer SymbolTable holds `:my::helper`,
    /// inner SymbolTable does NOT, with outer_symbols attached.
    #[test]
    fn runtime_sandbox_scope_leak_fires_with_outer_attached() {
        // Build the OUTER scope: stdlib + a user-defined helper.
        let (stdlib_sym, _, _) = stdlib_loaded();
        let mut outer_sym = stdlib_sym.clone();
        let helper_body = crate::parse_one!("42").expect("parse body");
        outer_sym.register_function(
            ":my::helper".to_string(),
            Arc::new(Function {
                name: Some(":my::helper".to_string()),
                params: vec![],
                type_params: vec![],
                param_types: vec![],
                ret_type: crate::types::TypeExpr::Path(":wat::core::i64".to_string()),
                rest_param: None,
                rest_param_type: None,
                body: FunctionBody::Wat(Arc::new(helper_body)),
                closed_env: None,
                rete: None,
                synthesized_for: None,
            }),
        );

        // Build the INNER scope: stdlib only, no `:my::helper`.
        // Attach outer_symbols so the runtime check can fire.
        let mut inner_sym = stdlib_sym.clone();
        inner_sym.outer_symbols = Some(Arc::new(outer_sym));

        // Construct the call: `(:my::helper)`.
        let call = crate::parse_one!("(:my::helper)").expect("parse call");

        let env = Environment::new();
        let result = eval_inner(&call, &env, &inner_sym);

        match result {
            Err(EvalBreak::Diagnostic(e)) => match e.kind() {
                RuntimeErrorKind::SandboxScopeLeak { offending_name, .. } => {
                    assert_eq!(offending_name, ":my::helper");
                }
                other => panic!("expected SandboxScopeLeak; got {:?}", other),
            },
            Err(other) => panic!("expected SandboxScopeLeak; got {:?}", other),
            Ok(v) => panic!("expected SandboxScopeLeak err; got Ok({:?})", v),
        }
    }

    /// Arc 140 slice 1 — when the offending name is NOT in the outer
    /// scope either, the runtime falls through to the existing
    /// `UnknownFunction` error. Confirms slice 1 doesn't misfire on
    /// genuine typos.
    #[test]
    fn runtime_unknown_function_when_outer_also_missing() {
        let (stdlib_sym, _, _) = stdlib_loaded();
        let outer_sym = stdlib_sym.clone();

        let mut inner_sym = stdlib_sym.clone();
        inner_sym.outer_symbols = Some(Arc::new(outer_sym));

        let call = crate::parse_one!("(:totally::made::up::name)").expect("parse call");

        let env = Environment::new();
        let result = eval_inner(&call, &env, &inner_sym);

        match result {
            Err(EvalBreak::Diagnostic(e)) => match e.kind() {
                RuntimeErrorKind::UnknownFunction(name) => {
                    assert_eq!(name, ":totally::made::up::name");
                }
                RuntimeErrorKind::SandboxScopeLeak { .. } => {
                    panic!("SandboxScopeLeak misfired on a genuinely-unknown name")
                }
                other => panic!("expected UnknownFunction; got {:?}", other),
            },
            other => panic!("expected UnknownFunction; got {:?}", other),
        }
    }

    /// Arc 140 slice 1 — when the SymbolTable has no outer_symbols
    /// attached (the entry program / non-sandboxed runtime), the
    /// runtime falls through to UnknownFunction even if some other
    /// table elsewhere has the name. The leak detection only runs
    /// for sandboxed sub-programs.
    #[test]
    fn runtime_no_leak_when_outer_not_attached() {
        let (stdlib_sym, _, _) = stdlib_loaded();
        let inner_sym = stdlib_sym.clone(); // outer_symbols stays None

        let call = crate::parse_one!("(:my::helper)").expect("parse call");
        let env = Environment::new();
        let result = eval_inner(&call, &env, &inner_sym);

        match result {
            Err(EvalBreak::Diagnostic(e)) => match e.kind() {
                RuntimeErrorKind::UnknownFunction(name) => {
                    assert_eq!(name, ":my::helper");
                }
                other => panic!("expected UnknownFunction; got {:?}", other),
            },
            other => panic!("expected UnknownFunction; got {:?}", other),
        }
    }

    /// Arc 138 slice 3a — every user-facing RuntimeError surfaced on
    /// real wat source carries `<file>:<line>:<col>:` in its rendered
    /// Display output. Canary uses UnboundSymbol — eval's
    /// `WatAST::Symbol` arm threads `span.clone()`. The runtime is
    /// the equivalent of slice 1's CheckError canary at the eval
    /// layer.
    #[test]
    fn arc138_runtime_error_message_carries_span() {
        let err = eval_expr("nonexistent-bare-symbol").unwrap_err();
        let rendered = format!("{}", err);
        // rune:lint(loose-assert) — variable Rust source file path / eval-synthetic label embedded in error Display output (varies by build environment)
        assert!(
            rendered.contains("<eval>:") || rendered.contains("src/") || rendered.contains(".rs:"),
            "RuntimeError Display must include source coordinates; rendered:\n{}",
            rendered
        );
    }

    /// Arc 143 slice 5b — `value_to_watast` bridges `Value::holon__HolonAST`.
    ///
    /// A `HolonAST::Symbol` whose content begins with `:` is a keyword.
    /// `holon_to_watast` maps it to `WatAST::Keyword`; `value_to_watast`
    /// must now thread through `holon_to_watast` instead of falling to the
    /// TypeMismatch catch-all.
    #[test]
    fn arc143_slice5b_value_to_watast_accepts_holon_ast() {
        use std::sync::Arc;
        let h = HolonAST::symbol(":foo");
        let v = Value::holon__HolonAST(Arc::new(h));
        let result = value_to_watast("test_op", v, crate::rust_caller_span!());
        match result {
            Ok(WatAST::Keyword(k, _)) => assert_eq!(k, ":foo"),
            other => panic!("expected Ok(WatAST::Keyword(\":foo\", _)), got {:?}", other),
        }
    }

    // ─── Arc 159 — untyped let bindings (new shape) ────────────────────

    /// Arc 159 test 1 — user's stated end goal: `(let ((x 2)) (+ x 1))` → 3.
    ///
    /// The new binding shape `(x 2)` (bare Symbol binder, no `:T`) must
    /// parse, infer, and evaluate correctly. This is the canonical end
    /// goal announced in arc 159 DESIGN.
    #[test]
    fn arc159_new_shape_basic_addition() {
        let src = r#"
            (:wat::core::let [x 2] (:wat::i64::+ x 1))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(3) => {}
            v => panic!("arc 159 user goal: expected 3:i64; got {:?}", v),
        }
    }

    /// Arc 159 test 2 — multi-binding sequential: later binding sees earlier.
    ///
    /// `(let ((a 1) (b (+ a 1))) b)` → 2. Sequential semantics (arc 154
    /// precedent) extended to new-shape bindings: `b`'s RHS evaluates in
    /// scope that includes `a`.
    #[test]
    fn arc159_new_shape_multi_binding_sequential() {
        let src = r#"
            (:wat::core::let
              [a 1
               b (:wat::i64::+ a 1)]
              b)
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(2) => {}
            v => panic!("arc 159 multi-binding: expected 2:i64; got {:?}", v),
        }
    }

    /// Arc 159 test 3 — closure capture through new shape.
    ///
    /// `(let ((x 2)) (fn () x))` — the closure captures `x` from the
    /// lexical scope introduced by the new-shape binding; calling it
    /// returns 2.
    #[test]
    fn arc159_new_shape_closure_capture() {
        let src = r#"
            (:wat::core::let
              [x 2]
              ((:wat::core::fn [] -> :wat::core::i64 x)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(2) => {}
            v => panic!("arc 159 closure capture: expected 2:i64; got {:?}", v),
        }
    }

    /// Arc 159 test 4 — sequential binding cross-reference: `(let ((a 1) (b (+ a 1))) (+ a b))` → 3.
    #[test]
    fn arc159_new_shape_sequential_cross_reference() {
        let src = r#"
            (:wat::core::let
              [a 1
               b (:wat::i64::+ a 1)]
              (:wat::i64::+ a b))
        "#;
        // a=1, b=2, a+b=3
        match eval_expr(src).unwrap() {
            Value::i64(3) => {}
            v => panic!(
                "arc 159 sequential cross-reference: expected 3:i64; got {:?}",
                v
            ),
        }
    }

    /// Arc 159 test 5 — nested let forms with new shape.
    ///
    /// Outer `(let ((x 10)) ...)` + inner `(let ((y (+ x 5))) y)` → 15.
    #[test]
    fn arc159_new_shape_nested_let() {
        let src = r#"
            (:wat::core::let
              [x 10]
              (:wat::core::let
                [y (:wat::i64::+ x 5)]
                y))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(15) => {}
            v => panic!("arc 159 nested let: expected 15:i64; got {:?}", v),
        }
    }

    /// Arc 159 test 9 — 2-element destructure with new-shape binder for
    /// the source tuple.
    ///
    /// `(let ((p ...) ((a b) p)) (+ a b))` where the outer binding uses
    /// new shape and the destructure uses the existing path. Verifies arc
    /// 158 v1's destructure-mangling bug does NOT recur: `((a b) p)` is
    /// a destructure binding (binder is all Symbols), not a new-shape or
    /// legacy binding.
    #[test]
    fn arc159_destructure_two_element_with_new_shape_source() {
        // Pre-bind `p` as a 2-tuple; then let new-shape `q` point to `p`;
        // then destructure `q` into `(a b)`. Exercises both paths.
        let src = r#"
            (:wat::core::let
              [[a b] p]
              (:wat::i64::+ a b))
        "#;
        let p = pair(Value::i64(3), Value::i64(4));
        match eval_with_binding(src, "p", p).unwrap() {
            Value::i64(7) => {}
            v => panic!(
                "arc 159 destructure 2-elem: expected 7:i64 (3+4); got {:?} — v1 destructure bug may have recurred",
                v
            ),
        }
    }

    /// Arc 159 test 10 — 3-element destructure still works.
    ///
    /// `(let (((a b c) tup)) (+ a (+ b c)))` where `tup` is a 3-tuple.
    /// The walker must not misclassify `((a b c) tup)` — it has no
    /// Keyword at binder[1], so it's a destructure.
    #[test]
    fn arc159_destructure_three_element() {
        let src = r#"
            (:wat::core::let
              [[a b c] tup]
              (:wat::i64::+ a (:wat::i64::+ b c)))
        "#;
        let tup = Value::Tuple(std::sync::Arc::new(vec![
            Value::i64(1),
            Value::i64(2),
            Value::i64(3),
        ]));
        match eval_with_binding(src, "tup", tup).unwrap() {
            Value::i64(6) => {}
            v => panic!(
                "arc 159 destructure 3-elem: expected 6:i64 (1+2+3); got {:?}",
                v
            ),
        }
    }

    /// Arc 159 test — mixed new-shape and destructure in one let.
    ///
    /// `(let ((q p) ((a b) q)) (+ a b))` — `q` is new-shape binding
    /// (bare Symbol); `((a b) q)` is destructure (all-Symbol binder).
    /// Exercises both paths in a single form.
    #[test]
    fn arc159_mixed_new_shape_and_destructure() {
        let src = r#"
            (:wat::core::let
              [q p
               [a b] q]
              (:wat::i64::+ a b))
        "#;
        let p = pair(Value::i64(5), Value::i64(6));
        match eval_with_binding(src, "p", p).unwrap() {
            Value::i64(11) => {}
            v => panic!(
                "arc 159 mixed new+destructure: expected 11:i64 (5+6); got {:?}",
                v
            ),
        }
    }

    // ─── Arc 170 slice 3 Gap A — keyword reflection primitives ─────────

    #[test]
    fn keyword_to_string_strips_leading_colon() {
        // Arc 109 "annihilate the angle bracket" — the third case used a
        // parametric keyword literal (`:wat::core::Vector<wat::core::i64>`);
        // a parametric type is no longer spellable as a single Keyword at
        // all (only the `:-` reference FORM survives, which is a List, not
        // a Keyword, and `keyword/to-string` operates on a Keyword). The
        // subject here — strip-leading-colon on a multi-segment `::` path
        // — is unaffected by the angle bracket, so it keeps exercising a
        // plain multi-segment keyword instead.
        assert_eq!(
            expect_string(eval_expr("(:wat::keyword::to-string :foo)").unwrap()),
            "foo"
        );
        assert_eq!(
            expect_string(eval_expr("(:wat::keyword::to-string :wat::core::i64)").unwrap()),
            "wat::core::i64"
        );
        assert_eq!(
            expect_string(
                eval_expr("(:wat::keyword::to-string :wat::core::Vector)")
                    .unwrap()
            ),
            "wat::core::Vector"
        );
    }

    #[test]
    fn keyword_from_string_prepends_colon() {
        let result = eval_expr(r#"(:wat::keyword::from-string "foo")"#).unwrap();
        match result {
            Value::wat__core__keyword(k) => assert_eq!(k.as_str(), ":foo"),
            other => panic!("expected keyword; got {:?}", other),
        }
        let result2 = eval_expr(r#"(:wat::keyword::from-string "wat::core::i64")"#).unwrap();
        match result2 {
            Value::wat__core__keyword(k) => assert_eq!(k.as_str(), ":wat::core::i64"),
            other => panic!("expected keyword; got {:?}", other),
        }
    }

    #[test]
    fn keyword_reflection_round_trip() {
        // Arc 109 "annihilate the angle bracket" — the third case used a
        // parametric keyword literal; see the comment on
        // `keyword_to_string_strips_leading_colon` above for why a
        // parametric type cannot be spelled as a single Keyword any more.
        // The subject (round-trip through a multi-segment `::` path) is
        // unaffected, so it keeps exercising a plain multi-segment
        // keyword instead.
        let cases = [
            (":foo", "foo"),
            (":wat::core::i64", "wat::core::i64"),
            (
                ":wat::kernel::Receiver",
                "wat::kernel::Receiver",
            ),
        ];
        for (kw, expected_text) in &cases {
            // to-string strips colon
            let text = expect_string(
                eval_expr(&format!("(:wat::keyword::to-string {})", kw)).unwrap(),
            );
            assert_eq!(&text, expected_text, "to-string({}) should strip ':'", kw);
            // from-string(to-string(k)) == k
            let roundtrip = eval_expr(&format!(
                r#"(:wat::keyword::from-string (:wat::keyword::to-string {}))"#,
                kw
            ))
            .unwrap();
            match roundtrip {
                Value::wat__core__keyword(k) => {
                    assert_eq!(k.as_str(), *kw, "round-trip failed for {}", kw)
                }
                other => panic!("expected keyword for {}; got {:?}", kw, other),
            }
        }
    }

    #[test]
    fn keyword_from_string_rejects_colon_prefix() {
        let err = eval_expr(r#"(:wat::keyword::from-string ":foo")"#).unwrap_err();
        let msg = format!("{}", err);
        // rune:lint(loose-assert) — Display embeds a Rust source file path/line/col prefix
        // (e.g. "src/runtime.rs:N:col:end_col:"); the line number shifts when lines are added
        // above the eval_expr call site, making full assert_eq! infeasible
        assert!(
            msg.contains("starts with ':'"),
            "expected 'starts with \":\"' in error; got: {}",
            msg
        );
    }

    // ─── Stone S-C.2c — Bucket C co-located unit test ─────────────────────────
    //
    // `to_holon_inner` is private; this is its only reachable home.
    // Contract: calling to_holon_inner on a base `Value::Aggregate(Record)` MUST return
    // `Err(..)` carrying the teaching message.  It MUST NOT panic or return `Ok`.
    #[test]
    fn to_holon_inner_base_record_returns_err_with_teaching_message() {
        let base = Value::Aggregate(Arc::new(AggregateValue::record(
            "my::Pt".to_string(),
            // `my::Pt` is a synthetic, never-registered test class (the contract under
            // test is `to_holon_inner`'s error message, not field naming) — positional
            // labels, not an invented semantic name.
            Arc::new(vec!["0".to_string(), "1".to_string()]),
            Arc::new(vec![Value::f64(1.0), Value::f64(2.0)]),
        )));
        let span = crate::rust_caller_span!();
        let result = to_holon_inner(base, &span);
        assert!(
            result.is_err(),
            "to_holon_inner(base_record) must return Err, not Ok"
        );
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            // rune:lint(loose-assert) — Display embeds a rust_caller_span!() from the test code
            // (e.g. "src/runtime.rs:N:col:"); line shifts when lines are added above the
            // rust_caller_span!() call, making full assert_eq! infeasible; these N checks test
            // the stable message body only (class name + teaching text)
            err_msg.contains("base record")
                && err_msg.contains("my::Pt")
                && err_msg.contains("has no holon flavor")
                && err_msg.contains(":wat::holon::defrecord"),
            "error must contain key teaching-message parts (base record, class name, defrecord hint); got: {}",
            err_msg
        );
    }

    // Arc 238 Stone 238.1 — co-located unit tests for Instant + Duration equality.
    // The external probe (probe_arc238_eq_completeness.rs) covers records/maps/sets at the
    // wat surface. Instant and Duration are not easily constructible at the wat surface
    // without time verbs, so we verify values_equal directly at the Rust layer here.

    #[test]
    fn values_equal_instant_same() {
        use chrono::TimeZone;
        let t = chrono::Utc.timestamp_opt(1_000_000, 0).unwrap();
        let a = Value::Instant(t);
        let b = Value::Instant(t);
        assert_eq!(values_equal(&a, &b), Some(true));
    }

    #[test]
    fn values_equal_instant_different() {
        use chrono::TimeZone;
        let t1 = chrono::Utc.timestamp_opt(1_000_000, 0).unwrap();
        let t2 = chrono::Utc.timestamp_opt(2_000_000, 0).unwrap();
        let a = Value::Instant(t1);
        let b = Value::Instant(t2);
        assert_eq!(values_equal(&a, &b), Some(false));
    }

    #[test]
    fn values_equal_duration_same() {
        let a = Value::Duration(123_456_789);
        let b = Value::Duration(123_456_789);
        assert_eq!(values_equal(&a, &b), Some(true));
    }

    #[test]
    fn values_equal_duration_different() {
        let a = Value::Duration(100);
        let b = Value::Duration(200);
        assert_eq!(values_equal(&a, &b), Some(false));
    }

    #[test]
    fn values_equal_wat_ast_same() {
        use std::sync::Arc;
        // Two structurally-identical WatAST nodes (IntLit, span-agnostic PartialEq).
        // crate::rust_caller_span!() is the synthetic sentinel — Span::eq is always true regardless.
        let ast_a = crate::ast::WatAST::IntLit(42, crate::rust_caller_span!());
        let ast_b = crate::ast::WatAST::IntLit(42, crate::rust_caller_span!());
        let a = Value::wat__WatAST(Arc::new(ast_a));
        let b = Value::wat__WatAST(Arc::new(ast_b));
        assert_eq!(values_equal(&a, &b), Some(true));
    }

    #[test]
    fn values_equal_wat_ast_different() {
        use std::sync::Arc;
        let ast_a = crate::ast::WatAST::IntLit(42, crate::rust_caller_span!());
        let ast_b = crate::ast::WatAST::IntLit(99, crate::rust_caller_span!());
        let a = Value::wat__WatAST(Arc::new(ast_a));
        let b = Value::wat__WatAST(Arc::new(ast_b));
        assert_eq!(values_equal(&a, &b), Some(false));
    }

    // Arc 255 Stone 255.1c-guard — manual perf harness for
    // `dispatch_keyword_head_value`. Drives the arithmetic hot path
    // (`:wat::i64::+`) directly (not through a wat program — an
    // interpreter loop's per-iteration cost is microseconds against the
    // dispatch call's nanoseconds, which would drown the signal).
    //
    // 296 Stone K, move 1, STOP-1: this belongs in `benches/` with its two
    // siblings (`perf_arc278_fire_baseline`), but `dispatch_keyword_head_value`
    // (this file, `fn dispatch_keyword_head_value`) has NO `pub` — it is
    // module-private, reachable only via `super::` from this `#[cfg(test)] mod
    // tests`. `benches/` is a separate crate target and can only see the crate's
    // `pub` surface, so relocating this benchmark would require making a hot
    // dispatch internal `pub` — an API change wearing a chore's clothes. STOP-1
    // forbids that trade, so this one stays a `#[cfg(test)]` unit test in place.
    // It is EXCLUDED from the default floor by `.config/nextest.toml`'s
    // `default-filter` (not `#[ignore]`) and included under `--profile slow`'s
    // `all()`, which re-admits everything default-filter excludes.
    //
    //   cargo nextest run --release --ignore-default-filter -E 'test(dispatch_keyword_head_value_perf)' --no-capture
    //   (or: cargo nextest run --release --profile slow -E 'test(dispatch_keyword_head_value_perf)')
    //
    // `std::hint::black_box` on both the args and the result stops LLVM
    // from proving the call is a pure, input-invariant function and
    // hoisting it out of the loop (loop-invariant code motion) or
    // constant-folding ITERS calls into one — a real risk here because
    // every iteration's arguments are otherwise identical. The
    // accumulator is folded with `wrapping_add` and asserted against a
    // closed-form expectation at the end, so the loop body's result is
    // both used and checked — the optimiser cannot delete it without
    // producing a wrong answer.
    #[test]
    fn dispatch_keyword_head_value_perf() {
        let (stdlib_sym, _stdlib_macros, _stdlib_types) = stdlib_loaded();
        let env = Environment::new();
        let list_span = crate::rust_caller_span!();
        let args = [
            WatAST::IntLit(1, crate::rust_caller_span!()),
            WatAST::IntLit(2, crate::rust_caller_span!()),
        ];

        const ITERS: u64 = 2_000_000;
        let mut acc: i64 = 0;
        let start = std::time::Instant::now();
        for _ in 0..ITERS {
            let head = std::hint::black_box(":wat::i64::+");
            let result = dispatch_keyword_head_value(
                head,
                std::hint::black_box(&args[..]),
                &list_span,
                &env,
                stdlib_sym,
            );
            match std::hint::black_box(result) {
                Ok(Value::i64(n)) => acc = acc.wrapping_add(n),
                other => panic!("unexpected dispatch result: {:?}", other),
            }
        }
        let elapsed = start.elapsed();
        let ns_per_op = elapsed.as_nanos() as f64 / ITERS as f64;

        // Closed form: every iteration adds exactly 1 + 2 = 3, folded with
        // wrapping_add — associative mod 2^64, so this equals a single
        // wrapping multiply. A mismatch means the loop body did not run
        // ITERS real dispatches.
        let expected = 3i64.wrapping_mul(ITERS as i64);
        assert_eq!(
            acc, expected,
            "accumulator mismatch — loop body was optimised away or short-circuited"
        );

        eprintln!(
            "255.1c-guard dispatch_keyword_head_value_perf: {:.2} ns/op over {} iters (elapsed {:?}, acc={})",
            ns_per_op, ITERS, elapsed, acc
        );
    }

    /// Arc 255 Stone 2a — the witness's own acceptance row: `(:wat::rete::i64::+ 1 2)` must
    /// evaluate to `3` THROUGH THE REGISTRY'S `alias_of` field, not through
    /// `dispatch_rete_op`'s `OpClass::Fallback` arm (`rete/vocabulary.rs`'s pre-existing,
    /// untouched row for this same name).
    ///
    /// ⚠ Deliberately calls `dispatch_keyword_head` DIRECTLY on a hand-built 2-arg AST —
    /// mirroring `dispatch_keyword_head_value_perf` just above, NOT a `.wat` fixture run
    /// through the full parse+CHECK+eval pipeline. `check.rs`'s `infer_list`
    /// (`crate::rete::vocabulary::rete_op_for` consulted before the registry, STOP-3 —
    /// untouched by this stone) still validates `:wat::rete::i64::+` against the OLD
    /// `Fallback`-class row's registered `TypeScheme` (4 params: `[I64, I64, Keyword, I64]`,
    /// `register_builtins`, `check.rs`) — a `.wat` source file spelling a bare 2-arg call to
    /// this name would FAIL STATIC TYPE CHECKING before ever reaching the runtime alias logic
    /// this test proves. Direct dispatch is therefore not a shortcut here; it is the only way
    /// to observe this stone's contract in isolation from that pre-existing, out-of-scope
    /// checker gap.
    ///
    /// ⛔ HOW THE REGISTRY PATH IS ESTABLISHED — and why a behavioural differential is NOT
    /// available for this witness, which is worth stating rather than faking.
    ///
    /// The witness is `:wat::rete::i64::>`, an `OpClass::Alias` row. Its old path and the new
    /// one produce the SAME answer by construction — that is what "alias" means — so no
    /// return value can distinguish them. The proof is therefore STRUCTURAL:
    /// `dispatch_keyword_head`'s alias check returns before this call can ever reach
    /// `dispatch_keyword_head_value`'s `RETE_PREFIX` gate, and that gate is the only route to
    /// `dispatch_rete_op`. `dispatch_keyword_head` has exactly one caller reachable from
    /// `eval_inner` for a keyword-headed form, so for this call `dispatch_rete_op` is provably
    /// never invoked.
    ///
    /// ★ The orchestrator supplied the differential the test cannot: re-pointing this row's
    /// `@alias` at `:wat::i64::<` and re-running flips the answer, which no change to
    /// `RETE_OPS` could produce. That sabotage is recorded in the stone's commit; it is not
    /// shipped as a test because a row whose `@alias` lies is not a state the corpus should
    /// hold.
    ///
    /// ⚠ The stone's FIRST witness was `:wat::rete::i64::+`, chosen by a DESIGN that recorded
    /// it as `Alias` class. It is `Fallback` — a 4-arg row carrying `:undefined` machinery —
    /// so registering it as a 2-arg alias made the 4-arg form unreachable and broke eight
    /// live rete tests. The rider implemented as briefed, measured the collision, and
    /// reported it rather than absorbing it; the witness moved here.
    #[test]
    fn alias_witness_dispatches_through_registry_not_dispatch_rete_op() {
        let (stdlib_sym, _stdlib_macros, _stdlib_types) = stdlib_loaded();
        let env = Environment::new();
        let list_span = crate::rust_caller_span!();
        let head = ":wat::rete::i64::>";
        let args = [
            WatAST::IntLit(2, crate::rust_caller_span!()),
            WatAST::IntLit(1, crate::rust_caller_span!()),
        ];

        // Positive control, read from the registry directly: this stone actually registered
        // the alias — if this is `None`/wrong, the test below proves nothing about THIS
        // stone's field, only that `:wat::i64::+` happens to work on its own.
        let entry = crate::intrinsic::registry()
            .lookup_entry(head)
            .expect("`:wat::rete::i64::>` must be a registered row — the alias witness");
        assert_eq!(
            entry.alias_of,
            Some(":wat::i64::>"),
            "the registered row's alias_of must name the witness's declared target"
        );
        assert!(
            entry.handler.is_none(),
            "STOP-2: the alias witness must carry no handler — a `Some` here means the field \
             is not carrying the dispatch, exactly the finding STOP-2 names"
        );

        let result = dispatch_keyword_head(head, &args, &list_span, &env, stdlib_sym);
        match result {
            Ok(tv) => match tv.value_owned() {
                Value::bool(b) => assert!(
                    b,
                    "(:wat::rete::i64::> 2 1) must evaluate to true via the alias re-dispatch \
                     to `:wat::i64::>` — a `false` here would mean the alias resolved to the \
                     wrong target"
                ),
                other => panic!("unexpected non-bool result: {:?}", other),
            },
            Err(e) => panic!(
                "(:wat::rete::i64::> 2 1) must dispatch cleanly through the alias, not error — \
                 an error here means neither the registry's `alias_of` nor any other door \
                 answered for a name the registry claims to know: {:?}",
                e
            ),
        }
    }

    /// Arc 300 stone C5b — `walk_match_clause`'s `RawClause::Compare` arm is unreachable
    /// with mixed-numeric operands through the checked rete `:wat::form::matches?` path
    /// (`check_comparison` unifies operand types first; see the design stone's
    /// reachability ruling). It has no wat-surface entry point, so it is driven directly
    /// here — this is its only executable regression coverage for the fix.
    fn walk_compare_bool(src: &str) -> bool {
        let clause = crate::parse_one!(src).expect("parse");
        let sym = SymbolTable::new();
        let (passed, _env) = crate::reflect::r#match::walk_match_clause(&clause, &[], &[], Environment::new(), &sym)
            .expect("walk_match_clause");
        passed
    }

    #[test]
    fn c5b_walk_match_clause_i64_f64_boundary_is_exact() {
        // RED at HEAD: coercing 2^53+1 down to f64 rounded it onto 2^53, comparing Equal.
        assert!(
            walk_compare_bool("(< 9007199254740992.0 9007199254740993)"),
            "(< 2^53.0 2^53+1) must be true"
        );
        assert!(
            !walk_compare_bool("(< 9007199254740993 9007199254740992.0)"),
            "(< 2^53+1 2^53.0) must stay false — pins the pre-fix accident"
        );
        assert!(
            walk_compare_bool("(> 9007199254740993 9007199254740992.0)"),
            "(> 2^53+1 2^53.0) must be true"
        );
        assert!(
            !walk_compare_bool("(>= 9007199254740992.0 9007199254740993)"),
            "(>= 2^53.0 2^53+1) must be false"
        );
    }

    /// This caller's NaN policy is `Equal` (preserved byte-for-byte, wart and all — the
    /// same posture `values_compare` has, and deliberately different from
    /// `compare_values`' `None`). Losing this would be STOP-2.
    #[test]
    fn c5b_walk_match_clause_nan_preserved_as_equal() {
        assert!(
            !walk_compare_bool("(< 1 (:wat::f64::/ 0.0 0.0))"),
            "(< 1 NaN) must stay false"
        );
        assert!(
            walk_compare_bool("(<= 1 (:wat::f64::/ 0.0 0.0))"),
            "(<= 1 NaN) must stay true — the separately-flagged, deliberately-preserved wart"
        );
    }

    /// STOP-1 regression guard: an unknown-type pair (not in this table's vocabulary of
    /// i64/u8/f64/String/bool/keyword) stays the silent-false Clara no-error result.
    #[test]
    fn c5b_walk_match_clause_not_numeric_stays_silent_false() {
        assert!(
            !walk_compare_bool(r#"(< "a" 1)"#),
            "an incomparable pair must stay silent-false, not error"
        );
    }

    // ─── Arc 296 G′ — `builtin_enum_variant_names` must see BOTH kinds of builtin ──────

    /// A Rust-registered builtin enum (`types.rs::register_builtin_types`, no `.wat`
    /// declaration) must resolve.
    #[test]
    fn arc296_gprime_builtin_enum_variant_names_rust_registered() {
        let names = builtin_enum_variant_names(":wat::holon::CosineOutcome", "Similarity");
        assert_eq!(names.as_ref(), &vec!["similarity".to_string()]);
    }

    /// A `.wat`-declared builtin enum (`defenum` in the bundled stdlib, e.g.
    /// `:wat::spawn::ServiceEvent` from `wat/spawn.wat`) must ALSO resolve — this is the
    /// exact gap the first draft of `builtin_enum_variant_names` had (backed only by
    /// `TypeEnv::with_builtins()`, which does not run the `.wat` stdlib pass): it panicked
    /// at runtime the first time a `ServiceEvent` was actually constructed. Caught by a
    /// gate-row-3 probe, not by this migration's own build — recorded here so it can't
    /// regress silently.
    #[test]
    fn arc296_gprime_builtin_enum_variant_names_wat_declared() {
        let names = builtin_enum_variant_names(":wat::spawn::ServiceEvent", "Message");
        assert_eq!(
            names.len(),
            2,
            "ServiceEvent::Message [idx <- i64  msg <- T]: {names:?}"
        );
    }
}

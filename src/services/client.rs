//! Per-thread stdio routing — the client half.
//!
//! Arc 170 slice 1f-α. The substrate ships three "thread-aware
//! helpers" that look up the calling thread's per-service channel
//! handles from a thread-local cell and run the mini-TCP block-on-
//! completion lockstep. Slice 1f-α delivers the substrate side;
//! slices 1f-β / γ / δ ship the wat-side service implementations,
//! the runtime orchestrator that populates ThreadIO from
//! `:wat::kernel::spawn-thread`, and the wat-cli boot integration.
//!
//! For slice 1f-α tests, the cell is populated by hand via
//! [`install_thread_io`] / [`uninstall_thread_io`]; later slices
//! call these from the spawn-thread / reap-thread orchestrator.
//!
//! The architecture is the wat-substrate analog of POSIX stdio:
//! every thread reaches three services through per-thread crossbeam
//! channel pairs. Mini-TCP discipline — every send paired with a
//! recv — turns "fire-and-forget" into "fire-and-wait-for-ack" so
//! shutdown cascades cleanly via scope-drop. See
//! [`docs/ZERO-MUTEX.md`] § Tier 3 + § Mini-TCP and arc 170
//! REALIZATIONS pass 15 + pass 16 for the locked architecture.

use std::cell::RefCell;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use crate::services::ServiceMsg;

/// Monotonic thread identifier. Mirrors the wat-side
/// `:wat::kernel::ThreadId` typealias-to-i64 settled in pass 18.
/// Slice 1f-γ will populate these from a monotonic counter in the
/// runtime orchestrator.
pub type ThreadId = i64;

/// Per-thread channel handles used by `:wat::kernel::println` /
/// `eprintln` / `readln`. Populated by `:wat::kernel::spawn-thread`
/// (slice 1f-γ); for slice 1f-α, populated by tests via
/// [`install_thread_io`].
///
/// All channel ends are owned (not Arc'd) — the thread that
/// owns the ThreadIO IS the thread that uses these channels.
///
/// Arc 214 Stone 8.1: ThreadIO does NOT hold a Sender<ServiceMsg<...>> for
/// the stdout/stderr services. Their input_tx is accessed via
/// sym.runtime_services().*_ctrl so the service peers' lifetimes are tied
/// solely to Arc<RuntimeServices> — enabling clean ProcessRuntime::drop
/// ordering.
///
/// Arc 214 Stone 8.2: ThreadIO no longer holds the old stdin_tx / old
/// stdin_reply_rx. The stdin half now mirrors the write pair exactly:
/// `stdin_reply_rx` is a `comms::thread::Receiver<Result<String, String>>`
/// that receives the line (Ok) or error (Err) from the StdInService peer.
pub struct ThreadIO {
    // ── stdout (Arc 214 Stone 8.1 — universe-resident write peer) ──────────
    //
    // NOTE: ThreadIO does NOT hold the service input_tx. The Req send goes
    // via sym.runtime_services().stdout_ctrl in eval_kernel_println. This
    // keeps the service peer's lifetime tied to RuntimeServices (RS-only),
    // not to every ThreadIO clone — so ProcessRuntime::drop can join the
    // peer after dropping RS without deadlocking on a ThreadIO-held sender.
    //
    /// Block here for the StdOutService's ack of "line emitted" routed
    /// back from the peer's reply registry. Populated by Register at
    /// thread registration; each println send+recv is the mini-TCP ack.
    pub stdout_reply_rx: crate::comms::thread::Receiver<Result<(), String>>,
    // ── stderr (Arc 214 Stone 8.1b — universe-resident write peer) ─────────
    //
    // Mirrors the stdout side exactly. The Req send goes via
    // sym.runtime_services().stderr_ctrl in eval_kernel_eprintln.
    //
    /// Block here for the StdErrService's ack of "line emitted" routed
    /// back from the peer's reply registry. Populated by Register at
    /// thread registration; each eprintln send+recv is the mini-TCP ack.
    pub stderr_reply_rx: crate::comms::thread::Receiver<Result<(), String>>,
    /// This thread's monotonic id — embedded in every Req so the service
    /// peers can route the Rep ack back to this thread's reply channels.
    /// Shared across stdout, stderr, and stdin services (one id per thread).
    pub thread_id: ThreadId,
    // ── stdin (Arc 214 Stone 8.2 — universe-resident read peer) ────────────
    //
    // Mirrors the write pair: the Req send goes via
    // sym.runtime_services().stdin_ctrl in eval_kernel_readln.
    //
    /// Block here for the StdInService's reply of "here is the line" routed
    /// back from the peer's reply registry. Populated by Register at
    /// thread registration; each readln send+recv is the mini-TCP round trip.
    /// Ok(line) = a line was read; Err(msg) = handle error (write-side failure);
    /// Recv Err = the loop disconnected (EOF cascade via assertion-failed!).
    pub stdin_reply_rx: crate::comms::thread::Receiver<Result<String, String>>,
}

thread_local! {
    /// Per-thread routing populated by the runtime orchestrator
    /// (or, in tests, by [`install_thread_io`]). `None` means
    /// "stdio services not running on this thread"; the three
    /// substrate primitives surface
    /// [`RuntimeError::ServiceNotRunning`] when they encounter
    /// `None`. Same `thread_local!` precedent as `CALL_STACK` in
    /// `src/runtime.rs`.
    static THREAD_IO: RefCell<Option<ThreadIO>> = const { RefCell::new(None) };
}

/// Install a [`ThreadIO`] into the calling thread's cell. Slice
/// 1f-γ will call this from `:wat::kernel::spawn-thread`'s
/// substrate primitive after registering the spawned thread with
/// each service. Slice 1f-α tests call this directly to populate
/// the per-test ThreadIO.
pub fn install_thread_io(io: ThreadIO) {
    THREAD_IO.with(|cell| {
        *cell.borrow_mut() = Some(io);
    });
}

/// Drain the calling thread's [`ThreadIO`], returning ownership to
/// the caller. Slice 1f-γ calls this when reaping a thread so the
/// channel handles drop in the orchestrator's controlled context;
/// slice 1f-α tests call this between tests to keep the
/// thread-local clean (cargo's test-thread reuse otherwise leaks
/// state across tests).
pub fn uninstall_thread_io() -> Option<ThreadIO> {
    THREAD_IO.with(|cell| cell.borrow_mut().take())
}

/// Internal accessor used by the three eval arms. Borrows the
/// ThreadIO for the duration of `f` and surfaces a clean
/// `ServiceNotRunning` diagnostic when the cell is empty.
pub(crate) fn with_thread_io<F, T>(op: &'static str, f: F) -> Result<T, crate::runtime::RuntimeError>
where
    F: FnOnce(&ThreadIO) -> Result<T, crate::runtime::RuntimeError>,
{
    use crate::runtime::{RuntimeError, RuntimeErrorKind};
    use crate::span::Span;
    THREAD_IO.with(|cell| match &*cell.borrow() {
        Some(io) => f(io),
        None => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ServiceNotRunning {
            op: op.into()
        } }),
    })
}

/// Three-Sender carrier per BRIEF Q5 + Q-carrier.
///
/// Arc 214 Stone 5.1 — ControlTx senders are comms::thread::Sender<Value>
/// (cascade-aware, depth-1) instead of bare crossbeam Senders.
/// Arc 214 Stone 8.1 — stdout_ctrl is now a Sender<ServiceMsg<()>> (the
/// universe-resident write peer's input channel) instead of a wat ControlTx.
/// Arc 214 Stone 8.1b — stderr_ctrl follows: now a Sender<ServiceMsg<()>>
/// for the universe-resident stderr write peer.
/// Arc 214 Stone 8.2 — stdin_ctrl follows: now a Sender<ServiceMsg<String>>
/// for the universe-resident stdin read peer.
#[derive(Clone)]
pub struct RuntimeServices {
    /// Arc 214 Stone 8.2 — the universe-resident StdInService read peer's
    /// input channel. Register/Deregister/Req flow through it.
    /// NOT cloned into ThreadIO — eval_kernel_readln accesses this via
    /// sym.runtime_services() so the peer's lifetime is tied solely to RS.
    pub stdin_ctrl: crate::comms::thread::Sender<ServiceMsg<String>>,
    /// Arc 214 Stone 8.1 — the universe-resident StdOutService write peer's
    /// input channel. Register/Deregister/Req flow through it.
    /// NOT cloned into ThreadIO — eval_kernel_println accesses this via
    /// sym.runtime_services() so the peer's lifetime is tied solely to RS.
    pub stdout_ctrl: crate::comms::thread::Sender<ServiceMsg<()>>,
    /// Arc 214 Stone 8.1b — the universe-resident StdErrService write peer's
    /// input channel. Register/Deregister/Req flow through it.
    /// NOT cloned into ThreadIO — eval_kernel_eprintln accesses this via
    /// sym.runtime_services() so the peer's lifetime is tied solely to RS.
    pub stderr_ctrl: crate::comms::thread::Sender<ServiceMsg<()>>,
}

impl std::fmt::Debug for RuntimeServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeServices")
            .field("stdin_ctrl", &"<Sender<ServiceMsg<String>>> (Stone 8.2 peer; accessed via sym.runtime_services())")
            .field("stdout_ctrl", &"<Sender<ServiceMsg<()>>> (Stone 8.1 peer; accessed via sym.runtime_services())")
            .field("stderr_ctrl", &"<Sender<ServiceMsg<()>>> (Stone 8.1b peer; accessed via sym.runtime_services())")
            .finish()
    }
}

/// Monotonic thread-id allocator. Starts at `1` so `0` is reserved as
/// a "no thread" sentinel for future use. Each `invoke_user_main` is
/// process-scoped; the counter survives across invocations, which is
/// fine — ids only need to be unique within a single orchestrator's
/// routing tables, and the wat-side services are torn down between
/// invocations.
static NEXT_THREAD_ID: AtomicI64 = AtomicI64::new(1);

/// Allocate a fresh monotonic [`ThreadId`]. Atomic, lock-free.
pub fn next_thread_id() -> ThreadId {
    NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst)
}

/// Allocate per-thread service channels; register with all three
/// universe-resident service peers (stdin, stdout, stderr); return the
/// populated [`ThreadIO`].
///
/// Arc 214 Stone 8.2 — stdin now mirrors the write pair: send
/// Register(tid, reply_tx) on the stdin peer's input channel; the peer
/// inserts the reply_tx into its HashMap keyed by tid. No bridge thread,
/// no wat-side channels.
///
/// On send failure (service shut down) returns
/// [`RuntimeError::ChannelDisconnected`]. Caller is responsible for
/// `install_thread_io` after this returns successfully.
pub fn register_thread_with_services(
    thread_id: ThreadId,
    services: &RuntimeServices,
) -> Result<ThreadIO, crate::runtime::RuntimeError> {
    use crate::runtime::{RuntimeError, RuntimeErrorKind};
    use crate::span::Span;
    const OP_ADD: &str = "register_thread_with_services";

    // ─── stdin (Arc 214 Stone 8.2 — universe-resident read peer) ──────
    //
    // Mirrors the write pair: allocate a per-thread reply pair,
    // send Register(tid, reply_tx) on the stdin peer's input channel.
    let (stdin_reply_tx, stdin_reply_rx) = crate::comms::thread::pair::<Result<String, String>>();
    services
        .stdin_ctrl
        .send(ServiceMsg::Register(thread_id, stdin_reply_tx))
        .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
            op: OP_ADD.into()
        } })?;

    // ─── stdout (Arc 214 Stone 8.1 — universe-resident write peer) ────
    //
    // No bridge thread, no wat-side channels. ThreadIO holds a per-thread
    // reply Receiver<Result<(),String>> for the ack back from the service's
    // reply registry.
    //
    // Registration: send Register(tid, reply_tx) on the service input so
    // the peer loop inserts the reply_tx into its HashMap keyed by tid.
    let (stdout_reply_tx, stdout_reply_rx) = crate::comms::thread::pair::<Result<(), String>>();
    services
        .stdout_ctrl
        .send(ServiceMsg::Register(thread_id, stdout_reply_tx))
        .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
            op: OP_ADD.into()
        } })?;

    // ─── stderr (Arc 214 Stone 8.1b — universe-resident write peer) ───
    //
    // Mirrors the stdout side exactly. No bridge thread.
    let (stderr_reply_tx, stderr_reply_rx) = crate::comms::thread::pair::<Result<(), String>>();
    services
        .stderr_ctrl
        .send(ServiceMsg::Register(thread_id, stderr_reply_tx))
        .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
            op: OP_ADD.into()
        } })?;

    Ok(ThreadIO {
        // NOTE: the service input_tx senders are NOT stored in ThreadIO
        // (Arc 214 Stone 8.1/8.1b/8.2 fix). The Req sends go via
        // sym.runtime_services().{stdin,stdout,stderr}_ctrl in
        // eval_kernel_{readln,println,eprintln} so the service peers'
        // lifetimes are tied solely to Arc<RuntimeServices>, not to every
        // ThreadIO.
        stdout_reply_rx,
        stderr_reply_rx,
        thread_id,
        stdin_reply_rx,
    })
}

/// Send Deregister messages to the universe-resident stdin, stdout, and
/// stderr peers for this thread_id.
/// Silent-fail on each send (services may be shutting down via scope-drop;
/// a failed send is "the service is already gone," the cleanup state we want).
pub fn deregister_thread_from_services(thread_id: ThreadId, services: &RuntimeServices) {
    // Arc 214 Stone 8.2 — stdin uses Deregister on the Rust-internal enum (mirrors write pair).
    let _ = services.stdin_ctrl.send(ServiceMsg::Deregister(thread_id));

    // Arc 214 Stone 8.1 — stdout uses Deregister on the Rust-internal enum.
    let _ = services.stdout_ctrl.send(ServiceMsg::Deregister(thread_id));

    // Arc 214 Stone 8.1b — stderr mirrors stdout.
    let _ = services.stderr_ctrl.send(ServiceMsg::Deregister(thread_id));
}

// ─── Slice 1f-γ — ambient stdio handles (orchestrator-facing) ──────────
//
// The orchestrator needs IOReader / IOWriter values for the three
// service spawns. Production wat-cli runs invoke_user_main inside a
// forked child whose fd 0/1/2 already point at the parent's stdio
// (or substituted pipes); the orchestrator wraps those fds via
// PipeReader / PipeWriter.
//
// Tests (the slice 1f-γ orchestrator-test rows) need to substitute
// in-memory or test-controlled handles so cargo's worker threads
// don't fight the host terminal. The chosen carrier is a per-thread
// "ambient stdio" cell: tests `install_ambient_stdio` before invoking
// the orchestrator-test entry point; production reaches the
// fall-through path which constructs PipeReader / PipeWriter around
// raw fd 0/1/2 on each invocation.
//
// Per-thread (not global) so cargo's parallel test threads don't
// race each other when each is running its own orchestrator
// instance. The orchestrator runs on the calling thread, so its
// initial read of the cell sees what THIS thread installed.

/// Per-thread ambient stdio carrier. Set by tests via
/// [`install_ambient_stdio`]; consumed by
/// [`crate::freeze::invoke_user_main`] when it spawns the three
/// services. `None` (the default) means "use real fd 0/1/2 via
/// PipeReader/PipeWriter."
pub struct AmbientStdio {
    pub stdin: Arc<dyn crate::io::WatReader>,
    pub stdout: Arc<dyn crate::io::WatWriter>,
    pub stderr: Arc<dyn crate::io::WatWriter>,
}

thread_local! {
    static AMBIENT_STDIO: RefCell<Option<AmbientStdio>> = const { RefCell::new(None) };
}

/// Install the calling thread's ambient stdio. Test-only entry point
/// — production wat-cli does NOT call this; the orchestrator falls
/// through to real fd 0/1/2 PipeReader/PipeWriter when the ambient
/// is None. Slice 1f-γ orchestrator tests use this to inject pipe
/// handles whose other ends the test thread controls.
pub fn install_ambient_stdio(stdio: AmbientStdio) {
    AMBIENT_STDIO.with(|cell| {
        *cell.borrow_mut() = Some(stdio);
    });
}

/// Drain the calling thread's ambient stdio. Tests call this between
/// rows to keep cargo's worker-thread reuse from leaking handles
/// across rows.
pub fn uninstall_ambient_stdio() -> Option<AmbientStdio> {
    AMBIENT_STDIO.with(|cell| cell.borrow_mut().take())
}

/// Take the calling thread's ambient stdio (consuming it) or return
/// `None` if no test has installed one. Called by the orchestrator
/// once per invoke_user_main; the orchestrator falls through to real
/// fd 0/1/2 wrappers when this returns `None`.
pub fn take_ambient_stdio() -> Option<AmbientStdio> {
    uninstall_ambient_stdio()
}

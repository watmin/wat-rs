//! Per-thread stdio routing — the client half.
//!
//! Arc 170 slice 1f-α. The substrate ships three "thread-aware
//! helpers" that look up the calling thread's per-service channel
//! handles from a thread-local cell and run the mini-TCP block-on-
//! completion lockstep. Slices 1f-β/γ/δ shipped the wat-side service
//! implementations, the runtime orchestrator that populates ThreadIO
//! from `:wat::kernel::spawn-thread`, and the wat-cli boot integration.
//!
//! The spawn-thread/reap orchestrator (runtime.rs) calls these in
//! production; tests call them directly.
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

use crate::services::{ServiceMsg, peer::ServiceInputSender};

/// Monotonic thread identifier. Mirrors the wat-side
/// `:wat::kernel::ThreadId` typealias-to-i64 settled in pass 18.
/// Allocated by `next_thread_id`; the spawn orchestrator assigns one per thread.
pub type ThreadId = i64;

/// Receiver of write-acks from a write-service peer (stdout/stderr).
pub type WriteAckRx = crate::comms::thread::Receiver<Result<(), String>>;
/// Receiver of read-replies from the stdin peer (the line, or the error).
pub type ReadReplyRx = crate::comms::thread::Receiver<Result<String, String>>;

/// Per-thread channel handles used by `:wat::kernel::println` /
/// `eprintln` / `readln`. Populated by the spawn orchestrator via
/// `register_thread_with_services`; tests populate via `install_thread_io`.
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
    pub stdout_reply_rx: WriteAckRx,
    // ── stderr (Arc 214 Stone 8.1b — universe-resident write peer) ─────────
    //
    // Mirrors the stdout side exactly. The Req send goes via
    // sym.runtime_services().stderr_ctrl in eval_kernel_eprintln.
    //
    /// Block here for the StdErrService's ack of "line emitted" routed
    /// back from the peer's reply registry. Populated by Register at
    /// thread registration; each eprintln send+recv is the mini-TCP ack.
    pub stderr_reply_rx: WriteAckRx,
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
    pub stdin_reply_rx: ReadReplyRx,
}

// rune:perspicere(intentional-structure) — RefCell<Option<T>> is the
// canonical thread_local interior-mutability idiom; the structure shows the
// reader exactly how borrow_mut/take interact.
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

/// Install a [`ThreadIO`] into the calling thread's cell. The spawn
/// orchestrator (runtime.rs) calls this after registering the spawned
/// thread with each service. Tests call this directly to populate
/// the per-test ThreadIO.
pub fn install_thread_io(io: ThreadIO) {
    THREAD_IO.with(|cell| {
        *cell.borrow_mut() = Some(io);
    });
}

/// Drain the calling thread's [`ThreadIO`], returning ownership to
/// the caller. The reap orchestrator (runtime.rs) calls this when
/// reaping a thread so the channel handles drop in the orchestrator's
/// controlled context; tests call this between tests to keep the
/// thread-local clean (cargo's test-thread reuse otherwise leaks
/// state across tests).
pub fn uninstall_thread_io() -> Option<ThreadIO> {
    THREAD_IO.with(|cell| cell.borrow_mut().take())
}

/// Internal accessor used by the three eval arms. Borrows the
/// ThreadIO for the duration of `f` and surfaces a clean
/// `ServiceNotRunning` diagnostic when the cell is empty.
pub(crate) fn with_thread_io<F, T>(op: &'static str, span: &crate::span::Span, f: F) -> Result<T, crate::runtime::RuntimeError>
where
    F: FnOnce(&ThreadIO) -> Result<T, crate::runtime::RuntimeError>,
{
    use crate::runtime::{RuntimeError, RuntimeErrorKind};
    THREAD_IO.with(|cell| match &*cell.borrow() {
        Some(io) => f(io),
        None => Err(RuntimeError { span: span.clone(), kind: RuntimeErrorKind::ServiceNotRunning {
            op: op.into()
        } }),
    })
}

/// Holds the three universe-resident service peer input channels; accessed via
/// `sym.runtime_services()` rather than ThreadIO so the peers' lifetimes tie
/// to `Arc<RuntimeServices>`, not to every per-thread cell.
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
    pub stdin_ctrl: ServiceInputSender<String>,
    /// Arc 214 Stone 8.1 — the universe-resident StdOutService write peer's
    /// input channel. Register/Deregister/Req flow through it.
    /// NOT cloned into ThreadIO — eval_kernel_println accesses this via
    /// sym.runtime_services() so the peer's lifetime is tied solely to RS.
    pub stdout_ctrl: ServiceInputSender<()>,
    /// Arc 214 Stone 8.1b — the universe-resident StdErrService write peer's
    /// input channel. Register/Deregister/Req flow through it.
    /// NOT cloned into ThreadIO — eval_kernel_eprintln accesses this via
    /// sym.runtime_services() so the peer's lifetime is tied solely to RS.
    pub stderr_ctrl: ServiceInputSender<()>,
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

/// Monotonic thread-id allocator. Starts at `1`; 0 is never allocated
/// (the counter starts at 1); no current consumer reads 0 as a sentinel.
/// Each `invoke_user_main` is process-scoped; the counter survives across
/// invocations, which is fine — ids only need to be unique within a single
/// orchestrator's routing tables, and the wat-side services are torn down
/// between invocations.
// rune:sequi(performance-counter) — uniqueness-only id allocator; no domain
// state crosses threads through the counter (the allocated tid travels
// VISIBLY in Register/Req messages); threading an AtomicI64 through every
// spawn-site signature trades real legibility for monadic purity. Documented
// bound: ZERO-MUTEX.md § honest caveats (hot atomic counters).
static NEXT_THREAD_ID: AtomicI64 = AtomicI64::new(1);

/// Allocate a fresh monotonic [`ThreadId`]. Atomic, lock-free.
pub fn next_thread_id() -> ThreadId {
    // Relaxed: uniqueness-only; no happens-before ordering required.
    NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed)
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
    caller_span: &crate::span::Span,
) -> Result<ThreadIO, crate::runtime::RuntimeError> {
    use crate::runtime::{RuntimeError, RuntimeErrorKind};
    const OP_ADD: &str = "register_thread_with_services";

    // ─── stdin (Arc 214 Stone 8.2 — universe-resident read peer) ──────
    //
    // Mirrors the write pair: allocate a per-thread reply pair,
    // send Register(tid, reply_tx) on the stdin peer's input channel.
    let (stdin_reply_tx, stdin_reply_rx): (_, ReadReplyRx) = crate::comms::thread::pair();
    services
        .stdin_ctrl
        .send(ServiceMsg::Register(thread_id, stdin_reply_tx))
        .map_err(|_| RuntimeError { span: caller_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
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
    let (stdout_reply_tx, stdout_reply_rx): (_, WriteAckRx) = crate::comms::thread::pair();
    services
        .stdout_ctrl
        .send(ServiceMsg::Register(thread_id, stdout_reply_tx))
        .map_err(|_| RuntimeError { span: caller_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
            op: OP_ADD.into()
        } })?;

    // ─── stderr (Arc 214 Stone 8.1b — universe-resident write peer) ───
    //
    // Mirrors the stdout side exactly. No bridge thread.
    let (stderr_reply_tx, stderr_reply_rx): (_, WriteAckRx) = crate::comms::thread::pair();
    services
        .stderr_ctrl
        .send(ServiceMsg::Register(thread_id, stderr_reply_tx))
        .map_err(|_| RuntimeError { span: caller_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
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

// rune:perspicere(intentional-structure) — RefCell<Option<T>> is the
// canonical thread_local interior-mutability idiom; the structure shows the
// reader exactly how borrow_mut/take interact.
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

/// Take the calling thread's ambient stdio (consuming it) or return
/// `None` if no test has installed one. Called by the orchestrator
/// once per invoke_user_main; the orchestrator falls through to real
/// fd 0/1/2 wrappers when this returns `None`. Tests call this between
/// rows to keep cargo's worker-thread reuse from leaking handles
/// across rows.
pub fn take_ambient_stdio() -> Option<AmbientStdio> {
    AMBIENT_STDIO.with(|cell| cell.borrow_mut().take())
}

// ─── Arc 259 — The Forced Hand: ambient program environment ──────────────────
//
// Homed alongside AMBIENT_STDIO (same pattern, same module) because both are
// per-thread runtime context carriers — the env is the program's identity for
// the duration of a peer's execution, exactly as ambient-stdio is its I/O
// identity. A `src/program/` home would scatter the context idiom; keeping
// both context threads here makes the pattern self-documenting.
//
// Unlike AMBIENT_STDIO (consumed once via `take`), the env is READ-MANY by any
// depth — `current_program_env` clones rather than takes. The RAII guard
// (save/restore) makes nested install and test isolation clean by construction.

thread_local! {
    // rune:perspicere(intentional-structure) — RefCell<Option<Value>> is the
    // canonical thread_local interior-mutability idiom (same as AMBIENT_STDIO).
    // The env lives here for the duration of a peer's execution and is read by
    // `(:wat::program::env)` from any call depth on this thread.
    static PROGRAM_ENV: RefCell<Option<crate::runtime::Value>> = const { RefCell::new(None) };
}

/// RAII guard returned by [`install_program_env`]. On drop, restores the
/// previous env (or `None` if there was none). Supports nested installs
/// and test isolation without any explicit teardown call.
pub struct EnvGuard {
    prior: Option<crate::runtime::Value>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        PROGRAM_ENV.with(|cell| {
            *cell.borrow_mut() = self.prior.take();
        });
    }
}

/// Install `env` as the calling thread's ambient program environment and
/// return a RAII guard. On guard drop the prior value is restored.
///
/// The env is a `:wat::program::Env` record `Value` (or a subtype — any
/// `Value::wat__Record` whose class descends from `wat::program::Env`).
/// Called by the post-bootstrap / pre-`:user::main` seam
/// (`invoke_user_main_orchestrated`) and directly by tests.
pub fn install_program_env(env: crate::runtime::Value) -> EnvGuard {
    PROGRAM_ENV.with(|cell| {
        let prior = cell.borrow_mut().replace(env);
        EnvGuard { prior }
    })
}

/// Clone and return the calling thread's ambient program environment,
/// or `None` if none has been installed. Read-many — does NOT consume
/// the installed value (unlike `take_ambient_stdio`). Called by the
/// `(:wat::program::env)` dispatch arm.
pub fn current_program_env() -> Option<crate::runtime::Value> {
    PROGRAM_ENV.with(|cell| cell.borrow().clone())
}

// ─── SELF_PEER thread-local (Arc 209 C0b.3a-0) ───────────────────────────────
//
// The process child's owner-link as a `SocketPeer'` value. Installed at the
// child-only seam `run_forms_as_server_child` (process/verbs.rs); never
// installed in root's `invoke_user_main_orchestrated`. Root callers get a
// clean error from `(:wat::program::self-peer …)`. Mirrors PROGRAM_ENV exactly:
// RefCell<Option<Value>>, RAII guard, read-many clone.

thread_local! {
    // rune:perspicere(intentional-structure) — mirrors PROGRAM_ENV above.
    // Lives for the duration of the spawned process child's `:user::main` run.
    // Read by `(:wat::program::self-peer :S :R)` from any call depth on this thread.
    static SELF_PEER: RefCell<Option<crate::runtime::Value>> = const { RefCell::new(None) };
}

/// RAII guard returned by [`install_self_peer`]. On drop, clears the
/// SELF_PEER slot. Mirrors [`EnvGuard`] (single-install, no prior to restore
/// since only the child-only seam installs this — there is no nesting).
pub struct SelfPeerGuard {
    _private: (),
}

impl Drop for SelfPeerGuard {
    fn drop(&mut self) {
        SELF_PEER.with(|cell| *cell.borrow_mut() = None);
    }
}

/// Install `peer` as the calling thread's self-peer (owner-link) and return
/// a RAII guard. On guard drop the slot is cleared.
///
/// Called only at the child-only seam `run_forms_as_server_child`
/// (`process/verbs.rs`), before `:user::main` runs. The guard is held for
/// the child's entire lifetime so the slot is set for any call depth.
pub fn install_self_peer(peer: crate::runtime::Value) -> SelfPeerGuard {
    SELF_PEER.with(|cell| *cell.borrow_mut() = Some(peer));
    SelfPeerGuard { _private: () }
}

/// Clone and return the calling thread's self-peer, or `None` if none has
/// been installed (i.e. this is the root process, not a spawned child).
/// Read-many — does NOT consume the installed value. Called by the
/// `(:wat::program::self-peer)` dispatch arm.
pub fn current_self_peer() -> Option<crate::runtime::Value> {
    SELF_PEER.with(|cell| cell.borrow().clone())
}

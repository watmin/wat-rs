//! Per-thread stdio routing — `:wat::kernel::println` /
//! `eprintln` / `readln` substrate primitives.
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

use crate::typed_channel::{Receiver, Sender};

use crate::ast::WatAST;
use crate::runtime::{eval, EnumValue, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;
use crate::typed_channel::{
    receiver_from_comms, sender_from_comms, ReceiverInner, SenderInner,
};

/// Monotonic thread identifier. Mirrors the wat-side
/// `:wat::kernel::ThreadId` typealias-to-i64 settled in pass 18.
/// Slice 1f-γ will populate these from a monotonic counter in the
/// runtime orchestrator.
pub type ThreadId = i64;

/// Arc 214 Stone 8.1 — Rust-internal input enum for the universe-resident
/// StdOutService peer. NEVER a wat message; the Rust service loop owns it.
///
/// - `Req(value)` carries a `Value::Struct` of `:wat::kernel::services::
///   StdOutService::Req {thread-id, line}`. The loop applies the wat handle
///   fn and routes the Rep ack back via the reply registry.
/// - `Register(tid, reply_tx)` inserts a per-thread reply sender so the loop
///   can route the ack back to the calling thread's `stdout_reply_rx`.
/// - `Deregister(tid)` removes the reply sender (thread reap).
#[derive(Debug)]
pub enum StdOutInput {
    Req(Value),
    Register(ThreadId, crate::comms::thread::Sender<Result<(), String>>),
    Deregister(ThreadId),
}

/// Arc 214 Stone 8.1 — handle returned from `spawn_stdio_service_peer`.
/// Caller (freeze.rs) sends Req/Register/Deregister messages on `input_tx`
/// and can join the service thread for clean teardown.
pub struct StdioServicePeer {
    pub input_tx: crate::comms::thread::Sender<StdOutInput>,
    pub join: std::thread::JoinHandle<()>,
}

/// Arc 214 Stone 8.1 — spawn the universe-resident StdOutService loop.
///
/// Looks up `:wat::kernel::services::StdOutService/handle` in `sym`, clones
/// sym for the service thread, and spawns a Rust loop that:
///   1. Receives `StdOutInput` messages on `input_rx`.
///   2. For `Req(v)`: applies the wat handle fn with `[v, writer.clone()]`,
///      extracts the `thread-id` from the Rep, routes a `()` ack to the
///      matching reply sender in the registry.
///   3. For `Register(tid, reply_tx)`: inserts into the reply registry.
///   4. For `Deregister(tid)`: removes from the registry.
///   5. Exits when `input_rx` disconnects (all `input_tx` senders dropped).
///
/// STOP-2: if applying the 2-arg handle or building the Req value fails at
/// runtime the loop logs and continues (ack is skipped for that Req — the
/// caller's println times out). Should not happen in production.
pub fn spawn_stdio_service_peer(
    handle_fn: Arc<crate::runtime::Function>,
    writer: Value,
    sym: crate::runtime::SymbolTable,
) -> StdioServicePeer {
    let (input_tx, input_rx) = crate::comms::thread::pair::<StdOutInput>();
    let join = std::thread::Builder::new()
        .name("wat-stdout-service-peer".to_string())
        .spawn(move || {
            let mut reply_registry: std::collections::HashMap<
                ThreadId,
                crate::comms::thread::Sender<Result<(), String>>,
            > = std::collections::HashMap::new();
            loop {
                let msg = match input_rx.recv() {
                    Ok(m) => m,
                    Err(_) => break, // all input_tx senders dropped → shutdown
                };
                match msg {
                    StdOutInput::Register(tid, reply_tx) => {
                        reply_registry.insert(tid, reply_tx);
                    }
                    StdOutInput::Deregister(tid) => {
                        reply_registry.remove(&tid);
                    }
                    StdOutInput::Req(req_value) => {
                        // Extract thread-id from the Req struct (field 0).
                        let thread_id: ThreadId = match &req_value {
                            Value::Struct(sv) if sv.fields.len() >= 1 => {
                                match &sv.fields[0] {
                                    Value::i64(n) => *n,
                                    _ => {
                                        eprintln!(
                                            "[wat substrate] stdout-peer: Req field[0] is not i64"
                                        );
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                eprintln!(
                                    "[wat substrate] stdout-peer: Req is not a Struct"
                                );
                                continue;
                            }
                        };
                        // Apply the wat handle fn: handle(req, writer).
                        let result = crate::runtime::apply_function(
                            Arc::clone(&handle_fn),
                            vec![req_value, writer.clone()],
                            &sym,
                            crate::rust_caller_span!(),
                        );
                        match result {
                            Ok(_rep) => {
                                // Route ack to the requesting thread.
                                if let Some(reply_tx) = reply_registry.get(&thread_id) {
                                    let _ = reply_tx.send(Ok(()));
                                }
                            }
                            Err(e) => {
                                // ZERO-MUTEX mini-TCP: EVERY Req gets a reply —
                                // a caller blocked in println must NEVER hang on a
                                // failed write. Route the error; println surfaces it
                                // as a RuntimeError (the ack means write-COMPLETED;
                                // acking a failure would be a lie, so the reply
                                // carries Result).
                                eprintln!(
                                    "[wat substrate] stdout-peer: handle failed: {}",
                                    e
                                );
                                if let Some(reply_tx) = reply_registry.get(&thread_id) {
                                    let _ = reply_tx.send(Err(format!("{}", e)));
                                }
                            }
                        }
                    }
                }
            }
        })
        .expect("std::thread::spawn for stdout service peer");
    StdioServicePeer { input_tx, join }
}

/// Arc 214 Stone 8.1 — kept for stderr/stdin architecture (old path).
/// Sent on the stdout tx; consumed by the wat-side StdOutService.
/// RETIRED for stdout in Stone 8.1 but kept because stdin/stderr still use
/// the old architecture. The Write / Add / Remove variants for stderr are
/// `StdErrServiceEvent` below; this type is now DEAD for stdout.
#[derive(Debug, Clone)]
pub enum StdOutServiceEvent {
    /// Caller's println rendered an EDN line; service writes
    /// it to fd 1 and acks.
    Write { line: String },
    /// Runtime registers a thread; service stores
    /// `(thread_id → (data_rx, ack_tx))` in its routing table.
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdOutServiceEvent>,
        ack_tx: Sender<()>,
    },
    /// Runtime reaps a thread; service drops the routing entry.
    Remove { thread_id: ThreadId },
}

/// Mirror of [`StdOutServiceEvent`] for fd 2.
#[derive(Debug, Clone)]
pub enum StdErrServiceEvent {
    Write { line: String },
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdErrServiceEvent>,
        ack_tx: Sender<()>,
    },
    Remove { thread_id: ThreadId },
}

/// Stdin's data variant is unit (the "give me next form"
/// request); the raw line comes back via the reply-tx.
///
/// Arc 170 slice 1f-ι — the reply channel carries the RAW EDN line
/// (a `String`), not a pre-parsed `Arc<HolonAST>`. The parse + coerce
/// to the caller's requested `T` happens substrate-side in
/// `eval_kernel_readln` (via `edn_to_typed_value`); the wat-side
/// StdInService no longer pre-parses with `:wat::edn::read`. This
/// locks the EDN-only stdio contract: `(:wat::kernel::readln -> :T)`
/// returns a native `T` for any wat type with EDN encoding/decoding.
#[derive(Debug, Clone)]
pub enum StdInServiceEvent {
    /// Caller's readln signals "next line please."
    Read,
    /// Runtime registers a thread; service stores
    /// `(thread_id → (data_rx, reply_tx))` in its routing table.
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdInServiceEvent>,
        reply_tx: Sender<String>,
    },
    Remove { thread_id: ThreadId },
}

/// Per-thread channel handles used by `:wat::kernel::println` /
/// `eprintln` / `readln`. Populated by `:wat::kernel::spawn-thread`
/// (slice 1f-γ); for slice 1f-α, populated by tests via
/// [`install_thread_io`].
///
/// All channel ends are owned (not Arc'd) — the thread that
/// owns the ThreadIO IS the thread that uses these channels.
/// crossbeam's Sender / Receiver are themselves `Send`; the
/// thread-local cell ensures only one thread accesses any given
/// ThreadIO instance.
///
/// Arc 214 Stone 8.1: ThreadIO does NOT hold a Sender<StdOutInput> for
/// the stdout service. The service input_tx is accessed in
/// eval_kernel_println via sym.runtime_services().stdout_ctrl so the
/// service peer's lifetime is tied solely to Arc<RuntimeServices> — not
/// to every ThreadIO clone — enabling clean ProcessRuntime::drop ordering.
pub struct ThreadIO {
    // ── stdout (Arc 214 Stone 8.1 — universe-resident peer) ────────────────
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
    /// This thread's monotonic id — embedded in every Req so the service
    /// peer can route the Rep ack back to this thread's reply channel.
    pub stdout_thread_id: ThreadId,
    // ── stderr (unchanged — old bridge architecture) ────────────────────────
    /// Send an Event (Write / Add / Remove) to the StdErrService.
    pub stderr_tx: Sender<StdErrServiceEvent>,
    /// Block here for the StdErrService's ack of "line emitted."
    pub stderr_ack_rx: Receiver<()>,
    // ── stdin (unchanged — old bridge architecture) ─────────────────────────
    /// Send an Event (Read / Add / Remove) to the StdInService.
    pub stdin_tx: Sender<StdInServiceEvent>,
    /// Receive the raw EDN line representing the next stdin form
    /// (arc 170 slice 1f-ι — pre-1f-ι this was `Arc<HolonAST>`).
    pub stdin_reply_rx: Receiver<String>,
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
fn with_thread_io<F, T>(op: &'static str, f: F) -> Result<T, RuntimeError>
where
    F: FnOnce(&ThreadIO) -> Result<T, RuntimeError>,
{
    THREAD_IO.with(|cell| match &*cell.borrow() {
        Some(io) => f(io),
        None => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ServiceNotRunning {
            op: op.into()
        } }),
    })
}

/// Shared one-arg helper — mirrors `edn_shim::require_one_arg`'s
/// shape. Inlined here to avoid leaking that helper across modules.
fn require_one_arg(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        } });
    }
    eval(&args[0], env, sym).map(|tv| tv.value_owned())
}

/// `(:wat::kernel::println v)` → `:wat::core::nil`. Serialize `v`
/// to compact EDN via `value_to_edn_with`; build a
/// `StdOutService::Req {thread-id, line}` struct Value; send it on
/// the universe-resident StdOutService peer's input channel; block on
/// `stdout_reply_rx` for the ack; return `Value::Unit`.
///
/// Arc 214 Stone 8.1 — replaced the old StdOutServiceEvent::Write +
/// bridge path with direct Req → service peer mini-TCP.
///
/// The Req send goes via `sym.runtime_services().stdout_ctrl` rather than
/// a ThreadIO-held sender. This keeps the service peer's lifetime tied
/// purely to RuntimeServices (the RS Arc), not to every ThreadIO — so
/// ProcessRuntime::drop can join the peer after dropping RS without
/// deadlocking on a ThreadIO-held sender that outlives the drop sequence.
pub fn eval_kernel_println(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::println";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write(&edn);
    // Access the service input_tx via sym.runtime_services() — not via ThreadIO —
    // so no clone of the sender lives in the ThreadIO struct.
    let services = sym.runtime_services().ok_or_else(|| RuntimeError {
        span: Span::unknown(),
        kind: RuntimeErrorKind::ServiceNotRunning { op: OP.into() },
    })?;
    with_thread_io(OP, |io| {
        // Build StdOutService::Req {thread-id, line} as a Value::Struct.
        let req = Value::Struct(Arc::new(crate::runtime::StructValue {
            type_name: ":wat::kernel::services::StdOutService::Req".into(),
            fields: vec![
                Value::i64(io.stdout_thread_id),
                Value::String(Arc::new(line.clone())),
            ],
        }));
        services
            .stdout_ctrl
            .send(StdOutInput::Req(req))
            .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        match io.stdout_reply_rx.recv() {
            Ok(Ok(())) => Ok(Value::Unit),
            // The service processed the Req but the write FAILED — surface it
            // (uniform with src/io.rs's IOWriter write-failure convention).
            Ok(Err(msg)) => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("stdout write failed: {}", msg),
            } }),
            Err(_) => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } }),
        }
    })
}

/// `(:wat::kernel::eprintln v)` → `:wat::core::nil`. Same shape as
/// [`eval_kernel_println`] but routed through the StdErrService
/// channel pair.
pub fn eval_kernel_eprintln(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::eprintln";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write(&edn);
    with_thread_io(OP, |io| {
        io.stderr_tx
            .send(StdErrServiceEvent::Write { line })
            .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        io.stderr_ack_rx
            .recv()
            .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        Ok(Value::Unit)
    })
}

/// `(:wat::kernel::readln -> :T)` → `:T`. Arc 170 slice 1f-ι.
///
/// Polymorphic in `T` via the call-site `-> :T` annotation (mirror
/// pattern of `:wat::core::Option/expect` / `:wat::core::Result/expect`
/// / `:wat::core::if`). Steps:
///   1. Read the call-site's `-> :T` annotation (head-position
///      arrow + type keyword; args = `[Symbol("->"), Keyword(":T")]`).
///   2. Signal the StdInService via the stdin req-tx.
///   3. Block on stdin reply-rx for the next RAW line.
///   4. Parse the line via `wat_edn::parse_owned`.
///   5. Coerce the parsed EDN to a wat `Value` of the declared `T`
///      via [`crate::edn_shim::edn_to_typed_value`]. On mismatch,
///      surfaces [`RuntimeError::EdnCoerceMismatch`].
///
/// The EDN-only stdio contract (locked 2026-05-10):
/// ```text
/// server: (:wat::kernel::println 42)                    → emits  42 (EDN i64)
/// reader: (:wat::kernel::readln -> :wat::core::i64)     → returns 42 (i64)
///
/// server: (:wat::kernel::println "foo")                 → emits  "foo" (EDN String, quoted)
/// reader: (:wat::kernel::readln -> :wat::core::String)  → returns "foo" (String)
/// ```
///
/// `T` is any wat type with EDN encoding/decoding: primitives, tuples,
/// Vector, Option, Result, user structs/enums, and
/// `:wat::holon::HolonAST` (when the caller explicitly wants raw AST
/// form). See the coercion table in
/// [`crate::edn_shim::edn_to_typed_value`].
pub fn eval_kernel_readln(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::readln";
    // Annotation shape: `(readln -> :T)` → args = [Symbol("->"), Keyword(":T")].
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "expected (:wat::kernel::readln -> :T) — 2 args (arrow + type keyword); got {}",
                args.len()
            )
        } });
    }
    match &args[0] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        other => {
            return Err(RuntimeError { span: other.span().clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "expected `->` as the first argument; (:wat::kernel::readln -> :T); got {}",
                    other.variant_name()
                )
            } });
        }
    }
    let target_ty = match &args[1] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e)
                } });
            }
        },
        other => {
            return Err(RuntimeError { span: other.span().clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "expected type keyword after `->`".into()
            } });
        }
    };
    with_thread_io(OP, |io| {
        io.stdin_tx
            .send(StdInServiceEvent::Read)
            .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        let line = io
            .stdin_reply_rx
            .recv()
            .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        let edn = wat_edn::parse_owned(&line).map_err(|e| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("EDN parse error reading stdin line {:?}: {}", line, e)
        } })?;
        crate::edn_shim::edn_to_typed_value(&target_ty, &edn, sym).map_err(|e| {
            RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::EdnCoerceMismatch {
                op: OP.into(),
                expected: e.expected,
                got: e.got,
                path: e.path
            } }
        })
    })
}

// ─── Slice 1f-γ — runtime-services carrier + bridge protocol ───────────
//
// The orchestrator owns three wat-side ControlTxs (one per service).
// `register_thread_with_services` allocates per-thread Rust-side AND
// wat-side channel pairs, spawns three tiny bridge threads (one per
// service) that translate Rust-typed `*ServiceEvent` payloads to
// Value::Enum payloads (and back for stdin replies), and sends a
// Value::Enum::Add event on each ControlTx so the wat-side service
// registers the routing entry. ThreadIO holds the Rust-side ends
// (substrate-typed); the wat-side service ends are owned by the
// bridges and by the service. This indirection is the consequence of
// slice 1f-α defining ThreadIO with Rust-typed channels (`Sender<
// StdOutServiceEvent>` etc.) while the wat-side service in
// `wat/kernel/services/{stdin,stdout,stderr}.wat` operates on
// Value-typed channels (`Sender<wat::kernel::services::StdOutService::
// Event>` which is `Sender<Value>` at runtime). Pass 18's "unified
// Event enum" describes shape parity — variants and semantics agree —
// but the carrier types differ across the substrate/wat boundary;
// the bridges are how those carriers meet. Surfaced as honest-delta.
//
// The carrier choice (Option B from BRIEF § honest-delta) is to thread
// `Arc<RuntimeServices>` through `SymbolTable` as a capability carrier
// next to `encoding_ctx` / `source_loader` / `macro_registry`. Per
// memory `feedback_capability_carrier.md` — new runtime capabilities
// attach to SymbolTable. SymbolTable is cloned per spawned thread
// (`thread_sym = sym.clone()` in `eval_kernel_spawn_thread`); the
// clone naturally propagates the carrier into child threads. When
// `invoke_user_main` returns and its augmented SymbolTable drops, the
// carrier's Arc count falls; once no live child thread holds a clone,
// the ControlTxs drop, the wat-side services' control-rxs disconnect,
// and the service driver loops exit. Scope-drop cascade by
// construction.
//
// Carrier alternative (A) was `OnceLock<RuntimeServices>` static.
// Rejected because OnceLock has no clear-on-exit semantics; sequential
// `invoke_user_main` invocations in one process (the cargo-test
// shape) would inherit the first set's services, breaking test
// isolation. Carrier alternative (C) thread-local was out per BRIEF.

/// Three-Sender carrier per BRIEF Q5 + Q-carrier. Wraps the wat-side
/// `Sender<wat::kernel::services::{Std{In,Out,Err}}Service::Event>`
/// ControlTxs the orchestrator allocated when spawning the three
/// services. Each is a `Value::wat__kernel__Sender` wrapping
/// `SenderInner::Crossbeam(crossbeam_channel::Sender<Value>)`.
///
/// The struct deliberately stores the inner `Sender<Value>` directly
/// (not the wrapped `Value`) so the bridge / register helpers can call
/// `.send` without `match`ing through `SenderInner` on every event.
/// The wat-side variant tag (`:wat::kernel::services::StdOutService::
/// Event` etc.) is encoded into each event's `Value::Enum`'s
/// `type_path` field at construction time.
/// Arc 214 Stone 5.1 — ControlTx senders are now comms::thread::Sender<Value>
/// (cascade-aware, depth-1) instead of bare crossbeam Senders.
/// Arc 214 Stone 8.1 — stdout_ctrl is now a Sender<StdOutInput> (the
/// universe-resident peer's input channel) instead of a wat ControlTx.
/// stdin_ctrl and stderr_ctrl remain as wat-side ControlTxs (old path).
#[derive(Clone)]
pub struct RuntimeServices {
    /// `Sender<wat::kernel::services::StdInService::Event>` ControlTx.
    pub stdin_ctrl: crate::comms::thread::Sender<Value>,
    /// Arc 214 Stone 8.1 — the universe-resident StdOutService peer's
    /// input channel. Register/Deregister/Req flow through it.
    /// NOT cloned into ThreadIO — eval_kernel_println accesses this via
    /// sym.runtime_services() so the peer's lifetime is tied solely to RS.
    pub stdout_ctrl: crate::comms::thread::Sender<StdOutInput>,
    /// `Sender<wat::kernel::services::StdErrService::Event>` ControlTx.
    pub stderr_ctrl: crate::comms::thread::Sender<Value>,
}

impl std::fmt::Debug for RuntimeServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeServices")
            .field("stdin_ctrl", &"<wat-side Sender<Value>>")
            .field("stdout_ctrl", &"<Sender<StdOutInput>> (Stone 8.1 peer; accessed via sym.runtime_services())")
            .field("stderr_ctrl", &"<wat-side Sender<Value>>")
            .finish()
    }
}

/// Helper — extract the inner `comms::thread::Sender<Value>` from a
/// `Value::wat__kernel__Sender`. Surfaces a clean diagnostic if the
/// caller passed something else or a tier-2 PipeFd variant (the
/// services emit a tier-1 ControlTx by construction — anything else
/// is a programmer error).
///
/// Arc 214 Stone 5.1 — now extracts from `SenderInner::Comms` instead of
/// the retired `SenderInner::Crossbeam`.
fn unwrap_value_sender(v: Value, label: &'static str) -> Result<crate::comms::thread::Sender<Value>, RuntimeError> {
    match v {
        Value::wat__kernel__Sender(inner) => match inner.as_ref() {
            SenderInner::Comms { sender: s, .. } => Ok(s.clone()),
            SenderInner::PipeFd { .. } => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: label.to_string(),
                expected: "tier-1 (comms::thread) Sender",
                got: Box::new(crate::runtime::ValueSnapshot::unavailable("tier-2 (pipe-fd) Sender"))
            } }),
        },
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: label.to_string(),
            expected: "wat::kernel::Sender<T>",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

/// Helper — extract the inner `comms::thread::Receiver<Value>` from a
/// `Value::wat__kernel__Receiver`. Sibling of [`unwrap_value_sender`].
///
/// Arc 214 Stone 5.1 — now extracts from `ReceiverInner::Comms` instead of
/// the retired `ReceiverInner::Crossbeam`.
fn unwrap_value_receiver(
    v: Value,
    label: &'static str,
) -> Result<crate::comms::thread::Receiver<Value>, RuntimeError> {
    match v {
        Value::wat__kernel__Receiver(inner) => match inner.as_ref() {
            ReceiverInner::Comms(r) => Ok(r.clone()),
            ReceiverInner::PipeFd(_) => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: label.to_string(),
                expected: "tier-1 (comms::thread) Receiver",
                got: Box::new(crate::runtime::ValueSnapshot::unavailable("tier-2 (pipe-fd) Receiver"))
            } }),
        },
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: label.to_string(),
            expected: "wat::kernel::Receiver<T>",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

/// Construct the wat-side Sender Value wrapping an existing
/// `comms::thread::Sender<Value>`. Mirrors
/// [`crate::typed_channel::sender_from_comms`] but takes the
/// already-allocated Sender directly.
///
/// Arc 214 Stone 5.1 — now takes comms::thread::Sender instead of
/// the retired crossbeam::Sender.
fn sender_value(tx: crate::comms::thread::Sender<Value>) -> Value {
    sender_from_comms(tx)
}

/// Construct the wat-side Receiver Value wrapping an existing
/// `comms::thread::Receiver<Value>`.
///
/// Arc 214 Stone 5.1 — now takes comms::thread::Receiver instead of
/// the retired crossbeam::Receiver.
fn receiver_value(rx: crate::comms::thread::Receiver<Value>) -> Value {
    receiver_from_comms(rx)
}

/// Helper — construct a `Value::Enum` for one of the three service-
/// Event variants. Field order matches the wat-side enum declarations
/// in `wat/kernel/services/{stdin,stdout,stderr}.wat`.
fn make_event_value(type_path: &str, variant: &str, fields: Vec<Value>) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: type_path.into(),
        variant_name: variant.into(),
        fields,
    }))
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

/// Allocate per-thread service channels; spawn three bridge threads
/// (substrate ↔ wat-side translation); send Add events on each of
/// the three services' ControlTxs in series (BRIEF Q2 ordering: stdin
/// → stdout → stderr, fd 0/1/2 natural order). Returns the populated
/// [`ThreadIO`].
///
/// On Add-send failure (service shut down) returns
/// [`RuntimeError::ChannelDisconnected`]. Caller is responsible for
/// `install_thread_io` after this returns successfully.
///
/// **Bridge protocol** — for each service, the bridge thread:
///   1. Recv on `rust_data_rx` — a Rust-typed `*ServiceEvent::Write`
///      / `Read`.
///   2. Build `Value::Enum` for the wat-side variant.
///   3. Send on `wat_data_tx` — the wat-side data Sender; flows to
///      the service's routing-table entry.
///   4. (stdout/stderr) Recv `()` on `wat_ack_rx` (Receiver<Value>
///      where the service sends `Value::Unit`); send `()` on
///      `rust_ack_tx` so the Rust-side caller of println/eprintln
///      unblocks.
///      (stdin) Recv `Value::String(line)` on `wat_reply_rx` (the
///      raw EDN line); send the unwrapped String on `rust_reply_tx`
///      so the readln caller unblocks (1f-ι; pre-1f-ι this was a
///      pre-parsed `Arc<HolonAST>`).
///   5. Repeat until `rust_data_rx` disconnects (orchestrator
///      dropped ThreadIO's `*_tx`), then exit.
///
/// The bridge is a `std::thread::spawn` (not `:wat::kernel::spawn-
/// thread`) so it doesn't trigger spawn-thread registration / does
/// not need a ThreadIO of its own.
pub fn register_thread_with_services(
    thread_id: ThreadId,
    services: &RuntimeServices,
) -> Result<ThreadIO, RuntimeError> {
    const OP_ADD: &str = "register_thread_with_services";

    // ─── stdin pair (Rust + wat) + bridge ──────────────────────────
    //
    // Rust-typed: ThreadIO holds (rust_stdin_tx, rust_stdin_reply_rx).
    // Bridge holds (rust_stdin_rx, rust_stdin_reply_tx).
    // Wat-side:   service holds (wat_stdin_data_rx, wat_stdin_reply_tx).
    // Bridge:    holds (wat_stdin_data_tx, wat_stdin_reply_rx).
    //
    // Arc 170 slice 1f-ι — reply payload changed from `Arc<HolonAST>`
    // (pre-1f-ι, the wat-side service pre-parsed via `:wat::edn::read`)
    // to `String` (the raw EDN line). The substrate parses + coerces
    // to the caller's requested `T` in `eval_kernel_readln`.
    let (rust_stdin_tx, rust_stdin_rx) =
        crate::typed_channel::bounded::<StdInServiceEvent>(1);
    let (rust_stdin_reply_tx, rust_stdin_reply_rx) =
        crate::typed_channel::bounded::<String>(1);
    // Arc 214 Stone 5.1 — use comms::thread::pair instead of crossbeam::bounded.
    let (wat_stdin_data_tx, wat_stdin_data_rx) =
        crate::comms::thread::pair::<Value>();
    let (wat_stdin_reply_tx, wat_stdin_reply_rx) =
        crate::comms::thread::pair::<Value>();

    spawn_stdin_bridge(
        thread_id,
        rust_stdin_rx,
        rust_stdin_reply_tx,
        wat_stdin_data_tx,
        wat_stdin_reply_rx,
    );

    // ─── stdout (Arc 214 Stone 8.1 — universe-resident peer) ──────────
    //
    // No bridge thread, no wat-side channels. ThreadIO holds a clone of
    // the service peer's input_tx (for Req sends) + a per-thread reply
    // Receiver<()> (for the ack back from the service's reply registry).
    //
    // Registration: send Register(tid, reply_tx) on the service input so
    // the peer loop inserts the reply_tx into its HashMap keyed by tid.
    let (stdout_reply_tx, stdout_reply_rx) = crate::comms::thread::pair::<Result<(), String>>();
    services
        .stdout_ctrl
        .send(StdOutInput::Register(thread_id, stdout_reply_tx))
        .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
            op: OP_ADD.into()
        } })?;

    // ─── stderr pair (Rust + wat) + bridge ─────────────────────────
    let (rust_stderr_tx, rust_stderr_rx) =
        crate::typed_channel::bounded::<StdErrServiceEvent>(1);
    let (rust_stderr_ack_tx, rust_stderr_ack_rx) = crate::typed_channel::bounded::<()>(1);
    // Arc 214 Stone 5.1 — use comms::thread::pair instead of crossbeam::bounded.
    let (wat_stderr_data_tx, wat_stderr_data_rx) =
        crate::comms::thread::pair::<Value>();
    let (wat_stderr_ack_tx, wat_stderr_ack_rx) = crate::comms::thread::pair::<Value>();

    spawn_stderr_bridge(
        thread_id,
        rust_stderr_rx,
        rust_stderr_ack_tx,
        wat_stderr_data_tx,
        wat_stderr_ack_rx,
    );

    // ─── Send Add events on the three ControlTxs (series; fd 0/1/2) ──
    //
    // BRIEF Q2: order = stdin → stdout → stderr (fd 0/1/2). Each Add
    // hands the service the wat-side data_rx + ack/reply_tx; the
    // service stores them in its routing table keyed by thread_id.
    let stdin_add = make_event_value(
        ":wat::kernel::services::StdInService::Event",
        "Add",
        vec![
            Value::i64(thread_id),
            receiver_value(wat_stdin_data_rx),
            sender_value(wat_stdin_reply_tx),
        ],
    );
    services
        .stdin_ctrl
        .send(stdin_add)
        .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
            op: OP_ADD.into()
        } })?;

    // stdout registration was handled above (Stone 8.1 — Register sent at pair allocation).

    let stderr_add = make_event_value(
        ":wat::kernel::services::StdErrService::Event",
        "Add",
        vec![
            Value::i64(thread_id),
            receiver_value(wat_stderr_data_rx),
            sender_value(wat_stderr_ack_tx),
        ],
    );
    services
        .stderr_ctrl
        .send(stderr_add)
        .map_err(|_| RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::ChannelDisconnected {
            op: OP_ADD.into()
        } })?;

    Ok(ThreadIO {
        // NOTE: stdout_input is NOT stored in ThreadIO (Arc 214 Stone 8.1 fix).
        // The send goes via sym.runtime_services().stdout_ctrl in eval_kernel_println
        // so the service peer's lifetime is tied solely to Arc<RuntimeServices>,
        // not to every ThreadIO clone.
        stdout_reply_rx,
        stdout_thread_id: thread_id,
        stderr_tx: rust_stderr_tx,
        stderr_ack_rx: rust_stderr_ack_rx,
        stdin_tx: rust_stdin_tx,
        stdin_reply_rx: rust_stdin_reply_rx,
    })
}

/// Send Remove events to stdin/stderr services and a Deregister message to
/// the universe-resident stdout peer for this thread_id.
/// Silent-fail on each send (services may be shutting down via scope-drop;
/// a failed send is "the service is already gone," the cleanup state we want).
pub fn deregister_thread_from_services(thread_id: ThreadId, services: &RuntimeServices) {
    let stdin_remove = make_event_value(
        ":wat::kernel::services::StdInService::Event",
        "Remove",
        vec![Value::i64(thread_id)],
    );
    let _ = services.stdin_ctrl.send(stdin_remove);

    // Arc 214 Stone 8.1 — stdout uses Deregister on the Rust-internal enum.
    let _ = services.stdout_ctrl.send(StdOutInput::Deregister(thread_id));

    let stderr_remove = make_event_value(
        ":wat::kernel::services::StdErrService::Event",
        "Remove",
        vec![Value::i64(thread_id)],
    );
    let _ = services.stderr_ctrl.send(stderr_remove);
}

/// stdin bridge — Rust `StdInServiceEvent::Read` → wat
/// `:wat::kernel::services::StdInService::Event::Read`; wat-side
/// reply (`Value::String(line)`) → Rust `String`.
///
/// Arc 170 slice 1f-ι — the bridge now forwards the RAW EDN line
/// (a `String`) instead of a pre-parsed `Arc<HolonAST>`. The
/// wat-side StdInService sends the raw line read from fd 0; the
/// substrate parses + coerces to the caller's requested `T` in
/// `eval_kernel_readln`.
///
/// Loop exits on any disconnect — the orchestrator's epilogue drops
/// the ThreadIO end, which collapses `rust_rx`; subsequent drop of
/// `wat_data_tx` collapses the service's routing-table data-rx.
fn spawn_stdin_bridge(
    thread_id: ThreadId,
    rust_rx: Receiver<StdInServiceEvent>,
    rust_reply_tx: Sender<String>,
    wat_data_tx: crate::comms::thread::Sender<Value>,
    wat_reply_rx: crate::comms::thread::Receiver<Value>,
) {
    let name = format!("wat-stdin-bridge::{}", thread_id);
    std::thread::Builder::new()
        .name(name)
        .spawn(move || loop {
            // 1. Recv Rust event from ThreadIO side.
            let event = match rust_rx.recv() {
                Ok(e) => e,
                Err(_) => break, // ThreadIO dropped; orderly bridge shutdown.
            };
            // 2. Translate to wat-side Event variant (only `Read` flows
            //    here from eval_kernel_readln; Add/Remove go via
            //    ControlTx directly and never traverse this bridge).
            let wat_event = match event {
                StdInServiceEvent::Read => make_event_value(
                    ":wat::kernel::services::StdInService::Event",
                    "Read",
                    vec![],
                ),
                StdInServiceEvent::Add { .. } | StdInServiceEvent::Remove { .. } => {
                    // Bridge sees Add/Remove only if a future caller
                    // wrongly routes them through ThreadIO. Drop +
                    // continue — no observable effect; defensive arm.
                    continue;
                }
            };
            // 3. Forward to wat-side service.
            if wat_data_tx.send(wat_event).is_err() {
                break; // service routing-table entry gone.
            }
            // 4. Block for wat-side reply (raw line as Value::String).
            let reply = match wat_reply_rx.recv() {
                Ok(v) => v,
                Err(_) => break, // service stopped sending replies.
            };
            // 5. Extract the raw line and forward to caller. Pre-1f-ι
            //    the wat-side service ran `:wat::edn::read` over the
            //    line and shipped the parsed Value; the bridge coerced
            //    to `Arc<HolonAST>`. Post-1f-ι the wat-side ships the
            //    raw line directly; the substrate parses + coerces to
            //    the caller's declared `T` in `eval_kernel_readln`.
            let line = match reply {
                Value::String(s) => (*s).clone(),
                _ => break, // Wat-side contract violated; close bridge.
            };
            if rust_reply_tx.send(line).is_err() {
                break; // caller dropped; bridge exits.
            }
        })
        .expect("std::thread::spawn for stdin bridge");
}

/// stderr bridge — sibling of the (now-deleted) stdout bridge; same shape, with
/// `StdErrServiceEvent` and `:wat::kernel::services::StdErrService::
/// Event` variants.
fn spawn_stderr_bridge(
    thread_id: ThreadId,
    rust_rx: Receiver<StdErrServiceEvent>,
    rust_ack_tx: Sender<()>,
    wat_data_tx: crate::comms::thread::Sender<Value>,
    wat_ack_rx: crate::comms::thread::Receiver<Value>,
) {
    let name = format!("wat-stderr-bridge::{}", thread_id);
    std::thread::Builder::new()
        .name(name)
        .spawn(move || loop {
            let event = match rust_rx.recv() {
                Ok(e) => e,
                Err(_) => break,
            };
            let wat_event = match event {
                StdErrServiceEvent::Write { line } => make_event_value(
                    ":wat::kernel::services::StdErrService::Event",
                    "Write",
                    vec![Value::String(Arc::new(line))],
                ),
                StdErrServiceEvent::Add { .. } | StdErrServiceEvent::Remove { .. } => {
                    continue;
                }
            };
            if wat_data_tx.send(wat_event).is_err() {
                break;
            }
            let _ack = match wat_ack_rx.recv() {
                Ok(v) => v,
                Err(_) => break,
            };
            if rust_ack_tx.send(()).is_err() {
                break;
            }
        })
        .expect("std::thread::spawn for stderr bridge");
}

/// Pull the wat-side ControlTx out of a service-spawn return value.
/// The wat-side `*Service/spawn` fns return
/// `:(Thread<nil,nil>, EventTx)`; this helper destructures the tuple
/// and unwraps the Sender to the inner `crossbeam::Sender<Value>`.
/// Used by [`crate::freeze::invoke_user_main`] after each service
/// spawn.
/// Arc 214 Stone 5.1 — now returns comms::thread::Sender<Value> instead of
/// the retired crossbeam::Sender<Value>.
pub fn extract_control_tx(
    spawn_result: Value,
    service_label: &'static str,
) -> Result<(Value, crate::comms::thread::Sender<Value>), RuntimeError> {
    let tuple = match spawn_result {
        Value::Tuple(t) => t,
        other => {
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: service_label.to_string(),
                expected: "(Thread, Sender) tuple from service spawn",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            } });
        }
    };
    if tuple.len() != 2 {
        return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
            head: service_label.to_string(),
            reason: format!(
                "service spawn returned tuple with {} fields; expected 2",
                tuple.len()
            )
        } });
    }
    let thread_value = tuple[0].clone();
    let ctrl_tx_value = tuple[1].clone();
    let ctrl_tx = unwrap_value_sender(ctrl_tx_value, service_label)?;
    Ok((thread_value, ctrl_tx))
}

/// Hidden re-export of the receiver-unwrap helper for the
/// orchestrator. Re-exposed because `invoke_user_main` needs to
/// `Thread/output recv` then `Thread/join-result` on each service
/// handle; the `Thread<nil,nil>` shape carries a `Receiver<Value>`
/// in its tuple-field-1 position.
/// Arc 214 Stone 5.1 — now returns comms::thread::Receiver<Value> instead of
/// the retired crossbeam::Receiver<Value>.
pub fn unwrap_receiver_for_orchestrator(
    v: Value,
    label: &'static str,
) -> Result<crate::comms::thread::Receiver<Value>, RuntimeError> {
    unwrap_value_receiver(v, label)
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

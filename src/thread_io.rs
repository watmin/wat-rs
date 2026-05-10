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

use crossbeam_channel::{Receiver, Sender};
use holon::HolonAST;

use crate::ast::WatAST;
use crate::runtime::{eval, EnumValue, Environment, RuntimeError, SymbolTable, Value};
use crate::span::Span;
use crate::typed_channel::{
    receiver_from_crossbeam, sender_from_crossbeam, ReceiverInner, SenderInner,
};

/// Monotonic thread identifier. Mirrors the wat-side
/// `:wat::kernel::ThreadId` typealias-to-i64 settled in pass 18.
/// Slice 1f-γ will populate these from a monotonic counter in the
/// runtime orchestrator.
pub type ThreadId = i64;

/// Per-pass-18 control-plane + data-plane union. Sent on the
/// stdout tx; consumed by the wat-side StdOutService.
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
/// request); the parsed HolonAST comes back via the reply-tx.
#[derive(Debug, Clone)]
pub enum StdInServiceEvent {
    /// Caller's readln signals "next form please."
    Read,
    /// Runtime registers a thread; service stores
    /// `(thread_id → (data_rx, reply_tx))` in its routing table.
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdInServiceEvent>,
        reply_tx: Sender<Arc<HolonAST>>,
    },
    Remove { thread_id: ThreadId },
}

/// Per-thread channel handles used by `:wat::kernel::println` /
/// `eprintln` / `readln`. Populated by `:wat::kernel::spawn-thread`
/// (slice 1f-γ); for slice 1f-α, populated by tests via
/// [`install_thread_io`].
///
/// All six channel ends are owned (not Arc'd) — the thread that
/// owns the ThreadIO IS the thread that uses these channels.
/// crossbeam's Sender / Receiver are themselves `Send`; the
/// thread-local cell ensures only one thread accesses any given
/// ThreadIO instance.
pub struct ThreadIO {
    /// Send an Event (Write / Add / Remove) to the StdOutService.
    pub stdout_tx: Sender<StdOutServiceEvent>,
    /// Block here for the StdOutService's ack of "line emitted."
    pub stdout_ack_rx: Receiver<()>,
    /// Send an Event (Write / Add / Remove) to the StdErrService.
    pub stderr_tx: Sender<StdErrServiceEvent>,
    /// Block here for the StdErrService's ack of "line emitted."
    pub stderr_ack_rx: Receiver<()>,
    /// Send an Event (Read / Add / Remove) to the StdInService.
    pub stdin_tx: Sender<StdInServiceEvent>,
    /// Receive the parsed HolonAST representing the next stdin form.
    pub stdin_reply_rx: Receiver<Arc<HolonAST>>,
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
        None => Err(RuntimeError::ServiceNotRunning {
            op: op.into(),
            span: Span::unknown(),
        }),
    })
}

/// Shared one-arg helper — mirrors `edn_shim::require_one_arg`'s
/// shape. Inlined here to avoid leaking that helper across modules.
fn require_one_arg(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
            span: Span::unknown(),
        });
    }
    eval(&args[0], env, sym)
}

/// `(:wat::kernel::println v)` → `:wat::core::nil`. Serialize `v`
/// to compact EDN via `value_to_edn_with`; send the resulting line
/// through this thread's stdout req-tx; block on stdout ack-rx;
/// return `Value::Unit` (the `:wat::core::nil` value).
pub fn eval_kernel_println(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::println";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write(&edn);
    with_thread_io(OP, |io| {
        io.stdout_tx
            .send(StdOutServiceEvent::Write { line })
            .map_err(|_| RuntimeError::ChannelDisconnected {
                op: OP.into(),
                span: Span::unknown(),
            })?;
        io.stdout_ack_rx
            .recv()
            .map_err(|_| RuntimeError::ChannelDisconnected {
                op: OP.into(),
                span: Span::unknown(),
            })?;
        Ok(Value::Unit)
    })
}

/// `(:wat::kernel::eprintln v)` → `:wat::core::nil`. Same shape as
/// [`eval_kernel_println`] but routed through the StdErrService
/// channel pair.
pub fn eval_kernel_eprintln(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::eprintln";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write(&edn);
    with_thread_io(OP, |io| {
        io.stderr_tx
            .send(StdErrServiceEvent::Write { line })
            .map_err(|_| RuntimeError::ChannelDisconnected {
                op: OP.into(),
                span: Span::unknown(),
            })?;
        io.stderr_ack_rx
            .recv()
            .map_err(|_| RuntimeError::ChannelDisconnected {
                op: OP.into(),
                span: Span::unknown(),
            })?;
        Ok(Value::Unit)
    })
}

/// `(:wat::kernel::readln)` → `:wat::holon::HolonAST`. Signal the
/// StdInService via the stdin req-tx; block on stdin reply-rx for
/// the next parsed HolonAST; lift it into a `Value::holon__HolonAST`.
pub fn eval_kernel_readln(
    args: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::readln";
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len(),
            span: Span::unknown(),
        });
    }
    with_thread_io(OP, |io| {
        io.stdin_tx
            .send(StdInServiceEvent::Read)
            .map_err(|_| RuntimeError::ChannelDisconnected {
                op: OP.into(),
                span: Span::unknown(),
            })?;
        let ast = io
            .stdin_reply_rx
            .recv()
            .map_err(|_| RuntimeError::ChannelDisconnected {
                op: OP.into(),
                span: Span::unknown(),
            })?;
        Ok(Value::holon__HolonAST(ast))
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
#[derive(Clone)]
pub struct RuntimeServices {
    /// `Sender<wat::kernel::services::StdInService::Event>` ControlTx.
    pub stdin_ctrl: Sender<Value>,
    /// `Sender<wat::kernel::services::StdOutService::Event>` ControlTx.
    pub stdout_ctrl: Sender<Value>,
    /// `Sender<wat::kernel::services::StdErrService::Event>` ControlTx.
    pub stderr_ctrl: Sender<Value>,
}

impl std::fmt::Debug for RuntimeServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeServices")
            .field("stdin_ctrl", &"<wat-side Sender<Value>>")
            .field("stdout_ctrl", &"<wat-side Sender<Value>>")
            .field("stderr_ctrl", &"<wat-side Sender<Value>>")
            .finish()
    }
}

/// Helper — extract the inner `crossbeam::Sender<Value>` from a
/// `Value::wat__kernel__Sender`. Surfaces a clean diagnostic if the
/// caller passed something else or a tier-2 PipeFd variant (the
/// services emit a tier-1 ControlTx by construction — anything else
/// is a programmer error).
fn unwrap_value_sender(v: Value, label: &'static str) -> Result<Sender<Value>, RuntimeError> {
    match v {
        Value::wat__kernel__Sender(inner) => match inner.as_ref() {
            SenderInner::Crossbeam(s) => Ok(s.clone()),
            SenderInner::PipeFd(_) => Err(RuntimeError::TypeMismatch {
                op: label.to_string(),
                expected: "tier-1 (crossbeam) Sender",
                got: "tier-2 (pipe-fd) Sender",
                span: Span::unknown(),
            }),
        },
        other => Err(RuntimeError::TypeMismatch {
            op: label.to_string(),
            expected: "wat::kernel::Sender<T>",
            got: other.type_name(),
            span: Span::unknown(),
        }),
    }
}

/// Helper — extract the inner `crossbeam::Receiver<Value>` from a
/// `Value::wat__kernel__Receiver`. Sibling of [`unwrap_value_sender`].
fn unwrap_value_receiver(
    v: Value,
    label: &'static str,
) -> Result<Receiver<Value>, RuntimeError> {
    match v {
        Value::wat__kernel__Receiver(inner) => match inner.as_ref() {
            ReceiverInner::Crossbeam(r) => Ok(r.clone()),
            ReceiverInner::PipeFd(_) => Err(RuntimeError::TypeMismatch {
                op: label.to_string(),
                expected: "tier-1 (crossbeam) Receiver",
                got: "tier-2 (pipe-fd) Receiver",
                span: Span::unknown(),
            }),
        },
        other => Err(RuntimeError::TypeMismatch {
            op: label.to_string(),
            expected: "wat::kernel::Receiver<T>",
            got: other.type_name(),
            span: Span::unknown(),
        }),
    }
}

/// Construct the wat-side Sender Value wrapping an existing
/// `crossbeam::Sender<Value>`. Mirrors
/// [`crate::typed_channel::sender_from_crossbeam`] but takes the
/// already-allocated Sender directly.
fn sender_value(tx: Sender<Value>) -> Value {
    sender_from_crossbeam(tx)
}

/// Construct the wat-side Receiver Value wrapping an existing
/// `crossbeam::Receiver<Value>`.
fn receiver_value(rx: Receiver<Value>) -> Value {
    receiver_from_crossbeam(rx)
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
///      (stdin) Recv `Value::holon__HolonAST(ast)` on `wat_reply_rx`;
///      send `ast` on `rust_reply_tx` so the readln caller unblocks.
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
    let (rust_stdin_tx, rust_stdin_rx) =
        crossbeam_channel::bounded::<StdInServiceEvent>(1);
    let (rust_stdin_reply_tx, rust_stdin_reply_rx) =
        crossbeam_channel::bounded::<Arc<HolonAST>>(1);
    let (wat_stdin_data_tx, wat_stdin_data_rx) =
        crossbeam_channel::bounded::<Value>(1);
    let (wat_stdin_reply_tx, wat_stdin_reply_rx) =
        crossbeam_channel::bounded::<Value>(1);

    spawn_stdin_bridge(
        thread_id,
        rust_stdin_rx,
        rust_stdin_reply_tx,
        wat_stdin_data_tx,
        wat_stdin_reply_rx,
    );

    // ─── stdout pair (Rust + wat) + bridge ─────────────────────────
    let (rust_stdout_tx, rust_stdout_rx) =
        crossbeam_channel::bounded::<StdOutServiceEvent>(1);
    let (rust_stdout_ack_tx, rust_stdout_ack_rx) = crossbeam_channel::bounded::<()>(1);
    let (wat_stdout_data_tx, wat_stdout_data_rx) =
        crossbeam_channel::bounded::<Value>(1);
    let (wat_stdout_ack_tx, wat_stdout_ack_rx) = crossbeam_channel::bounded::<Value>(1);

    spawn_stdout_bridge(
        thread_id,
        rust_stdout_rx,
        rust_stdout_ack_tx,
        wat_stdout_data_tx,
        wat_stdout_ack_rx,
        "stdout",
    );

    // ─── stderr pair (Rust + wat) + bridge ─────────────────────────
    let (rust_stderr_tx, rust_stderr_rx) =
        crossbeam_channel::bounded::<StdErrServiceEvent>(1);
    let (rust_stderr_ack_tx, rust_stderr_ack_rx) = crossbeam_channel::bounded::<()>(1);
    let (wat_stderr_data_tx, wat_stderr_data_rx) =
        crossbeam_channel::bounded::<Value>(1);
    let (wat_stderr_ack_tx, wat_stderr_ack_rx) = crossbeam_channel::bounded::<Value>(1);

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
        .map_err(|_| RuntimeError::ChannelDisconnected {
            op: OP_ADD.into(),
            span: Span::unknown(),
        })?;

    let stdout_add = make_event_value(
        ":wat::kernel::services::StdOutService::Event",
        "Add",
        vec![
            Value::i64(thread_id),
            receiver_value(wat_stdout_data_rx),
            sender_value(wat_stdout_ack_tx),
        ],
    );
    services
        .stdout_ctrl
        .send(stdout_add)
        .map_err(|_| RuntimeError::ChannelDisconnected {
            op: OP_ADD.into(),
            span: Span::unknown(),
        })?;

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
        .map_err(|_| RuntimeError::ChannelDisconnected {
            op: OP_ADD.into(),
            span: Span::unknown(),
        })?;

    Ok(ThreadIO {
        stdout_tx: rust_stdout_tx,
        stdout_ack_rx: rust_stdout_ack_rx,
        stderr_tx: rust_stderr_tx,
        stderr_ack_rx: rust_stderr_ack_rx,
        stdin_tx: rust_stdin_tx,
        stdin_reply_rx: rust_stdin_reply_rx,
    })
}

/// Send Remove events to all three services for this thread_id.
/// Silent-fail on each send (BRIEF Q3: services may be shutting down
/// via scope-drop; a failed send is "the service is already gone,"
/// which is exactly the cleanup state we want).
pub fn deregister_thread_from_services(thread_id: ThreadId, services: &RuntimeServices) {
    let stdin_remove = make_event_value(
        ":wat::kernel::services::StdInService::Event",
        "Remove",
        vec![Value::i64(thread_id)],
    );
    let _ = services.stdin_ctrl.send(stdin_remove);

    let stdout_remove = make_event_value(
        ":wat::kernel::services::StdOutService::Event",
        "Remove",
        vec![Value::i64(thread_id)],
    );
    let _ = services.stdout_ctrl.send(stdout_remove);

    let stderr_remove = make_event_value(
        ":wat::kernel::services::StdErrService::Event",
        "Remove",
        vec![Value::i64(thread_id)],
    );
    let _ = services.stderr_ctrl.send(stderr_remove);
}

/// Coerce a wat-side `Value` to `Arc<HolonAST>`. Mirrors
/// `runtime::value_to_holon`'s primitive cases. Used by the stdin
/// bridge to bridge over the wat-side service's use of
/// `:wat::edn::read` (which returns a generic Value, not a
/// `Value::holon__HolonAST`). Returns `None` for shapes we don't
/// know how to coerce — caller closes the bridge so the reader
/// surfaces a clean disconnect.
fn value_to_holon_ast(v: &Value) -> Option<Arc<HolonAST>> {
    match v {
        Value::holon__HolonAST(h) => Some(Arc::clone(h)),
        Value::i64(n) => Some(Arc::new(HolonAST::i64(*n))),
        Value::f64(x) => Some(Arc::new(HolonAST::f64(*x))),
        Value::bool(b) => Some(Arc::new(HolonAST::bool_(*b))),
        Value::String(s) => Some(Arc::new(HolonAST::string(s.as_str()))),
        Value::wat__core__keyword(k) => Some(Arc::new(HolonAST::symbol(k.as_str()))),
        _ => None,
    }
}

/// stdin bridge — Rust `StdInServiceEvent::Read` → wat
/// `:wat::kernel::services::StdInService::Event::Read`; wat-side
/// reply (`Value::holon__HolonAST(ast)`) → Rust `Arc<HolonAST>`.
///
/// Loop exits on any disconnect — the orchestrator's epilogue drops
/// the ThreadIO end, which collapses `rust_rx`; subsequent drop of
/// `wat_data_tx` collapses the service's routing-table data-rx.
fn spawn_stdin_bridge(
    thread_id: ThreadId,
    rust_rx: Receiver<StdInServiceEvent>,
    rust_reply_tx: Sender<Arc<HolonAST>>,
    wat_data_tx: Sender<Value>,
    wat_reply_rx: Receiver<Value>,
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
            // 4. Block for wat-side reply (parsed HolonAST).
            let reply = match wat_reply_rx.recv() {
                Ok(v) => v,
                Err(_) => break, // service stopped sending replies.
            };
            // 5. Coerce to Arc<HolonAST> and forward to caller. The
            //    wat-side service uses `:wat::edn::read` to parse the
            //    line (returning a generic `Value`); the BRIEF
            //    contract for `(:wat::kernel::readln)` returns
            //    `:wat::holon::HolonAST`. The wat-side service does
            //    not currently re-wrap via `:wat::holon::leaf`; we
            //    coerce here defensively. Mirrors `value_to_holon`'s
            //    primitive arm.
            let ast = match value_to_holon_ast(&reply) {
                Some(a) => a,
                None => break,
            };
            if rust_reply_tx.send(ast).is_err() {
                break; // caller dropped; bridge exits.
            }
        })
        .expect("std::thread::spawn for stdin bridge");
}

/// stdout bridge — Rust `StdOutServiceEvent::Write { line }` → wat
/// `:wat::kernel::services::StdOutService::Event::Write { line }`;
/// wat-side ack (`Value::Unit`) → Rust `()`.
fn spawn_stdout_bridge(
    thread_id: ThreadId,
    rust_rx: Receiver<StdOutServiceEvent>,
    rust_ack_tx: Sender<()>,
    wat_data_tx: Sender<Value>,
    wat_ack_rx: Receiver<Value>,
    label: &'static str,
) {
    let name = format!("wat-{}-bridge::{}", label, thread_id);
    std::thread::Builder::new()
        .name(name)
        .spawn(move || loop {
            let event = match rust_rx.recv() {
                Ok(e) => e,
                Err(_) => break,
            };
            let wat_event = match event {
                StdOutServiceEvent::Write { line } => make_event_value(
                    ":wat::kernel::services::StdOutService::Event",
                    "Write",
                    vec![Value::String(Arc::new(line))],
                ),
                StdOutServiceEvent::Add { .. } | StdOutServiceEvent::Remove { .. } => {
                    continue;
                }
            };
            if wat_data_tx.send(wat_event).is_err() {
                break;
            }
            // Wat-side service sends Value::Unit (nil) as ack.
            let _ack = match wat_ack_rx.recv() {
                Ok(v) => v,
                Err(_) => break,
            };
            if rust_ack_tx.send(()).is_err() {
                break;
            }
        })
        .expect("std::thread::spawn for stdout bridge");
}

/// stderr bridge — sibling of the stdout bridge; same shape, with
/// `StdErrServiceEvent` and `:wat::kernel::services::StdErrService::
/// Event` variants.
fn spawn_stderr_bridge(
    thread_id: ThreadId,
    rust_rx: Receiver<StdErrServiceEvent>,
    rust_ack_tx: Sender<()>,
    wat_data_tx: Sender<Value>,
    wat_ack_rx: Receiver<Value>,
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
pub fn extract_control_tx(
    spawn_result: Value,
    service_label: &'static str,
) -> Result<(Value, Sender<Value>), RuntimeError> {
    let tuple = match spawn_result {
        Value::Tuple(t) => t,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: service_label.to_string(),
                expected: "(Thread, Sender) tuple from service spawn",
                got: other.type_name(),
                span: Span::unknown(),
            });
        }
    };
    if tuple.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: service_label.to_string(),
            reason: format!(
                "service spawn returned tuple with {} fields; expected 2",
                tuple.len()
            ),
            span: Span::unknown(),
        });
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
pub fn unwrap_receiver_for_orchestrator(
    v: Value,
    label: &'static str,
) -> Result<Receiver<Value>, RuntimeError> {
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

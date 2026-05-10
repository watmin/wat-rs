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
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use holon::HolonAST;

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, SymbolTable, Value};
use crate::span::Span;

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

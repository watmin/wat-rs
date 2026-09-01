//! Kernel sub-module mirroring `src/intrinsic/kernel/identity.rs` — arc 109
//! Stone B (the seven kernel sub-modules). Five items backing the edge
//! file's five verbs — `eval_peer_pid`, `eval_peer_process`,
//! `eval_peer_wire`, `eval_address_wire`, `eval_require_wire_address` — the
//! impls `peer-pid`/`peer-process`/`peer-wire?`/`address-wire?`/
//! `require-wire-address` each delegate to, one subject ("what is this
//! peer or address") asked three ways (project, probe, require).
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};

/// `(:wat::kernel::peer-pid peer)` — arc 170 capability circuit, stone 2.
///
/// PURE PROJECTION of the far-end child pid off a peer value — no effect, no
/// signalling. Reads the pid off the `Pidfd` the peer already holds:
/// - a **process** peer → `(:wat::core::Option::Some child-pid)` — the pid of the
///   forked child on the far end (`bundle.peer.pidfd.pid()`).
/// - a **thread** peer → `:None` — the far end is a cell in THIS process, so there
///   is no separate pid.
///
/// The pid is an identity only (reuse-unsafe for `kill()` per `Pidfd::pid` doc);
/// it becomes an entry in the capability allow-set later.
pub(crate) fn eval_peer_pid(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::peer-pid";
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
    let peer_val = eval_inner(&args[0], env, sym)?.value_owned();

    match &peer_val {
        // Process peer: reach the Pidfd through the bundle → (Some pid).
        Value::RustOpaque(inner)
            if inner.type_path == crate::kernel::spawn::PROCESS_PEER_TYPE_PATH =>
        {
            let cell: &std::sync::Arc<
                crate::rust_deps::custodia::ThreadOwnedCell<
                    Option<crate::kernel::spawn::ProcessSelectable>,
                >,
            > = crate::rust_deps::marshal::downcast_ref_opaque(
                inner,
                crate::kernel::spawn::PROCESS_PEER_TYPE_PATH,
                OP,
                list_span.clone(),
            )?;
            let out = cell
                .with_ref(OP, |opt_bundle| -> Result<Value, EvalBreak> {
                    match opt_bundle {
                        None => Err(RuntimeError::new(
                            list_span.clone(),
                            RuntimeErrorKind::MalformedForm {
                                head: OP.into(),
                                reason: "peer already closed".into(),
                            },
                        )
                        .into()),
                        Some(crate::kernel::spawn::ProcessSelectable::Spawned(bundle)) => {
                            let pid = bundle.peer.pidfd.pid() as i64;
                            Ok(Value::Option(std::sync::Arc::new(Some(Value::i64(pid)))))
                        }
                        // arc 292 L3 — a timer peer has no child; peer-pid is meaningless.
                        Some(crate::kernel::spawn::ProcessSelectable::Timer(_)) => {
                            Err(RuntimeError::new(
                                list_span.clone(),
                                RuntimeErrorKind::MalformedForm {
                                    head: OP.into(),
                                    reason: "peer-pid: not supported on a timer peer".into(),
                                },
                            )
                            .into())
                        }
                    }
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(out)
        }
        // Thread peer: the far end is a cell in this process → :None.
        Value::RustOpaque(inner)
            if inner.type_path == crate::kernel::spawn::THREAD_PEER_TYPE_PATH =>
        {
            Ok(Value::Option(std::sync::Arc::new(None)))
        }
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "peer ((Thread :- [I O]) | (Process :- [I O]))",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::peer-process peer)` — DESIGN-STONE-a-service-that-measures-
/// itself.md A1.
///
/// PURE PROJECTION, un-erasing the concrete locus a `(Peer :- [I O])`-typed value
/// already holds at runtime even though the static type widened it to the
/// abstract `Peer` — exactly the situation a defservice `Handle`'s lineage
/// `handle` field is in (`wat/spawn.wat:265` `Launched.handle <- (Peer :- [Sh Lu])`,
/// deliberate so `stop` stays locus-agnostic). `spawn-program` returns the
/// concrete `(Process :- [I O])`/`(Thread :- [I O])`; the RustOpaque type-path tag never
/// lies about which one it is, regardless of what the checker widened the
/// static type to. So:
/// - a **process** peer → `Some` the SAME peer value, now nameable
///   `(Process :- [I O])` — good enough to hand straight to `:wat::kernel::signal`.
/// - a **thread** peer → `None` — a thread has no process to signal.
///
/// Same shape as `eval_peer_pid` (arc 170 capability circuit stone 2): a
/// runtime tag read, no effect, no signal. Unlike `peer-pid` this does not
/// need to reach inside the bundle for a field — the peer value itself IS
/// the answer, just re-tagged at the type level (via the Option wrapper).
pub(crate) fn eval_peer_process(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::peer-process";
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
    let peer_val = eval_inner(&args[0], env, sym)?.value_owned();

    match &peer_val {
        Value::RustOpaque(inner)
            if inner.type_path == crate::kernel::spawn::PROCESS_PEER_TYPE_PATH =>
        {
            Ok(Value::Option(std::sync::Arc::new(Some(peer_val.clone()))))
        }
        Value::RustOpaque(inner)
            if inner.type_path == crate::kernel::spawn::THREAD_PEER_TYPE_PATH =>
        {
            Ok(Value::Option(std::sync::Arc::new(None)))
        }
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "peer ((Thread :- [I O]) | (Process :- [I O]))",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::peer-wire? peer)` — DESIGN-STONE-the-client-validates-
/// locally.md STOP-3.
///
/// PURE PROJECTION, un-erasing the one fact `send'`/`try-send'` already branch
/// on internally (`peer.is_socket_tier()`, `eval_peer_send_prime` above) but
/// never surfaced to wat: a client-generated method needs this BEFORE it
/// decides whether to measure a request's encoded size at all — measuring on a
/// thread-tier peer would be "a full serialization onto the one path whose
/// entire point is not serializing" (the stone's own words), not a 2x cost but
/// zero-to-full on every call. `c`'s runtime tag is always `PEER_TYPE_PATH`
/// (the unified connection object send'/recv' already operate on — NOT the
/// `PROCESS_PEER_TYPE_PATH`/`THREAD_PEER_TYPE_PATH` lineage-handle tags
/// `peer-process` reads, a different peer kind entirely).
/// - socket-tier (a wire; `send'` would call `send_wire`) → `true`.
/// - thread-tier (shared memory; `send'` never encodes) → `false`.
/// - already closed (`None`) → `false`: nothing to measure against a peer with
///   no live transport either way, and the caller's own `send'` will face the
///   real `Closed`/`Lost` outcome regardless of this answer.
pub(crate) fn eval_peer_wire(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::peer-wire?";
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
    let peer_val = eval_inner(&args[0], env, sym)?.value_owned();

    match &peer_val {
        Value::RustOpaque(inner) if inner.type_path == crate::kernel::spawn::PEER_TYPE_PATH => {
            let cell: &crate::kernel::spawn::PeerCell =
                crate::rust_deps::marshal::downcast_ref_opaque(
                    inner,
                    crate::kernel::spawn::PEER_TYPE_PATH,
                    OP,
                    list_span.clone(),
                )?;
            let is_wire = cell
                .with_ref(OP, |opt_peer| match opt_peer {
                    None => false,
                    Some(peer) => peer.is_socket_tier(),
                })
                .map_err(Into::<EvalBreak>::into)?;
            Ok(Value::bool(is_wire))
        }
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "peer (unified (Peer :- [S R]))",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::address-wire? addr)` — 293.W.2e.
///
/// PURE PROJECTION of `Address::portable_form().is_some()`. Some = wire
/// (SocketAddress; a process may hold and dial it). None = shared memory
/// (crossbeam; only a thread in that address space may dial it).
/// Downcast reuses `eval_connect_prime`'s ADDRESS_TYPE_PATH match.
pub(crate) fn eval_address_wire(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::address-wire?";
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
    let addr_val = eval_inner(&args[0], env, sym)?.value_owned();
    let addr: &crate::kernel::address::Address = match addr_val {
        Value::RustOpaque(ref inner)
            if inner.type_path == crate::kernel::spawn::ADDRESS_TYPE_PATH =>
        {
            use crate::rust_deps::marshal::downcast_ref_opaque;
            downcast_ref_opaque(
                inner.as_ref(),
                crate::kernel::spawn::ADDRESS_TYPE_PATH,
                OP,
                args[0].span().clone(),
            )?
        }
        ref other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Address :- [S R])",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    Ok(Value::bool(addr.portable_form().is_some()))
}

/// `(:wat::kernel::require-wire-address x)` — 293.W.2f. Runtime identity;
/// the Wire check lives in `infer_require_wire_address`.
pub(crate) fn eval_require_wire_address(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::require-wire-address";
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
    Ok(eval_inner(&args[0], env, sym)?.value_owned())
}

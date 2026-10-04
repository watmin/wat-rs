//! Kernel sub-module mirroring `src/intrinsic/kernel/message.rs` — arc 109
//! Stone B (the seven kernel sub-modules). Six items backing the edge
//! file's five verbs (`send`, `try-send`, `recv`, `select`, `poll`) plus
//! the private helper they share: `wrap_connect_request` unpacks a
//! connect-request `Value` and wraps the server `Peer'` end on the current
//! thread — used by `eval_poll_prime`'s listener arm (`accept'` reaches the
//! same wrap through `Listener::accept_as_value` in `src/kernel/listener.rs`,
//! not through this helper). ONE helper, its module's own caller.
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::kernel::error::loci_died_disconnected;
use crate::kernel::outcome::{
    recv_outcome_closed, recv_outcome_from_decoded, recv_outcome_lost, recv_outcome_message,
    recv_outcome_shutdown, send_outcome_closed, send_outcome_from_error, send_outcome_sent,
    try_send_outcome_closed, try_send_outcome_lost, try_send_outcome_sent,
    try_send_outcome_would_block,
};
use crate::runtime::{builtin_enum_variant_names, eval_inner, message_only_failure, no_field_names};
use crate::span::Span;
use crate::value::{
    EnumValue, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use std::sync::Arc;

/// Unpack a connect-request `Value` and wrap the server `Peer'` end on the
/// current thread.
///
/// Called by both `eval_accept_prime` (after a blocking `typed_recv` on the
/// listener) and by the 2-arg `eval_peer_select_prime` (after the listener
/// arm of `sel.select()` fires).  ONE helper, TWO callers — the wrap logic
/// is not duplicated.
///
/// The connect-request is a `Value::Tuple` `(req_rx: Receiver, resp_tx: Sender)`
/// minted by `connect'` and uniquely owned at the point of receipt:
/// `Arc::try_unwrap` succeeds.  Returns the server `(Peer' :- [R S])` opaque.
pub(crate) fn wrap_connect_request(cr: Value, span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::accept"; // same context for error messages
                                             // Unpack the connect-request tuple: (req_rx: Receiver, resp_tx: Sender).
    let mut items: Vec<Value> = match cr {
        Value::Tuple(arc) => match Arc::try_unwrap(arc) {
            Ok(vec) => vec,
            Err(arc) => (*arc).clone(),
        },
        other => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "connect-request must be a Tuple; got {:?}",
                        other.type_name()
                    ),
                },
            )
            .into());
        }
    };
    if items.len() != 2 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "connect-request tuple must have 2 elements; got {}",
                    items.len()
                ),
            },
        )
        .into());
    }
    // Move items out in reverse order so indices stay valid.
    let resp_tx_val = items.remove(1);
    let req_rx_val = items.remove(0);
    // Extract req_rx (Receiver<Value>) — moved out, unique owner.
    let req_rx = match req_rx_val {
        Value::wat__kernel__Receiver(arc) => match Arc::try_unwrap(arc) {
            Ok(crate::channel::ReceiverInner::Comms(rx)) => rx,
            Ok(_) => {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "connect-request rx is not a comms (thread-tier) receiver".into(),
                    },
                )
                .into());
            }
            Err(_) => {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "connect-request rx has unexpected additional references".into(),
                    },
                )
                .into());
            }
        },
        other => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "Receiver (connect-request req_rx)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    // Extract resp_tx (Sender<Value>) — moved out, unique owner.
    let resp_tx = match resp_tx_val {
        Value::wat__kernel__Sender(arc) => match Arc::try_unwrap(arc) {
            Ok(crate::channel::SenderInner::Comms { sender, .. }) => sender,
            Err(_) => {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "connect-request tx has unexpected additional references".into(),
                    },
                )
                .into());
            }
        },
        other => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "Sender (connect-request resp_tx)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    // Wrap the server (Peer' :- [R S]) end on THIS thread (custody holds).
    use crate::kernel::peer::Peer;
    use crate::kernel::spawn::PEER_TYPE_PATH;
    use crate::rust_deps::custodia::ThreadOwnedCell;
    use crate::rust_deps::marshal::make_rust_opaque;
    Ok(make_rust_opaque(
        PEER_TYPE_PATH,
        Arc::new(ThreadOwnedCell::new(Some(Peer::from_thread(
            resp_tx, req_rx,
        )))),
    ))
}

/// Excursus 003 stone G — encode `payload` for a WIRE peer, or refuse AT THE SENDER.
///
/// A wire promises the value comes back whole, so a value the EDN writer can only render as a
/// payload-less tag (an `Lru`, a `Sender`, a fn — anything with no wire form) must not be shipped: it
/// used to go out as `#tag nil`, the sender was told `Sent`, and only the RECEIVER learned — as a
/// `Lost` whose message spoke of retired syntax. The compile-time wall at the wire-peer producers
/// cannot see it when the peer is opened inside a generic fn (`(Box :- [T])`, `T` still a parameter).
///
/// The refusal RAISES, at `list_span` (the user's call), like the timer arm's misuse raise below: it
/// is a programmer misuse, not a gone peer, so it is no `SendOutcome` — `Sent/Closed/Stopped/Lost`
/// all describe the channel, never the value. Which values refuse is the WRITER's own set
/// ([`crate::edn::render::value_to_wire_edn_string`]), never a parallel list here.
///
/// Excursus 003 stone H — returns the strict [`WireFrame`](crate::edn::render::WireFrame) every wire
/// sink requires, and is `pub(crate)` so every wire VERB raises the same refusal: `send`/`try-send`
/// here, a process-tier `after` (`kernel/resource.rs`), and `println`/`pprintln` in a spawned process,
/// whose stdout IS its wire (`services/verbs.rs`).
pub(crate) fn encode_for_wire(
    op: &str,
    payload: &Value,
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<crate::edn::render::WireFrame, RuntimeError> {
    encode_for_wire_as(op, payload, list_span, sym, "sent", "")
}

/// [`encode_for_wire`], with the verb the refusal names the payload by (`"sent"`, `"printed"`) and a
/// `note` (empty, or one sentence led by a space) naming why this verb is a wire.
pub(crate) fn encode_for_wire_as(
    op: &str,
    payload: &Value,
    list_span: &Span,
    sym: &SymbolTable,
    verb: &str,
    note: &str,
) -> Result<crate::edn::render::WireFrame, RuntimeError> {
    use crate::edn::render::{value_to_wire_edn_string, WireEncodeError};
    value_to_wire_edn_string(payload, sym.types().map(|a| a.as_ref())).map_err(|e| match e {
        WireEncodeError::Encode(e) => e,
        WireEncodeError::NotPure { type_name } => RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "a wire peer carries only pure data — the value {verb} ({}) contains a {}, \
                     which is not pure and has no wire form.{note} Send records, scalars, or pure \
                     enums over a wire; a handle stays in-locus (a thread peer carries any value).",
                    wat_type_label(payload),
                    type_name
                ),
            },
        ),
    })
}

/// The wat type a refusal names for the value sent: a record's or enum's own class, else the
/// value's type name — always with its leading `:`.
fn wat_type_label(v: &Value) -> String {
    let t = match v {
        Value::Aggregate(a) => a.class.to_string(),
        Value::Enum(e) => e.type_path.to_string(),
        other => other.type_name().to_string(),
    };
    if t.starts_with(':') { t } else { format!(":{t}") }
}

/// `(:wat::kernel::send peer payload)` — Stone 4.6a-ii / Arc 258.5b-ii.
///
/// Thread': `peer.send(value)` Value pass-through (crossbeam, no serialisation).
/// Process': encode payload via value_to_edn + wat_edn::write → peer.send(String).
/// Peer' thread-tier: `peer.send(value)` Value pass-through.
/// Peer' socket-tier: encode with sym.types() in eval → `peer.send_wire(String)`.
/// Returns `nil`.  Use-after-close (Option is None) → RuntimeError.
pub(crate) fn eval_peer_send_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::send";
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
    let peer_val = eval_inner(&args[0], env, sym)?;
    let payload_val = eval_inner(&args[1], env, sym)?;

    match &peer_val {
        Value::RustOpaque(inner)
            if inner.type_path == crate::kernel::spawn::THREAD_PEER_TYPE_PATH =>
        {
            let cell: &std::sync::Arc<
                crate::rust_deps::custodia::ThreadOwnedCell<
                    Option<crate::kernel::peer::Thread<Value, Value>>,
                >,
            > = crate::rust_deps::marshal::downcast_ref_opaque(
                inner,
                crate::kernel::spawn::THREAD_PEER_TYPE_PATH,
                OP,
                list_span.clone(),
            )?;
            let outcome = cell
                .with_ref(OP, |opt_peer| -> Result<Value, EvalBreak> {
                    Ok(match opt_peer {
                        None => send_outcome_closed(),
                        Some(peer) => match peer.send(payload_val) {
                            Ok(()) => send_outcome_sent(),
                            Err(e) => send_outcome_from_error(&e),
                        },
                    })
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(outcome)
        }
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
            // Arc 258.5b — thread sym.types() into the encoder so records cross
            // the process wire with named fields (e.g. {:x 7, :y 35}) rather than
            // positional fallback ({:field-0 7, :field-1 35}). The decoder on the
            // receiver side uses sym.types() too (arc 258.5b / 272 6c.2), so the
            // named-field map round-trips exactly. Before 258.5b, send' called
            // value_to_edn (no registry) and recv' expected a `-> :T` hint.
            // Excursus 003 stone G — the strict wire encode: a value with no wire form raises here.
            let wire = encode_for_wire(OP, &payload_val, list_span, sym)?;
            let outcome = cell
                .with_ref(OP, |opt_bundle| -> Result<Value, EvalBreak> {
                    match opt_bundle {
                        None => Ok(send_outcome_closed()),
                        Some(crate::kernel::spawn::ProcessSelectable::Spawned(bundle)) => {
                            Ok(match bundle.send_wire(wire.clone()) {
                                Ok(()) => send_outcome_sent(),
                                Err(e) => send_outcome_from_error(&e),
                            })
                        }
                        // arc 292 L3 — timers are select'-only; send' is not supported. Not a
                        // "gone peer" case (the SendOutcome wall's remit) — a genuine
                        // programmer misuse (wrong peer kind), so it still raises.
                        Some(crate::kernel::spawn::ProcessSelectable::Timer(_)) => {
                            Err(RuntimeError::new(
                                list_span.clone(),
                                RuntimeErrorKind::MalformedForm {
                                    head: OP.into(),
                                    reason: "cannot send to a timer peer (timers are select-only)"
                                        .into(),
                                },
                            )
                            .into())
                        }
                    }
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(outcome)
        }
        // Arc 209 C0b.2e-i-b / Arc 258.5b-ii — unified Peer' arm.
        //
        // Thread-tier peers: send Value in-process via crossbeam (no serialisation).
        // Socket-tier peers: encode with sym.types() in the eval layer → ship the
        //   wire String via Peer::send_wire (Sender<String> raw passthrough).
        //
        // Symmetric with recv': the decode side already threads sym.types() through
        // decode_trusted_wire in eval_peer_recv_prime. Arc 258.5b's thread-local
        // injection is gone — the encode type env travels honestly as a function-local.
        Value::RustOpaque(inner) if inner.type_path == crate::kernel::spawn::PEER_TYPE_PATH => {
            let cell: &crate::kernel::spawn::PeerCell =
                crate::rust_deps::marshal::downcast_ref_opaque(
                    inner,
                    crate::kernel::spawn::PEER_TYPE_PATH,
                    OP,
                    list_span.clone(),
                )?;
            let outcome = cell
                .with_ref(OP, |opt_peer| -> Result<Value, EvalBreak> {
                    Ok(match opt_peer {
                        None => send_outcome_closed(),
                        Some(peer) if peer.is_socket_tier() => {
                            // Socket-tier: encode with type registry in eval, ship the wire String.
                            // Excursus 003 stone G — strict: a value with no wire form raises here.
                            let wire = encode_for_wire(OP, &payload_val, list_span, sym)?;
                            match peer.send_wire(wire) {
                                Ok(()) => send_outcome_sent(),
                                Err(e) => send_outcome_from_error(&e),
                            }
                        }
                        Some(peer) => {
                            // Thread-tier: pass Value in-process, no serialisation.
                            match peer.send(payload_val) {
                                Ok(()) => send_outcome_sent(),
                                Err(e) => send_outcome_from_error(&e),
                            }
                        }
                    })
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(outcome)
        }
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "peer ((Thread :- [I O]) | (Process :- [I O]) | (Peer :- [S R]))",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::try-send peer payload)` — Arc 278 Stone 1a / Phase 3a
/// (`BRIEF-send-wall-3a-try-send-outcome.md`).
///
/// Best-effort, NON-BLOCKING twin of `send'` for the unified `(Peer' :- [S R])`. Same
/// type contract for the payload (unifies with the peer's I) but the write NEVER
/// blocks: a full kernel buffer (peer not draining) or a gone peer is a
/// **silent skip** at the transport level — but unlike Phase-1 `send'`, the
/// caller-visible result is now an honest `TrySendOutcome`
/// (`Sent`/`WouldBlock`/`Closed`/`Lost`), not a swallowed `nil`. Used by the
/// serve loop's over-FOO `Rejected` arm to reply `Reply::Failed{cause}` to a
/// client that may be blocked mid-`send` on an extreme oversized frame: if it
/// isn't reading its reply side, the reply is skipped (`WouldBlock`) and the
/// connection is evicted regardless (the client learns via EPIPE on its own
/// `send`), so one client can never wedge the loop — see `service.wat:1167`.
pub(crate) fn eval_peer_try_send_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::try-send";
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
    let peer_val = eval_inner(&args[0], env, sym)?;
    let payload_val = eval_inner(&args[1], env, sym)?;

    match &peer_val {
        // Unified Peer' arm (the serve loop's `clients` are PEER_TYPE_PATH — socket
        // tier on process, thread tier on thread). Best-effort: any TRANSPORT failure is a
        // faced TrySendOutcome value, never a raise. A payload with no wire form on the socket
        // tier is not a transport failure but a misuse, and raises (excursus 003 stone G).
        Value::RustOpaque(inner) if inner.type_path == crate::kernel::spawn::PEER_TYPE_PATH => {
            let cell: &crate::kernel::spawn::PeerCell =
                crate::rust_deps::marshal::downcast_ref_opaque(
                    inner,
                    crate::kernel::spawn::PEER_TYPE_PATH,
                    OP,
                    list_span.clone(),
                )?;
            // Excursus 003 stone H — the payload is encoded ONLY on the socket arm, where it is
            // shipped. The thread tier ships the `Value` itself (a thread peer carries any value), so
            // an encode there could only refuse a legal program: it used to, raising a `:wat::edn::write`
            // error for a thread-tier `try-send` of a holon that `send` delivers fine. On the socket
            // arm the encode is STRICT (`encode_for_wire`) and a refusal RAISES — `?` propagates out of
            // `with_ref`'s `Result` callback. It is never lossy: an `#wat.edn/Unencodable` marker put on
            // the wire as if it were the payload would be worse than either outcome.
            let outcome = cell
                .with_ref(OP, |opt_peer| -> Result<Value, EvalBreak> {
                    Ok(match opt_peer {
                        // Already closed → Closed (never an error).
                        None => try_send_outcome_closed(),
                        Some(peer) if peer.is_socket_tier() => {
                            let wire = encode_for_wire(OP, &payload_val, list_span, sym)?;
                            match peer.try_send_wire(wire) {
                                crate::kernel::peer::TrySendResult::Sent => try_send_outcome_sent(),
                                crate::kernel::peer::TrySendResult::Full => {
                                    try_send_outcome_would_block()
                                }
                                crate::kernel::peer::TrySendResult::Disconnected => {
                                    try_send_outcome_lost(loci_died_disconnected())
                                }
                            }
                        }
                        Some(peer) => match peer.try_send(payload_val.clone()) {
                            crate::kernel::peer::TrySendResult::Sent => try_send_outcome_sent(),
                            crate::kernel::peer::TrySendResult::Full => {
                                try_send_outcome_would_block()
                            }
                            crate::kernel::peer::TrySendResult::Disconnected => {
                                try_send_outcome_lost(loci_died_disconnected())
                            }
                        },
                    })
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(outcome)
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

pub(crate) fn eval_peer_recv_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::recv";

    // Arc 258.5b — recv' is 1-arg only; `-> :T` ascription is illegal on recv'.
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "recv takes exactly one argument (peer); got {}. \
                     `-> :T` is a function-return annotation only — it is illegal on recv. \
                     The type flows from the consumer or the self-describing EDN wire.",
                    args.len()
                ),
            },
        )
        .into());
    }

    let peer_val = eval_inner(&args[0], env, sym)?;

    match &peer_val {
        Value::RustOpaque(inner)
            if inner.type_path == crate::kernel::spawn::THREAD_PEER_TYPE_PATH =>
        {
            let cell: &std::sync::Arc<
                crate::rust_deps::custodia::ThreadOwnedCell<
                    Option<crate::kernel::peer::Thread<Value, Value>>,
                >,
            > = crate::rust_deps::marshal::downcast_ref_opaque(
                inner,
                crate::kernel::spawn::THREAD_PEER_TYPE_PATH,
                OP,
                list_span.clone(),
            )?;
            let result = cell
                .with_ref(OP, |opt_peer| -> Result<Value, EvalBreak> {
                    // Arc 278 the recv'-outcome wall — recv' returns a matchable
                    // `(RecvOutcome :- [O])`, never raises on close/crash (a raise unwinds
                    // past the reader = mute). Ok → Message; Disconnected (clean EOF,
                    // incl. use-after-close: the peer is gone) → Closed; Crashed(reason)
                    // → Lost(<structured Failure carrying the crash reason>).
                    match opt_peer {
                        None => Ok(recv_outcome_closed()),
                        Some(peer) => Ok(match peer.recv() {
                            Ok(v) => recv_outcome_message(v),
                            Err(e) => {
                                use crate::kernel::spawn::PeerRecvError;
                                match e {
                                    PeerRecvError::Crashed(crash_reason) => recv_outcome_lost(
                                        crash_reason,
                                        sym.types().map(|a| a.as_ref()),
                                    ),
                                    PeerRecvError::Disconnected => recv_outcome_closed(),
                                    // A stop was requested; the peer is alive. NOT Closed —
                                    // Closed means a genuine clean EOF (arc 170).
                                    PeerRecvError::Shutdown => recv_outcome_shutdown(),
                                }
                            }
                        }),
                    }
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(result)
        }
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
            // Arc 278 the recv'-outcome wall — the process arm returns a matchable
            // `(RecvOutcome :- [O])`. The EDN decode moves INSIDE the closure so a decode
            // failure surfaces as `Lost(<Failure>)` (abnormal loss carrying its reason),
            // never a raise. Ok+decode → Message; Crashed(reason) → Lost; Disconnected
            // (clean EOF / use-after-close) → Closed. Timer stays a static-usage raise.
            //
            // Arc 258.5b / 272 6a-i / step 5 / 6c.2 — recv' is the TRUSTED peer wire: decode
            // through the capability-reconstructing door with the full type registry.
            // Every Peer is lineage BY CONSTRUCTION — a spawn handle / self-peer is
            // inherited; an accept'd peer passed OnlyMyPeers (euid + pid∈allow-set);
            // a connect'd peer passed OnlyThisPeer (euid + kernel-vouched pid == minter
            // pid stamped in the address capability; the autobind name is an exclusive-bind
            // rendezvous token, not a secret; the SO_PEERCRED checks are the security).
            // So "bytes from a lineage peer" holds on every leg. The EDN wire is
            // self-describing (post-234.7: tagged records/structs/enums + typed scalars)
            // — sym.types() reconstructs user records; no declared target type is needed.
            let result = cell
                .with_ref(OP, |opt_bundle| -> Result<Value, EvalBreak> {
                    match opt_bundle {
                        None => Ok(recv_outcome_closed()),
                        Some(crate::kernel::spawn::ProcessSelectable::Spawned(bundle)) => {
                            use crate::kernel::spawn::PeerRecvError;
                            Ok(match bundle.recv() {
                                Ok(edn_str) => match crate::edn::render::decode_trusted_wire(
                                    &edn_str,
                                    sym.types().map(|a| a.as_ref()),
                                    sym.encoding_ctx().map(|a| a.as_ref()),
                                ) {
                                    Ok(v) => recv_outcome_message(v),
                                    Err(e) => recv_outcome_lost(format!("recv EDN decode failed: {}", e), sym.types().map(|a| a.as_ref())),
                                },
                                // The crash reason is the full `#wat.kernel/ProcessPanics [...]`
                                // envelope text — carried as the Lost cause's Failure/message.
                                Err(PeerRecvError::Crashed(crash_reason)) => {
                                    recv_outcome_lost(crash_reason, sym.types().map(|a| a.as_ref()))
                                }
                                Err(PeerRecvError::Disconnected) => recv_outcome_closed(),
                                // A stop was requested; the peer is alive. NOT Closed —
                                // Closed means a genuine clean EOF (arc 170).
                                Err(PeerRecvError::Shutdown) => recv_outcome_shutdown(),
                            })
                        }
                        // arc 292 L3 — timers are select'-only; recv' is not supported.
                        // A static-usage error (not a peer-read outcome) → still a raise.
                        Some(crate::kernel::spawn::ProcessSelectable::Timer(_)) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                head: OP.into(),
                                reason: "recv on a timer peer is not supported; place it in a select set".into(),
                            })
                        .into()),
                    }
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(result)
        }
        // Arc 209 C0b.2e-i-b — unified Peer' arm: thread-tier and socket-tier peers both
        // box their recv endpoint as `Box<dyn CommReceiver<Value>>`. Decoding is internal
        // to the boxed transport impl — `peer.recv()` returns `Value` directly.
        Value::RustOpaque(inner) if inner.type_path == crate::kernel::spawn::PEER_TYPE_PATH => {
            let cell: &crate::kernel::spawn::PeerCell =
                crate::rust_deps::marshal::downcast_ref_opaque(
                    inner,
                    crate::kernel::spawn::PEER_TYPE_PATH,
                    OP,
                    list_span.clone(),
                )?;
            // Arc 278 the recv'-outcome wall — the unified peer arm returns a matchable
            // `(RecvOutcome :- [O])`. Ok+decode → Message (or Lost if the decoded value is a
            // reserved `Reply::Failed`, via recv_outcome_from_decoded); a raw wire
            // Failed(reason) or an abnormal far-side crash (PeerCrashed, whose to_string
            // IS the reason-free administrative sentinel a CLIENT gets) → Lost; a clean
            // Disconnected/Shutdown/FrameTooLarge / use-after-close → Closed.
            let result = cell
                .with_ref(OP, |opt_peer| -> Result<Value, EvalBreak> {
                    match opt_peer {
                        None => Ok(recv_outcome_closed()),
                        // Arc 272 6b-ii-α — socket-tier self-peer: recv the raw EDN wire
                        // string and decode via the trusted-wire door with sym.types().
                        // peer.recv() decodes internally via Value::from_wire (no type
                        // registry) and fails on user-defined record tags
                        // (e.g. `#user/Counter {:base 1000}`). Using recv_wire() +
                        // decode_trusted_wire reconstructs the record correctly.
                        Some(peer) if peer.is_socket_tier() => {
                            match peer.recv_wire() {
                                Ok(wire) => Ok(
                                    match crate::edn::render::decode_trusted_wire(
                                        &wire,
                                        sym.types().map(|a| a.as_ref()),
                                        sym.encoding_ctx().map(|a| a.as_ref()),
                                    ) {
                                        Ok(v) => recv_outcome_from_decoded(
                                            v,
                                            sym.types().map(|a| a.as_ref()),
                                        ),
                                        Err(e) => recv_outcome_lost(
                                            format!("recv EDN decode failed: {}", e),
                                            sym.types().map(|a| a.as_ref()),
                                        ),
                                    },
                                ),
                                Err(e) => Ok(match &e {
                                    // A raw wire failure carries its real reason.
                                    crate::comms::RecvError::Failed(reason) => recv_outcome_lost(
                                        reason.clone(),
                                        sym.types().map(|a| a.as_ref()),
                                    ),
                                    // Abnormal far-side crash — the reason-free administrative
                                    // sentinel (owner's crash channel holds the full reason).
                                    crate::comms::RecvError::PeerCrashed => recv_outcome_lost(
                                        e.to_string(),
                                        sym.types().map(|a| a.as_ref()),
                                    ),
                                    // Arc 170 — a stop was requested; the peer is ALIVE and the
                                    // channel is open. This arm used to be folded into the
                                    // wildcard below under the comment "genuine clean close",
                                    // which is what reported "peer closed" for a healthy peer and
                                    // is what a months-long sigterm flake was made of.
                                    crate::comms::RecvError::Shutdown => recv_outcome_shutdown(),
                                    // Genuine clean close (Disconnected).
                                    // NOTE: FrameTooLarge still lands here and is ALSO not a clean
                                    // close — an over-budget frame reported as EOF. Arc 278 minted
                                    // the Rejected path for it; out of scope for this fix, named
                                    // rather than silently inherited.
                                    _ => recv_outcome_closed(),
                                }),
                            }
                        }
                        // Thread-tier peer: peer.recv() returns a decoded Value directly.
                        Some(peer) => Ok(match peer.recv() {
                            Ok(v) => recv_outcome_from_decoded(v, sym.types().map(|a| a.as_ref())),
                            Err(e) => match &e {
                                crate::comms::RecvError::Failed(reason) => recv_outcome_lost(
                                    reason.clone(),
                                    sym.types().map(|a| a.as_ref()),
                                ),
                                crate::comms::RecvError::PeerCrashed => recv_outcome_lost(
                                    e.to_string(),
                                    sym.types().map(|a| a.as_ref()),
                                ),
                                // Arc 170 — see the socket-tier arm above: a stop request is not
                                // a close, and calling it one is the lie this fix removes.
                                crate::comms::RecvError::Shutdown => recv_outcome_shutdown(),
                                _ => recv_outcome_closed(),
                            },
                        }),
                    }
                })
                .map_err(Into::<EvalBreak>::into)??;
            Ok(result)
        }
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "peer ((Thread :- [I O]) | (Process :- [I O]) | (Peer :- [S R]))",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::select peers)` — Stone 4.6b / Stone 259 Lost-locus.
///
/// Blocks until one peer's output is ready; returns a
/// `:wat::spawn::ServiceEvent`:
/// - `Ok(value)` from a peer → `ServiceEvent::Message { idx, msg }`.
/// - EOF on output, crash channel has reason → `ServiceEvent::Lost { idx, cause }`.
/// - EOF on output, crash channel empty (clean exit) → `ServiceEvent::Closed { idx }`.
///
/// Dispatch:
/// - `peers` must be a non-empty `Value::Vec`.
/// - All elements must be the same tier (the first element's type_path decides).
///   A mismatched element → TypeMismatch (check forbids it; runtime refuses honestly).
/// - Empty vector → MalformedForm "select over an empty vector would block forever".
/// - `None` Option (peer already closed) → MalformedForm "peer already closed".
/// - Thread tier: builds a `comms::thread::Select`, registers output receivers,
///   blocks; on EOF reads the crash channel → `Lost`/`Closed`; on value → `Message`.
/// - Process tier: same with `comms::process::Select`; decodes EDN String → Value;
///   on EOF reads the err channel → `Lost`/`Closed`.
/// - Shutdown fires → MalformedForm "select' interrupted by shutdown".
pub(crate) fn eval_peer_select_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::select";
    // Arc 209 Stone C0b.2e-i-c: select' is 1-arg-only (fan-in over homogeneous peers).
    // The 3-arg service multiplexer is poll' — use (poll' self listener clients) instead.
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "select takes one peer vector (fan-in); got {} args. \
                     The 3-arg service multiplexer is poll.",
                    args.len()
                ),
            },
        )
        .into());
    }
    let peers_val = eval_inner(&args[0], env, sym)?;
    let peers_vec = match peers_val {
        Value::Vec(ref v) => v.clone(),
        other => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "Vector of peers",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };

    if peers_vec.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "select over an empty vector would block forever".into(),
            },
        )
        .into());
    }

    // Determine tier from first element.
    let first_type_path = match &peers_vec[0] {
        Value::RustOpaque(inner) => inner.type_path,
        other => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "peer ((Thread :- [I O]) | (Process :- [I O]))",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into())
        }
    };

    if first_type_path == crate::kernel::spawn::THREAD_PEER_TYPE_PATH {
        // ── Thread tier ────────────────────────────────────────────────────────
        // Downcast all elements to Arc<ThreadOwnedCell<Option<Thread<Value,Value>>>>.
        type ThreadCell = std::sync::Arc<
            crate::rust_deps::custodia::ThreadOwnedCell<
                Option<crate::kernel::peer::Thread<Value, Value>>,
            >,
        >;
        let mut arcs: Vec<&ThreadCell> = Vec::with_capacity(peers_vec.len());
        for (i, peer) in peers_vec.iter().enumerate() {
            match peer {
                Value::RustOpaque(inner)
                    if inner.type_path == crate::kernel::spawn::THREAD_PEER_TYPE_PATH =>
                {
                    let cell: &ThreadCell = crate::rust_deps::marshal::downcast_ref_opaque(
                        inner,
                        crate::kernel::spawn::THREAD_PEER_TYPE_PATH,
                        OP,
                        list_span.clone(),
                    )?;
                    arcs.push(cell);
                }
                other => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!(
                                "peers[{}] has wrong tier (expected Thread): {:?}",
                                i,
                                ValueSnapshot::of(other)
                            ),
                        },
                    )
                    .into())
                }
            }
        }

        // Acquire ref_guard for each cell — simultaneous shared borrows.
        let mut guards: Vec<
            crate::rust_deps::custodia::RefGuard<
                '_,
                Option<crate::kernel::peer::Thread<Value, Value>>,
            >,
        > = Vec::with_capacity(arcs.len());
        for arc in &arcs {
            guards.push(
                arc.ref_guard(OP, list_span.clone())
                    .map_err(EvalBreak::from)?,
            );
        }

        // Verify none are closed; collect &output and &crash receivers.
        // Both are borrowed from the same guard, so we need the guards alive
        // across the select — collect as parallel slices.
        let mut output_rxs: Vec<&crate::comms::thread::Receiver<Value>> =
            Vec::with_capacity(guards.len());
        let mut crash_rxs: Vec<&crate::comms::thread::Receiver<String>> =
            Vec::with_capacity(guards.len());
        for (i, guard) in guards.iter().enumerate() {
            match &**guard {
                None => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!("peer already closed (index {})", i),
                        },
                    )
                    .into())
                }
                Some(peer) => {
                    output_rxs.push(&peer.output);
                    crash_rxs.push(&peer.crash);
                }
            }
        }

        // Build Select over output receivers only.
        let mut sel = crate::comms::thread::Select::new();
        for rx in &output_rxs {
            sel.recv(*rx);
        }

        // Block until ready; demux EOF via the crash channel (mirrors Thread::recv).
        const SELECT_EVENT_TYPE_THREAD: &str = ":wat::spawn::ServiceEvent";
        match sel.select() {
            crate::comms::SelectOutcome::Recv { index, result } => {
                let peer_idx = index.0 as i64;
                match result {
                    Ok(msg) => Ok(Value::Enum(Arc::new(EnumValue {
                        type_path: SELECT_EVENT_TYPE_THREAD.into(),
                        variant_name: "Message".into(),
                        names: builtin_enum_variant_names(SELECT_EVENT_TYPE_THREAD, "Message"),
                        fields: vec![Value::i64(peer_idx), msg],
                    }))),
                    Err(_) => {
                        // Output EOF — classify death via the shared helper.
                        use crate::kernel::spawn::{classify_peer_death, PeerDeath};
                        let event = match classify_peer_death(crash_rxs[index.0].recv()) {
                            PeerDeath::Lost(reason) => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE_THREAD.into(),
                                variant_name: "Lost".into(),
                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE_THREAD, "Lost"),
                                fields: vec![Value::i64(peer_idx), message_only_failure(reason)],
                            })),
                            PeerDeath::Closed => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE_THREAD.into(),
                                variant_name: "Closed".into(),
                                names: builtin_enum_variant_names(
                                    SELECT_EVENT_TYPE_THREAD,
                                    "Closed",
                                ),
                                fields: vec![Value::i64(peer_idx)],
                            })),
                            // Arc 278 #73 — a stop woke this peer's read. It is NOT a
                            // per-peer event (nothing happened to peers[idx]; the world
                            // is stopping), so it carries no index: `Shutdown` is the
                            // nullary terminate signal this surface already has. The
                            // self-peer/admin arm below reaches the SAME variant for the
                            // same underlying reason, so this is parity, not a widening.
                            PeerDeath::Shutdown => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE_THREAD.into(),
                                variant_name: "Shutdown".into(),
                                names: no_field_names(),
                                fields: vec![],
                            })),
                        };
                        Ok(event)
                    }
                }
            }
            crate::comms::SelectOutcome::Shutdown => Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "select interrupted by shutdown".into(),
                },
            )
            .into()),
            crate::comms::SelectOutcome::Listener => {
                unreachable!("thread-tier Select has no listener arm")
            }
        }
    } else if first_type_path == crate::kernel::spawn::PROCESS_PEER_TYPE_PATH {
        // ── Process tier ───────────────────────────────────────────────────────
        type ProcessCell = std::sync::Arc<
            crate::rust_deps::custodia::ThreadOwnedCell<
                Option<crate::kernel::spawn::ProcessSelectable>,
            >,
        >;
        let mut arcs: Vec<&ProcessCell> = Vec::with_capacity(peers_vec.len());
        for (i, peer) in peers_vec.iter().enumerate() {
            match peer {
                Value::RustOpaque(inner)
                    if inner.type_path == crate::kernel::spawn::PROCESS_PEER_TYPE_PATH =>
                {
                    let cell: &ProcessCell = crate::rust_deps::marshal::downcast_ref_opaque(
                        inner,
                        crate::kernel::spawn::PROCESS_PEER_TYPE_PATH,
                        OP,
                        list_span.clone(),
                    )?;
                    arcs.push(cell);
                }
                other => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!(
                                "peers[{}] has wrong tier (expected Process): {:?}",
                                i,
                                ValueSnapshot::of(other)
                            ),
                        },
                    )
                    .into())
                }
            }
        }

        // Acquire ref_guard for each cell.
        let mut guards: Vec<
            crate::rust_deps::custodia::RefGuard<
                '_,
                Option<crate::kernel::spawn::ProcessSelectable>,
            >,
        > = Vec::with_capacity(arcs.len());
        for arc in &arcs {
            guards.push(
                arc.ref_guard(OP, list_span.clone())
                    .map_err(EvalBreak::from)?,
            );
        }

        const SELECT_EVENT_TYPE: &str = ":wat::spawn::ServiceEvent";

        // Verify none are closed; collect &output and &err receivers.
        // Both are borrowed from the same guard, so we need guards alive
        // across the select — collect as parallel slices.
        // arc 292 L3 — err_rxs is now Option<&Receiver<String>>: Spawned has a real
        // err channel; Timer(rx) has none (it never crashes — EOF = Closed, not Lost).
        let mut output_rxs: Vec<&crate::comms::process::Receiver<String>> =
            Vec::with_capacity(guards.len());
        let mut err_rxs: Vec<Option<&crate::comms::process::Receiver<String>>> =
            Vec::with_capacity(guards.len());
        for (i, guard) in guards.iter().enumerate() {
            match &**guard {
                None => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!("peer already closed (index {})", i),
                        },
                    )
                    .into())
                }
                Some(crate::kernel::spawn::ProcessSelectable::Spawned(bundle)) => {
                    output_rxs.push(&bundle.peer.output);
                    err_rxs.push(Some(&bundle.err));
                }
                // arc 292 L3 — timer peer: output is the timer rx; no err channel.
                Some(crate::kernel::spawn::ProcessSelectable::Timer(rx)) => {
                    output_rxs.push(rx);
                    err_rxs.push(None);
                }
            }
        }

        // Build Select over output receivers only.
        let mut sel = crate::comms::process::Select::new();
        for rx in &output_rxs {
            sel.recv(*rx);
        }

        // Block until ready; demux EOF via the err channel (mirrors ProcessPeerBundle::recv).
        match sel.select().map_err(|io_err| {
            EvalBreak::from(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("select io_uring error: {}", io_err),
                },
            ))
        })? {
            crate::comms::SelectOutcome::Recv { index, result } => {
                match result {
                    // The ONE door (annihilation of the two-door deadlock):
                    // classify_peer_error owns the FrameTooLarge teardown (no err
                    // read → no deadlock) AND the true-EOF err read. recv'
                    // (ProcessPeerBundle::recv) routes through the SAME fn — a
                    // cap-violation surfaces as Lost{cap reason} consistently.
                    // arc 292 L3 — timer peers have no err channel (err_rxs[i] = None);
                    // EOF on a timer rx always means Closed (the timer fired and is done).
                    Err(e) => {
                        use crate::kernel::spawn::{classify_peer_error, PeerDeath};
                        let peer_idx = index.0 as i64;
                        let event = match err_rxs[index.0] {
                            Some(err_rx) => match classify_peer_error(&e, err_rx) {
                                PeerDeath::Lost(reason) => Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE.into(),
                                    variant_name: "Lost".into(),
                                    names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Lost"),
                                    fields: vec![
                                        Value::i64(peer_idx),
                                        message_only_failure(reason),
                                    ],
                                })),
                                PeerDeath::Closed => Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE.into(),
                                    variant_name: "Closed".into(),
                                    names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Closed"),
                                    fields: vec![Value::i64(peer_idx)],
                                })),
                                // Arc 278 #73 — see the thread-tier arm above. A stop is
                                // not a fact about peers[idx]; it carries no index.
                                PeerDeath::Shutdown => Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE.into(),
                                    variant_name: "Shutdown".into(),
                                    names: no_field_names(),
                                    fields: vec![],
                                })),
                            },
                            // Timer peer: no err channel; EOF always means clean Closed.
                            None => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE.into(),
                                variant_name: "Closed".into(),
                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Closed"),
                                fields: vec![Value::i64(peer_idx)],
                            })),
                        };
                        Ok(event)
                    }
                    Ok(edn_str) => {
                        // Arc 258.5b / 272 6a-i / step 5 / 6c.2 — select' is the TRUSTED peer wire:
                        // decode through the capability door with the full type registry.
                        let value = crate::edn::render::decode_trusted_wire(
                            &edn_str,
                            sym.types().map(|a| a.as_ref()),
                            sym.encoding_ctx().map(|a| a.as_ref()),
                        )
                        .map_err(|e| {
                            EvalBreak::from(RuntimeError::new(
                                list_span.clone(),
                                RuntimeErrorKind::MalformedForm {
                                    head: OP.into(),
                                    reason: format!("select EDN decode failed: {}", e),
                                },
                            ))
                        })?;
                        let peer_idx = index.0 as i64;
                        Ok(Value::Enum(Arc::new(EnumValue {
                            type_path: SELECT_EVENT_TYPE.into(),
                            variant_name: "Message".into(),
                            names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Message"),
                            fields: vec![Value::i64(peer_idx), value],
                        })))
                    }
                }
            }
            crate::comms::SelectOutcome::Shutdown => Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "select interrupted by shutdown".into(),
                },
            )
            .into()),
            crate::comms::SelectOutcome::Listener => {
                unreachable!("process-tier 1-arg select has no listener arm")
            }
        }
    } else if first_type_path == crate::kernel::spawn::PEER_TYPE_PATH {
        // ── Bare Peer' (a provisioned connection — no spawned worker behind it) ──
        // Arc 209 Stone C0 / C0b.2e-i-b — a service select's over the server ends of the
        // peer-pair' connections it has provisioned.  The unified `Peer` boxes its rx
        // endpoint; recover the concrete `&thread::Receiver<Value>` via `as_any` (i-a
        // foundation).  Socket-backed connection peers in `select'` are C0b.3a-ii.
        let mut arcs: Vec<&crate::kernel::spawn::PeerCell> = Vec::with_capacity(peers_vec.len());
        for (i, peer) in peers_vec.iter().enumerate() {
            match peer {
                Value::RustOpaque(inner)
                    if inner.type_path == crate::kernel::spawn::PEER_TYPE_PATH =>
                {
                    let cell: &crate::kernel::spawn::PeerCell =
                        crate::rust_deps::marshal::downcast_ref_opaque(
                            inner,
                            crate::kernel::spawn::PEER_TYPE_PATH,
                            OP,
                            list_span.clone(),
                        )?;
                    arcs.push(cell);
                }
                other => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!(
                                "peers[{}] has wrong tier (expected Peer): {:?}",
                                i,
                                ValueSnapshot::of(other)
                            ),
                        },
                    )
                    .into())
                }
            }
        }

        let mut guards: Vec<
            crate::rust_deps::custodia::RefGuard<'_, Option<crate::kernel::peer::Peer>>,
        > = Vec::with_capacity(arcs.len());
        for arc in &arcs {
            guards.push(
                arc.ref_guard(OP, list_span.clone())
                    .map_err(EvalBreak::from)?,
            );
        }

        // Bare Peer' has no crash channel (it is a connection peer, not a spawned worker).
        // EOF = clean disconnect only → :Closed. :Lost is for spawned workers.
        const SELECT_EVENT_TYPE_PEER: &str = ":wat::spawn::ServiceEvent";

        // ── Dispatch on the reactor class of the (homogeneous) peer set ───────────
        // arc 278 Stone 1 — a unified `Peer'` timer (from `after`) is a real `Peer'`, so
        // `select'` must accept it at BOTH tiers (a process-tier `after` yields a socket-
        // backed `Peer'`). This closes the C0b.3a-ii deferral for `select'`, mirroring
        // `poll'`'s already-shipped Fd client arm (`select_raw` + `decode_trusted_wire`).
        let first_class = match &*guards[0] {
            Some(peer) => peer.rx.reactor_class(),
            None => {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "peer already closed (index 0)".into(),
                    },
                )
                .into())
            }
        };

        match first_class {
            crate::comms::ReactorClass::InMemory => {
                // ── Thread tier: crossbeam Select over &thread::Receiver<Value> ──────
                let mut receivers: Vec<&crate::comms::thread::Receiver<Value>> =
                    Vec::with_capacity(guards.len());
                for (i, guard) in guards.iter().enumerate() {
                    match &**guard {
                        None => {
                            return Err(RuntimeError::new(
                                list_span.clone(),
                                RuntimeErrorKind::MalformedForm {
                                    head: OP.into(),
                                    reason: format!("peer already closed (index {})", i),
                                },
                            )
                            .into())
                        }
                        Some(peer) => {
                            match peer.rx.as_any().downcast_ref::<crate::comms::thread::Receiver<Value>>() {
                                Some(rx) => receivers.push(rx),
                                None => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                        head: OP.into(),
                                        reason: format!(
                                            "peers[{}]: mixed-tier select set (a non-crossbeam peer \
                                             among crossbeam peers) is not a representable-good state",
                                            i
                                        ),
                                    }).into()),
                            }
                        }
                    }
                }

                let mut sel = crate::comms::thread::Select::new();
                for rx in &receivers {
                    sel.recv(*rx);
                }
                match sel.select() {
                    crate::comms::SelectOutcome::Recv { index, result } => {
                        let peer_idx = index.0 as i64;
                        match result {
                            Ok(msg) => Ok(Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE_PEER.into(),
                                variant_name: "Message".into(),
                                names: builtin_enum_variant_names(
                                    SELECT_EVENT_TYPE_PEER,
                                    "Message",
                                ),
                                fields: vec![Value::i64(peer_idx), msg],
                            }))),
                            // EOF — bare connection peer left gracefully (no crash channel).
                            Err(_) => Ok(Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE_PEER.into(),
                                variant_name: "Closed".into(),
                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE_PEER, "Closed"),
                                fields: vec![Value::i64(peer_idx)],
                            }))),
                        }
                    }
                    crate::comms::SelectOutcome::Shutdown => Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "select interrupted by shutdown".into(),
                        },
                    )
                    .into()),
                    crate::comms::SelectOutcome::Listener => {
                        unreachable!("thread-tier Peer Select has no listener arm")
                    }
                }
            }
            crate::comms::ReactorClass::Fd => {
                // ── Process tier: process::Select over ONE io_uring ring ────────────
                // Recover &process::Receiver<Value> from each unified Peer' (mirrors the
                // poll' Fd client arm). No self-peer / listener — select' is peers-only.
                let mut receivers: Vec<&crate::comms::process::Receiver<Value>> =
                    Vec::with_capacity(guards.len());
                for (i, guard) in guards.iter().enumerate() {
                    match &**guard {
                        None => {
                            return Err(RuntimeError::new(
                                list_span.clone(),
                                RuntimeErrorKind::MalformedForm {
                                    head: OP.into(),
                                    reason: format!("peer already closed (index {})", i),
                                },
                            )
                            .into())
                        }
                        Some(peer) => {
                            match peer
                                .rx
                                .as_any()
                                .downcast_ref::<crate::comms::process::Receiver<Value>>()
                            {
                                Some(rx) => receivers.push(rx),
                                None => {
                                    return Err(RuntimeError::new(
                                        list_span.clone(),
                                        RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: format!(
                                            "peers[{}]: mixed-tier select set (a non-socket peer \
                                             among socket peers) is not a representable-good state",
                                            i
                                        ),
                                        },
                                    )
                                    .into())
                                }
                            }
                        }
                    }
                }

                let mut sel = crate::comms::process::Select::<Value>::new();
                for rx in &receivers {
                    sel.recv(*rx);
                }
                // select_raw() → raw wire bytes (select() would call Value::from_wire with
                // NO type registry and fail on user enum/record payloads); decode with the
                // full registry via decode_trusted_wire — same as the poll' client arm.
                match sel.select_raw().map_err(|io_err| {
                    EvalBreak::from(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!("select (process tier) io_uring error: {}", io_err),
                        },
                    ))
                })? {
                    crate::comms::SelectOutcome::Recv { index, result } => {
                        let peer_idx = index.0 as i64;
                        match result {
                            Ok(raw_bytes) => {
                                let wire_str = std::str::from_utf8(&raw_bytes).map_err(|_| {
                                    EvalBreak::from(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: "select (process tier): peer message is not valid UTF-8".into(),
                                        }))
                                })?;
                                let msg = crate::edn::render::decode_trusted_wire(
                                    wire_str,
                                    sym.types().map(|a| a.as_ref()),
                                    sym.encoding_ctx().map(|a| a.as_ref()),
                                )
                                .map_err(|e| {
                                    EvalBreak::from(RuntimeError::new(
                                        list_span.clone(),
                                        RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: format!(
                                                "select (process tier) EDN decode failed: {}",
                                                e
                                            ),
                                        },
                                    ))
                                })?;
                                Ok(Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE_PEER.into(),
                                    variant_name: "Message".into(),
                                    names: builtin_enum_variant_names(
                                        SELECT_EVENT_TYPE_PEER,
                                        "Message",
                                    ),
                                    fields: vec![Value::i64(peer_idx), msg],
                                })))
                            }
                            // EOF — bare connection peer left gracefully (no crash channel).
                            Err(_) => Ok(Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE_PEER.into(),
                                variant_name: "Closed".into(),
                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE_PEER, "Closed"),
                                fields: vec![Value::i64(peer_idx)],
                            }))),
                        }
                    }
                    crate::comms::SelectOutcome::Shutdown => Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "select interrupted by shutdown".into(),
                        },
                    )
                    .into()),
                    crate::comms::SelectOutcome::Listener => {
                        unreachable!("process-tier peers-only select has no listener arm")
                    }
                }
            }
        }
    } else {
        Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "peer ((Thread :- [I O]) | (Process :- [I O]) | (Peer :- [I O]))",
                got: Box::new(ValueSnapshot::unavailable(first_type_path)),
            },
        )
        .into())
    }
}

/// Excursus 003 strike T2 + T3 — decode one client wire frame into the `ServiceEvent` value
/// `poll'`'s two client-message arms (the main arm and the re-poll retry) both build. ONE DECODE
/// DOOR (T3 Target §1): `decode_trusted_wire` — unconditionally strict, everywhere — is the only
/// decode. On a SHAPE-class refusal (`EdnReadErrorKind::is_shape_defect`), the outer tag still
/// PARSED (only typed reconstruction failed), so the op is in hand:
/// `crate::edn::render::shape_defect_request_malformed` re-walks the untyped `:req` sub-tree
/// against the op's own declared request type — the SAME walker `:wat::edn::validate` calls — to
/// build `ServiceEvent::RequestMalformed`'s `(path, expected, got)` (Target §2/§3: one shared
/// formatter, the contract's rendering). A non-shape refusal (an unparseable frame, an
/// unknown/unsupported tag — the message never identifies any op at all), or a shape refusal
/// whose op cannot be resolved, goes straight to `ServiceEvent::Malformed` with the STRUCTURED
/// cause (Target §1 — `EdnReadError::to_record`, never `message_only_failure(format!(...))`'s
/// flattened prose).
fn decode_client_message_event(peer_idx: i64, wire_str: &str, sym: &SymbolTable, event_type: &str) -> Value {
    let types = sym.types().map(|a| a.as_ref());
    let ctx = sym.encoding_ctx().map(|a| a.as_ref());
    match crate::edn::render::decode_trusted_wire(wire_str, types, ctx) {
        // ServiceEvent::Message [idx <- i64  msg <- Value]
        Ok(msg) => Value::Enum(Arc::new(EnumValue {
            type_path: event_type.into(),
            variant_name: "Message".into(),
            names: builtin_enum_variant_names(event_type, "Message"),
            fields: vec![Value::i64(peer_idx), msg],
        })),
        Err(e) if e.kind.is_shape_defect() => {
            match types.and_then(|t| crate::edn::render::shape_defect_request_malformed(wire_str, t, ctx)) {
                // ServiceEvent::RequestMalformed [idx <- i64  op <- String  path <- Vector<String>  expected <- String  got <- String]
                Some((op, coerce_err)) => Value::Enum(Arc::new(EnumValue {
                    type_path: event_type.into(),
                    variant_name: "RequestMalformed".into(),
                    names: builtin_enum_variant_names(event_type, "RequestMalformed"),
                    fields: vec![
                        Value::i64(peer_idx),
                        Value::String(Arc::new(op)),
                        Value::Vec(Arc::new(
                            wat_reader::identifier::dot_path_segments(&coerce_err.path)
                                .into_iter()
                                .map(|seg| Value::String(Arc::new(seg.to_string())))
                                .collect(),
                        )),
                        Value::String(Arc::new(coerce_err.expected)),
                        Value::String(Arc::new(coerce_err.got)),
                    ],
                })),
                // The op couldn't be resolved (shouldn't happen for a genuine shape defect, but
                // this is the honest fallback, not a panic) — generic Malformed, structured cause.
                None => Value::Enum(Arc::new(EnumValue {
                    type_path: event_type.into(),
                    variant_name: "Malformed".into(),
                    names: builtin_enum_variant_names(event_type, "Malformed"),
                    fields: vec![Value::i64(peer_idx), crate::runtime::edn_read_error_failure(&e)],
                })),
            }
        }
        // ServiceEvent::Malformed [idx <- i64  cause <- Failure]
        Err(e) => Value::Enum(Arc::new(EnumValue {
            type_path: event_type.into(),
            variant_name: "Malformed".into(),
            names: builtin_enum_variant_names(event_type, "Malformed"),
            fields: vec![Value::i64(peer_idx), crate::runtime::edn_read_error_failure(&e)],
        })),
    }
}

pub(crate) fn eval_poll_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::poll";
    const SELECT_EVENT_TYPE: &str = ":wat::spawn::ServiceEvent";

    // ── arg 0: self-peer → PEER_TYPE_PATH opaque ──────────────────────────────
    // The self-peer is the spawned worker's own (Peer' :- [O I]) (tx=output_tx, rx=input_rx).
    // We only need its .rx (= input_rx); watching it makes the RAII drain the wake.
    // Arc 209 C0b.2e-i-b: Peer is now non-generic (boxed); recover the concrete
    // &thread::Receiver<Value> via as_any (i-a foundation, shipped aac27fb5).
    let self_peer_val = eval_inner(&args[0], env, sym)?;
    let self_peer_cell: crate::kernel::spawn::PeerCell = match &self_peer_val {
        Value::RustOpaque(inner) if inner.type_path == crate::kernel::spawn::PEER_TYPE_PATH => {
            crate::rust_deps::marshal::downcast_ref_opaque::<crate::kernel::spawn::PeerCell>(
                inner,
                crate::kernel::spawn::PEER_TYPE_PATH,
                OP,
                list_span.clone(),
            )?
            .clone()
        }
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Peer :- [S R]) (self-peer, the owner/supervisor link)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let self_guard = self_peer_cell
        .ref_guard(OP, list_span.clone())
        .map_err(EvalBreak::from)?;
    // Determine self-peer's reactor class (thread vs process) for tier dispatch.
    // The concrete receiver type is recovered below after the class is confirmed.
    let self_peer_class: crate::comms::ReactorClass = match &*self_guard {
        None => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "poll: self-peer already closed".into(),
                },
            )
            .into());
        }
        Some(peer) => peer.rx.reactor_class(),
    };

    // ── arg 1: listener → Listener' (unified Listener entity, arc 209 C0b.2e-ii) ─
    let listener_val = eval_inner(&args[1], env, sym)?;
    let listener_opaque: &crate::kernel::listener::Listener = match &listener_val {
        Value::RustOpaque(inner) if inner.type_path == crate::kernel::spawn::LISTENER_TYPE_PATH => {
            crate::rust_deps::marshal::downcast_ref_opaque::<crate::kernel::listener::Listener>(
                inner,
                crate::kernel::spawn::LISTENER_TYPE_PATH,
                OP,
                args[1].span().clone(),
            )?
        }
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Listener :- [S R]) (unified Listener entity from listener)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let listener_class = listener_opaque.inner.reactor_class();

    // ── arg 2: peers → Vec of PEER_TYPE_PATH opaques ──────────────────────────
    let peers_val = eval_inner(&args[2], env, sym)?;
    let peers_vec = match peers_val {
        Value::Vec(ref v) => v.clone(),
        other => {
            return Err(RuntimeError::new(
                args[2].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Vector :- [(Peer :- [I O])]) (connected client peers)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // Downcast all peer elements to PeerCell.
    // Arc 209 C0b.2e-i-b: PeerCell is now the non-generic unified peer cell.
    let mut peer_arcs: Vec<crate::kernel::spawn::PeerCell> = Vec::with_capacity(peers_vec.len());
    for (i, peer) in peers_vec.iter().enumerate() {
        match peer {
            Value::RustOpaque(inner) if inner.type_path == crate::kernel::spawn::PEER_TYPE_PATH => {
                let cell: &crate::kernel::spawn::PeerCell =
                    crate::rust_deps::marshal::downcast_ref_opaque(
                        inner,
                        crate::kernel::spawn::PEER_TYPE_PATH,
                        OP,
                        list_span.clone(),
                    )?;
                peer_arcs.push(cell.clone());
            }
            other => {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                            "poll: peers[{}] must be a Peer (connected client); got {:?}",
                            i,
                            ValueSnapshot::of(other)
                        ),
                    },
                )
                .into());
            }
        }
    }

    // Acquire RefGuard for each peer cell (needed for both tiers).
    let mut peer_guards: Vec<
        crate::rust_deps::custodia::RefGuard<'_, Option<crate::kernel::peer::Peer>>,
    > = Vec::with_capacity(peer_arcs.len());
    for arc in &peer_arcs {
        peer_guards.push(
            arc.ref_guard(OP, list_span.clone())
                .map_err(EvalBreak::from)?,
        );
    }

    // ── Verify reactor_class homogeneity across self-peer + listener + all clients ──
    // self_peer_class already computed above; listener_class just computed.
    // Check client peers match the self-peer class.
    if listener_class != self_peer_class {
        return Err(RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "poll: listener tier ({:?}) does not match self-peer tier ({:?}) — \
                     mixed-tier service is not a representable-good state",
                    listener_class, self_peer_class
                ),
            },
        )
        .into());
    }
    for (i, guard) in peer_guards.iter().enumerate() {
        match &**guard {
            None => {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("poll: client peer already closed (index {})", i),
                    },
                )
                .into());
            }
            Some(peer) => {
                let client_class = peer.rx.reactor_class();
                if client_class != self_peer_class {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!(
                                "poll: peers[{}] tier ({:?}) does not match self-peer tier \
                                 ({:?}) — mixed-tier service is not a representable-good state",
                                i, client_class, self_peer_class
                            ),
                        },
                    )
                    .into());
                }
            }
        }
    }

    // ── Dispatch to tier-specific Select ──────────────────────────────────────
    use crate::comms::ReactorClass;
    match self_peer_class {
        ReactorClass::InMemory => {
            // ── Thread tier: crossbeam Select ─────────────────────────────────
            // Extract &thread::Receiver<Value> from each peer via as_any (i-a).
            let self_rx: &crate::comms::thread::Receiver<Value> = match &*self_guard {
                Some(peer) => peer
                    .rx
                    .as_any()
                    .downcast_ref::<crate::comms::thread::Receiver<Value>>()
                    .expect("reactor_class InMemory implies thread::Receiver"),
                None => unreachable!("closed check done above"),
            };
            let listener_rx: &crate::comms::thread::Receiver<Value> = &listener_opaque
                .inner
                .as_any_ref()
                .downcast_ref::<crate::kernel::listener::CrossbeamListener>()
                .expect("reactor_class InMemory implies CrossbeamListener")
                .rx;
            let mut peer_rxs: Vec<&crate::comms::thread::Receiver<Value>> =
                Vec::with_capacity(peer_guards.len());
            for guard in &peer_guards {
                match &**guard {
                    Some(peer) => peer_rxs.push(
                        peer.rx
                            .as_any()
                            .downcast_ref::<crate::comms::thread::Receiver<Value>>()
                            .expect("reactor_class InMemory implies thread::Receiver"),
                    ),
                    None => unreachable!("closed check done above"),
                }
            }
            // ── Build Select: self-peer at index 0, listener at 1, clients at 2..=N+1 ──
            let mut sel = crate::comms::thread::Select::new();
            sel.recv(self_rx); // index 0 = self-peer (owner link — RAII drain wakes this)
            sel.recv(listener_rx); // index 1 = listener
            for rx in &peer_rxs {
                sel.recv(*rx); // indices 2..=N+1 = peers[0..N-1]
            }
            // ── Block until one fires ──────────────────────────────────────────
            let event_value = match sel.select() {
                crate::comms::SelectOutcome::Shutdown => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "select interrupted by shutdown".into(),
                        },
                    )
                    .into());
                }
                crate::comms::SelectOutcome::Recv { index, result } => {
                    if index.0 == 0 {
                        // ── Self-peer arm (index 0): owner↔service lineage channel ──────
                        // Arc 291 3a-i: inspect `result`.
                        //   Ok(msg)  → ServiceEvent::Admin{msg}  (owner sent an admin op)
                        //   Err(_)   → ServiceEvent::Shutdown     (owner dropped handle — RAII drain)
                        // Previously this arm always returned :Shutdown without inspecting
                        // `result` — that discarded messages the owner sent before dropping.
                        // [[arc-291-3a-i: admin/data facet split foundation]]
                        match result {
                            Ok(msg) => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE.into(),
                                variant_name: "Admin".into(),
                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Admin"),
                                fields: vec![msg],
                            })),
                            Err(_) => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE.into(),
                                variant_name: "Shutdown".into(),
                                names: no_field_names(),
                                fields: vec![],
                            })),
                        }
                    } else if index.0 == 1 {
                        // ── Listener arm: a client is dialing ─────────────────────────
                        let cr = result.map_err(|_| {
                            EvalBreak::from(RuntimeError::new(
                                list_span.clone(),
                                RuntimeErrorKind::MalformedForm {
                                    head: OP.into(),
                                    reason: "poll: listener recv failed — address was dropped"
                                        .into(),
                                },
                            ))
                        })?;
                        let peer_value = wrap_connect_request(cr, list_span)?;
                        // ServiceEvent::Connection [peer <- (Peer' :- [I O])]
                        Value::Enum(Arc::new(EnumValue {
                            type_path: SELECT_EVENT_TYPE.into(),
                            variant_name: "Connection".into(),
                            names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Connection"),
                            fields: vec![peer_value],
                        }))
                    } else {
                        // ── Client peer arm: peers[k-2] fired ─────────────────────────
                        // index 0 = self-peer, index 1 = listener → subtract 2 for peer idx.
                        let peer_idx = (index.0 - 2) as i64;
                        match result {
                            Ok(msg) => {
                                // ServiceEvent::Message [idx <- i64  msg <- O]
                                Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE.into(),
                                    variant_name: "Message".into(),
                                    names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Message"),
                                    fields: vec![Value::i64(peer_idx), msg],
                                }))
                            }
                            Err(_) => {
                                // Output EOF — bare Peer' has no crash channel, so there
                                // is no abnormal-exit distinction here.  The canonical
                                // Lost-vs-Closed classifier is
                                // `crate::kernel::spawn::classify_peer_death`; poll' keeps
                                // emitting :Closed because bare peers carry no crash
                                // channel.  Upgrading poll' to emit :Lost requires adding
                                // a crash channel to `Peer` (peer.rs) — the next slice.
                                // ServiceEvent::Closed [idx <- i64]
                                Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE.into(),
                                    variant_name: "Closed".into(),
                                    names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Closed"),
                                    fields: vec![Value::i64(peer_idx)],
                                }))
                            }
                        }
                    }
                }
                crate::comms::SelectOutcome::Listener => {
                    unreachable!("thread-tier poll Select has no listener arm")
                }
            };
            Ok(event_value)
        }

        ReactorClass::Fd => {
            // ── Process tier: process::Select over ONE io_uring ring ──────────
            // Arc 209 C0b.3a-ii — DEADLOCK-SURFACE: the self-peer is index 0.
            // The owner dropping the spawn-program' handle → the child's input
            // pipe (fd0) closes → the self-peer's process::Receiver sees EOF →
            // process::Select fires Recv{0} → we return ServiceEvent::Shutdown →
            // the loop exits → RAII reaps. The RAII drain the runtime already
            // runs on owner-drop IS the wake. NO cooperative Stop, NO shutdown
            // channel. [[feedback_vended_primitives_never_deadlock]]

            // Extract &process::Receiver<Value> from self-peer via as_any (i-a).
            let self_proc_rx: &crate::comms::process::Receiver<Value> = match &*self_guard {
                Some(peer) => peer
                    .rx
                    .as_any()
                    .downcast_ref::<crate::comms::process::Receiver<Value>>()
                    .expect("reactor_class Fd implies process::Receiver"),
                None => unreachable!("closed check done above"),
            };

            // Extract the socket listener's raw fd for the accept-arm.
            use std::os::fd::AsRawFd;
            let socket_listener: &crate::kernel::listener::SocketListener = listener_opaque
                .inner
                .as_any_ref()
                .downcast_ref::<crate::kernel::listener::SocketListener>()
                .expect("reactor_class Fd implies SocketListener");
            let listen_raw_fd = socket_listener.listener.as_raw_fd();

            // Extract &process::Receiver<Value> for each client peer.
            let mut client_proc_rxs: Vec<&crate::comms::process::Receiver<Value>> =
                Vec::with_capacity(peer_guards.len());
            for guard in &peer_guards {
                match &**guard {
                    Some(peer) => client_proc_rxs.push(
                        peer.rx
                            .as_any()
                            .downcast_ref::<crate::comms::process::Receiver<Value>>()
                            .expect("reactor_class Fd implies process::Receiver"),
                    ),
                    None => unreachable!("closed check done above"),
                }
            }

            // ── Build process::Select ──────────────────────────────────────────
            // index 0 = self-peer (owner link — self-peer EOF IS the termination wake)
            // indices 1..=N = clients[0..N-1]  (NB: NOT +2; listener is the accept-arm)
            // listener arm = SelectOutcome::Listener  (no recv index)
            let mut sel = crate::comms::process::Select::<Value>::new();
            sel.recv(self_proc_rx); // index 0
            for rx in &client_proc_rxs {
                sel.recv(*rx); // indices 1..=N
            }
            sel.listener(listen_raw_fd);

            // ── Block until one fires ──────────────────────────────────────────
            // Arc 272 6b-ii-β: use select_raw() to get raw frame bytes for Recv
            // outcomes. select() calls Value::from_wire (no type registry) which
            // fails for user-defined enum/record types (e.g. Op::Increment).
            // select_raw() returns Vec<u8>; the client arm decodes with
            // decode_trusted_wire(wire, sym.types()) to reconstruct user values.
            let event_value = match sel.select_raw().map_err(|io_err| {
                EvalBreak::from(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("poll (process tier) io_uring error: {}", io_err),
                    },
                ))
            })? {
                crate::comms::SelectOutcome::Shutdown => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "poll interrupted by substrate shutdown".into(),
                        },
                    )
                    .into());
                }
                crate::comms::SelectOutcome::Recv { index, result } => {
                    if index.0 == 0 {
                        // ── Self-peer arm (index 0): owner↔service lineage channel ──────
                        // Arc 291 3a-i: inspect `result`.
                        //   Ok(raw_bytes) → decode → ServiceEvent::Admin{msg}  (owner sent admin op)
                        //   Err(_)        → ServiceEvent::Shutdown              (owner dropped handle)
                        // Previously always returned :Shutdown without inspecting `result`.
                        // [[arc-291-3a-i: admin/data facet split foundation]]
                        match result {
                            Ok(raw_bytes) => {
                                let wire_str = std::str::from_utf8(&raw_bytes).map_err(|_| {
                                    EvalBreak::from(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: "poll (process tier): admin message is not valid UTF-8".into(),
                                        }))
                                })?;
                                let msg = crate::edn::render::decode_trusted_wire(
                                    wire_str,
                                    sym.types().map(|a| a.as_ref()),
                                    sym.encoding_ctx().map(|a| a.as_ref()),
                                )
                                .map_err(|e| {
                                    EvalBreak::from(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: format!(
                                                "poll (process tier): admin message decode failed: {}",
                                                e
                                            ),
                                        }))
                                })?;
                                Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE.into(),
                                    variant_name: "Admin".into(),
                                    names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Admin"),
                                    fields: vec![msg],
                                }))
                            }
                            Err(_) => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE.into(),
                                variant_name: "Shutdown".into(),
                                names: no_field_names(),
                                fields: vec![],
                            })),
                        }
                    } else {
                        // ── Client peer arm: clients[k-1] fired (k = index, k ≥ 1) ──
                        // NB: process layout is 0=self-peer, 1..=N=clients (the listener
                        // is the accept-arm, NOT a recv index) → peer_idx = index - 1.
                        let peer_idx = (index.0 - 1) as i64;
                        match result {
                            Ok(raw_bytes) => {
                                // Arc 272 6b-ii-β: decode the raw wire bytes with
                                // decode_trusted_wire so user-defined enum/record values
                                // (e.g. Op::Increment(IncrementRequest{n:5})) are
                                // reconstructed correctly via the type registry.
                                let wire_str = std::str::from_utf8(&raw_bytes).map_err(|_| {
                                    EvalBreak::from(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: "poll (process tier): client message is not valid UTF-8".into(),
                                        }))
                                })?;
                                // Arc 278 no-hidden-failures — a client message we cannot
                                // decode is NOT service-fatal. On decode failure, build the
                                // rich reason as a first-class Failure and return
                                // ServiceEvent::Malformed{idx, cause} INSTEAD of raising (the
                                // old `?` here was the DoS: one bad message killed the whole
                                // service, and its reason vanished on the EPIPE'd err pipe).
                                // The serve loop replies the cause to THIS client (Reply::Failed)
                                // and keeps serving — the peer is ALIVE. Excursus 003 strike T2 —
                                // `decode_client_message_event` is the one door for this (the
                                // structured cause + the shape-defect lenient-retry-for-
                                // :RequestMalformed, Targets §1/§2); see its own doc.
                                decode_client_message_event(peer_idx, wire_str, sym, SELECT_EVENT_TYPE)
                            }
                            // Arc 278 Stone 1a — over-FOO is a 400-class CLIENT error, NOT a
                            // 500-class internal crash. A frame exceeding THIS service's declared
                            // hard frame limit `FOO` (RecvError::FrameTooLarge) routes to
                            // ServiceEvent::Rejected{idx, cause}: the serve loop TELLS that client
                            // (`Reply::Failed{cause}` via a non-blocking try-send'), EVICTS just that
                            // connection (discarding the un-read oversized residual that would desync
                            // the wire), and KEEPS SERVING everyone else. NOT the reason-free `Closed`
                            // (mute), NOT the terminal `Lost` (whose `eprintln` is wat's panic — a
                            // client-triggerable service crash = DoS). `message_only_failure` mirrors
                            // the Malformed construction above.
                            Err(crate::comms::RecvError::FrameTooLarge) => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE.into(),
                                variant_name: "Rejected".into(),
                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Rejected"),
                                fields: vec![
                                    Value::i64(peer_idx),
                                    message_only_failure("request too large — exceeded this service's \
                                         max-frame-bytes limit; request rejected, connection closed".to_string()),
                                ],
                            })),
                            // Genuine clean EOF (Disconnected / Shutdown), a reason-free abnormal
                            // reset (PeerCrashed — administrative, owner-crash-channel only), or a
                            // raw transport Failed(reason) (kept at the HEAD `Closed` path — its
                            // reason-surfacing is a separate stone, NOT this over-FOO disposition).
                            // A bare Peer' has no crash channel here → clean `Closed`.
                            // ServiceEvent::Closed [idx <- i64]
                            Err(_) => Value::Enum(Arc::new(EnumValue {
                                type_path: SELECT_EVENT_TYPE.into(),
                                variant_name: "Closed".into(),
                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Closed"),
                                fields: vec![Value::i64(peer_idx)],
                            })),
                        }
                    }
                }
                crate::comms::SelectOutcome::Listener => {
                    // ── Accept-arm: a client is dialing over the socket ────────
                    // Non-blocking accept loop (C0b.3a-i invariant: the listener fd
                    // is non-blocking; a spurious POLLIN → EWOULDBLOCK → re-accept).
                    // Mirrors SocketListener::accept but returns ServiceEvent::Connection.
                    use std::os::fd::OwnedFd;
                    loop {
                        match socket_listener.listener.accept() {
                            Ok((stream, _addr)) => {
                                // Arc 209 C0b.3b-b — THE GATE: the kernel vouches for the
                                // connector's {pid,uid,gid}; serve only an authorized one,
                                // else bounce the stranger (drop + re-accept).
                                let cred = crate::comms::process::peer_cred(stream.as_raw_fd())
                                    .map_err(|e| {
                                        RuntimeError::new(
                                            list_span.clone(),
                                            RuntimeErrorKind::MalformedForm {
                                                head: OP.into(),
                                                reason: format!(
                                            "poll (process tier): peer_cred on accepted socket: {}",
                                            e
                                        ),
                                            },
                                        )
                                    })?;
                                if !socket_listener.authorizes(&cred) {
                                    drop(stream); // bounce the stranger — close the accepted fd
                                    continue; // back to socket_listener.listener.accept()
                                }
                                let peer_value = {
                                    // Arc 258.5b-ii: reinterpret Sender<Value> as Sender<String>.
                                    // Arc 278 Stone 1: the accepted receiver reads client requests
                                    // at the service's declared hard frame limit `FOO`
                                    // (socket_listener.max_frame_bytes), NOT the global default.
                                    let (tx, rx) = crate::comms::process::sender_receiver_from_fd_with_budget::<
                                        Value,
                                    >(
                                        OwnedFd::from(stream),
                                        socket_listener.max_frame_bytes,
                                    )
                                    .map_err(|e| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: format!(
                                                "poll (process tier): wrap socket stream \
                                                 failed: {}",
                                                e
                                            ),
                                        }))?;
                                    use crate::kernel::peer::Peer;
                                    use crate::kernel::spawn::PEER_TYPE_PATH;
                                    use crate::rust_deps::custodia::ThreadOwnedCell;
                                    use crate::rust_deps::marshal::make_rust_opaque;
                                    make_rust_opaque(
                                        PEER_TYPE_PATH,
                                        Arc::new(ThreadOwnedCell::new(Some(Peer::from_socket(
                                            tx.reinterpret::<String>(),
                                            rx,
                                        )))),
                                    )
                                };
                                // ServiceEvent::Connection [peer <- (Peer' :- [I O])]
                                break Value::Enum(Arc::new(EnumValue {
                                    type_path: SELECT_EVENT_TYPE.into(),
                                    variant_name: "Connection".into(),
                                    names: builtin_enum_variant_names(
                                        SELECT_EVENT_TYPE,
                                        "Connection",
                                    ),
                                    fields: vec![peer_value],
                                }));
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // Spurious POLLIN — re-poll via another select iteration.
                                // Rebuild the Select with the same arms and retry.
                                // Arc 272 6b-ii-β: use select_raw() here too so client
                                // messages are decoded with decode_trusted_wire.
                                let mut sel2 = crate::comms::process::Select::<Value>::new();
                                sel2.recv(self_proc_rx);
                                for rx in &client_proc_rxs {
                                    sel2.recv(*rx);
                                }
                                sel2.listener(listen_raw_fd);
                                match sel2.select_raw().map_err(|io_err| {
                                    EvalBreak::from(RuntimeError::new(
                                        list_span.clone(),
                                        RuntimeErrorKind::MalformedForm {
                                            head: OP.into(),
                                            reason: format!(
                                                "poll (process tier) re-poll after \
                                                 WouldBlock: {}",
                                                io_err
                                            ),
                                        },
                                    ))
                                })? {
                                    crate::comms::SelectOutcome::Listener => continue,
                                    other => {
                                        // Another arm fired during re-poll — recurse via
                                        // returning the event: re-enter on next call.
                                        // This is conservative: hand back a non-Listener
                                        // outcome by converting it to an event value.
                                        // Actually we need to handle the non-listener arm
                                        // here. But this path is hit extremely rarely
                                        // (spurious POLLIN on a non-blocking UDS listener);
                                        // return the fired event properly.
                                        let ev = match other {
                                            crate::comms::SelectOutcome::Shutdown => {
                                                return Err(RuntimeError::new(
                                                    list_span.clone(),
                                                    RuntimeErrorKind::MalformedForm {
                                                        head: OP.into(),
                                                        reason: "poll interrupted by substrate \
                                                                 shutdown (re-poll)"
                                                            .into(),
                                                    },
                                                )
                                                .into());
                                            }
                                            crate::comms::SelectOutcome::Recv {
                                                index: idx2,
                                                result: res2,
                                            } => {
                                                if idx2.0 == 0 {
                                                    Value::Enum(Arc::new(EnumValue {
                                                        type_path: SELECT_EVENT_TYPE.into(),
                                                        variant_name: "Shutdown".into(),
                                                        names: no_field_names(),
                                                        fields: vec![],
                                                    }))
                                                } else {
                                                    let pidx = (idx2.0 - 1) as i64;
                                                    match res2 {
                                                        Ok(raw_bytes2) => {
                                                            // Arc 272 6b-ii-β: decode with
                                                            // trusted wire for user-defined types.
                                                            let ws2 = std::str::from_utf8(&raw_bytes2).map_err(|_| {
                                                                EvalBreak::from(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                                                                        head: OP.into(),
                                                                        reason: "poll (process tier re-poll): client message is not valid UTF-8".into(),
                                                                    }))
                                                            })?;
                                                            // Arc 278 no-hidden-failures — a client
                                                            // message we cannot decode is NOT
                                                            // service-fatal: return Malformed{idx,cause}
                                                            // instead of raising (mirrors the main
                                                            // client arm above). Excursus 003 strike
                                                            // T2 — `decode_client_message_event` is
                                                            // the one door (structured cause +
                                                            // shape-defect lenient retry); see its doc.
                                                            decode_client_message_event(pidx, ws2, sym, SELECT_EVENT_TYPE)
                                                        }
                                                        // Arc 278 Stone 1a — over-FOO → Rejected
                                                        // here too (parity with the main client
                                                        // arm), so an over-budget frame is never
                                                        // muted even on the rare re-poll path.
                                                        Err(crate::comms::RecvError::FrameTooLarge) => Value::Enum(Arc::new(EnumValue {
                                                            type_path: SELECT_EVENT_TYPE.into(),
                                                            variant_name: "Rejected".into(),
                                                            names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Rejected"),
                                                            fields: vec![
                                                                Value::i64(pidx),
                                                                message_only_failure("request too large — exceeded this service's max-frame-bytes limit; request rejected, connection closed".to_string()),
                                                            ],
                                                        })),
                                                        Err(_) => Value::Enum(Arc::new(
                                                            EnumValue {
                                                                type_path: SELECT_EVENT_TYPE
                                                                    .into(),
                                                                variant_name: "Closed".into(),
                                                                names: builtin_enum_variant_names(SELECT_EVENT_TYPE, "Closed"),
                                                                fields: vec![Value::i64(pidx)],
                                                            },
                                                        )),
                                                    }
                                                }
                                            }
                                            crate::comms::SelectOutcome::Listener => {
                                                unreachable!()
                                            }
                                        };
                                        break ev;
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(RuntimeError::new(
                                    list_span.clone(),
                                    RuntimeErrorKind::MalformedForm {
                                        head: OP.into(),
                                        reason: format!(
                                            "poll (process tier): non-blocking accept \
                                             failed: {}",
                                            e
                                        ),
                                    },
                                )
                                .into());
                            }
                        }
                    }
                }
            };
            Ok(event_value)
        }
    }
}

#[cfg(test)]
mod excursus_003_t2_gates {
    //! Excursus 003 strike T2 — GT2a/GT2b/GT2c, mutation-proven in RELEASE.
    //!
    //! Builds a REAL frozen world (a tiny `defsurface`/`defservice`, via `startup_from_source` —
    //! the SAME macro-expansion path a production `defservice` goes through), then constructs
    //! the malformed WIRE BYTES directly in Rust — never through a wat-level `:wat::edn::read`,
    //! which strict decode now refuses by design (see the strike's report for the raw-frame-door
    //! finding this sidesteps). The wrongly-shaped `Value` is built with the raw
    //! `AggregateValue`/`EnumValue` constructors (Rust is not bound by the wat type checker) and
    //! serialized with the REAL writer (`value_to_edn_with`) — exactly the bytes a malicious
    //! client's socket would carry, never a hand-typed literal and never an in-process Value
    //! handed to the server directly.
    use super::*;
    use crate::edn::render::{decode_trusted_wire, value_to_wire_edn_string};
    use crate::freeze::{startup_from_source, FrozenWorld};
    use crate::load::loader::InMemoryLoader;
    use crate::value::value::AggregateValue;

    const SURFACE_SRC: &str = r#"
(:wat::core::defsurface :t::edn2::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :t::edn2::Bag::PutRequest [items <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :t::edn2::Bag::PutResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :t::edn2::Bag  req <- :t::edn2::Bag::PutRequest]
     -> :t::edn2::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :t::edn2::bag-svc
  :satisfies :t::edn2::Bag
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :impls
  [(put [s ctx req]
     (:wat::service::Outcome.Reply {:state s :reply (:t::edn2::Bag::PutResponse.Ok {:n 0})}))])
"#;

    pub(super) fn frozen_world() -> FrozenWorld {
        startup_from_source(
            SURFACE_SRC,
            Some(concat!(file!(), ":", line!())),
            Arc::new(InMemoryLoader::new()),
        )
        .expect("the tiny Bag surface+service must freeze cleanly")
    }

    /// A wrongly-shaped `Op::Put{req: PutRequest{items: [1,2,3]}}`, serialized through the REAL
    /// writer. `items` is declared `Vector<String>`; this hands it integers.
    pub(super) fn malformed_put_wire(types: &crate::types::TypeEnv) -> String {
        let bad_items = Value::Vec(Arc::new(vec![Value::i64(1), Value::i64(2), Value::i64(3)]));
        let bad_req = Value::Aggregate(Arc::new(AggregateValue::record(
            "t::edn2::Bag::PutRequest".to_string(),
            Arc::new(vec!["items".to_string()]),
            Arc::new(vec![bad_items]),
        )));
        let bad_op = Value::Enum(Arc::new(EnumValue {
            type_path: ":t::edn2::Bag::Op".to_string(),
            variant_name: "Put".to_string(),
            names: Arc::new(vec!["req".to_string()]),
            fields: vec![bad_req],
        }));
        value_to_wire_edn_string(&bad_op, Some(types))
            .expect("the wire encoder must render a Value it was handed")
            .into_string()
    }

    /// GT2a — the decode cause is STRUCTURED. The malformed frame's strict decode fails with
    /// `FieldTypeMismatch`, classified a shape defect, and `.to_record()` is the declared
    /// `:wat::edn::FieldTypeMismatch` record — not a stringified Fault.
    /// Mutation: in `EdnReadError::to_record`, change the `FieldTypeMismatch` arm to build a
    /// `wat::edn::ReadError` (or any other class) instead. RED.
    #[test]
    fn gt2a_decode_cause_is_structured_not_prose() {
        let w = frozen_world();
        let wire = malformed_put_wire(&w.types);
        let err = decode_trusted_wire(&wire, Some(&w.types), w.symbols.encoding_ctx().map(|a| a.as_ref()))
            .expect_err("a wrong-typed items field must be strictly refused");
        assert!(
            err.kind.is_shape_defect(),
            "a FieldTypeMismatch must be classified a shape defect; got {:?}",
            err.kind
        );
        match err.to_record() {
            Value::Aggregate(a) => assert_eq!(
                a.class.as_ref(),
                "wat::edn::FieldTypeMismatch",
                "the structured cause must be the declared wat record, never a Fault/prose"
            ),
            other => panic!("expected Value::Aggregate(FieldTypeMismatch), got {other:?}"),
        }
    }

    // GT2b RETIRED by strike T3 (RULING 2026-10-01): it asserted the LENIENT RETRY
    // (`decode_trusted_wire_lenient`) restored a bindable `req` so `ServiceEvent::Message` fired
    // — exactly the hole T3 closes (Strike T's undeclared-key wall was reopened by that retry).
    // T3 deletes the lenient door outright: the SAME wire this test built now yields
    // `ServiceEvent::RequestMalformed` directly (never `Message`), proven below by
    // `excursus_003_t3_gates::gt3b_wrong_field_type_gives_the_contract`, which reuses
    // `malformed_put_wire` and asserts the `(path, expected, got)` triple against the live
    // `:wat::edn::validate` door instead of a lenient-decode round-trip.

    /// GT2c — a non-shape failure (an unknown tag — the message never identifies any op) stays
    /// the generic `ServiceEvent::Malformed`, never retried (there is no `req` to validate
    /// because there is no op), and its cause is STILL structured (Target §1 covers EVERY decode
    /// failure, not only the shape-class ones `is_shape_defect` singles out for the retry).
    /// Mutation: in `decode_client_message_event`'s `Err(e) =>` arm, build
    /// `crate::runtime::message_only_failure(format!("{e}"))` instead of
    /// `crate::runtime::edn_read_error_failure(&e)`. RED — the cause reverts to a `wat::core::
    /// Fault` carrying flattened prose, exactly the regression Target §1 exists to catch.
    #[test]
    fn gt2c_non_shape_failure_stays_generic_malformed_with_structured_cause() {
        let w = frozen_world();
        let wire = "#t.edn2.Bag/Op.NoSuchOp {:req {:items []}}".to_string();
        let event = decode_client_message_event(3, &wire, &w.symbols, ":wat::spawn::ServiceEvent");
        match event {
            Value::Enum(e) if e.variant_name == "Malformed" => {
                assert_eq!(e.fields[0], Value::i64(3));
                match &e.fields[1] {
                    Value::Aggregate(failure) => match &failure.fields[0] {
                        // `Op` IS a registered enum — only the variant name `NoSuchOp` is
                        // unregistered — so this is `EnumVariantNotFound`, not `UnknownTag`.
                        Value::Aggregate(err_rec) => assert_eq!(
                            err_rec.class.as_ref(),
                            "wat::edn::EnumVariantNotFound",
                            "a non-shape refusal's cause must still be a structured :wat::edn:: record"
                        ),
                        other => panic!("expected Failure.error to be a structured Aggregate, got {other:?}"),
                    },
                    other => panic!("expected ServiceEvent::Malformed.cause to be a Failure Aggregate, got {other:?}"),
                }
            }
            other => panic!(
                "expected ServiceEvent::Malformed (an unknown op must never be retried, since \
                 there is no req to validate), got {other:?}"
            ),
        }
    }
}

#[cfg(test)]
mod excursus_003_t3_gates {
    //! Excursus 003 strike T3 — GT3a/GT3b/GT3c/GT3d, mutation-proven in RELEASE.
    //!
    //! Reuses `excursus_003_t2_gates::{frozen_world, malformed_put_wire}` (the identical tiny
    //! `Bag` surface+service) — one fixture, two strikes' worth of gates over it.
    use super::excursus_003_t2_gates::{frozen_world, malformed_put_wire};
    use super::*;
    use crate::ast::WatAST;
    use crate::edn::render::{decode_trusted_wire, edn_to_typed_value};
    use crate::types::TypeExpr;
    use crate::value::{Function, FunctionBody};

    /// GT3a — an undeclared key on the wire (the orchestrator's exact probe quoted in the
    /// brief: valid `items`, plus a stray `:stray 1`) is refused as `RequestMalformed`, not
    /// silently laundered into `Message` by a lenient retry.
    /// Mutation: restore a lenient retry (reinstate `decode_trusted_wire_lenient` and the
    /// `Err(e) if e.kind.is_shape_defect() => decode_trusted_wire_lenient(...)` arm inside
    /// `decode_client_message_event`). RED — the stray key gets silently dropped by the
    /// lenient re-decode and the event reverts to `ServiceEvent::Message`, exactly the hole
    /// this strike closes (driven and confirmed during this strike's own verification).
    #[test]
    fn gt3a_undeclared_key_is_refused_on_the_wire() {
        let w = frozen_world();
        let wire = r#"#t.edn2.Bag/Op.Put {:req #t.edn2.Bag/PutRequest {:items ["a"] :stray 1}}"#;
        let event = decode_client_message_event(9, wire, &w.symbols, ":wat::spawn::ServiceEvent");
        match &event {
            Value::Enum(e) if e.variant_name == "RequestMalformed" => {
                assert_eq!(e.fields[0], Value::i64(9), "idx must round-trip");
                assert_eq!(e.fields[1], Value::String(Arc::new("Put".to_string())), "op must name the variant");
                match &e.fields[2] {
                    Value::Vec(segs) => assert!(segs.is_empty(), "the unknown-field refusal has no sub-path: {segs:?}"),
                    other => panic!("expected path Vector, got {other:?}"),
                }
                assert_eq!(e.fields[3], Value::String(Arc::new(":t::edn2::Bag::PutRequest".to_string())));
                assert_eq!(e.fields[4], Value::String(Arc::new("unknown field :stray".to_string())));
            }
            Value::Enum(e) => panic!(
                "expected ServiceEvent::RequestMalformed (the undeclared key must be refused, \
                 not laundered by a lenient retry); got ServiceEvent::{}",
                e.variant_name
            ),
            other => panic!("expected Value::Enum(ServiceEvent), got {other:?}"),
        }
    }

    /// GT3b — a wrong field type gives the SAME contract `:wat::edn::validate` would. Two
    /// INDEPENDENT computations over the SAME malformed wire: (1) the production door
    /// (`decode_client_message_event` → `shape_defect_request_malformed`) and (2) this test's
    /// own direct call to `edn_to_typed_value` (the identical walker `:wat::edn::validate`
    /// calls) against the `:req` sub-tree parsed out of the SAME wire bytes — both calling the
    /// ONE shared formatter (`edn_to_typed_value_inner`), never a hand-converted string.
    /// Mutation: inside `shape_defect_request_malformed`, replace the
    /// `edn_to_typed_value_inner(req_ty, req_edn, ...)` call with a hand-rolled rendering (e.g.
    /// T's own `Mismatch`-style `got`/`path` convention). RED — this test's independently
    /// computed `(path, expected, got)` stops matching the production door's.
    #[test]
    fn gt3b_wrong_field_type_gives_the_contract() {
        let w = frozen_world();
        let wire = malformed_put_wire(&w.types);

        // (1) the production door.
        let event = decode_client_message_event(7, &wire, &w.symbols, ":wat::spawn::ServiceEvent");
        let (op, path, expected, got) = match &event {
            Value::Enum(e) if e.variant_name == "RequestMalformed" => {
                let op = match &e.fields[1] {
                    Value::String(s) => s.as_str().to_string(),
                    other => panic!("expected op String, got {other:?}"),
                };
                let path: Vec<String> = match &e.fields[2] {
                    Value::Vec(segs) => segs
                        .iter()
                        .map(|v| match v {
                            Value::String(s) => s.as_str().to_string(),
                            other => panic!("expected path segment String, got {other:?}"),
                        })
                        .collect(),
                    other => panic!("expected path Vector, got {other:?}"),
                };
                let expected = match &e.fields[3] {
                    Value::String(s) => s.as_str().to_string(),
                    other => panic!("expected expected String, got {other:?}"),
                };
                let got = match &e.fields[4] {
                    Value::String(s) => s.as_str().to_string(),
                    other => panic!("expected got String, got {other:?}"),
                };
                (op, path, expected, got)
            }
            other => panic!("expected ServiceEvent::RequestMalformed, got {other:?}"),
        };
        assert_eq!(op, "Put");

        // (2) the independent validate-door computation: parse the SAME wire, pull the raw
        // `:req` sub-tree, and walk it against the declared PutRequest type directly.
        let parsed = wat_edn::parse_owned(&wire).expect("the real writer's own output must re-parse");
        let req_edn = match &parsed {
            wat_edn::Value::Tagged(_, body) => match body.as_ref() {
                wat_edn::Value::Map(entries) => entries
                    .iter()
                    .find_map(|(k, v)| match k {
                        wat_edn::Value::Keyword(kw) if kw.name() == "req" => Some(v),
                        _ => None,
                    })
                    .expect(":req key must be present"),
                other => panic!("expected Tagged-body Map, got {other:?}"),
            },
            other => panic!("expected Tagged, got {other:?}"),
        };
        let target = TypeExpr::Path(":t::edn2::Bag::PutRequest".to_string());
        let mismatch = edn_to_typed_value(&target, req_edn, &w.symbols)
            .expect_err("the wrong-typed items field must still be refused by validate");

        assert_eq!(path, wat_reader::identifier::dot_path_segments(&mismatch.path));
        assert_eq!(expected, mismatch.expected);
        assert_eq!(got, mismatch.got);
    }

    /// GT3c — the codegen replies `:RequestMalformed`.
    ///
    /// **The door, measured, not assumed.** `ServiceEvent::RequestMalformed` is built ONLY by
    /// `decode_client_message_event` on a SHAPE-class refusal from the process-tier socket
    /// decode (`eval_poll_prime`'s client-peer arm, `src/kernel/message.rs`) — the thread tier
    /// passes `Value`s through crossbeam VERBATIM and never decodes a wire at all (this file's
    /// own doc, `wat/spawn.wat`'s `ServiceEvent` doc). Reaching it honestly therefore requires
    /// a REAL spawned process-tier `defservice`'s ACCEPTED socket connection, fed genuinely
    /// malformed BYTES from outside wat's typed layer (the checker refuses constructing a
    /// wrong-shaped request value in wat at all — T's own measurement). T2 measured this exact
    /// gap and did not mint it: "a Rust-level helper reaching a real spawned `defservice`'s
    /// accepted socket from outside wat — new kernel/test surface this strike does not mint
    /// without the builder's sign-off" (`wat-tests/service-request-malformed.wat`'s retired-probe
    /// comment). This strike's own scope fence lists the SAME gap OUT: "a raw-socket test door
    /// (still missing, flagged by T2)". Confirmed independently here: no Rust or wat-level test
    /// surface in this tree reaches a spawned defservice's accepted connection's raw fd, and the
    /// thread tier structurally cannot produce this event (no decode occurs on it at all).
    ///
    /// **So: no honest in-process door reaches the REAL `poll'`-triggered serve loop with a
    /// genuinely malformed frame.** This gate instead runs the REAL macro expansion
    /// (`startup_from_source`, the exact pipeline a production `defservice` goes through) and
    /// extracts the GENERATED `:t::edn2::bag-svc::serve` function's own AST — never a hand
    /// copy — to assert STRUCTURALLY that the `Put` op's `ServiceEvent.RequestMalformed`
    /// dispatch arm sends that op's OWN `:RequestMalformed` built from the reply-variant /
    /// RequestMalformed-variant constructors (the SAME ones `shape-guarded`'s pre-existing
    /// `:wat::edn::validate` Invalid arm already uses), never the generic `Reply::Failed`.
    /// Mutation: in `wat/service.wat`'s `rm-send-arm` template, replace
    /// `(~rm-ctor-kw {:path rm-path :expected rm-expected :got rm-got})` with
    /// `(~reply-failed-kw {:cause ...})` (the brief's own named mutation: "send the event to the
    /// generic Reply::Failed arm"). RED — the extracted AST's `:resp` construction no longer
    /// names a `.RequestMalformed`-suffixed ctor with `path`/`expected`/`got` fields.
    #[test]
    fn gt3c_codegen_replies_requestmalformed() {
        let w = frozen_world();
        let serve_fn: std::sync::Arc<Function> = w
            .symbols
            .functions_iter()
            .find(|(name, _)| name.as_str() == ":t::edn2::bag-svc::serve")
            .unwrap_or_else(|| panic!("the generated serve fn must be registered"))
            .1
            .clone();
        let FunctionBody::Wat(body) = &serve_fn.body else {
            panic!("serve's body must be a wat AST, not a native stub");
        };

        let rm_arm = find_bracket_arm_by_keyword_head(body, ":wat::spawn::ServiceEvent.RequestMalformed")
            .unwrap_or_else(|| panic!("serve's top-level match must carry a RequestMalformed arm"));
        let put_arm = find_bracket_arm_by_string(rm_arm, "Put")
            .unwrap_or_else(|| panic!("the RequestMalformed op-dispatch must carry a \"Put\" arm"));
        let WatAST::List(put_items, _) = put_arm else {
            panic!("the \"Put\" arm's body must be a (match (send ...) ...) List, got {put_arm:?}");
        };
        let send_call = put_items.get(1).unwrap_or_else(|| panic!("match form has no scrutinee: {put_arm:?}"));
        let WatAST::List(send_items, _) = send_call else {
            panic!("the match scrutinee must be a (send ...) call, got {send_call:?}");
        };
        assert!(
            matches!(&send_items[0], WatAST::Keyword(k, _) if k == ":wat::kernel::send"),
            "expected a (:wat::kernel::send ...) call, got {send_items:?}"
        );
        let reply_call = send_items.get(2).unwrap_or_else(|| panic!("send call has no reply arg: {send_items:?}"));
        let WatAST::List(reply_items, _) = reply_call else {
            panic!("the reply must be a constructor call, got {reply_call:?}");
        };
        let reply_kw = match &reply_items[0] {
            WatAST::Keyword(k, _) => k.clone(),
            other => panic!("expected the reply ctor's keyword head, got {other:?}"),
        };
        // Exact, not loose: `compose-variant`'s own rendering (`wat_reader::identifier::
        // compose_variant`) is `<ns>.<Variant>`, byte-identical — derivable, not guessed.
        assert_eq!(reply_kw, ":t::edn2::Bag::Reply.Put", "expected the Put reply variant exactly");
        let WatAST::Map(reply_fields, _) = &reply_items[1] else {
            panic!("expected the reply ctor's {{:resp ...}} map, got {:?}", reply_items[1]);
        };
        let resp_val = reply_fields
            .iter()
            .find_map(|(k, v)| matches!(k, WatAST::Keyword(kk, _) if kk == ":resp").then_some(v))
            .unwrap_or_else(|| panic!("reply ctor map has no :resp key: {reply_fields:?}"));
        let WatAST::List(rm_items, _) = resp_val else {
            panic!(":resp value must be a constructor call, got {resp_val:?}");
        };
        let rm_kw = match &rm_items[0] {
            WatAST::Keyword(k, _) => k.clone(),
            other => panic!("expected the :resp ctor's keyword head, got {other:?}"),
        };
        assert_eq!(
            rm_kw, ":t::edn2::Bag::PutResponse.RequestMalformed",
            "the Put op's RequestMalformed event must reply THAT op's own :RequestMalformed \
             variant (reusing shape-guarded's ctor) — got {rm_kw}, which looks like the generic \
             Reply::Failed collapse the mutation names"
        );
        let WatAST::Map(rm_fields, _) = &rm_items[1] else {
            panic!("expected the RequestMalformed ctor's field map, got {:?}", rm_items[1]);
        };
        let field_names: Vec<String> = rm_fields
            .iter()
            .filter_map(|(k, _)| match k {
                WatAST::Keyword(kk, _) => Some(kk.clone()),
                _ => None,
            })
            .collect();
        for expected_field in [":path", ":expected", ":got"] {
            assert!(
                field_names.iter().any(|f| f == expected_field),
                "RequestMalformed ctor must carry {expected_field}, got fields {field_names:?}"
            );
        }
    }

    /// Structural AST search: the first match arm (inside `match_form`, a
    /// `(:wat::core::match scrutinee arm...)` List) whose pattern is headed by the keyword
    /// `head` — either a bare unit-variant keyword or a `(keyword {...})` tagged-variant
    /// pattern. Returns that arm's BODY (never the pattern).
    fn find_bracket_arm_by_keyword_head<'a>(match_form: &'a WatAST, head: &str) -> Option<&'a WatAST> {
        let WatAST::List(items, _) = match_form else { return None };
        items.iter().skip(2).find_map(|arm| {
            let WatAST::Vector(parts, _) = arm else { return None };
            // A tagged-variant pattern with a field-destructure map is a 3-element arm vector
            // `[Keyword {field-map} body]` (the map IS part of the pattern); a unit/bare-keyword
            // pattern with no destructure is 2 elements `[Keyword body]`.
            match parts.as_slice() {
                [WatAST::Keyword(k, _), WatAST::Map(_, _), body] if k == head => Some(body),
                [WatAST::Keyword(k, _), body] if k == head => Some(body),
                _ => None,
            }
        })
    }

    /// Structural AST search: the first match arm whose pattern is the STRING LITERAL `s` —
    /// `rm-serve-arms`' own per-op dispatch convention (service.wat).
    fn find_bracket_arm_by_string<'a>(match_form: &'a WatAST, s: &str) -> Option<&'a WatAST> {
        let WatAST::List(items, _) = match_form else { return None };
        items.iter().skip(2).find_map(|arm| {
            let WatAST::Vector(parts, _) = arm else { return None };
            if parts.len() != 2 {
                return None;
            }
            matches!(&parts[0], WatAST::StringLit(lit, _) if lit == s).then_some(&parts[1])
        })
    }

    /// Structural AST search: the first match arm whose pattern is the bare wildcard `_` —
    /// `rm-op`'s own unreachable-op fallback convention (service.wat).
    fn find_bracket_arm_by_wildcard<'a>(match_form: &'a WatAST) -> Option<&'a WatAST> {
        let WatAST::List(items, _) = match_form else { return None };
        items.iter().skip(2).find_map(|arm| {
            let WatAST::Vector(parts, _) = arm else { return None };
            if parts.len() != 2 {
                return None;
            }
            matches!(&parts[0], WatAST::Symbol(id, _) if id.as_str() == "_").then_some(&parts[1])
        })
    }

    /// Does `node`, or anything nested inside it, carry the keyword `kw` as a call head /
    /// literal? Generic recursive search via [`WatAST::children`] — used to prove a subtree
    /// carries NO `:wat::kernel::assertion-failed!` anywhere, not merely "not at the top".
    fn ast_contains_keyword(node: &WatAST, kw: &str) -> bool {
        if matches!(node, WatAST::Keyword(k, _) if k == kw) {
            return true;
        }
        node.children().iter().any(|c| ast_contains_keyword(c, kw))
    }

    /// Excursus 003 strike B3 item 5(a) — the unreachable-op fallback arm of the
    /// `ServiceEvent.RequestMalformed` op-dispatch (`rm-op`'s trailing `_` arm,
    /// `wat/service.wat`) is a VALUE, not a panic. Same door as GT3c above: the generated
    /// `:t::edn2::bag-svc::serve` function's OWN AST, extracted structurally — never a hand
    /// copy — must show the fallback arm sending a generic `Reply::Failed` built from a
    /// `Fault` naming the impossible op, and must carry NO `:wat::kernel::assertion-failed!`
    /// anywhere in its body (the panic class this item kills).
    ///
    /// Mutation (recorded in the strike report): revert `wat/service.wat`'s fallback arm back
    /// to `(:wat::kernel::assertion-failed! ...)` — RED, `ast_contains_keyword` finds it.
    #[test]
    fn b3_item5a_unreachable_op_arm_replies_failed_not_panic() {
        let w = frozen_world();
        let serve_fn: std::sync::Arc<Function> = w
            .symbols
            .functions_iter()
            .find(|(name, _)| name.as_str() == ":t::edn2::bag-svc::serve")
            .unwrap_or_else(|| panic!("the generated serve fn must be registered"))
            .1
            .clone();
        let FunctionBody::Wat(body) = &serve_fn.body else {
            panic!("serve's body must be a wat AST, not a native stub");
        };

        let rm_arm = find_bracket_arm_by_keyword_head(body, ":wat::spawn::ServiceEvent.RequestMalformed")
            .unwrap_or_else(|| panic!("serve's top-level match must carry a RequestMalformed arm"));
        let WatAST::List(rm_items, _) = rm_arm else {
            panic!("the RequestMalformed arm's body must be a (match rm-op ...) List, got {rm_arm:?}");
        };
        assert!(
            matches!(&rm_items[0], WatAST::Keyword(k, _) if k == ":wat::core::match"),
            "expected a (:wat::core::match rm-op ...) form, got {rm_items:?}"
        );
        let fallback_arm = find_bracket_arm_by_wildcard(rm_arm)
            .unwrap_or_else(|| panic!("the rm-op dispatch must carry a trailing `_` fallback arm"));

        assert!(
            !ast_contains_keyword(fallback_arm, ":wat::kernel::assertion-failed!"),
            "the unreachable-op fallback arm must not call assertion-failed! anywhere — got {fallback_arm:?}"
        );

        let WatAST::List(let_items, _) = fallback_arm else {
            panic!("the fallback arm's body must be a (let [...] (match (send ...) ...)) List, got {fallback_arm:?}");
        };
        assert!(
            matches!(&let_items[0], WatAST::Keyword(k, _) if k == ":wat::core::let"),
            "expected a (:wat::core::let [...] ...) form, got {let_items:?}"
        );
        let inner_match = let_items.get(2).unwrap_or_else(|| panic!("let form has no body: {let_items:?}"));
        let WatAST::List(inner_items, _) = inner_match else {
            panic!("the let body must be a (match (send ...) ...) List, got {inner_match:?}");
        };
        let send_call = inner_items.get(1).unwrap_or_else(|| panic!("match form has no scrutinee: {inner_match:?}"));
        let WatAST::List(send_items, _) = send_call else {
            panic!("the match scrutinee must be a (send ...) call, got {send_call:?}");
        };
        assert!(
            matches!(&send_items[0], WatAST::Keyword(k, _) if k == ":wat::kernel::send"),
            "expected a (:wat::kernel::send ...) call, got {send_items:?}"
        );
        let reply_call = send_items.get(2).unwrap_or_else(|| panic!("send call has no reply arg: {send_items:?}"));
        let WatAST::List(reply_items, _) = reply_call else {
            panic!("the reply must be a constructor call, got {reply_call:?}");
        };
        let reply_kw = match &reply_items[0] {
            WatAST::Keyword(k, _) => k.clone(),
            other => panic!("expected the reply ctor's keyword head, got {other:?}"),
        };
        assert_eq!(reply_kw, ":t::edn2::Bag::Reply.Failed", "expected the generic Reply::Failed variant");
    }

    /// GT3d — one door. A grep proves nothing (it cannot see whether two decode sites share an
    /// implementation or merely look similar); this drives it: the GENERAL untrusted door
    /// (`read_edn`) and the TRUSTED wire door (`decode_trusted_wire`) are handed the IDENTICAL
    /// malformed bytes and must refuse with the IDENTICAL structured kind — proof they share
    /// the ONE `reconstruct_*` family (`allow_caps` is the only difference between the two
    /// doors now; strictness is no longer a second axis). T's GT1 (`value_conforms`'s own
    /// pinned arms, `src/edn/render.rs`) and T2's GT2a/GT2c (this file, just above) run
    /// unmodified in the SAME floor — their continued green is this gate's other half.
    /// Mutation: reintroduce a `strict: bool` parameter on `reconstruct_record` defaulting
    /// `read_edn`'s call to lenient while leaving `decode_trusted_wire`'s strict (a literal
    /// restoration of the two-door split). RED — the two doors' refusals diverge (one refuses,
    /// one silently accepts).
    #[test]
    fn gt3d_one_door_general_and_trusted_decode_agree() {
        let w = frozen_world();
        let wire = malformed_put_wire(&w.types);

        let trusted_err = decode_trusted_wire(&wire, Some(&w.types), w.symbols.encoding_ctx().map(|a| a.as_ref()))
            .expect_err("the trusted door must refuse the wrong-typed items field");
        let general_err = crate::edn::render::read_edn(&wire, Some(&w.types), w.symbols.encoding_ctx().map(|a| a.as_ref()))
            .expect_err("the general untrusted door must refuse the IDENTICAL value — one decode door, not two");

        match (&trusted_err.kind, &general_err.kind) {
            (
                crate::edn::render::EdnReadErrorKind::FieldTypeMismatch { type_path: tp1, field: f1, expected: e1, got: g1 },
                crate::edn::render::EdnReadErrorKind::FieldTypeMismatch { type_path: tp2, field: f2, expected: e2, got: g2 },
            ) => {
                assert_eq!(tp1, tp2);
                assert_eq!(f1, f2);
                assert_eq!(e1, e2);
                assert_eq!(g1, g2);
            }
            other => panic!(
                "expected both doors to refuse with the IDENTICAL FieldTypeMismatch — one shared \
                 reconstruct_* family, not two decoders; got {other:?}"
            ),
        }
    }
}

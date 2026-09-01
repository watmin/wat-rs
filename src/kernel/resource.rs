//! Kernel sub-module mirroring `src/intrinsic/kernel/resource.rs` — arc 109
//! Stone B (the seven kernel sub-modules). Twelve items backing the edge
//! file's fourteen verbs (`pipe`/`spawn-thread`/`spawn-process` delegate
//! elsewhere already — `src/io.rs` and `src/kernel/spawn.rs` — and are not
//! this module's): `HandlePool::{new,pop,finish}`, `after`, `close`,
//! `signal`, `listener`, `connect`, `accept`, `allow`, `deny`, plus
//! `SIGNAL_TYPE`.
//!
//! `SIGNAL_TYPE` is the type path of the closed `:wat::kernel::Signal`
//! argument enum `eval_signal` decodes — it sat mid-block among stone A's
//! outcome-vocabulary consts (deliberately left there; see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-B-the-seven-kernel-submodules.md`)
//! because it names an argument type, not an outcome constructor. It has
//! exactly one consumer, `eval_signal` below, so it comes here with it.
//!
//! `eval_listener_prime`'s thread-tier arm calls `bound_names()` (the
//! `:wat::spawn::Bound` field-name helper) twice — that helper is not one
//! of this module's twelve items; it landed in `src/kernel/source.rs`
//! (the source-position family's own private helper) because that is
//! where the brief's item list puts it. A cross-module `use` reaches it
//! back from here.
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::declare::parse::is_type_arg_shaped;
use crate::kernel::outcome::{
    close_outcome_closed, close_outcome_failed, close_outcome_signaled, signal_outcome_delivered,
    signal_outcome_failed,
};
use crate::kernel::source::bound_names;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    AggregateValue, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use std::sync::Arc;
use wat_macros::restricted_to;

/// Arc 209 C0b.2c / C0b.2e-i-b / Arc 258.5b-ii — wrap a connected `UnixStream` as a
/// `(:wat::kernel::listener host …)` — Arc 209 Stone C0b.1 / C0b.2c / C0b.2d.
///
/// Thread tier (C0b.1): `(listener' (thread) :S :R)` — mints a crossbeam rendezvous
/// channel and returns `Tuple[(Listener' :- [S R]), (Address' :- [S R])]` (raw Receiver / raw Sender).
/// 3 args: host, :S, :R.
///
/// Process tier (C0b.2d → arc 272): `(listener' (process) :S :R)` — autobinds an abstract-namespace
/// UDS (kernel-minted, exclusive-bind, not a chosen name) and returns `Bound{ listener, address }`
/// mirroring the thread tier. 3 args: host, :S, :R. The legacy 2-arg named form (`socket-address'`
/// opaque) was annihilated in arc 272 step 5 (guessable names → squattable; autobind is the only
/// rendezvous). The SO_PEERCRED uid+pid checks are the security; the autobind name is an
/// exclusive-bind rendezvous token, not a secret.
///
/// The host value (args[0]) is evaluated at runtime to dispatch between tiers; arity is
/// validated AFTER host dispatch (thread=3, process=2).
pub(crate) fn eval_listener_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::listener";
    // Need at least the host arg to dispatch.
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: 0,
            },
        )
        .into());
    }
    // Evaluate the host to dispatch between thread and process tiers.
    let host_val = eval_inner(&args[0], env, sym)?.value_owned();
    let is_process = matches!(&host_val,
        Value::Aggregate(a) if a.class.as_ref() == "wat::spawn::ProcessOpts");

    if is_process {
        // Arc 272 — 3-arg AUTOBIND form `(listener' (process) :S :R)`: mint a kernel-unique,
        // exclusive-bind abstract address (kernel-minted, not a chosen name → no collision, no
        // squatting) and return `(Bound :- [S R]){listener, address}`, MIRRORING the thread tier.
        // The address is the capability `connect'` dials. (The 2-arg `(host addr)` named form
        // below is LEGACY — annihilated in arc 272 step 5 with the rest of the name-discovery
        // stack.) The SO_PEERCRED uid+pid checks are the security; the autobind name is the
        // exclusive-bind rendezvous token, not a secret.
        // Arc 278 Stone 1 — the process form accepts an OPTIONAL 4th arg: the
        // service's declared hard frame limit `FOO` (bytes-per-read), threaded to
        // the accepted-connection receivers via `SocketListener`. 3 args → default
        // `DEFAULT_MAX_FRAME_BYTES` (512 KiB); 4 args → the declared `FOO`.
        if args.len() == 3 || args.len() == 4 {
            for i in [1usize, 2usize] {
                if !is_type_arg_shaped(&args[i]) {
                    return Err(RuntimeError::new(
                        args[i].span().clone(),
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
            // Evaluate the optional per-service frame budget `FOO` (arg 3).
            let max_frame_bytes: usize = if args.len() == 4 {
                match eval_inner(&args[3], env, sym)?.value_owned() {
                    Value::i64(n) if n > 0 => n as usize,
                    other => {
                        return Err(RuntimeError::new(args[3].span().clone(), RuntimeErrorKind::MalformedForm {
                                head: OP.into(),
                                reason: format!(
                                    "argument 3 (:max-frame-bytes FOO) must be a positive i64; got {:?}",
                                    other.type_name()
                                ),
                            })
                        .into());
                    }
                }
            } else {
                crate::edn::render::DEFAULT_MAX_FRAME_BYTES
            };
            // autobind_listener creates the socket SOCK_NONBLOCK (the C0b.3a-i invariant).
            let (ul, name_bytes) = crate::comms::process::autobind_listener(128).map_err(|e| {
                RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("autobind UDS listener: {}", e),
                    },
                )
            })?;
            use crate::kernel::address::Address;
            use crate::kernel::listener::Listener;
            use crate::kernel::spawn::{ADDRESS_TYPE_PATH, LISTENER_TYPE_PATH};
            use crate::rust_deps::marshal::make_rust_opaque;
            // Arc 272 6c.2 — stamp the minter pid at autobind so the connect gate can verify the
            // kernel-vouched answerer pid against it. SAFETY: getpid() is always-succeeds, no args.
            let minter_pid = unsafe { libc::getpid() };
            return Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
                "wat::spawn::Bound".into(),
                bound_names(),
                vec![
                    make_rust_opaque(
                        LISTENER_TYPE_PATH,
                        Listener::from_socket(ul, max_frame_bytes),
                    ),
                    make_rust_opaque(
                        ADDRESS_TYPE_PATH,
                        Address::from_socket_name_bytes(name_bytes, minter_pid),
                    ),
                ],
            ))));
        }
        // Arc 272 step 5 — the process listener is AUTOBIND-ONLY. The legacy 2-arg named form
        // `(listener' (process) <socket-address'>)` is ANNIHILATED with the rest of the
        // name-discovery stack: a chosen name is guessable hence squattable, so all rendezvous is
        // the kernel-minted exclusive-bind autobind capability (the 3-arg form above), handed
        // over the lineage channel. Anything but the 3-arg autobind form is an arity error.
        Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into())
    } else {
        // Thread tier (C0b.1): 3 args — host, :S, :R.
        if args.len() != 3 {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::ArityMismatch {
                    op: OP.into(),
                    expected: 3,
                    got: args.len(),
                },
            )
            .into());
        }
        // Validate args[1] and args[2] are type keywords (args[0] is the host expression).
        for i in [1usize, 2usize] {
            if !is_type_arg_shaped(&args[i]) {
                return Err(RuntimeError::new(
                    args[i].span().clone(),
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
        // Mint the crossbeam rendezvous channel.
        // Listener' = rx (the service accept-side, wrapped as Listener entity);
        // Address' = tx (the client dial-side, wrapped as Address entity — C0b.2e-iii).
        let (tx, rx) = crate::comms::thread::pair::<Value>();
        // Arc 209 C0b.2e-ii — wrap rx as the unified Listener entity.
        // Arc 209 C0b.2e-iii — wrap tx as the unified Address entity (was raw Sender).
        use crate::kernel::address::Address;
        use crate::kernel::listener::Listener;
        use crate::kernel::spawn::{ADDRESS_TYPE_PATH, LISTENER_TYPE_PATH};
        use crate::rust_deps::marshal::make_rust_opaque;
        Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
            "wat::spawn::Bound".into(),
            bound_names(),
            vec![
                make_rust_opaque(LISTENER_TYPE_PATH, Listener::from_crossbeam(rx)),
                make_rust_opaque(ADDRESS_TYPE_PATH, Address::from_thread(tx)),
            ],
        ))))
    }
}

/// `(:wat::kernel::connect addr)` — Arc 209 Stone C0b.1 / C0b.2c / C0b.2e-iii.
///
/// Arc 209 C0b.2e-iii: `addr` is now a unified `Address'` opaque (both thread and
/// process tiers). Downcasts the opaque to `Address`, calls `inner.connect(sym, span)`,
/// wraps the returned `Peer` as a `PEER_TYPE_PATH` opaque.  One arm, two impls.
///
/// (Formerly two arms: thread tier dispatched on `Value::wat__kernel__Sender`;
/// process tier dispatched on `SOCKET_ADDRESS_TYPE_PATH`. Both bodies moved verbatim
/// into `ThreadAddress::connect` / `SocketAddress::connect` in `kernel/address.rs`.)
pub(crate) fn eval_connect_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::connect";
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
    // Arc 209 C0b.2e-iii — one arm: downcast the Address' opaque → inner.connect.
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
    addr.connect_as_value(sym, list_span)
}

/// `(:wat::kernel::accept listener)` — Arc 209 Stone C0b.1 / C0b.2c.
///
/// Thread tier (C0b.1): block on the rendezvous `Listener'` (a raw
/// `Receiver`) until a connect-request arrives; unpack the server's raw
/// halves `(req_rx, resp_tx)`; wrap the server `(Peer' :- [R S])` end on THIS
/// thread (custody holds).  Returns the server `Peer'`.
///
/// Process tier (C0b.2c): downcast the `SocketListener'` opaque to
/// `&UnixListener`, call `.accept()` (blocks until a connection — the
/// honest wire-wait), wrap the accepted stream as a unified `(Peer' :- [R S])`.
/// Returns `(Peer' :- [R S])`.
pub(crate) fn eval_accept_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::accept";
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
    let listener_val = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc 209 C0b.2e-ii — ONE arm: downcast the unified Listener entity.
    match listener_val {
        Value::RustOpaque(ref inner)
            if inner.type_path == crate::kernel::spawn::LISTENER_TYPE_PATH =>
        {
            use crate::kernel::listener::Listener;
            use crate::rust_deps::marshal::downcast_ref_opaque;
            let listener: &Listener = downcast_ref_opaque(
                inner.as_ref(),
                crate::kernel::spawn::LISTENER_TYPE_PATH,
                OP,
                args[0].span().clone(),
            )?;
            listener.accept_as_value(sym, list_span)
        }
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Listener :- [S R]) (unified Listener entity from listener)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Arc 209 C0b.3b-b — `(:wat::kernel::allow listener pid)` → `nil`.
///
/// Inserts `pid` into the `SocketListener`'s allow-set. Process-tier only: a
/// `CrossbeamListener` has no allow-set (the crossbeam handle IS the grant).
pub(crate) fn eval_allow_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::allow";
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
    let listener_val = eval_inner(&args[0], env, sym)?.value_owned();
    match listener_val {
        Value::RustOpaque(ref inner)
            if inner.type_path == crate::kernel::spawn::LISTENER_TYPE_PATH =>
        {
            use crate::kernel::listener::{Listener, SocketListener};
            use crate::rust_deps::marshal::downcast_ref_opaque;
            let listener: &Listener = downcast_ref_opaque(
                inner.as_ref(),
                crate::kernel::spawn::LISTENER_TYPE_PATH,
                OP,
                args[0].span().clone(),
            )?;
            match listener.inner.as_any_ref().downcast_ref::<SocketListener>() {
                Some(sl) => {
                    let pid_val = eval_inner(&args[1], env, sym)?.value_owned();
                    let pid = match pid_val {
                        Value::i64(n) => n as i32,
                        other => {
                            return Err(RuntimeError::new(
                                args[1].span().clone(),
                                RuntimeErrorKind::TypeMismatch {
                                    op: OP.into(),
                                    expected: "i64 (pid)",
                                    got: Box::new(ValueSnapshot::of(&other)),
                                },
                            )
                            .into());
                        }
                    };
                    sl.allow(pid, list_span.clone())?;
                    Ok(Value::Unit)
                }
                None => Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "allow is a process-tier service gate; \
                                 a thread listener's handle IS the grant"
                            .into(),
                    },
                )
                .into()),
            }
        }
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Listener :- [S R]) (unified Listener entity from listener)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Arc 209 C0b.3b-b — `(:wat::kernel::deny listener pid)` → `nil`.
///
/// Removes `pid` from the `SocketListener`'s allow-set (future accepts by that pid bounce).
/// Process-tier only: a `CrossbeamListener` has no allow-set (the crossbeam handle IS the grant).
pub(crate) fn eval_deny_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::deny";
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
    let listener_val = eval_inner(&args[0], env, sym)?.value_owned();
    match listener_val {
        Value::RustOpaque(ref inner)
            if inner.type_path == crate::kernel::spawn::LISTENER_TYPE_PATH =>
        {
            use crate::kernel::listener::{Listener, SocketListener};
            use crate::rust_deps::marshal::downcast_ref_opaque;
            let listener: &Listener = downcast_ref_opaque(
                inner.as_ref(),
                crate::kernel::spawn::LISTENER_TYPE_PATH,
                OP,
                args[0].span().clone(),
            )?;
            match listener.inner.as_any_ref().downcast_ref::<SocketListener>() {
                Some(sl) => {
                    let pid_val = eval_inner(&args[1], env, sym)?.value_owned();
                    let pid = match pid_val {
                        Value::i64(n) => n as i32,
                        other => {
                            return Err(RuntimeError::new(
                                args[1].span().clone(),
                                RuntimeErrorKind::TypeMismatch {
                                    op: OP.into(),
                                    expected: "i64 (pid)",
                                    got: Box::new(ValueSnapshot::of(&other)),
                                },
                            )
                            .into());
                        }
                    };
                    sl.deny(pid, list_span.clone())?;
                    Ok(Value::Unit)
                }
                None => Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "deny is a process-tier service gate; \
                                 a thread listener's handle IS the grant"
                            .into(),
                    },
                )
                .into()),
            }
        }
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Listener :- [S R]) (unified Listener entity from listener)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::HandlePool::new name handles)` — build a pool of
/// N handles of the same type. `name` surfaces in error messages; the
/// pool drains as callers `pop` and asserts empty at `finish`.
///
/// Implementation: a bounded crossbeam channel of size N pre-filled
/// with the given handles, whose sender is dropped immediately so
/// further puts are impossible. Consumers `pop` via `recv` (the sender
/// is already gone so recv returns immediately on empty); `finish`
/// checks the channel is empty. No Mutex; the channel's
/// lock-free multi-consumer semantics are the synchronization.
pub(crate) fn eval_handle_pool_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::kernel::HandlePool::new".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let name = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::String(s) => s,
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::kernel::HandlePool::new".into(),
                    expected: "String",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let handles = match eval_inner(&args[1], env, sym)?.value_owned() {
        Value::Vec(v) => v,
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::kernel::HandlePool::new".into(),
                    expected: "wat::core::Vector",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let n = handles.len();
    // Zero-handle pools are legal — a pool with zero handles whose
    // `finish` is called immediately asserts true vacuously. Callers
    // that pre-count capacity may hit N=0 for degenerate cases.
    let (tx, rx) = crossbeam_channel::bounded::<Value>(n.max(1));
    for v in handles.iter() {
        if tx.send(v.clone()).is_err() {
            // The rx is local to this scope; send cannot fail.
            unreachable!("newly-built channel receiver must be alive");
        }
    }
    // Drop tx so the channel's is_empty discipline reads "fully
    // drained" once every handle is popped.
    drop(tx);
    Ok(Value::wat__kernel__HandlePool {
        name,
        rx: Arc::new(rx),
    })
}

/// `(:wat::kernel::HandlePool::pop pool)` — claim one handle. Returns
/// the claimed value. If the pool is empty, returns a
/// MalformedForm error naming the pool — callers are expected to
/// pop exactly the count they committed to at construction.
pub(crate) fn eval_handle_pool_pop(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::kernel::HandlePool::pop".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let (name, rx) = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::wat__kernel__HandlePool { name, rx } => (name, rx),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::kernel::HandlePool::pop".into(),
                    expected: "wat::kernel::HandlePool",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    // The sender was dropped at pool construction — recv() returns immediately
    // (either a value or Err on empty). Equivalent to try_recv without the
    // try_recv surface (Stone 214 1b-ii-ε: try_recv annihilated from substrate).
    match rx.recv() {
        Ok(v) => Ok(v),
        Err(_) => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::kernel::HandlePool::pop".into(),
                reason: format!(
                    "{}: no handles left to claim (pool drained or mis-counted at construction)",
                    name
                ),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::HandlePool::finish pool)` — assert the pool is
/// empty and return `:()`. Callers call this at the end of wiring to
/// catch orphaned handles BEFORE any thread runs. If handles remain
/// (an orphan — typically a mis-counted handle budget at
/// construction), returns a MalformedForm error naming the pool and
/// the orphan count. This is the "claim or panic" discipline from
/// FOUNDATION's Pipeline Discipline rule 2.
pub(crate) fn eval_handle_pool_finish(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::kernel::HandlePool::finish".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let (name, rx) = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::wat__kernel__HandlePool { name, rx } => (name, rx),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::kernel::HandlePool::finish".into(),
                    expected: "wat::kernel::HandlePool",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let remaining = rx.len();
    if remaining != 0 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::kernel::HandlePool::finish".into(),
            reason: format!(
                "{}: {} orphaned handle(s) — deadlock risk (every handle must be claimed before finish)",
                name, remaining
            )
        }).into());
    }
    Ok(Value::Unit)
}

/// DESIGN-STONE-process-signal-owner-to-child.md — the type path of the closed
/// signal enum `:wat::kernel::signal` takes as its second arg (registered in
/// `types.rs`). Users construct variants via the generic unit-variant path
/// (`:wat::kernel::Signal::User1`, etc. — `register_enum_methods`); nothing in
/// this file constructs a `Signal` value.
pub(crate) const SIGNAL_TYPE: &str = ":wat::kernel::Signal";

/// `(:wat::kernel::close peer)` — Stone 4.6a-ii; Arc 278 the close' OUTCOME WALL.
///
/// Consumes the peer (takes the Option, leaving None for subsequent calls).
/// Returns a matchable `:wat::kernel::CloseOutcome` for every HANDLEABLE outcome:
///   Thread' clean join       → `Closed[exit = None]`   (no OS exit code).
///   Thread' join panic       → `Failed[cause]`.
///   Process' clean exit      → `Closed[exit = Some(code)]`.
///   Process' terminated      → `Signaled[signal]`.
///   Process' wait fail / stop → `Failed[cause]`.
/// Only the MUST-NEVER-HAPPEN cases stay raises: double-close / use-after-close
/// ("peer already closed"), close' on a timer peer (arc-292 L3), and arity/type
/// mismatch (checker-prevented; defensive).
// Arc 259 S2d — restricted to `:wat::kernel::` callers. Teardown is RAII Drop;
// a :user:: fn calling close' is a check error. The user never holds the rope.
#[restricted_to(":wat::kernel::close", ":wat::kernel::")]
pub(crate) fn eval_peer_close_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::close";
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
            let mut thread = cell
                .with_mut(OP, list_span.clone(), |opt_peer| opt_peer.take())
                .map_err(EvalBreak::from)?
                .ok_or_else(|| {
                    EvalBreak::from(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "peer already closed".into(),
                        },
                    ))
                })?;
            // drain_and_join: drop input Sender FIRST (worker's recv' raises → worker
            // exits), then join. Idempotent via Option::take — the subsequent Drop on
            // `thread` is a no-op (arc 259 S2b drain-before-join invariant).
            // Arc 278 the close' OUTCOME WALL: a join panic is a HANDLEABLE close
            // failure → a matchable `CloseOutcome::Failed`, not a raise. A clean join
            // → `Closed[exit = None]` (a thread has no OS exit code — loci-agnostic, R32).
            if let Some(Err(_)) = thread.drain_and_join() {
                return Ok(close_outcome_failed(
                    "Thread peer join failed (thread panicked)".into(),
                ));
            }
            Ok(close_outcome_closed(None))
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
            let selectable = cell
                .with_mut(OP, list_span.clone(), |opt_bundle| opt_bundle.take())
                .map_err(EvalBreak::from)?
                .ok_or_else(|| {
                    EvalBreak::from(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "peer already closed".into(),
                        },
                    ))
                })?;
            match selectable {
                crate::kernel::spawn::ProcessSelectable::Spawned(bundle) => {
                    // Consume the bundle: close channels, then wait for the child.
                    // We need to extract the peer from the bundle first (bundle has _lifeline_w field too).
                    // Arc 278 the close' OUTCOME WALL: a wait failure is a HANDLEABLE close
                    // failure → `CloseOutcome::Failed`, not a raise.
                    let exit_status = match bundle.peer.wait() {
                        Ok(status) => status,
                        Err(io_err) => {
                            return Ok(close_outcome_failed(format!(
                                "Process peer wait failed: {}",
                                io_err
                            )));
                        }
                    };
                    match exit_status {
                        // Clean exit → Closed[Some(code)] (a process carries an OS exit code).
                        crate::process::ExitStatus::Exited(code) => {
                            Ok(close_outcome_closed(Some(code as i64)))
                        }
                        // Terminated by a signal → the matchable Signaled variant.
                        crate::process::ExitStatus::Signaled(sig) => {
                            Ok(close_outcome_signaled(sig as i64))
                        }
                        // Stopped (SIGSTOP/ptrace) but NOT terminated during teardown is an
                        // ABNORMAL close, not a kill: `Signaled` means *terminated by a
                        // signal*, which a stopped-not-reaped child is not (four-Q Honest).
                        // So it maps to `Failed`, carrying the stop signal in its cause.
                        crate::process::ExitStatus::Stopped(sig) => Ok(close_outcome_failed(
                            format!("Process peer stopped by signal {}", sig),
                        )),
                    }
                }
                // arc 292 L3 — timer peers are consumed by select'; close' is not supported.
                // Drop the rx (fd closed by Drop); no child to wait on.
                crate::kernel::spawn::ProcessSelectable::Timer(_) => {
                    Err(EvalBreak::from(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason:
                                "close on a timer peer is not supported (it is consumed by select)"
                                    .into(),
                        },
                    )))
                }
            }
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

/// `(:wat::kernel::signal proc sig)` — DESIGN-STONE-process-signal-owner-to-
/// child.md; BRIEF-process-signal-p2-mint.md.
///
/// STOP-1: `(Process :- [I O])` ONLY — no shared codegen with Thread'/Peer' (a thread
/// peer has no process to signal). STOP-3: routes through `Pidfd::send_signal`,
/// never `kill(pid, sig)` (`clone.rs:215-216` documents why the bare PID is
/// unsafe to reuse). STOP-4: `Kill` sends and returns; it does NOT reap —
/// `ChildHandle::Drop`/`close'` remain the only paths that reap.
///
/// Unlike `close'`, this does NOT consume the peer (`with_ref`, not
/// `with_mut` + `take`) — a process may be signalled any number of times
/// before it is closed.
///
/// STOP-2 (own probe, 2026-08-03): `SignalOutcome::Gone` was NOT minted —
/// `pidfd_send_signal` against a child that had exited but was deliberately
/// left un-reaped returned `Ok(())`, not ESRCH (delivery to a zombie is a
/// silent no-op). ESRCH appeared only against an ALREADY-REAPED pidfd, and
/// nothing in this substrate reaps a `Process` peer's pidfd except `close'`,
/// which consumes it — so the only way to reach that state through THIS verb
/// is to call it on an already-closed peer, which is intercepted below
/// (`"peer already closed"`) before the syscall ever runs. A live `signal`
/// call cannot observe ESRCH.
pub(crate) fn eval_signal(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::signal";
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
    let peer_val = eval_inner(&args[0], env, sym)?.value_owned();
    let sig_val = eval_inner(&args[1], env, sym)?.value_owned();

    // STOP-6: no `_` wildcard — every Signal variant is named explicitly.
    let sig_posix: libc::c_int = match &sig_val {
        Value::Enum(e) if e.type_path == SIGNAL_TYPE => match e.variant_name.as_str() {
            "User1" => libc::SIGUSR1,
            "User2" => libc::SIGUSR2,
            "Hangup" => libc::SIGHUP,
            "Interrupt" => libc::SIGINT,
            "Terminate" => libc::SIGTERM,
            "Kill" => libc::SIGKILL,
            other_variant => return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "unknown Signal variant {other_variant:?} (substrate bug: checker-prevented)"
                    ),
                })
            .into()),
        },
        other => return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "sig (:wat::kernel::Signal)",
                got: Box::new(ValueSnapshot::of(other)),
            })
        .into()),
    };

    match &peer_val {
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

            // Local sentinel: the `with_ref` closure cannot early-return an
            // `Err` (it is FnOnce(&T) -> R, not -> Result<R, _>) — classify
            // first, act on the classification after the borrow ends.
            enum SignalAttempt {
                Sent(std::io::Result<()>),
                Timer,
                Closed,
            }
            let attempt = cell
                .with_ref(OP, |opt_bundle| match opt_bundle {
                    Some(crate::kernel::spawn::ProcessSelectable::Spawned(bundle)) => {
                        SignalAttempt::Sent(bundle.peer.pidfd.send_signal(sig_posix))
                    }
                    Some(crate::kernel::spawn::ProcessSelectable::Timer(_)) => SignalAttempt::Timer,
                    None => SignalAttempt::Closed,
                })
                .map_err(EvalBreak::from)?;

            match attempt {
                SignalAttempt::Closed => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "peer already closed".into(),
                    })
                .into()),
                // arc 292 L3 — timer peers carry no child process to signal.
                SignalAttempt::Timer => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "signal on a timer peer is not supported (it is consumed by select)".into(),
                    })
                .into()),
                SignalAttempt::Sent(Ok(())) => Ok(signal_outcome_delivered()),
                // STOP-7: EINVAL (unrepresentable signal — the enum forbids it) and
                // EBADF (closed pidfd — a substrate bug) are must-never-happen: raises,
                // not outcomes. Every other io failure is a genuine handleable `Failed`.
                SignalAttempt::Sent(Err(io_err)) => match io_err.raw_os_error() {
                    Some(libc::EINVAL) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!(
                                "pidfd_send_signal: EINVAL (unrepresentable signal — substrate bug): {}",
                                io_err
                            ),
                        })
                    .into()),
                    Some(libc::EBADF) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!(
                                "pidfd_send_signal: EBADF (closed pidfd — substrate bug): {}",
                                io_err
                            ),
                        })
                    .into()),
                    _ => Ok(signal_outcome_failed(format!("pidfd_send_signal failed: {}", io_err))),
                },
            }
        }
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "proc (Process :- [I O])",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Implement `(:wat::kernel::after peer-kind duration msg)` — arc 292 L3 timer peer.
///
/// arg0 is a `:wat::program::PeerKind` enum value (`:thread` or `:process`), selecting
/// the tier for the one-shot timer. arc 278 Stone 1: returns a UNIFIED `(Peer' :- [nil O])`
/// value (`PEER_TYPE_PATH` opaque RustOpaque) — a real peer whose recv fires the `msg`
/// once, then EOFs — so it drops into `poll'`/`select'` by construction:
///   - `:thread` → crossbeam `after`-backed unified peer (futex, no background thread).
///   - `:process` → timerfd-backed unified peer (io_uring reactor, same Select as
///     accepted socket connections).
///
/// Three args:
/// - `args[0]`: peer-kind — must evaluate to `:wat::program::PeerKind` enum value.
/// - `args[1]`: duration — must evaluate to `Value::Duration(nanos: i64)`, non-negative.
/// - `args[2]`: msg — any `Value`; becomes the timer's output payload.
pub(crate) fn eval_kernel_after(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::after";
    if args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }

    // arg 0: peer-kind — evaluate and match the PeerKind enum VALUE.
    // `:wat::program::PeerKind::thread` / `:process` evaluate to
    // Value::Enum { type_path=":wat::program::PeerKind", variant_name="thread"/"process", fields=[] }.
    let peer_kind_val = eval_inner(&args[0], env, sym)?.value_owned();
    let is_thread_tier = match &peer_kind_val {
        Value::Enum(ev)
            if ev.type_path.as_str() == ":wat::program::PeerKind" && ev.fields.is_empty() =>
        {
            match ev.variant_name.as_str() {
                "thread" => true,
                "process" => false,
                _other => {
                    return Err(RuntimeError::new(
                        args[0].span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected:
                                ":wat::program::PeerKind (e.g. :wat::program::PeerKind::process)",
                            got: Box::new(ValueSnapshot::of(&peer_kind_val)),
                        },
                    )
                    .into());
                }
            }
        }
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::program::PeerKind (e.g. :wat::program::PeerKind::process)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };

    // arg 1: duration — must be Value::Duration(nanos: i64), non-negative.
    let duration_val = eval_inner(&args[1], env, sym)?.value_owned();
    let nanos: i64 = match &duration_val {
        Value::Duration(n) => *n,
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::time::Duration value (e.g. (:wat::time::Millisecond 50))",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    if nanos < 0 {
        return Err(RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("after: duration must be non-negative; got {} nanos", nanos),
            },
        )
        .into());
    }

    // arg 2: msg — any Value.
    let msg = eval_inner(&args[2], env, sym)?.value_owned();

    // Build the std::time::Duration from nanos.
    let std_dur = std::time::Duration::from_nanos(nanos as u64);

    use crate::rust_deps::custodia::ThreadOwnedCell;
    use crate::rust_deps::marshal::make_rust_opaque;

    // arc 278 Stone 1 — the timer is built in the CORRECT location: a UNIFIED
    // `(Peer' :- [nil O])` (`PEER_TYPE_PATH`), NOT a tier-specific `Thread'`/`Process'`
    // `(Timer' :- [O])`. A timer is a real peer whose recv fires the `msg` once, then
    // EOFs — so it drops into `poll'` (and `select'`) BY CONSTRUCTION, exactly
    // like an accepted connection (`Listener::accept_as_value`). The tier is still
    // chosen by `peer-kind`, but both tiers now yield the same `PEER_TYPE_PATH`
    // value; the vestigial tier-open `Timer'` type + its fusion machinery are
    // retired in check.rs. A timer has NO input, so the peer's send endpoint is a
    // dead sender (its receiver is dropped immediately; it is never used).
    use crate::kernel::spawn::PEER_TYPE_PATH;

    if is_thread_tier {
        // ── Thread tier ───────────────────────────────────────────────────────
        // The timer Receiver<Value> (crossbeam::after, futex-based, no background
        // thread; fires exactly once after std_dur) IS the unified peer's `rx`.
        let output_rx = crate::comms::thread::timer(std_dur, msg);

        // Dead send endpoint: a crossbeam pair whose receiver is dropped.
        let (dead_tx, dead_rx) = crate::comms::thread::pair::<Value>();
        drop(dead_rx);

        let peer = crate::kernel::peer::Peer::from_thread(dead_tx, output_rx);
        let cell: crate::kernel::spawn::PeerCell =
            std::sync::Arc::new(ThreadOwnedCell::new(Some(peer)));
        Ok(make_rust_opaque(PEER_TYPE_PATH, cell))
    } else {
        // ── Process tier ─────────────────────────────────────────────────────
        // Encode msg to a wire frame (tagged EDN + '\n') — same framing as send'
        // and as a real socket peer's frames, so `poll'`/`select'` decode it via
        // `decode_trusted_wire` identically to any accepted connection.
        let edn_node = crate::edn::render::value_to_edn_with(&msg, sym.types().map(|a| a.as_ref()));
        let edn_str = wat_edn::write(&edn_node);
        let mut frame: Vec<u8> = edn_str.into_bytes();
        frame.push(b'\n');

        // A timerfd-backed `process::Receiver<Value>` — the SAME `Source::Timer`
        // that backed the old `Receiver<String>`; the frame bytes are tier-agnostic
        // so only the decode type param differs. This IS the unified peer's `rx`.
        let output_rx =
            crate::comms::process::timer::<Value>(std_dur, frame).map_err(|io_err| {
                EvalBreak::from(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("after: timerfd creation failed: {}", io_err),
                    },
                ))
            })?;

        // Dead send endpoint: a socketpair whose receiver end is dropped.
        let (dead_tx, dead_rx) = crate::comms::process::pair::<Value>().map_err(|io_err| {
            EvalBreak::from(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("after: dead-sender socket creation failed: {}", io_err),
                },
            ))
        })?;
        drop(dead_rx);

        let peer =
            crate::kernel::peer::Peer::from_socket(dead_tx.reinterpret::<String>(), output_rx);
        let cell: crate::kernel::spawn::PeerCell =
            std::sync::Arc::new(ThreadOwnedCell::new(Some(peer)));
        Ok(make_rust_opaque(PEER_TYPE_PATH, cell))
    }
}

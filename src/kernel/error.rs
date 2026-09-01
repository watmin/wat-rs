//! Kernel sub-module mirroring `src/intrinsic/kernel/error.rs` — arc 109
//! Stone 4a (`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-died-error-
//! cluster-decomposes.md`, map item 4a). Sixteen items: the edge's four
//! delegate fns (`eval_died_error_message` / `eval_died_error_to_failure` /
//! `eval_failure_message` / `eval_failure_location`), the `loci_died_*`
//! family (`loci_died_error_from_reason` / `loci_died_from_send_error` /
//! `loci_died_disconnected`), the `thread_died_error_*` family (`panic` /
//! `runtime` / `shutdown`), the chain/EDN helpers (`single_died_chain` /
//! `thread_crash_panic_edn` / `thread_crash_runtime_edn`), and three
//! private helpers (`died_error_payload_message` / `edn_is_loci_died_chain` /
//! `failure_error_field`).
//!
//! Measured, and stated so it can be re-checked rather than believed: no
//! file outside `src/intrinsic/kernel/error.rs` (the edge) and
//! `src/kernel/{message,outcome,spawn}.rs` (this home) CALLS any of the
//! sixteen. Four other files name them in prose only. To re-derive, grep
//! each name tree-wide and discard lines whose first non-space characters
//! are `//`, `///`, `//!` or `*`; what remains is confined to those four
//! files. (An earlier brief put "seventeen sites" here; that number summed
//! call expressions with `use`-specifiers and did not survive being
//! re-counted — the claim above is the one that does.)
//! ★ The two that prove it hardest:
//! `thread_crash_panic_edn` and `thread_crash_runtime_edn` have ZERO
//! callers left in `runtime.rs` — their only consumer is
//! `src/kernel/spawn.rs`, orphaned in the megafile exactly as stone A's
//! `accept_outcome_*` were.
//!
//! `eval_error_names` — an `_error_`-named, `_names`-suffixed helper right
//! beside this cluster in `runtime.rs` — is NOT part of this home: its only
//! caller is `runtime_error_to_eval_error_value`, the `:wat::core::EvalError`
//! vocabulary beside `wrap_as_eval_result`/`eval_form_ast` that serves
//! `intrinsic/holon/atom.rs`'s `eval-*` verbs. A `_names` suffix is a naming
//! convention, not a membership test.
//!
//! The `:wat::core::Fault`/`Failure` diagnostic vocabulary this cluster
//! CALLS (`fault_value`, `failure_names`, `location_names`, `frame_names`,
//! `failure_value_from_assertion_payload`, `record_field_by_name`, …) stays
//! in `runtime.rs` — map item 4d, the genuinely shared residue consumed by
//! `edn`/`host`/`types`/`resolve`/`assertion`/`comms`/`kernel`/`distribution`,
//! deliberately unassigned.
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::runtime::{
    builtin_enum_variant_names, eval_inner, failure_value_from_assertion_payload,
    message_only_failure, no_field_names, record_field_by_name,
};
use crate::span::Span;
use crate::value::{
    EnumValue, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use std::sync::Arc;

/// Build a `:wat::kernel::ThreadDiedError::Panic` enum value
/// (arc 060 + arc 105c). Variant carries two fields:
/// `message: String` always populated; `failure: (Option :- [Failure])`
/// populated when the panic was an `AssertionPayload` carrying
/// arc 064's structured actual / expected / location / frames
/// info, `:None` for plain panics.
pub(crate) fn thread_died_error_panic(
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
) -> Value {
    let failure_field = match assertion {
        Some(p) => {
            // Build a :wat::kernel::Failure Value::Aggregate(Struct) out of the
            // AssertionPayload's owned fields. Same shape arc 064
            // produced via the now-deleted build_failure helper in
            // src/sandbox.rs.
            Value::Option(Arc::new(Some(failure_value_from_assertion_payload(p))))
        }
        None => Value::Option(Arc::new(None)),
    };
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "Panic".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "Panic"),
        fields: vec![Value::String(Arc::new(message)), failure_field],
    }))
}

/// Build a `:wat::kernel::ThreadDiedError::RuntimeError(message)`
/// enum value (arc 060).
pub(crate) fn thread_died_error_runtime(message: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "RuntimeError".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "RuntimeError"),
        fields: vec![Value::String(Arc::new(message))],
    }))
}

/// Build a `:wat::kernel::LociDiedError::Stopped`
/// (unit variant) enum value (arc 170 Slice A).
/// Produced when the process-wide shutdown signal fires during recv, or
/// (arc 278 send-mirrors-recv) when it fires while a `Sender::send` is
/// polled-blocked waiting for pipe room — the wat-visible variant is
/// `Stopped` (arc-170 intueri cast: nothing on the wat side is "shutting
/// down", a stop was merely requested), while the Rust signal that triggers
/// it keeps its own uniform `shutdown` vocabulary, hence this fn's name.
/// Distinguishable from ChannelDisconnected: the channel partner did
/// NOT drop — the process is stopping. Used by [`loci_died_from_send_error`]
/// (send' side); the recv' side builds its own inline copy in
/// `recv_outcome_shutdown`.
pub(crate) fn thread_died_error_shutdown() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "Stopped".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `(:wat::kernel::Failure/message f) -> :String` — arc 278 the string-wrap
/// annihilation. DERIVED accessor: reads `error.message` off the Failure's
/// mandatory `:wat::core::Error` field. (`message` is no longer a stored field;
/// storing it alongside `error` would duplicate and could drift — four-questions
/// Fork B.) Every existing `Failure/message` reader keeps working unchanged.
pub(crate) fn eval_failure_message(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::Failure/message";
    let error = failure_error_field(OP, args, env, sym, list_span)?;
    let types = sym.types().map(|a| a.as_ref());
    match record_field_by_name(&error, "message", types) {
        Some(v @ Value::String(_)) => Ok(v),
        _ => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "String at :wat::core::Error/message",
                got: Box::new(ValueSnapshot::unavailable(
                    "error has no String message field",
                )),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::Failure/location f) -> (:Option :- [:wat::kernel::Location])` — arc 278
/// the string-wrap annihilation. DERIVED accessor: reads `error.location` (a
/// mandatory `:wat::kernel::Location` on the error) and wraps it in `Some` to keep
/// the accessor's historic `(Option :- [Location])` return shape.
pub(crate) fn eval_failure_location(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::Failure/location";
    let error = failure_error_field(OP, args, env, sym, list_span)?;
    let types = sym.types().map(|a| a.as_ref());
    match record_field_by_name(&error, "location", types) {
        Some(loc @ Value::Aggregate(_)) => Ok(Value::Option(Arc::new(Some(loc)))),
        _ => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Location at :wat::core::Error/location",
                got: Box::new(ValueSnapshot::unavailable("error has no Location field")),
            },
        )
        .into()),
    }
}

/// Shared arity-1 eval + `error`-field extraction for the derived `Failure/*`
/// accessors. Evaluates the single Failure arg and returns its `error` field
/// (the raised `:wat::core::Error`).
pub(crate) fn failure_error_field(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
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
    let val = eval_inner(&args[0], env, sym)?.value_owned();
    let types = sym.types().map(|a| a.as_ref());
    match record_field_by_name(&val, "error", types) {
        Some(e) => Ok(e),
        None => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::kernel::Failure (with an `error` field)",
                got: Box::new(ValueSnapshot::of(&val)),
            },
        )
        .into()),
    }
}

/// Arc 113 slice 1 — wrap a single DiedError Value in a
/// `Vec<DiedError>` chain.
///
/// The Vec is the chain. Head = the immediate peer that died; tail
/// = whatever killed it, transitively. Consumers reach for
/// `(:wat::core::first chain)` to recover the head when they don't
/// care about the trail.
pub(crate) fn single_died_chain(died: Value) -> Value {
    Value::Vec(Arc::new(vec![died]))
}

/// Arc 278 no-hidden-failures — render a THREAD-peer PANIC death as the SAME
/// bare `(Vector :- [LociDiedError])` EDN line the process tier emits from
/// `emit_chain_envelope` (`value_to_edn_with` → `wat_edn::write`), so the
/// parent's [`loci_died_error_from_reason`] bridges it STRUCTURALLY — the raised
/// Fault rides in `Panic.failure` — instead of falling to the opaque string-wrap
/// (which resurrected the arc-278-annihilated string-wrap for a structured
/// `AssertionPayload`). Thread tier now loci-agnostic-equal to the process tier.
pub(crate) fn thread_crash_panic_edn(
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
    types: Option<&crate::types::TypeEnv>,
) -> String {
    let chain = single_died_chain(thread_died_error_panic(message, assertion));
    let edn = crate::edn::render::value_to_edn_with(&chain, types);
    wat_edn::write(&edn)
}

/// Arc 278 no-hidden-failures — the RuntimeError sibling of
/// [`thread_crash_panic_edn`]. Mirrors the process tier's
/// `process_died_error_runtime_value`: the RuntimeError crosses the wire as
/// structured `to_wire_edn` (its `:message`/`:location`/`:causes` floor), NOT
/// `re.to_string()` prose, wrapped in the same bare `(Vector :- [LociDiedError])` line.
pub(crate) fn thread_crash_runtime_edn(
    re: &RuntimeError,
    types: Option<&crate::types::TypeEnv>,
) -> String {
    let chain = single_died_chain(thread_died_error_runtime(crate::edn::contract::to_wire_edn(re)));
    let edn = crate::edn::render::value_to_edn_with(&chain, types);
    wat_edn::write(&edn)
}

/// Derive the human message from a `LociDiedError` variant's carried payload.
///
/// Arc 278 "errors first-class EDN" (stone 1) — `StartupError`'s payload is now
/// the structured `:wat::core::Error` record (a `Value::Aggregate` whose FIRST
/// field is the `:message` String, by the floor order `message`/`location`/
/// `causes`). Every OTHER carrying variant (`Panic` / `RuntimeError` /
/// `EntryFormFailure` / `MainSignature` / `BadReturn`) still carries a bare
/// `Value::String`. This accessor accepts BOTH: a structured Error record →
/// its `:message`; a bare String → itself. (A legacy String-wrapped
/// `StartupError` payload — e.g. the setpgid OS-level `FlatMessage` path — also
/// lands on the String arm.)
pub(crate) fn died_error_payload_message(v: &Value) -> Option<Arc<String>> {
    match v {
        Value::String(s) => Some(s.clone()),
        // Structured `:wat::core::Error` record: `:message` is field 0.
        Value::Aggregate(a) => match a.fields.first() {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// `(:wat::kernel::LociDiedError/message err) -> :String` — arc 278 the
/// LociDiedError stone (one loci-agnostic accessor; the two dead
/// `Thread/ProcessDiedError/message` siblings collapsed here).
///
/// Extracts the carried String from any `:wat::kernel::LociDiedError`
/// variant; returns a constant string for the unit variants
/// (`Disconnected` / `Shutdown`). Routes around the wat-side
/// enum-variant pattern-matcher gap — callers ask for a generic
/// message without discriminating variants.
///
/// Field 0 is `message` for `Panic` / `RuntimeError` /
/// `StartupError` / `EntryFormFailure` / `MainSignature` / `BadReturn`.
pub(crate) fn eval_died_error_message(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    expected_type_path: &'static str,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let op_string = format!("{}/message", expected_type_path);
    let op: &str = &op_string;
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
    let val = eval_inner(&args[0], env, sym)?.value_owned();
    match val {
        Value::Enum(ev) if ev.type_path == expected_type_path => {
            match ev.variant_name.as_str() {
                // Arc 170 slice 1i — EntryFormFailure / MainSignature / BadReturn /
                // RuntimeError / Panic carry a String at field 0. Arc 278 stone 1 —
                // StartupError carries a structured `:wat::core::Error` record; the
                // message is DERIVED from its `:message` (see `died_error_payload_message`).
                "Panic" | "RuntimeError" | "StartupError" | "EntryFormFailure"
                | "MainSignature" | "BadReturn" => {
                    match ev.fields.first().and_then(died_error_payload_message) {
                        Some(s) => Ok(Value::String(s)),
                        None => Err(RuntimeError::new(
                            args[0].span().clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "String or :wat::core::Error inside *DiedError variant",
                                got: Box::new(ValueSnapshot::unavailable("non-message payload")),
                                // arc 138: no — matching on Value::Enum fields; no AST element
                            },
                        )
                        .into()),
                    }
                }
                "Disconnected" => Ok(Value::String(Arc::new("disconnected".to_string()))),
                // arc 170 Slice A — a stop was requested during recv. Wat-visible name is
                // "Stopped" (arc-170 intueri cast RULING A), not Rust's "shutdown".
                "Stopped" => Ok(Value::String(Arc::new("process stopped".to_string()))),
                _ => Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: op.into(),
                        expected: "*DiedError variant",
                        got: Box::new(ValueSnapshot::unavailable("unknown *DiedError variant")),
                        // arc 138: no — matching on Value::Enum variant_name; no AST element
                    },
                )
                .into()),
            }
        }
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::kernel::*DiedError",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// True when `v` is a bare `(Vector :- [LociDiedError])` death chain — a
/// `Vector` whose first element is a `#wat.kernel.LociDiedError/…` tagged
/// value. Arc 278: the chain crosses bare (no `ProcessPanics` wrapper); the
/// head element's own tag is the self-describing marker.
pub(crate) fn edn_is_loci_died_chain(v: &wat_edn::OwnedValue) -> bool {
    if let wat_edn::OwnedValue::Vector(items) = v {
        if let Some(wat_edn::OwnedValue::Tagged(tag, _)) = items.first() {
            return tag.namespace() == "wat.kernel.LociDiedError";
        }
    }
    false
}

/// Shared backbone for ThreadDiedError/to-failure and
/// ProcessDiedError/to-failure — variants are identical; only the
/// expected type_path differs.
pub(crate) fn eval_died_error_to_failure(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    expected_type_path: &'static str,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let op_string = format!("{}/to-failure", expected_type_path);
    let op: &str = &op_string;
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
    let val = eval_inner(&args[0], env, sym)?.value_owned();
    match val {
        Value::Enum(ev) if ev.type_path == expected_type_path => {
            match ev.variant_name.as_str() {
                "Panic" => {
                    let msg = match ev.fields.first() {
                        Some(Value::String(s)) => (**s).clone(),
                        _ => {
                            return Err(RuntimeError::new(
                                args[0].span().clone(),
                                RuntimeErrorKind::TypeMismatch {
                                    op: op.into(),
                                    expected: "String at Panic.message",
                                    got: Box::new(ValueSnapshot::unavailable(
                                        "non-String at field 0",
                                    )),
                                    // arc 138: no — matching on Value::Enum fields; no AST element
                                },
                            )
                            .into());
                        }
                    };
                    // Field 1 is declared `(Option :- [Failure])`. The
                    // EDN reader's reconstruct_struct + Tagged
                    // arms (arc 113 slice 3) wrap Option layers
                    // back during bridge, so both wat-side
                    // builds and post-EDN round trips arrive
                    // here as `Value::Option(_)`. `Some(failure)`
                    // → return the inner Failure clone; `None` →
                    // fall through to message-only.
                    if let Some(Value::Option(opt)) = ev.fields.get(1) {
                        if let Some(failure) = opt.as_ref() {
                            return Ok(failure.clone());
                        }
                    }
                    Ok(message_only_failure(msg))
                }
                // Arc 170 slice 1i — EntryFormFailure / MainSignature / BadReturn /
                // RuntimeError carry one String field. Arc 278 stone 1 — StartupError
                // carries a structured `:wat::core::Error`; `died_error_payload_message`
                // derives its `:message`. Both map to a message-only Failure.
                "RuntimeError" | "StartupError" | "EntryFormFailure" | "MainSignature"
                | "BadReturn" => match ev.fields.first().and_then(died_error_payload_message) {
                    Some(s) => Ok(message_only_failure((*s).clone())),
                    None => Err(RuntimeError::new(
                        args[0].span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "String or :wat::core::Error at *DiedError payload",
                            got: Box::new(ValueSnapshot::unavailable(
                                "non-message payload at field 0",
                            )),
                            // arc 138: no — matching on Value::Enum fields; no AST element
                        },
                    )
                    .into()),
                },
                "Disconnected" => Ok(message_only_failure("disconnected".to_string())),
                // arc 170 Slice A — a stop was requested during recv. Wat-visible name is
                // "Stopped" (arc-170 intueri cast RULING A), not Rust's "shutdown".
                "Stopped" => Ok(message_only_failure("process stopped".to_string())),
                _ => Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: op.into(),
                        expected: "*DiedError variant",
                        got: Box::new(ValueSnapshot::unavailable("unknown *DiedError variant")),
                        // arc 138: no — matching on Value::Enum variant_name; no AST element
                    },
                )
                .into()),
            }
        }
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::kernel::*DiedError",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Turn a peer's crash-channel `reason` into the single
/// `:wat::kernel::LociDiedError` that `RecvOutcome::Lost` carries (arc 278 the
/// LociDiedError stone).
///
/// A process peer emits its death as a self-describing BARE `(Vector :- [LociDiedError])`
/// EDN line (the annihilated `#wat.kernel/ProcessPanics` wrapper is gone); we parse
/// it via generic `edn::read` and take the HEAD (the immediate peer death) — the
/// chain is a container-level Vector, and Lost holds ONE. A single tagged
/// `#wat.kernel.LociDiedError/…` line is bridged as-is. Any other reason (a
/// thread-peer plain message, a socket-tier administrative sentinel, a
/// decode-failure note) is an opaque death message → wrapped as
/// `LociDiedError::Panic{message: reason, failure: None}`.
pub(crate) fn loci_died_error_from_reason(reason: String, types: Option<&crate::types::TypeEnv>) -> Value {
    let trimmed = reason.trim();
    if let Ok(parsed) = wat_edn::parse_owned(trimmed) {
        // A bare (Vector :- [LociDiedError]) death chain → bridge + take the head.
        if edn_is_loci_died_chain(&parsed) {
            // ctx=None: this decodes only the fixed core `LociDiedError` enum — never a
            // user-declared HolonRecord class — so no EncodingCtx is ever needed here.
            if let Ok(Value::Vec(items)) = crate::edn::render::edn_to_value(&parsed, types, None) {
                if let Some(head) = items.first() {
                    return head.clone();
                }
            }
        }
        // A single #wat.kernel.LociDiedError/… tagged value → bridge as-is.
        if let wat_edn::OwnedValue::Tagged(tag, _) = &parsed {
            if tag.namespace() == "wat.kernel.LociDiedError" {
                if let Ok(v) = crate::edn::render::edn_to_value(&parsed, types, None) {
                    return v;
                }
            }
        }
    }
    // Opaque reason — wrap as a Panic carrying the raw death message.
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "Panic".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "Panic"),
        fields: vec![
            Value::String(Arc::new(reason)),
            Value::Option(Arc::new(None)),
        ],
    }))
}

/// `:wat::kernel::LociDiedError::Disconnected []` — the peer's receiving end is
/// gone (EPIPE). Arc 278 BRIEF-send-carries-its-cause (#70) minted this as the
/// only cause send' could honestly report; arc 278 send-mirrors-recv
/// (`DESIGN-STONE-send-mirrors-recv.md`) has since given `comms::thread::
/// Sender::send` and `comms::process::Sender::send` a real `SendError` enum
/// (`Disconnected`/`Shutdown`/`FrameTooLarge`/`Failed`) mirroring `RecvError` —
/// see [`loci_died_from_send_error`] for the full mapping. This fn now builds
/// specifically the `Disconnected` cause, not a stand-in for "whatever send
/// failed for."
pub(crate) fn loci_died_disconnected() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "Disconnected".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// Map a `comms::SendError<T>` to its `:wat::kernel::LociDiedError` cause —
/// arc 278 send-mirrors-recv. Mirrors the recv-side match on `RecvError` at
/// this same call site's twin (`eval_peer_recv_prime`):
/// - `Disconnected` → `LociDiedError::Disconnected` (EPIPE, honest as-is).
/// - `Shutdown` → `LociDiedError::Stopped` — now producible, because
///   `Sender::send` polls the shutdown broadcast mid-write instead of
///   blocking uncancellably (the gap `loci_died_disconnected`'s old doc
///   named: "not yet producible from any live send' call site").
/// - `Failed(_, reason)` → `LociDiedError::RuntimeError(reason)`, carrying
///   the real io error text instead of discarding it.
///
/// `SendError` has no `FrameTooLarge` arm (arc 278 "cut the cap, prove the
/// poll arm" removed the sender-side pre-write cap check — the transport
/// cannot know which *op* is being sent, so it can never hold the right
/// budget; that check moves to the generated client method in a later
/// strike). The receiver's `RecvError::FrameTooLarge` is unaffected.
pub(crate) fn loci_died_from_send_error<T>(e: &crate::comms::SendError<T>) -> Value {
    match e {
        crate::comms::SendError::Disconnected(_) => loci_died_disconnected(),
        crate::comms::SendError::Shutdown(_) => thread_died_error_shutdown(),
        crate::comms::SendError::Failed(_, reason) => thread_died_error_runtime(reason.clone()),
    }
}

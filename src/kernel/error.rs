//! Kernel sub-module mirroring `src/intrinsic/kernel/error.rs` — arc 109
//! Stone 4a (`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-died-error-
//! cluster-decomposes.md`, map item 4a). Sixteen items: the edge's four
//! delegate fns (`eval_died_error_message` / `eval_died_error_to_failure` /
//! `eval_failure_message` / `eval_failure_location`), the `loci_died_*`
//! family (`loci_died_error_from_reason` / `loci_died_from_send_error` /
//! `loci_died_disconnected`), the `thread_died_error_*` family (`panic` /
//! `runtime` / `shutdown`), the chain/EDN helpers (`single_died_chain` /
//! `thread_crash_panic_edn` / `thread_crash_runtime_edn`), and three
//! private helpers (`edn_is_loci_died_chain` / `failure_error_field` / and,
//! since excursus 003 step 3b replaced the arc-278 stone-1 String-or-Error
//! branching this doc originally counted, `failure_payload_message`).
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
//! Excursus 003 strike E retired `eval_error_names` and its only caller
//! `runtime_error_to_eval_error_value` outright (the `:wat::core::EvalError`
//! vocabulary beside `wrap_as_eval_result`/`eval_form_ast` that served
//! `intrinsic/holon/atom.rs`'s `eval-*` verbs) — `wrap_as_eval_result` now calls
//! `runtime_error_failure` (below) directly, so there is no longer a sibling
//! helper near this cluster to carve a boundary against.
//!
//! The `:wat::core::Fault`/`Failure` diagnostic vocabulary this cluster
//! CALLS (`fault_value`, `failure_names`, `span_names`, `frame_names`,
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
    flat_message_failure, message_only_failure, no_field_names, record_field_by_name,
    runtime_error_failure,
};
use crate::span::Span;
use crate::value::{
    EnumValue, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use std::sync::Arc;

// ⛔ GENERATED FROM `wat/kernel/diagnostics.wat`. Builder ruling 2026-09-06:
// LociDiedError is declared in wat; Rust sources from it. Add a variant there
// and this type follows — there is no Rust list to keep in step.
::wat_source_derive::wat_enum_from!(
    pub(crate) enum LociDiedError,
    "wat/kernel/diagnostics.wat",
    ":wat::kernel::LociDiedError"
);

fn loci_died_value(variant: LociDiedError, names: Arc<Vec<String>>, fields: Vec<Value>) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: LociDiedError::WAT_TYPE_PATH.into(),
        variant_name: variant.as_str().into(),
        names,
        fields,
    }))
}

/// Build a `:wat::kernel::LociDiedError::Panic` enum value (arc 060 + arc 105c;
/// excursus 003 step 3b — the envelope carries `Failure`). ONE mandatory field now:
/// every panic, assertion-carrying or plain, carries a real `Failure` — a plain
/// panic's `failure.error` is a `:wat::core::Fault` synthesized from `message`
/// (`flat_message_failure`'s `#[track_caller]` site, since a bare panic has no
/// location of its own — measured: the panic hook only captures a location for an
/// `AssertionPayload`). The separate `message` field this variant used to carry is
/// gone; its text is `failure.error.message` (`eval_died_error_message` derives it).
pub(crate) fn thread_died_error_panic(
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
) -> Value {
    let failure_field = match assertion {
        // Build a :wat::kernel::Failure Value::Aggregate(Struct) out of the
        // AssertionPayload's owned fields. Same shape arc 064
        // produced via the now-deleted build_failure helper in
        // src/sandbox.rs.
        Some(p) => failure_value_from_assertion_payload(p),
        None => flat_message_failure(message),
    };
    loci_died_value(
        LociDiedError::Panic,
        builtin_enum_variant_names(LociDiedError::WAT_TYPE_PATH, LociDiedError::Panic.as_str()),
        vec![failure_field],
    )
}

/// Build a `:wat::kernel::LociDiedError::RuntimeError(failure)` enum value (arc 060;
/// excursus 003 step 3b) from a FLAT message with no `RuntimeError` of its own —
/// `SendError::Failed`'s io-error reason is the one live caller
/// ([`loci_died_from_send_error`]). `failure.error` is a synthesized `:wat::core::Fault`
/// (`flat_message_failure`). A genuine `RuntimeError` goes through
/// [`thread_died_error_runtime_from_error`] instead, which preserves its declared
/// `:wat::runtime::<Kind>` record and its own already-captured frames.
pub(crate) fn thread_died_error_runtime(message: String) -> Value {
    loci_died_value(
        LociDiedError::RuntimeError,
        builtin_enum_variant_names(
            LociDiedError::WAT_TYPE_PATH,
            LociDiedError::RuntimeError.as_str(),
        ),
        vec![flat_message_failure(message)],
    )
}

/// Build a `:wat::kernel::LociDiedError::RuntimeError(failure)` enum value from a
/// GENUINE `RuntimeError` (excursus 003 step 3b, item 2's "From a RuntimeError"
/// branch) — `failure.error` is `re.to_record()` (step 3a's declared
/// `:wat::runtime::<Kind>` record, its class carried structurally); `frames` /
/// `frames-elided` are the error's OWN already-captured trace (step 2), never
/// re-snapshotted. The thread-tier sibling of
/// [`crate::process::died::process_died_error_runtime_from_error`]; used by
/// [`thread_crash_runtime_edn`].
pub(crate) fn thread_died_error_runtime_from_error(re: &RuntimeError) -> Value {
    loci_died_value(
        LociDiedError::RuntimeError,
        builtin_enum_variant_names(
            LociDiedError::WAT_TYPE_PATH,
            LociDiedError::RuntimeError.as_str(),
        ),
        vec![runtime_error_failure(re)],
    )
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
    loci_died_value(LociDiedError::Stopped, no_field_names(), vec![])
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

/// `(:wat::kernel::Failure/location f) -> (:Option :- [:wat::core::Span])` — arc 278
/// the string-wrap annihilation. DERIVED accessor: reads `error.location` (a
/// mandatory `:wat::core::Span` on the error) and wraps it in `Some` to keep
/// the accessor's historic `(Option :- [Span])` return shape.
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
                expected: "Span at :wat::core::Error/location",
                got: Box::new(ValueSnapshot::unavailable("error has no Span field")),
            },
        )
        .into()),
    }
}

/// `(:wat::kernel::Failure/actual f) -> (:Option :- [:wat::core::String])` — excursus 003
/// strike A (F2). `actual`/`expected` LEFT `Failure`'s own stored fields (they meant
/// something only for an assertion, and duplicated `:wat::runtime::AssertionFailed`'s own
/// fields); this is now a DERIVED accessor, the same shape `Failure/message` /
/// `Failure/location` already use. Reads `error.actual` when `error` is an
/// `AssertionFailed` record (it has that field); returns `:wat::core::Option::None` for
/// every other error kind (a `Fault`, a `DivisionByZero`, …, none of which carry it) —
/// `record_field_by_name` already answers `None` when the named field is absent on the
/// error's own registered type, so no per-class branch is needed here.
pub(crate) fn eval_failure_actual(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::Failure/actual";
    let error = failure_error_field(OP, args, env, sym, list_span)?;
    let types = sym.types().map(|a| a.as_ref());
    match record_field_by_name(&error, "actual", types) {
        Some(v @ Value::Option(_)) => Ok(v),
        _ => Ok(Value::Option(Arc::new(None))),
    }
}

/// `(:wat::kernel::Failure/expected f) -> (:Option :- [:wat::core::String])` — the
/// `expected` sibling of [`eval_failure_actual`]; same derivation, same fallback.
pub(crate) fn eval_failure_expected(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::Failure/expected";
    let error = failure_error_field(OP, args, env, sym, list_span)?;
    let types = sym.types().map(|a| a.as_ref());
    match record_field_by_name(&error, "expected", types) {
        Some(v @ Value::Option(_)) => Ok(v),
        _ => Ok(Value::Option(Arc::new(None))),
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
///
/// Excursus 003 strike A — cascade-aware: when `assertion` carries an `upstream_chain`
/// (arc 113 slice 2, `result::expect` re-panicking on an Err that already carried a
/// death chain), this thread's death is conj'd onto its FRONT (`conj_died_chain`)
/// instead of always emitting a singleton chain. A `None` upstream is unaffected —
/// `conj_died_chain(fresh, None)` is exactly `single_died_chain(fresh)` — so every
/// existing (non-cascading) caller's output is byte-identical to before. Now also the
/// ONE renderer `wat::panic_hook`'s hook calls for every `AssertionPayload` panic
/// (the main-thread / uncaught case), so a cascaded assertion reaches stderr with its
/// whole chain regardless of which locus caught it.
pub(crate) fn thread_crash_panic_edn(
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
    types: Option<&crate::types::TypeEnv>,
) -> String {
    let upstream = assertion.as_ref().and_then(|p| p.upstream_chain.clone());
    let chain = crate::process::died::conj_died_chain_value(
        thread_died_error_panic(message, assertion),
        upstream,
    );
    crate::edn::render::value_to_edn_string_lossy(&chain, types)
}

/// Arc 278 no-hidden-failures — the RuntimeError sibling of
/// [`thread_crash_panic_edn`]. Excursus 003 step 3b: the RuntimeError crosses the
/// wire as its OWN declared `:wat::runtime::<Kind>` record, structurally, inside a
/// `Failure` (`thread_died_error_runtime_from_error`) — never `to_wire_edn`/
/// `to_string()` prose — wrapped in the same bare `(Vector :- [LociDiedError])` line.
pub(crate) fn thread_crash_runtime_edn(
    re: &RuntimeError,
    types: Option<&crate::types::TypeEnv>,
) -> String {
    let chain = single_died_chain(thread_died_error_runtime_from_error(re));
    crate::edn::render::value_to_edn_string_lossy(&chain, types)
}

/// Excursus 003 step 3b — derive the human headline from a failure-carrying
/// `LociDiedError` variant's ONE `:wat::kernel::Failure` payload: `failure.error.message`.
/// Every one of the failure-carrying variants (`Panic` / `RuntimeError` / `StartupError` /
/// `MainSignature`) now shares this ONE shape — no
/// per-variant String-vs-structured-Error branching left to do (that was the whole
/// defect this step cures: the field used to be a bare `String` holding the error's
/// OWN serialized EDN for five of the six, and a structured `:wat::core::Error` for
/// the sixth). `types` resolves `error`'s and `message`'s field offsets by name
/// (`record_field_by_name`) — the SAME two-hop read `eval_failure_message` performs
/// for the wat-level `Failure/message` accessor, applied here to a `Failure` already
/// in hand rather than one still to be evaluated from an AST arg.
fn failure_payload_message(
    failure: &Value,
    types: Option<&crate::types::TypeEnv>,
) -> Option<Arc<String>> {
    let error = record_field_by_name(failure, "error", types)?;
    match record_field_by_name(&error, "message", types)? {
        Value::String(s) => Some(s),
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
/// Field 0 is the `:wat::kernel::Failure` payload for `Panic` / `RuntimeError` /
/// `StartupError` / `MainSignature` (excursus 003 step 3b; `EntryFormFailure` /
/// `BadReturn` retired at strike A); the derived message is `failure.error.message`.
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
    let types = sym.types().map(|a| a.as_ref());
    match val {
        Value::Enum(ev) if ev.type_path == LociDiedError::WAT_TYPE_PATH => {
            match ev.variant_name.parse::<LociDiedError>() {
                // Excursus 003 step 3b — every failure variant carries ONE
                // `:wat::kernel::Failure` at field 0; the message derives from
                // `failure.error.message` (`failure_payload_message`).
                Ok(LociDiedError::Panic)
                | Ok(LociDiedError::RuntimeError)
                | Ok(LociDiedError::StartupError)
                | Ok(LociDiedError::MainSignature) => {
                    match ev.fields.first().and_then(|f| failure_payload_message(f, types)) {
                        Some(s) => Ok(Value::String(s)),
                        None => Err(RuntimeError::new(
                            args[0].span().clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: ":wat::kernel::Failure inside *DiedError variant",
                                got: Box::new(ValueSnapshot::unavailable("non-Failure payload")),
                                // arc 138: no — matching on Value::Enum fields; no AST element
                            },
                        )
                        .into()),
                    }
                }
                Ok(LociDiedError::Disconnected) => {
                    Ok(Value::String(Arc::new("disconnected".to_string())))
                }
                // arc 170 Slice A — a stop was requested during recv. Wat-visible name is
                // "Stopped" (arc-170 intueri cast RULING A), not Rust's "shutdown".
                Ok(LociDiedError::Stopped) => {
                    Ok(Value::String(Arc::new("process stopped".to_string())))
                }
                Err(()) => Err(RuntimeError::new(
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
            return crate::edn::render::tag_is_variant_of(tag, LociDiedError::WAT_TYPE_PATH);
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
        Value::Enum(ev) if ev.type_path == LociDiedError::WAT_TYPE_PATH => {
            match ev.variant_name.parse::<LociDiedError>() {
                // Excursus 003 step 3b — field 0 IS the `:wat::kernel::Failure` now,
                // for every one of the failure-carrying variants (Panic included: its
                // separate `message` field and `Option<Failure>` field 1 are gone,
                // collapsed into this one mandatory `Failure`). `to-failure` is just
                // "hand back what is already there" — no more message-only synthesis
                // for a variant that carries real structure.
                Ok(LociDiedError::Panic)
                | Ok(LociDiedError::RuntimeError)
                | Ok(LociDiedError::StartupError)
                | Ok(LociDiedError::MainSignature) => match ev.fields.first() {
                    Some(failure @ Value::Aggregate(_)) => Ok(failure.clone()),
                    _ => Err(RuntimeError::new(
                        args[0].span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: ":wat::kernel::Failure at *DiedError field 0",
                            got: Box::new(ValueSnapshot::unavailable("non-Failure at field 0")),
                            // arc 138: no — matching on Value::Enum fields; no AST element
                        },
                    )
                    .into()),
                },
                Ok(LociDiedError::Disconnected) => {
                    Ok(message_only_failure("disconnected".to_string()))
                }
                // arc 170 Slice A — a stop was requested during recv. Wat-visible name is
                // "Stopped" (arc-170 intueri cast RULING A), not Rust's "shutdown".
                Ok(LociDiedError::Stopped) => {
                    Ok(message_only_failure("process stopped".to_string()))
                }
                Err(()) => Err(RuntimeError::new(
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
            //
            // Excursus 003 step 3b, item 4 — the OLD code discarded this Result with
            // `if let Ok(...)`, so a registration gap (a variant's cause type not
            // registered in `types`) took the SAME silent exit as a truly opaque
            // reason and came out looking like a plain, causeless panic. The shape
            // already told us this WAS a death-chain line; a decode failure here is
            // never opaque, and must say so.
            return match crate::edn::render::edn_to_value(&parsed, types, None) {
                Ok(Value::Vec(items)) => match items.first() {
                    Some(head) => head.clone(),
                    None => decode_failed_panic(
                        "death report decoded as an empty chain (Vector :- [LociDiedError]) \
                         with no head element"
                            .to_string(),
                    ),
                },
                Ok(other) => decode_failed_panic(format!(
                    "death report chain decoded to {}, not a Vector",
                    other.type_name()
                )),
                Err(e) => decode_failed_panic(format!("death report chain failed to decode: {e}")),
            };
        }
        // A single LociDiedError tagged value → bridge as-is. Same non-silent
        // treatment as the chain arm above: the tag already says this is a death
        // report, so a decode failure here names why, rather than falling to opaque.
        if let wat_edn::OwnedValue::Tagged(tag, _) = &parsed {
            if crate::edn::render::tag_is_variant_of(tag, LociDiedError::WAT_TYPE_PATH) {
                return match crate::edn::render::edn_to_value(&parsed, types, None) {
                    Ok(v) => v,
                    Err(e) => decode_failed_panic(format!("death report failed to decode: {e}")),
                };
            }
        }
    }
    // Genuinely opaque reason: not EDN at all, or EDN that is not shaped like a
    // LociDiedError chain or a single tagged variant — an OS-level error string, a
    // "recv EDN decode failed: …" prose note describing the *message* payload
    // (unrelated to the death-report shape), or a raw crash line that never became
    // well-formed EDN. Wrap as a Panic carrying the raw death message.
    loci_died_value(
        LociDiedError::Panic,
        builtin_enum_variant_names(LociDiedError::WAT_TYPE_PATH, LociDiedError::Panic.as_str()),
        vec![message_only_failure(reason)],
    )
}

/// Excursus 003 step 3b, item 4 — the shape of `reason` said "this is a
/// `LociDiedError`", but decoding it failed. Distinguishable from a genuinely
/// opaque reason (`loci_died_error_from_reason`'s final fallback): both wrap as
/// `Panic`, but this one's `Failure/message` NAMES the decode failure (and the
/// `EdnReadError` behind it) instead of silently repeating the raw bytes as if
/// nothing had been recognized at all.
fn decode_failed_panic(why: String) -> Value {
    loci_died_value(
        LociDiedError::Panic,
        builtin_enum_variant_names(LociDiedError::WAT_TYPE_PATH, LociDiedError::Panic.as_str()),
        vec![message_only_failure(why)],
    )
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
    loci_died_value(LociDiedError::Disconnected, no_field_names(), vec![])
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

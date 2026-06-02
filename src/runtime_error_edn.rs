//! Arc 233 Stone 233.3 — Errors-as-EDN extension.
//!
//! Generalizes arc 211b's `#wat.kernel/AssertionFailure` pattern across
//! all 28 [`crate::runtime::RuntimeError`] variants. Each variant
//! serializes as a tagged EDN envelope:
//!
//! ```text
//! #wat.kernel/<VariantName> {:field1 <edn-value> :field2 <edn-value> ...}
//! ```
//!
//! Wire format: single line, newline-terminated, parallel to
//! `#wat.kernel/AssertionFailure` (arc 211b) and
//! `#wat.kernel/ProcessPanics` (arc 170 slice 1i).
//!
//! ## Naming parallel with arc 211b
//!
//! Arc 211b introduced `#wat.kernel/AssertionFailure` for *panic*
//! payloads (when `assertion-failed!` is used inside a sandbox).
//! `RuntimeError::AssertionFailed` uses `#wat.kernel/AssertionFailed`
//! (variant name, present-tense) — both derive from the same assertion
//! machinery but live on distinct envelope types. The naming difference
//! is intentional: `AssertionFailure` = panic envelope;
//! `AssertionFailed` = runtime-error envelope.

use std::borrow::Cow;
use std::io::Write;
use wat_edn::{Keyword, OwnedValue, Tag};

use crate::runtime::{ClauseAttempt, ClauseFailureReason, Provenance, RuntimeError, RuntimeErrorKind, ValueSnapshot};
use crate::span::Span;

// ─── Public API ─────────────────────────────────────────────────────────────

/// Serialize a [`RuntimeError`] to a tagged [`OwnedValue`].
///
/// Each variant maps to `#wat.kernel/<VariantName> {<fields>}`.
/// Struct fields become EDN map keyword entries. Tuple-variant fields
/// get descriptive key names, never positional `:0 :1`.
pub fn runtime_error_to_edn(err: &RuntimeError) -> OwnedValue {
    // Pattern A: span lives at the outer struct; kind carries variant data.
    // Read self.span once; match self.kind for variant-specific fields.
    let span = &err.span;
    match &err.kind {
        // ── Tuple variants (simple: String + Span or just Span) ──────────
        RuntimeErrorKind::UnboundSymbol(name) => {
            tagged("UnboundSymbol", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::UnknownFunction(path) => {
            tagged("UnknownFunction", map2(
                kw("path"), str_val(path),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::ParamShadowsBuiltin(name) => {
            tagged("ParamShadowsBuiltin", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::DivisionByZero => {
            tagged("DivisionByZero", map1(
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::DuplicateDefine(name) => {
            tagged("DuplicateDefine", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::ReservedPrefix(prefix) => {
            tagged("ReservedPrefix", map2(
                kw("prefix"), str_val(prefix),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::DeclarationInExpressionPosition(head) => {
            tagged("DeclarationInExpressionPosition", map2(
                kw("head"), str_val(head),
                kw("span"), span_val(span),
            ))
        }

        // ── Struct variants ──────────────────────────────────────────────
        RuntimeErrorKind::NotCallable { got } => {
            tagged("NotCallable", map2(
                kw("got"), snap_val(got),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::TypeMismatch { op, expected, got } => {
            tagged("TypeMismatch", map4(
                kw("op"), str_val(op),
                kw("expected"), str_val(expected),
                kw("got"), snap_val(got),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::ArityMismatch { op, expected, got } => {
            tagged("ArityMismatch", map4(
                kw("op"), str_val(op),
                kw("expected"), OwnedValue::Integer(*expected as i64),
                kw("got"), OwnedValue::Integer(*got as i64),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::BadCondition { got } => {
            tagged("BadCondition", map2(
                kw("got"), snap_val(got),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::MalformedForm { head, reason } => {
            tagged("MalformedForm", OwnedValue::Map(vec![
                (kw("head"), str_val(head)),
                (kw("reason"), str_val(reason)),
                (kw("span"), span_val(span)),
            ]))
        }
        RuntimeErrorKind::EvalForbidsMutationForm { head } => {
            tagged("EvalForbidsMutationForm", map2(
                kw("head"), str_val(head),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::UserMainMissing => {
            // Freeze pair: span is Span::unknown(); elide from EDN.
            tagged("UserMainMissing", OwnedValue::Map(vec![]))
        }
        RuntimeErrorKind::EvalVerificationFailed { err } => {
            // Freeze pair: span is Span::unknown(); elide from EDN.
            // Lazy fallback: HashError rendered as Display string.
            tagged("EvalVerificationFailed", map1(
                kw("error"), str_val(&format!("{}", err)),
            ))
        }
        RuntimeErrorKind::ChannelDisconnected { op } => {
            tagged("ChannelDisconnected", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::NoEncodingCtx { op } => {
            tagged("NoEncodingCtx", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::NoSourceLoader { op } => {
            tagged("NoSourceLoader", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::NoMacroRegistry { op } => {
            tagged("NoMacroRegistry", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::MacroExpansionFailed { op, reason } => {
            tagged("MacroExpansionFailed", OwnedValue::Map(vec![
                (kw("op"), str_val(op)),
                (kw("reason"), str_val(reason)),
                (kw("span"), span_val(span)),
            ]))
        }
        RuntimeErrorKind::PatternMatchFailed { value_type } => {
            tagged("PatternMatchFailed", map2(
                kw("value-type"), str_val(value_type),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::EffectfulInStep { op } => {
            tagged("EffectfulInStep", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::NoStepRule { op } => {
            tagged("NoStepRule", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::AssertionFailed { message, actual, expected } => {
            // Mirrors #wat.kernel/AssertionFailure (arc 211b panic
            // envelope) but as a RuntimeError variant — see module doc.
            tagged("AssertionFailed", OwnedValue::Map(vec![
                (kw("message"), str_val(message)),
                (kw("actual"), opt_str_val(actual.as_deref())),
                (kw("expected"), opt_str_val(expected.as_deref())),
                (kw("span"), span_val(span)),
            ]))
        }
        RuntimeErrorKind::SandboxScopeLeak { offending_name, outer_define_span } => {
            // Multi-span: outer span = call_span (in err.span); secondary = outer_define_span.
            tagged("SandboxScopeLeak", OwnedValue::Map(vec![
                (kw("offending-name"), str_val(offending_name)),
                (kw("call-span"), span_val(span)),
                (kw("outer-define-span"), span_val(outer_define_span)),
            ]))
        }
        RuntimeErrorKind::ServiceNotRunning { op } => {
            tagged("ServiceNotRunning", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeErrorKind::EdnCoerceMismatch { op, expected, got, path } => {
            tagged("EdnCoerceMismatch", OwnedValue::Map(vec![
                (kw("op"), str_val(op)),
                (kw("expected"), str_val(expected)),
                (kw("got"), str_val(got)),
                (kw("path"), str_val(path)),
                (kw("span"), span_val(span)),
            ]))
        }
        RuntimeErrorKind::UnknownField { record_class, field, available } => {
            let available_edn = OwnedValue::Vector(
                available.iter().map(|s| str_val(s)).collect(),
            );
            tagged("UnknownField", OwnedValue::Map(vec![
                (kw("record-class"), str_val(record_class)),
                (kw("field"), str_val(field)),
                (kw("available"), available_edn),
                (kw("span"), span_val(span)),
            ]))
        }
        // Stone 237.4 — rich NoMatchingClause with structured ClauseAttempt list.
        RuntimeErrorKind::NoMatchingClause { name, called_arity, called_args, attempted_clauses } => {
            let called_args_edn = OwnedValue::Vector(
                called_args.iter().map(|s| snap_val(s)).collect(),
            );
            let attempted_edn = OwnedValue::Vector(
                attempted_clauses.iter().map(|a| clause_attempt_to_edn(a)).collect(),
            );
            tagged("NoMatchingClause", OwnedValue::Map(vec![
                (kw("name"), str_val(name)),
                (kw("called-arity"), OwnedValue::Integer(*called_arity as i64)),
                (kw("called-args"), called_args_edn),
                (kw("attempted-clauses"), attempted_edn),
                (kw("span"), span_val(span)),
            ]))
        }
        // Stone 237.4 — rich PostconditionFailed with ensure snapshot + dual spans.
        RuntimeErrorKind::PostconditionFailed { defclause_name, clause_index, ensure_expr_snapshot, returned_value, ensure_span } => {
            // Multi-span: outer span = body_span (in err.span); secondary = ensure_span.
            tagged("PostconditionFailed", OwnedValue::Map(vec![
                (kw("defclause-name"), str_val(defclause_name)),
                (kw("clause-index"), OwnedValue::Integer(*clause_index as i64)),
                (kw("ensure-expr-snapshot"), str_val(ensure_expr_snapshot)),
                (kw("returned-value"), snap_val(returned_value)),
                (kw("body-span"), span_val(span)),
                (kw("ensure-span"), span_val(ensure_span)),
            ]))
        }
    }
}

/// Serialize a [`ValueSnapshot`] to an EDN map.
///
/// Maps `{:type "...", :rendered "...", :provenance <provenance-edn>}`.
pub fn value_snapshot_to_edn(snap: &ValueSnapshot) -> OwnedValue {
    OwnedValue::Map(vec![
        (kw("type"), str_val(snap.type_name)),
        (kw("rendered"), str_val(&snap.rendered)),
        (kw("provenance"), provenance_to_edn(&snap.provenance)),
    ])
}

/// Serialize a [`Provenance`] to tagged EDN.
///
/// - `Unknown` → `nil`
/// - `Literal { span }` → `#wat.kernel/Literal {:span <map>}`
/// - `SymbolBound { binding_span, head_span }` → `#wat.kernel/SymbolBound {:binding-span ... :head-span ...}`
/// - `RuntimeBuilt { producer, call_span }` → `#wat.kernel/RuntimeBuilt {:producer "..." :call-span ...}`
pub fn provenance_to_edn(prov: &Provenance) -> OwnedValue {
    match prov {
        Provenance::Unknown => OwnedValue::Nil,
        Provenance::Literal { span } => {
            tagged("Literal", map1(kw("span"), span_val(span)))
        }
        Provenance::SymbolBound { binding_span, head_span } => {
            tagged("SymbolBound", map2(
                kw("binding-span"), span_val(binding_span),
                kw("head-span"), span_val(head_span),
            ))
        }
        Provenance::RuntimeBuilt { producer, call_span } => {
            tagged("RuntimeBuilt", map2(
                kw("producer"), str_val(producer),
                kw("call-span"), span_val(call_span),
            ))
        }
    }
}

/// Emit a `#wat.kernel/<VariantName> {<fields>}\n` envelope to `out`.
///
/// This is the HARD CUT wire format for RuntimeErrors crossing
/// IPC boundaries — no Display-text fallback.
pub fn emit_runtime_error_envelope<W: Write>(out: &mut W, err: &RuntimeError) {
    let edn_value = runtime_error_to_edn(err);
    let variant_name = variant_name(err);
    let line = format!("#wat.kernel/{} {}\n", variant_name, wat_edn::write(&edn_value));
    let _ = out.write_all(line.as_bytes());
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Extract the variant name string from a RuntimeError (for the wire
/// format prefix). Must stay in sync with the match arms in
/// `runtime_error_to_edn`.
fn variant_name(err: &RuntimeError) -> &'static str {
    match &err.kind {
        RuntimeErrorKind::UnboundSymbol(..) => "UnboundSymbol",
        RuntimeErrorKind::UnknownFunction(..) => "UnknownFunction",
        RuntimeErrorKind::NotCallable { .. } => "NotCallable",
        RuntimeErrorKind::TypeMismatch { .. } => "TypeMismatch",
        RuntimeErrorKind::ArityMismatch { .. } => "ArityMismatch",
        RuntimeErrorKind::BadCondition { .. } => "BadCondition",
        RuntimeErrorKind::MalformedForm { .. } => "MalformedForm",
        RuntimeErrorKind::ParamShadowsBuiltin(..) => "ParamShadowsBuiltin",
        RuntimeErrorKind::DivisionByZero => "DivisionByZero",
        RuntimeErrorKind::DuplicateDefine(..) => "DuplicateDefine",
        RuntimeErrorKind::ReservedPrefix(..) => "ReservedPrefix",
        RuntimeErrorKind::DeclarationInExpressionPosition(..) => "DeclarationInExpressionPosition",
        RuntimeErrorKind::EvalForbidsMutationForm { .. } => "EvalForbidsMutationForm",
        RuntimeErrorKind::UserMainMissing => "UserMainMissing",
        RuntimeErrorKind::EvalVerificationFailed { .. } => "EvalVerificationFailed",
        RuntimeErrorKind::ChannelDisconnected { .. } => "ChannelDisconnected",
        RuntimeErrorKind::NoEncodingCtx { .. } => "NoEncodingCtx",
        RuntimeErrorKind::NoSourceLoader { .. } => "NoSourceLoader",
        RuntimeErrorKind::NoMacroRegistry { .. } => "NoMacroRegistry",
        RuntimeErrorKind::MacroExpansionFailed { .. } => "MacroExpansionFailed",
        RuntimeErrorKind::PatternMatchFailed { .. } => "PatternMatchFailed",
        RuntimeErrorKind::EffectfulInStep { .. } => "EffectfulInStep",
        RuntimeErrorKind::NoStepRule { .. } => "NoStepRule",
        RuntimeErrorKind::AssertionFailed { .. } => "AssertionFailed",
        RuntimeErrorKind::SandboxScopeLeak { .. } => "SandboxScopeLeak",
        RuntimeErrorKind::ServiceNotRunning { .. } => "ServiceNotRunning",
        RuntimeErrorKind::EdnCoerceMismatch { .. } => "EdnCoerceMismatch",
        RuntimeErrorKind::UnknownField { .. } => "UnknownField",
        RuntimeErrorKind::NoMatchingClause { .. } => "NoMatchingClause",
        RuntimeErrorKind::PostconditionFailed { .. } => "PostconditionFailed",
    }
}

// ─── Low-level builders (eliminate boilerplate) ──────────────────────────────

fn kw(name: &'static str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

fn str_val(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}

fn opt_str_val(s: Option<&str>) -> OwnedValue {
    match s {
        Some(v) => OwnedValue::String(Cow::Owned(v.to_owned())),
        None => OwnedValue::Nil,
    }
}

fn span_val(span: &Span) -> OwnedValue {
    crate::panic_hook::span_to_edn(span)
}

fn snap_val(snap: &ValueSnapshot) -> OwnedValue {
    value_snapshot_to_edn(snap)
}

/// Stone 237.4 — serialize a [`ClauseAttempt`] to a tagged EDN map.
///
/// Each attempt renders as `#wat.kernel/ClauseAttempt {:clause-index N ...
/// :failure-reason #wat.kernel/<Reason> {...}}`.
fn clause_attempt_to_edn(attempt: &ClauseAttempt) -> OwnedValue {
    let arg_types_edn = OwnedValue::Vector(
        attempt.declared_arg_types.iter().map(|s| str_val(s)).collect(),
    );
    let reason_edn = clause_failure_reason_to_edn(&attempt.failure_reason);
    tagged("ClauseAttempt", OwnedValue::Map(vec![
        (kw("clause-index"), OwnedValue::Integer(attempt.clause_index as i64)),
        (kw("declared-arity"), OwnedValue::Integer(attempt.declared_arity as i64)),
        (kw("declared-arg-types"), arg_types_edn),
        (kw("failure-reason"), reason_edn),
    ]))
}

/// Stone 237.4 — serialize a [`ClauseFailureReason`] to a tagged EDN value.
///
/// Each variant renders as `#wat.kernel/<VariantName> {<fields>}`:
/// - `ArityMismatch` → `#wat.kernel/ArityMismatch {:expected N :got N}`
/// - `ArgTypeMismatch` → `#wat.kernel/ArgTypeMismatch {:position N :expected "..." :got "..."}`
/// - `GuardFalse` → `#wat.kernel/GuardFalse nil`
fn clause_failure_reason_to_edn(reason: &ClauseFailureReason) -> OwnedValue {
    match reason {
        ClauseFailureReason::ArityMismatch { expected, got } => {
            tagged("ArityMismatch", OwnedValue::Map(vec![
                (kw("expected"), OwnedValue::Integer(*expected as i64)),
                (kw("got"), OwnedValue::Integer(*got as i64)),
            ]))
        }
        ClauseFailureReason::ArgTypeMismatch { position, expected, got } => {
            tagged("ArgTypeMismatch", OwnedValue::Map(vec![
                (kw("position"), OwnedValue::Integer(*position as i64)),
                (kw("expected"), str_val(expected)),
                (kw("got"), str_val(got)),
            ]))
        }
        ClauseFailureReason::GuardFalse => {
            tagged("GuardFalse", OwnedValue::Nil)
        }
    }
}

fn tagged(variant: &'static str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns("wat.kernel", variant), Box::new(body))
}

fn map1(k1: OwnedValue, v1: OwnedValue) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1)])
}

fn map2(k1: OwnedValue, v1: OwnedValue, k2: OwnedValue, v2: OwnedValue) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1), (k2, v2)])
}

fn map4(
    k1: OwnedValue, v1: OwnedValue,
    k2: OwnedValue, v2: OwnedValue,
    k3: OwnedValue, v3: OwnedValue,
    k4: OwnedValue, v4: OwnedValue,
) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1), (k2, v2), (k3, v3), (k4, v4)])
}

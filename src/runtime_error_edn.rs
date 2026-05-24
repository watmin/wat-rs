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

use crate::runtime::{Provenance, RuntimeError, ValueSnapshot};
use crate::span::Span;

// ─── Public API ─────────────────────────────────────────────────────────────

/// Serialize a [`RuntimeError`] to a tagged [`OwnedValue`].
///
/// Each variant maps to `#wat.kernel/<VariantName> {<fields>}`.
/// Struct fields become EDN map keyword entries. Tuple-variant fields
/// get descriptive key names, never positional `:0 :1`.
pub fn runtime_error_to_edn(err: &RuntimeError) -> OwnedValue {
    match err {
        // ── Tuple variants (simple: String + Span or just Span) ──────────
        RuntimeError::UnboundSymbol(name, span) => {
            tagged("UnboundSymbol", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::UnknownFunction(path, span) => {
            tagged("UnknownFunction", map2(
                kw("path"), str_val(path),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::ParamShadowsBuiltin(name, span) => {
            tagged("ParamShadowsBuiltin", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::DivisionByZero(span) => {
            tagged("DivisionByZero", map1(
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::DuplicateDefine(name, span) => {
            tagged("DuplicateDefine", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::ReservedPrefix(prefix, span) => {
            tagged("ReservedPrefix", map2(
                kw("prefix"), str_val(prefix),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::DeclarationInExpressionPosition(head, span) => {
            tagged("DeclarationInExpressionPosition", map2(
                kw("head"), str_val(head),
                kw("span"), span_val(span),
            ))
        }

        // ── Struct variants ──────────────────────────────────────────────
        RuntimeError::NotCallable { got, span } => {
            tagged("NotCallable", map2(
                kw("got"), snap_val(got),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::TypeMismatch { op, expected, got, span } => {
            tagged("TypeMismatch", map4(
                kw("op"), str_val(op),
                kw("expected"), str_val(expected),
                kw("got"), snap_val(got),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::ArityMismatch { op, expected, got, span } => {
            tagged("ArityMismatch", map4(
                kw("op"), str_val(op),
                kw("expected"), OwnedValue::Integer(*expected as i64),
                kw("got"), OwnedValue::Integer(*got as i64),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::BadCondition { got, span } => {
            tagged("BadCondition", map2(
                kw("got"), snap_val(got),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::MalformedForm { head, reason, span } => {
            tagged("MalformedForm", OwnedValue::Map(vec![
                (kw("head"), str_val(head)),
                (kw("reason"), str_val(reason)),
                (kw("span"), span_val(span)),
            ]))
        }
        RuntimeError::EvalForbidsMutationForm { head, span } => {
            tagged("EvalForbidsMutationForm", map2(
                kw("head"), str_val(head),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::UserMainMissing => {
            tagged("UserMainMissing", OwnedValue::Map(vec![]))
        }
        RuntimeError::EvalVerificationFailed { err } => {
            // Lazy fallback: HashError rendered as Display string.
            // A future arc can deepen to a structured EDN map if needed.
            tagged("EvalVerificationFailed", map1(
                kw("error"), str_val(&format!("{}", err)),
            ))
        }
        RuntimeError::ChannelDisconnected { op, span } => {
            tagged("ChannelDisconnected", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::NoEncodingCtx { op, span } => {
            tagged("NoEncodingCtx", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::NoSourceLoader { op, span } => {
            tagged("NoSourceLoader", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::NoMacroRegistry { op, span } => {
            tagged("NoMacroRegistry", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::MacroExpansionFailed { op, reason, span } => {
            tagged("MacroExpansionFailed", OwnedValue::Map(vec![
                (kw("op"), str_val(op)),
                (kw("reason"), str_val(reason)),
                (kw("span"), span_val(span)),
            ]))
        }
        RuntimeError::PatternMatchFailed { value_type, span } => {
            tagged("PatternMatchFailed", map2(
                kw("value-type"), str_val(value_type),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::EffectfulInStep { op, span } => {
            tagged("EffectfulInStep", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::NoStepRule { op, span } => {
            tagged("NoStepRule", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::TryPropagate(value) => {
            // Carry the err-payload value as a ValueSnapshot (type + rendered).
            // A future arc can deepen to full structured encoding.
            let snap = ValueSnapshot::of(value);
            tagged("TryPropagate", map1(
                kw("value"), snap_val(&snap),
            ))
        }
        RuntimeError::OptionPropagate => {
            tagged("OptionPropagate", OwnedValue::Map(vec![]))
        }
        RuntimeError::TailCall { func, args, call_span } => {
            // Internal control-flow signal; render function name + arg
            // count + span. Reaching the user is a bug, so rich detail
            // is secondary to not panicking during serialization.
            let fn_name = func.name.as_deref().unwrap_or("<anonymous>").to_owned();
            tagged("TailCall", OwnedValue::Map(vec![
                (kw("fn-name"), str_val(&fn_name)),
                (kw("arg-count"), OwnedValue::Integer(args.len() as i64)),
                (kw("call-span"), span_val(call_span)),
            ]))
        }
        RuntimeError::AssertionFailed { message, actual, expected, span } => {
            // Mirrors #wat.kernel/AssertionFailure (arc 211b panic
            // envelope) but as a RuntimeError variant — see module doc.
            tagged("AssertionFailed", OwnedValue::Map(vec![
                (kw("message"), str_val(message)),
                (kw("actual"), opt_str_val(actual.as_deref())),
                (kw("expected"), opt_str_val(expected.as_deref())),
                (kw("span"), span_val(span)),
            ]))
        }
        RuntimeError::SandboxScopeLeak { offending_name, call_span, outer_define_span } => {
            tagged("SandboxScopeLeak", OwnedValue::Map(vec![
                (kw("offending-name"), str_val(offending_name)),
                (kw("call-span"), span_val(call_span)),
                (kw("outer-define-span"), span_val(outer_define_span)),
            ]))
        }
        RuntimeError::ServiceNotRunning { op, span } => {
            tagged("ServiceNotRunning", map2(
                kw("op"), str_val(op),
                kw("span"), span_val(span),
            ))
        }
        RuntimeError::EdnCoerceMismatch { op, expected, got, path, span } => {
            tagged("EdnCoerceMismatch", OwnedValue::Map(vec![
                (kw("op"), str_val(op)),
                (kw("expected"), str_val(expected)),
                (kw("got"), str_val(got)),
                (kw("path"), str_val(path)),
                (kw("span"), span_val(span)),
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
    match err {
        RuntimeError::UnboundSymbol(..) => "UnboundSymbol",
        RuntimeError::UnknownFunction(..) => "UnknownFunction",
        RuntimeError::NotCallable { .. } => "NotCallable",
        RuntimeError::TypeMismatch { .. } => "TypeMismatch",
        RuntimeError::ArityMismatch { .. } => "ArityMismatch",
        RuntimeError::BadCondition { .. } => "BadCondition",
        RuntimeError::MalformedForm { .. } => "MalformedForm",
        RuntimeError::ParamShadowsBuiltin(..) => "ParamShadowsBuiltin",
        RuntimeError::DivisionByZero(..) => "DivisionByZero",
        RuntimeError::DuplicateDefine(..) => "DuplicateDefine",
        RuntimeError::ReservedPrefix(..) => "ReservedPrefix",
        RuntimeError::DeclarationInExpressionPosition(..) => "DeclarationInExpressionPosition",
        RuntimeError::EvalForbidsMutationForm { .. } => "EvalForbidsMutationForm",
        RuntimeError::UserMainMissing => "UserMainMissing",
        RuntimeError::EvalVerificationFailed { .. } => "EvalVerificationFailed",
        RuntimeError::ChannelDisconnected { .. } => "ChannelDisconnected",
        RuntimeError::NoEncodingCtx { .. } => "NoEncodingCtx",
        RuntimeError::NoSourceLoader { .. } => "NoSourceLoader",
        RuntimeError::NoMacroRegistry { .. } => "NoMacroRegistry",
        RuntimeError::MacroExpansionFailed { .. } => "MacroExpansionFailed",
        RuntimeError::PatternMatchFailed { .. } => "PatternMatchFailed",
        RuntimeError::EffectfulInStep { .. } => "EffectfulInStep",
        RuntimeError::NoStepRule { .. } => "NoStepRule",
        RuntimeError::TryPropagate(..) => "TryPropagate",
        RuntimeError::OptionPropagate => "OptionPropagate",
        RuntimeError::TailCall { .. } => "TailCall",
        RuntimeError::AssertionFailed { .. } => "AssertionFailed",
        RuntimeError::SandboxScopeLeak { .. } => "SandboxScopeLeak",
        RuntimeError::ServiceNotRunning { .. } => "ServiceNotRunning",
        RuntimeError::EdnCoerceMismatch { .. } => "EdnCoerceMismatch",
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

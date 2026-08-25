//! Arc 233 Stone 233.3 — Errors-as-EDN extension.
//!
//! Formerly contained a hand-written `runtime_error_to_edn` match over all 28
//! [`crate::runtime::RuntimeError`] variants. Arc 298.3 deleted that serializer;
//! `RuntimeErrorKind` now carries `#[derive(wat_edn::ToEdn)]` and the
//! `impl ToEdn for RuntimeError` wrapper delegates to
//! `splice_span(self.kind.to_edn(), &self.span)`.
//!
//! ## What remains here
//!
//! - `emit_runtime_error_envelope`: public IPC wire-format writer
//! - `edn_path_segments`: via-helper for `EdnCoerceMismatch.path`
//! - `impl ToEdn / WatError` for `RuntimeError`, `ValueSnapshot`, `Provenance`,
//!   `ClauseAttempt` (the four building-block types that still need explicit impls)
//! - Low-level EDN builders used by the building-block impls
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

use crate::runtime::{ClauseAttempt, ClauseFailureReason, RuntimeError, ValueSnapshot};
use crate::value::Provenance;
use crate::span::Span;

// ─── Public API ─────────────────────────────────────────────────────────────

/// Emit a `#wat.kernel/<VariantName> {<fields>}\n` envelope to `out`.
///
/// This is the HARD CUT wire format for RuntimeErrors crossing
/// IPC boundaries — no Display-text fallback.
///
/// Arc 298.3: delegates to `err.to_edn()` (the derive-generated form)
/// and writes the tagged EDN line. Replaces the deleted `runtime_error_to_edn`
/// + `variant_name` pair.
pub fn emit_runtime_error_envelope<W: Write>(out: &mut W, err: &RuntimeError) {
    use crate::edn::contract::ToEdn;
    let line = format!("{}\n", wat_edn::write(&err.to_edn()));
    let _ = out.write_all(line.as_bytes());
}

/// Arc 298.3 — serialize a dot-notation path string to a vector of segments.
///
/// Used as `#[to_edn(via = crate::edn::error::edn_path_segments)]`
/// on `EdnCoerceMismatch.path` so the wire form stays `["seg1" "seg2"]`
/// rather than `"seg1.seg2"` — matching the hand-written serializer.
pub(crate) fn edn_path_segments(path: &str) -> OwnedValue {
    OwnedValue::Vector(
        wat_reader::identifier::dot_path_segments(path).into_iter().map(str_val).collect(),
    )
}

// ─── ToEdn + WatError impls ──────────────────────────────────────────────────

impl crate::edn::contract::ToEdn for RuntimeError {
    /// Pattern A: derive on RuntimeErrorKind generates the variant body;
    /// `:span` appended via `span.to_edn()` (Stone B: the derive-generated
    /// typed record replaces the hand-built `splice_span` helper).
    fn to_edn(&self) -> OwnedValue {
        use crate::edn::contract::edn_kw;
        let kind_val = self.kind().to_edn();
        match kind_val {
            OwnedValue::Tagged(tag, body) => {
                let mut fields = match *body {
                    OwnedValue::Map(f) => f,
                    other => vec![(edn_kw("body"), other)],
                };
                fields.push((edn_kw("span"), self.span().to_edn()));
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(fields)))
            }
            other => other,
        }
    }
}

impl crate::edn::contract::WatError for RuntimeError {
    /// Concise single-line headline: the span-free kind Display's first line
    /// (no `file:line` prefix — that lives in `:location`; no multi-line
    /// actual/expected detail — that lives in the structured variant fields).
    fn message(&self) -> String {
        crate::edn::contract::first_line(self.kind().to_string())
    }
    fn location(&self) -> OwnedValue {
        crate::edn::contract::location_from_span(self.span())
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> OwnedValue {
        use crate::edn::contract::ToEdn;
        crate::edn::contract::strip_span_from_tagged(self.to_edn())
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

impl crate::edn::contract::ToEdn for ValueSnapshot {
    fn to_edn(&self) -> OwnedValue {
        value_snapshot_to_edn(self)
    }
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

impl crate::edn::contract::ToEdn for Provenance {
    fn to_edn(&self) -> OwnedValue {
        provenance_to_edn(self)
    }
}

/// Arc 298.3 — `impl ToEdn for ClauseAttempt` wraps the free function so
/// the derive's `Vec<ClauseAttempt>::to_edn()` serializes each element.
impl crate::edn::contract::ToEdn for ClauseAttempt {
    fn to_edn(&self) -> OwnedValue {
        clause_attempt_to_edn(self)
    }
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

// ─── Low-level builders ──────────────────────────────────────────────────────

fn kw(name: &'static str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

fn str_val(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}

fn span_val(span: &Span) -> OwnedValue {
    use crate::edn::contract::ToEdn;
    span.to_edn()
}

fn tagged(variant: &'static str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(crate::error_ns::KERNEL, variant), Box::new(body))
}

fn map1(k1: OwnedValue, v1: OwnedValue) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1)])
}

fn map2(k1: OwnedValue, v1: OwnedValue, k2: OwnedValue, v2: OwnedValue) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1), (k2, v2)])
}

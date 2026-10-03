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
use wat_edn::{Keyword, OwnedValue, Tag};

use crate::edn::contract::edn_tag_dotted;
use crate::runtime::{ClauseAttempt, ClauseFailureReason, RuntimeError, ValueSnapshot};
use crate::value::Provenance;
use crate::span::Span;

// ─── Public API ─────────────────────────────────────────────────────────────

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
    ///
    /// Excursus 003 D3 — wired now, not left captured-and-unseen: `:frames`
    /// (wat frames innermost-first, THEN the one Rust frame — "user first",
    /// D3's own ordering) and `:frames-elided` (always present; 0 when the live
    /// stack fit under the cap) follow `:span`. Excursus 003 step 3c: this remains
    /// the shape of this type's plain `ToEdn` impl (kept — see `WatError::variant`'s
    /// own doc below for who still needs it), but it is NOT the wire anymore:
    /// `WatError::variant()` strips `:span`/`:frames`/`:frames-elided` back out, so
    /// `error_edn()` (`to_wire_edn`/`Debug`/`Display`, and every embedding via
    /// `error_edn_of`/`error_edn_of_boxed`) never shows them — frames live on
    /// `:wat::kernel::Failure` alone (step 3b).
    fn to_edn(&self) -> OwnedValue {
        use crate::edn::contract::edn_kw;
        let kind_val = self.kind().to_edn();
        let frames_val = OwnedValue::Vector(
            self.wat_frames()
                .iter()
                .map(crate::value::frame::Frame::to_edn)
                .chain(std::iter::once(self.rust_frame().to_edn()))
                .collect(),
        );
        match kind_val {
            OwnedValue::Tagged(tag, body) => {
                let mut fields = match *body {
                    OwnedValue::Map(f) => f,
                    other => vec![(edn_kw("body"), other)],
                };
                fields.push((edn_kw("span"), self.span().to_edn()));
                fields.push((edn_kw("frames"), frames_val));
                fields.push((
                    edn_kw("frames-elided"),
                    OwnedValue::Integer(self.frames_elided() as i64),
                ));
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
    fn location(&self) -> crate::span::Span {
        crate::edn::contract::location_from_span(self.span())
    }
    /// Excursus 003 step 3c — the wire's `variant()` strips BOTH `:span` (the floor
    /// owns `:location`) AND `:frames`/`:frames-elided` (they live on `Failure`, step
    /// 3b, never on a standalone error — see [`crate::edn::contract::strip_frames_from_tagged`]).
    /// `self.to_edn()` (this type's plain [`crate::edn::contract::ToEdn`] impl, above) is UNCHANGED and still
    /// carries all three: it is the shape ~30 golden-backed regression tests
    /// (`probe_arc298_3_runtime_derive_identical`, `probe_stone_233_3_runtime_error_edn`,
    /// `probe_arc237_stone4_rich_errors`, …) pin as the derive's byte-identical output,
    /// and it is also what `StartupError::to_edn_values`'s `--check-output edn|json`
    /// reads for a `StartupError::Runtime` (a registration-time, not mid-execution,
    /// failure — named and kept, not force-deleted, per this step's own brief).
    fn variant(&self) -> OwnedValue {
        use crate::edn::contract::{strip_frames_from_tagged, strip_span_from_tagged, ToEdn};
        strip_frames_from_tagged(strip_span_from_tagged(self.to_edn()))
    }
}

/// Serialize a [`ValueSnapshot`] to an EDN map.
///
/// Maps `{:type "...", :rendered "..."}`. Excursus 003 strike C: `:provenance`
/// removed per the 2026-09-27 ruling item 4 — the field left `ValueSnapshot`
/// itself (`src/value/observe.rs`); `provenance_to_edn`/`ToEdn for Provenance`
/// below are untouched (general `Provenance` EDN serialization, not specific
/// to this writer) — see the strike's report for their own measured use.
pub fn value_snapshot_to_edn(snap: &ValueSnapshot) -> OwnedValue {
    OwnedValue::Map(vec![
        (kw("type"), str_val(snap.type_name)),
        (kw("rendered"), str_val(&snap.rendered)),
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
/// Excursus 003 strike B2, item 5: `ClauseFailureReason` is a genuine wat `defenum`
/// (`:wat::kernel::ClauseFailureReason`, `wat/kernel/diagnostics.wat`), so its variants
/// carry the DOTTED tag `#wat.kernel/ClauseFailureReason.<Variant>` here too — matching
/// `RuntimeError::to_record`'s own writer (`clause_failure_reason_value`,
/// `src/value/runtime_records.rs`), which already rendered dotted via the registered
/// enum. The two writers used to disagree (this one was flat); both now call the SAME
/// dot-join (`edn_tag_dotted` → `wat_edn::Tag::enum_variant`, the same helper
/// `#[to_edn(qualified)]` emits), closing the gap step 3a's comment named:
/// - `ArityMismatch` → `#wat.kernel/ClauseFailureReason.ArityMismatch {:expected N :got N}`
/// - `ArgTypeMismatch` → `#wat.kernel/ClauseFailureReason.ArgTypeMismatch {:position N :expected "..." :got "..."}`
/// - `GuardFalse` → `#wat.kernel/ClauseFailureReason.GuardFalse {}`
fn clause_failure_reason_to_edn(reason: &ClauseFailureReason) -> OwnedValue {
    match reason {
        ClauseFailureReason::ArityMismatch { expected, got } => {
            edn_tag_dotted("ClauseFailureReason", "ArityMismatch", OwnedValue::Map(vec![
                (kw("expected"), OwnedValue::Integer(*expected as i64)),
                (kw("got"), OwnedValue::Integer(*got as i64)),
            ]))
        }
        ClauseFailureReason::ArgTypeMismatch { position, expected, got } => {
            edn_tag_dotted("ClauseFailureReason", "ArgTypeMismatch", OwnedValue::Map(vec![
                (kw("position"), OwnedValue::Integer(*position as i64)),
                (kw("expected"), str_val(expected)),
                (kw("got"), str_val(got)),
            ]))
        }
        ClauseFailureReason::GuardFalse => {
            // A unit variant's body is `{}`, never a bare `nil` (arc 278 A.0 — a
            // bare-nil tagged body is retired; `coerce_enum_path`'s
            // `EnumVariant::Unit` arm refuses anything else). The old FLAT
            // `#wat.kernel/GuardFalse` predates that convention and got away with
            // `nil` because nothing decoded it typed; the dotted form must match
            // what `to_record()`'s own Unit-variant render already produces.
            edn_tag_dotted("ClauseFailureReason", "GuardFalse", OwnedValue::Map(Vec::new()))
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

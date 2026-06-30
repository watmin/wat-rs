//! Arc 296 — `ToEdn` trait: the ONE serialization contract for every
//! error/diagnostic type.
//!
//! Before this arc, EDN serialization was a pile of ad-hoc free functions
//! (`runtime_error_to_edn`, `macro_error_to_edn`, `startup_error_to_edn`,
//! `payload_to_edn`, `span_to_edn`, …). Each function was wired manually;
//! a new error type shipped with no EDN form and nothing stopped it.
//!
//! Arc 296 mints ONE contract: every error/diagnostic type implements this
//! trait. The existing free functions become the impl bodies (or thin
//! wrappers around them). The serialization boundary (arc 296 slice 5)
//! is generic over `ToEdn` — a non-`ToEdn` error has no path to the
//! wire, making stringly diagnostics uncompilable.
//!
//! ## The compile fence (the wall)
//!
//! [`to_wire_edn`] is the single, named, generic conversion from an error to
//! the text that crosses the process boundary (the `ProcessDiedError`
//! payload). The `process_died_error_*_value` builders, `emit_structured_edn`
//! and the `--check-output` consumers are all **generic over `ToEdn`** — they
//! accept `impl ToEdn`, never a raw `String`. Adding a new error variant that
//! does NOT implement `ToEdn` produces a compile error at the first call site
//! that tries to reach the wire — the mistake is unrepresentable by
//! construction, not caught at runtime. A `compile_fail` doc-test on
//! [`to_wire_edn`] proves the wall is real, not aspirational.
//!
//! Genuinely message-only failures (a syscall error string, a return-type
//! name) travel through the same generic boundary as a [`FlatMessage`] —
//! they too are a `ToEdn` value, so the boundary never has to accept a bare
//! `String`.
//!
//! [`OwnedValue`] itself implements `ToEdn` as a passthrough (identity),
//! so pre-computed EDN values can be passed to the boundary without
//! unwrapping and re-wrapping.

use wat_edn::OwnedValue;

/// Serialize `self` to a structured tagged [`OwnedValue`].
///
/// Every error and diagnostic type implements this trait. The wire and IPC
/// boundaries are generic over `ToEdn`, so a type that does not implement
/// this trait cannot reach the wire.
///
/// ## Contract
/// - The returned value MUST be structured tagged EDN, NOT a bare
///   `OwnedValue::String`. A `String` payload at the wire boundary is a
///   violation of the "EDN all the way down" principle.
/// - Implementations delegate to the existing named free functions
///   (`runtime_error_to_edn`, `macro_error_to_edn`, …) so behavior is
///   byte-identical to the pre-trait path. The free functions are thin
///   wrappers once the impls exist.
pub trait ToEdn {
    fn to_edn(&self) -> OwnedValue;
}

/// Identity implementation: an already-serialized [`OwnedValue`] is itself
/// the EDN form.
///
/// This impl is the passthrough at the wire boundary — code that builds an
/// `OwnedValue` directly (e.g. via `make_simple_edn`) can pass it to the
/// generic `impl ToEdn` boundary without loss of type safety. The constraint
/// still holds: any type that is NOT already an `OwnedValue` and carries no
/// `ToEdn` impl cannot reach the wire.
impl ToEdn for OwnedValue {
    fn to_edn(&self) -> OwnedValue {
        self.clone()
    }
}

// ─── Shared low-level EDN builders ───────────────────────────────────────────
//
// One canonical home for the tag/keyword/string/int/span constructors so a
// new `ToEdn` impl does not copy the helpers a sixth time (the older
// `runtime_error_edn.rs` / `macros/error_edn.rs` / `check/error_edn.rs`
// serializers each carry a private copy; new impls call these instead).

use std::borrow::Cow;
use wat_edn::{Keyword, Tag};

/// `#wat.kernel/<variant> <body>` — the kernel-namespaced tagged envelope.
pub(crate) fn edn_tag(variant: &str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns("wat.kernel", variant), Box::new(body))
}

/// A keyword EDN value (`:name`). Accepts a dynamic string.
pub(crate) fn edn_kw(name: &str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

/// A string EDN value.
pub(crate) fn edn_str(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}

/// An integer EDN value.
pub(crate) fn edn_int(n: i64) -> OwnedValue {
    OwnedValue::Integer(n)
}

/// A span EDN value (`{:file … :line … :col …}`).
pub(crate) fn edn_span(span: &crate::span::Span) -> OwnedValue {
    crate::panic_hook::span_to_edn(span)
}

/// Append a `:key {span}` entry to `fields`, but ONLY when the span is known.
/// Unknown spans are elided (the same discipline as `push_span` in the
/// check serializer).
pub(crate) fn push_span_field(
    fields: &mut Vec<(OwnedValue, OwnedValue)>,
    key: &str,
    span: &crate::span::Span,
) {
    if !span.is_unknown() {
        fields.push((edn_kw(key), edn_span(span)));
    }
}

// ─── The wire boundary (the structural wall) ─────────────────────────────────

/// Convert any error to its wire EDN text **through its [`ToEdn`] impl**.
///
/// This is the single, named, generic conversion from an error to the text
/// that crosses the process boundary (the `ProcessDiedError` payload) or the
/// `--check-output` / structured-test stream. It is **generic over `ToEdn`**:
/// a type that does not implement the trait is a COMPILE error here, so it has
/// no path to the wire. This is the structural wall arc 296 promises —
/// "serialize a non-EDN-able error" has no representable form, it is not merely
/// "we currently happen to convert them all."
///
/// A type with NO `ToEdn` impl cannot reach the boundary:
///
/// ```compile_fail
/// struct NotSerializable;
/// // ERROR[E0277]: `NotSerializable: ToEdn` is not satisfied.
/// let _: String = wat::to_edn::to_wire_edn(&NotSerializable);
/// ```
///
/// A type that implements `ToEdn` passes:
///
/// ```
/// // `Span` implements `ToEdn`, so it reaches the boundary.
/// let span = wat::span::Span::unknown();
/// let _text: String = wat::to_edn::to_wire_edn(&span);
/// ```
pub fn to_wire_edn(e: &impl ToEdn) -> String {
    wat_edn::write(&e.to_edn())
}

/// The honest [`ToEdn`] form for a genuinely message-only failure — a syscall
/// error string, a `:user::main` return-type name. The string IS the datum
/// (there is no span, no kind, no structured sub-fields to lose); this is NOT
/// a stringified structured error.
///
/// Serializes to `#wat.kernel/<tag> {:<key> "<message>"}`. Using this type
/// (rather than passing a raw `String`) keeps the wire boundary generic over
/// `ToEdn` — even flat messages travel as a `ToEdn` value, so the boundary
/// never has to accept a bare `String`.
pub(crate) struct FlatMessage<'a> {
    pub tag: &'a str,
    pub key: &'a str,
    pub message: &'a str,
}

impl ToEdn for FlatMessage<'_> {
    fn to_edn(&self) -> OwnedValue {
        edn_tag(
            self.tag,
            OwnedValue::Map(vec![(edn_kw(self.key), edn_str(self.message))]),
        )
    }
}

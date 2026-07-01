//! Arc 296 / Arc 298.3 — EDN serializers for `MacroError` and `StartupError`.
//!
//! Arc 298.3 deleted `macro_error_to_edn`; `MacroErrorKind` now carries
//! `#[derive(wat_macros::ToEdn)]` and the `impl ToEdn for MacroError` wrapper
//! delegates to `splice_span(self.kind.to_edn(), &self.span)`.
//!
//! `startup_error_to_edn` is KEPT (transparent passthrough, no smuggle hazard).
//!
//! ## What remains here
//!
//! - `startup_error_to_edn`: public serializer for the startup pipeline
//! - `impl ToEdn / WatError` for `MacroError` (Pattern A: kind derive + splice_span)
//! - `impl ToEdn / WatError` for `StartupError` (transparent delegating wrapper)
//! - Low-level EDN builders used by `startup_error_to_edn` and `StartupError` impls

use std::borrow::Cow;
use wat_edn::{Keyword, OwnedValue, Tag};

use crate::macros::error::MacroError;
use crate::freeze::StartupError;

// ─── Public API ──────────────────────────────────────────────────────────────

/// Serialize a [`StartupError`] to a tagged [`OwnedValue`].
///
/// Every variant that carries a structured underlying error delegates to that
/// error's own `ToEdn` impl, so the wire value is fully navigable (span +
/// kind + fields) — no `:detail` prose blob smuggling structure in a string:
///
/// - `Macro` → `MacroError::to_edn` (the full typed cause chain).
/// - `Runtime` → `RuntimeError::to_edn` (arc 298.3: derive-generated).
/// - `Parse` → `ParseError::to_edn` (span + variant fields).
/// - `Config` → `ConfigError::to_edn` (Pattern A, span + fields).
/// - `Load` → `LoadError::to_edn` (Pattern A; nested `ParseError` structured).
/// - `Type` → `TypeError::to_edn` (Pattern A, span + 18 variants' fields).
/// - `Resolve` → `ResolveError::to_edn` (vector of structured references).
/// - `Check` → `CheckErrors::to_edn` (`#wat.kernel/CheckErrors {:errors […]}`,
///   each `CheckError` a navigable tagged value).
/// - `Stdlib` → `StdlibError::to_edn` (Pattern A, span + fields).
///
/// The phase is inferable from the returned tag's variant name (the same
/// convention `Macro`/`Runtime` already used). The ONLY variant that carries
/// a genuinely flat human message with no span/cause/structured fields is
/// `SigmaFn(String)` — a bare diagnostic string from the sigma-fn registration
/// path — so its `:detail` is honest, not a deferral.
pub fn startup_error_to_edn(err: &StartupError) -> OwnedValue {
    use crate::to_edn::ToEdn;
    match err {
        StartupError::Macro(e) => e.to_edn(),
        StartupError::Runtime(e) => e.to_edn(),
        StartupError::Parse(e) => e.to_edn(),
        StartupError::Config(e) => e.to_edn(),
        StartupError::Load(e) => e.to_edn(),
        StartupError::Type(e) => e.to_edn(),
        StartupError::Resolve(e) => e.to_edn(),
        StartupError::Check(e) => e.to_edn(),
        StartupError::Stdlib(e) => e.to_edn(),
        // SigmaFn carries a bare String message (no span, no kind, no
        // structured fields — see `StartupError::SigmaFn(String)`), so a
        // `:detail` string is the honest serialization, not a deferral.
        StartupError::SigmaFn(msg) => tagged(
            "SigmaFnError",
            OwnedValue::Map(vec![(kw("detail"), str_val(msg))]),
        ),
    }
}

// ─── ToEdn + WatError impls ──────────────────────────────────────────────────

impl crate::to_edn::ToEdn for MacroError {
    /// Arc 298.3 — Pattern A: derive on MacroErrorKind generates the
    /// variant body; `splice_span` appends `:span` from the outer struct.
    /// Replaces the deleted hand-written `macro_error_to_edn` match.
    fn to_edn(&self) -> OwnedValue {
        use crate::to_edn::splice_span;
        splice_span(self.kind.to_edn(), &self.span)
    }
}

impl crate::to_edn::WatError for MacroError {
    /// Concise single-line headline. The two nested-cause variants drop the
    /// embedded cause text (the cause is now carried structurally under
    /// `:cause` in floor form); every other variant uses the span-free kind
    /// Display's first line.
    fn message(&self) -> String {
        use crate::macros::error::MacroErrorKind;
        match &self.kind {
            MacroErrorKind::ProgramBodyEvalFailed { macro_name, .. } => {
                format!("macro {} — program body eval failed", macro_name)
            }
            MacroErrorKind::MacroEvalRuntimeFailed { .. } => {
                "macro_eval: runtime::eval failed".to_string()
            }
            _ => crate::to_edn::first_line(self.kind.to_string()),
        }
    }
    fn location(&self) -> OwnedValue {
        crate::to_edn::location_from_span(&self.span)
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> OwnedValue {
        use crate::to_edn::ToEdn;
        crate::to_edn::strip_span_from_tagged(self.to_edn())
    }
}

impl crate::to_edn::ToEdn for crate::freeze::StartupError {
    fn to_edn(&self) -> OwnedValue {
        startup_error_to_edn(self)
    }
}

impl crate::to_edn::WatError for crate::freeze::StartupError {
    /// `StartupError` is a TRANSPARENT wrapper: its `WatError` methods delegate
    /// to the inner error so `error_edn()` reconstructs the inner error's floor
    /// form EXACTLY (inner tag, inner `:message`, inner `:location`, inner
    /// `:causes`). The phase already lives in the inner error's tag
    /// (`#wat.kernel/MacroError`, `#wat.kernel/CheckErrors`, …), so no outer
    /// phase floor is layered on top (which would overwrite the inner
    /// `:location` with `nil`). The only genuinely flat arm is `SigmaFn`.
    fn message(&self) -> String {
        use crate::freeze::StartupError as SE;
        match self {
            SE::Macro(e) => e.message(),
            SE::Runtime(e) => e.message(),
            SE::Parse(e) => e.message(),
            SE::Config(e) => e.message(),
            SE::Load(e) => e.message(),
            SE::Type(e) => e.message(),
            SE::Resolve(e) => e.message(),
            SE::Check(e) => e.message(),
            SE::Stdlib(e) => e.message(),
            SE::SigmaFn(msg) => crate::to_edn::first_line(msg.clone()),
        }
    }
    fn location(&self) -> OwnedValue {
        use crate::freeze::StartupError as SE;
        match self {
            SE::Macro(e) => e.location(),
            SE::Runtime(e) => e.location(),
            SE::Parse(e) => e.location(),
            SE::Config(e) => e.location(),
            SE::Load(e) => e.location(),
            SE::Type(e) => e.location(),
            SE::Resolve(e) => e.location(),
            SE::Check(e) => e.location(),
            SE::Stdlib(e) => e.location(),
            SE::SigmaFn(_) => OwnedValue::Nil,
        }
    }
    fn causes(&self) -> OwnedValue {
        use crate::freeze::StartupError as SE;
        match self {
            SE::Macro(e) => e.causes(),
            SE::Runtime(e) => e.causes(),
            SE::Parse(e) => e.causes(),
            SE::Config(e) => e.causes(),
            SE::Load(e) => e.causes(),
            SE::Type(e) => e.causes(),
            SE::Resolve(e) => e.causes(),
            SE::Check(e) => e.causes(),
            SE::Stdlib(e) => e.causes(),
            SE::SigmaFn(_) => OwnedValue::Vector(vec![]),
        }
    }
    /// Delegates to the inner error's `variant()` (its own tagged, span-stripped
    /// map). `error_edn()` then composes the inner floor from the delegated
    /// `message`/`location`/`causes`, so the result IS the inner error's
    /// `error_edn()`. The `SigmaFn` arm carries a bare diagnostic string.
    fn variant(&self) -> OwnedValue {
        use crate::freeze::StartupError as SE;
        match self {
            SE::Macro(e) => e.variant(),
            SE::Runtime(e) => e.variant(),
            SE::Parse(e) => e.variant(),
            SE::Config(e) => e.variant(),
            SE::Load(e) => e.variant(),
            SE::Type(e) => e.variant(),
            SE::Resolve(e) => e.variant(),
            SE::Check(e) => e.variant(),
            SE::Stdlib(e) => e.variant(),
            SE::SigmaFn(msg) => tagged(
                "SigmaFnError",
                OwnedValue::Map(vec![(kw("detail"), str_val(msg))]),
            ),
        }
    }
}

// ─── Low-level builders ──────────────────────────────────────────────────────

fn tagged(variant: &'static str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns("wat.kernel", variant), Box::new(body))
}

fn kw(name: &'static str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

fn str_val(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}


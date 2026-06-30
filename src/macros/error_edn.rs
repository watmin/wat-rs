//! Arc 296 — EDN serializers for `MacroError` and `StartupError`.
//!
//! Extends arc 233's `runtime_error_to_edn` pattern upward through the
//! startup pipeline. Each error type serializes as a tagged EDN envelope:
//!
//! ```text
//! #wat.kernel/MacroError {:phase :macro :span {:file "…" :line N :col N}
//!                         :kind  #wat.kernel/MacroEvalRuntimeFailed
//!                                  {:cause #wat.kernel/UnboundSymbol {:name "str" :span …}}}
//! #wat.kernel/StartupError/Macro {:cause #wat.kernel/MacroError {…}}
//! ```
//!
//! ## Mirror of `runtime_error_edn.rs`
//!
//! Same builder helpers (`kw`, `str_val`, `tagged`, `span_val`); same
//! `#wat.kernel/<VariantName>` tag convention; same goal — no prose
//! strings carrying structured data at IPC boundaries.

use std::borrow::Cow;
use wat_edn::{Keyword, OwnedValue, Tag};

use crate::macros::error::{MacroError, MacroErrorKind};
use crate::freeze::StartupError;
use crate::span::Span;

// ─── Public API ──────────────────────────────────────────────────────────────

/// Serialize a [`MacroError`] to a tagged [`OwnedValue`].
///
/// Variant mapping:
/// - `MalformedTemplate` → `#wat.kernel/MalformedTemplate {:reason "…" :span {…}}`
/// - `ProgramBodyEvalFailed` → `#wat.kernel/ProgramBodyEvalFailed {:macro-name "…" :cause <MacroError>}`
/// - `MacroEvalRuntimeFailed` → `#wat.kernel/MacroEvalRuntimeFailed {:cause <RuntimeError>}`
/// - All other variants → `#wat.kernel/<VariantName> {:span {…} :detail "…"}`
///
/// The outer `MacroError` span lives at the struct level; each variant's
/// map includes it under `:span`. This mirrors Pattern A from arc 243.
pub fn macro_error_to_edn(err: &MacroError) -> OwnedValue {
    let span = &err.span;
    match &err.kind {
        MacroErrorKind::MalformedTemplate { reason } => {
            tagged("MalformedTemplate", map2(
                kw("reason"), str_val(reason),
                kw("span"), span_val(span),
            ))
        }
        MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause } => {
            tagged("ProgramBodyEvalFailed", OwnedValue::Map(vec![
                (kw("macro-name"), str_val(macro_name)),
                (kw("span"), span_val(span)),
                (kw("cause"), macro_error_to_edn(cause)),
            ]))
        }
        MacroErrorKind::MacroEvalRuntimeFailed { cause } => {
            tagged("MacroEvalRuntimeFailed", map2(
                kw("span"), span_val(span),
                kw("cause"), crate::runtime_error_edn::runtime_error_to_edn(cause),
            ))
        }
        MacroErrorKind::DuplicateMacro(name) => {
            tagged("DuplicateMacro", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        MacroErrorKind::ReservedPrefix(name) => {
            tagged("ReservedPrefix", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        MacroErrorKind::MalformedDefmacro { reason } => {
            tagged("MalformedDefmacro", map2(
                kw("reason"), str_val(reason),
                kw("span"), span_val(span),
            ))
        }
        MacroErrorKind::ArityMismatch { name, expected, got } => {
            tagged("ArityMismatch", OwnedValue::Map(vec![
                (kw("name"), str_val(name)),
                (kw("expected"), OwnedValue::Integer(*expected as i64)),
                (kw("got"), OwnedValue::Integer(*got as i64)),
                (kw("span"), span_val(span)),
            ]))
        }
        MacroErrorKind::ArityTooFew { name, minimum, got } => {
            tagged("ArityTooFew", OwnedValue::Map(vec![
                (kw("name"), str_val(name)),
                (kw("minimum"), OwnedValue::Integer(*minimum as i64)),
                (kw("got"), OwnedValue::Integer(*got as i64)),
                (kw("span"), span_val(span)),
            ]))
        }
        MacroErrorKind::UnboundMacroParam { name } => {
            tagged("UnboundMacroParam", map2(
                kw("name"), str_val(name),
                kw("span"), span_val(span),
            ))
        }
        MacroErrorKind::SpliceNotSequence { name, got } => {
            tagged("SpliceNotSequence", OwnedValue::Map(vec![
                (kw("name"), str_val(name)),
                (kw("got"), str_val(got)),
                (kw("span"), span_val(span)),
            ]))
        }
        MacroErrorKind::ExpansionDepthExceeded { limit } => {
            tagged("ExpansionDepthExceeded", map2(
                kw("limit"), OwnedValue::Integer(*limit as i64),
                kw("span"), span_val(span),
            ))
        }
        MacroErrorKind::RefusedInMacro { head } => {
            tagged("RefusedInMacro", map2(
                kw("head"), str_val(head),
                kw("span"), span_val(span),
            ))
        }
        MacroErrorKind::ProgramBodyIntroducesName { macro_name, binder } => {
            tagged("ProgramBodyIntroducesName", OwnedValue::Map(vec![
                (kw("macro-name"), str_val(macro_name)),
                (kw("binder"), str_val(binder)),
                (kw("span"), span_val(span)),
            ]))
        }
    }
}

/// Serialize a [`StartupError`] to a tagged [`OwnedValue`].
///
/// Every variant that carries a structured underlying error delegates to that
/// error's own `ToEdn` impl, so the wire value is fully navigable (span +
/// kind + fields) — no `:detail` prose blob smuggling structure in a string:
///
/// - `Macro` → `MacroError::to_edn` (the full typed cause chain).
/// - `Runtime` → `RuntimeError::to_edn` (arc 233's serializer).
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
        StartupError::Runtime(e) => crate::runtime_error_edn::runtime_error_to_edn(e),
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

// ─── ToEdn impls ─────────────────────────────────────────────────────────────

impl crate::to_edn::ToEdn for MacroError {
    fn to_edn(&self) -> OwnedValue {
        macro_error_to_edn(self)
    }
}

impl crate::to_edn::ToEdn for crate::freeze::StartupError {
    fn to_edn(&self) -> OwnedValue {
        startup_error_to_edn(self)
    }
}

// ─── Low-level builders (mirrors runtime_error_edn.rs) ───────────────────────

fn tagged(variant: &'static str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns("wat.kernel", variant), Box::new(body))
}

fn kw(name: &'static str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

fn str_val(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}

fn span_val(span: &Span) -> OwnedValue {
    crate::panic_hook::span_to_edn(span)
}

fn map2(k1: OwnedValue, v1: OwnedValue, k2: OwnedValue, v2: OwnedValue) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1), (k2, v2)])
}

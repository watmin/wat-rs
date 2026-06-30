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
/// The `Macro` variant delegates to `macro_error_to_edn`, surfacing the
/// full typed cause chain. The `Runtime` variant delegates to
/// `runtime_error_to_edn`. All other variants carry their display string
/// as `:detail` (arc 296 covers only the macro and runtime paths — other
/// startup variants are left with prose detail for now).
///
/// Tag convention: `#wat.kernel/StartupPhaseError {:phase :<variant> ...}`
/// for string-only variants; the Macro and Runtime variants return the
/// richer typed EDN directly (the phase is inferable from the tag).
pub fn startup_error_to_edn(err: &StartupError) -> OwnedValue {
    match err {
        // Arc 296 target: Macro variant carries the fully structured cause chain.
        StartupError::Macro(macro_err) => {
            macro_error_to_edn(macro_err)
        }
        // Runtime variant delegates to arc 233's serializer.
        StartupError::Runtime(e) => {
            crate::runtime_error_edn::runtime_error_to_edn(e)
        }
        // All other variants: structured envelope with `:phase` discriminator.
        StartupError::Parse(e) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("parse"),
                kw("detail"), str_val(&e.to_string()),
            ))
        }
        StartupError::Config(e) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("config"),
                kw("detail"), str_val(&e.to_string()),
            ))
        }
        StartupError::Load(e) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("load"),
                kw("detail"), str_val(&e.to_string()),
            ))
        }
        StartupError::Type(e) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("type"),
                kw("detail"), str_val(&e.to_string()),
            ))
        }
        StartupError::Resolve(e) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("resolve"),
                kw("detail"), str_val(&e.to_string()),
            ))
        }
        StartupError::Check(e) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("check"),
                kw("detail"), str_val(&e.to_string()),
            ))
        }
        StartupError::Stdlib(e) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("stdlib"),
                kw("detail"), str_val(&e.to_string()),
            ))
        }
        StartupError::SigmaFn(msg) => {
            tagged("StartupPhaseError", map2(
                kw("phase"), kw("sigma-fn"),
                kw("detail"), str_val(msg),
            ))
        }
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

fn map1(k1: OwnedValue, v1: OwnedValue) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1)])
}

fn map2(k1: OwnedValue, v1: OwnedValue, k2: OwnedValue, v2: OwnedValue) -> OwnedValue {
    OwnedValue::Map(vec![(k1, v1), (k2, v2)])
}

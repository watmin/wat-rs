//! Arc 296 / Arc 298.3 — EDN serializers for `MacroError` and `StartupError`.
//!
//! Arc 298.3 deleted `macro_error_to_edn`; `MacroErrorKind` now carries
//! `#[derive(wat_edn::ToEdn)]` and the `impl ToEdn for MacroError` wrapper
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
/// - `Validator` → the boxed [`crate::freeze::validator::FreezeValidatorError`]'s own
///   `to_edn` by dynamic dispatch — a registered `FreezeValidator` (e.g. the rete `defrule`
///   wall) keeps its own namespace tag (`#wat.rete/…`) through the box.
/// - `Stdlib` → `StdlibError::to_edn` (Pattern A, span + fields).
///
/// The phase is inferable from the returned tag's variant name (the same
/// convention `Macro`/`Runtime` already used). The ONLY variant that carries
/// a genuinely flat human message with no span/cause/structured fields is
/// `SigmaFn(String)` — a bare diagnostic string from the sigma-fn registration
/// path — so its `:detail` is honest, not a deferral.
pub fn startup_error_to_edn(err: &StartupError) -> OwnedValue {
    use crate::edn::contract::ToEdn;
    match err {
        StartupError::Macro(e) => e.to_edn(),
        StartupError::Runtime(e) => e.to_edn(),
        StartupError::Parse(e) => e.to_edn(),
        StartupError::Config(e) => e.to_edn(),
        StartupError::Load(e) => e.to_edn(),
        StartupError::Type(e) => e.to_edn(),
        StartupError::Resolve(e) => e.to_edn(),
        StartupError::Check(e) => e.to_edn(),
        StartupError::Validator(e) => e.to_edn(),
        StartupError::Stdlib(e) => e.to_edn(),
        // SigmaFn carries a bare String message (no kind, no structured
        // fields — see `StartupError::SigmaFn(String, Span)`), so a
        // `:detail` string is the honest serialization, not a deferral. The
        // span is the raising Rust site (excursus 003 step 3c) and surfaces
        // through `WatError::location`, not here (raw `to_edn()` mirrors the
        // pre-floor shape).
        StartupError::SigmaFn(msg, _) => tagged(
            "SigmaFnError",
            OwnedValue::Map(vec![(kw("detail"), str_val(msg))]),
        ),
        // MainSignature carries a bare String message (no kind — see
        // `StartupError::MainSignature(String, Span)`), same shape as SigmaFn.
        StartupError::MainSignature(msg, _) => tagged(
            "MainSignatureError",
            OwnedValue::Map(vec![(kw("detail"), str_val(msg))]),
        ),
    }
}

// ─── ToEdn + WatError impls ──────────────────────────────────────────────────

impl crate::edn::contract::ToEdn for MacroError {
    /// Pattern A: derive on MacroErrorKind generates the variant body;
    /// `:span` appended via `span.to_edn()` (Stone B).
    fn to_edn(&self) -> OwnedValue {
        use crate::edn::contract::edn_kw;
        let kind_val = self.kind.to_edn();
        match kind_val {
            OwnedValue::Tagged(tag, body) => {
                let mut fields = match *body {
                    OwnedValue::Map(f) => f,
                    other => vec![(edn_kw("body"), other)],
                };
                fields.push((edn_kw("span"), self.span.to_edn()));
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(fields)))
            }
            other => other,
        }
    }
}

impl crate::edn::contract::WatError for MacroError {
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
            _ => crate::edn::contract::first_line(self.kind.to_string()),
        }
    }
    fn location(&self) -> crate::span::Span {
        crate::edn::contract::location_from_span(&self.span)
    }
    fn variant(&self) -> OwnedValue {
        use crate::edn::contract::ToEdn;
        crate::edn::contract::strip_span_from_tagged(self.to_edn())
    }
}

impl crate::edn::contract::ToEdn for crate::freeze::StartupError {
    fn to_edn(&self) -> OwnedValue {
        startup_error_to_edn(self)
    }
}

impl crate::edn::contract::WatError for crate::freeze::StartupError {
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
            // Excursus 003 step 3c: `FreezeValidatorError` now requires `WatError`
            // (`src/freeze/validator.rs`), so the boxed validator error delegates
            // exactly like every other arm — the `first_line(e.to_string())` mask is gone.
            SE::Validator(e) => e.message(),
            SE::Stdlib(e) => e.message(),
            SE::SigmaFn(msg, _) => crate::edn::contract::first_line(msg.clone()),
            SE::MainSignature(msg, _) => crate::edn::contract::first_line(msg.clone()),
        }
    }
    fn location(&self) -> crate::span::Span {
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
            // Excursus 003 step 3c: delegates to the boxed validator's own location
            // (a `ReteCheckErrors`, whose location is its first item's — never `nil`).
            SE::Validator(e) => e.location(),
            SE::Stdlib(e) => e.location(),
            // A genuinely flat message: the location is the raising Rust site,
            // captured at construction (`crate::rust_caller_span!()`).
            SE::SigmaFn(_, span) => span.clone(),
            SE::MainSignature(_, span) => span.clone(),
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
            // Excursus 003 step 3c: delegates to the boxed validator's own `variant()`
            // (its tagged, span-stripped map) — the concrete namespace (e.g. #wat.rete/…)
            // survives by dynamic dispatch, the box never re-tags it.
            SE::Validator(e) => e.variant(),
            SE::Stdlib(e) => e.variant(),
            SE::SigmaFn(msg, _) => tagged(
                "SigmaFnError",
                OwnedValue::Map(vec![(kw("detail"), str_val(msg))]),
            ),
            SE::MainSignature(msg, _) => tagged(
                "MainSignatureError",
                OwnedValue::Map(vec![(kw("detail"), str_val(msg))]),
            ),
        }
    }
}

// ─── Low-level builders ──────────────────────────────────────────────────────

fn tagged(variant: &'static str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(crate::error_ns::MACRO, variant), Box::new(body))
}

fn kw(name: &'static str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

fn str_val(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}

// ─── Excursus 003 sweep S3 — the macro taxonomy's declaration gates ──────────
//
// G-list, G-strict (per the brief's per-strike gate list;
// docs/excursus/2026/09/003-the-little-wat-findings/
// BRIEF-shape-sweep-every-startup-error-is-a-declared-record.md), the same
// shape S1/S2 used.
//
// G-mirror (no golden moved) is a `git diff --stat -- '*.edn'` check, not a
// Rust assertion — stated in the strike report, not here.
#[cfg(test)]
mod excursus_003_s3_gates {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use crate::edn::contract::WatError;
    use crate::edn::render::decode_trusted_wire;
    use crate::macros::error::{MacroError, MacroErrorKind};
    use crate::runtime::{RuntimeError, RuntimeErrorKind};
    use crate::span::Span;
    use crate::types::TypeEnv;

    fn s() -> Span {
        Span::new(Arc::new("test.wat".to_string()), 1, 0)
    }

    /// One instance of every `MacroErrorKind` variant (16, measured against
    /// `src/macros/error.rs:42`), paired with its Rust variant name — the
    /// same name `error_edn()` tags it with on the wire (`#wat.macro/<Name>`).
    fn all_variants() -> Vec<(&'static str, MacroErrorKind)> {
        vec![
            ("DuplicateMacro", MacroErrorKind::DuplicateMacro("my-macro".into())),
            ("ReservedPrefix", MacroErrorKind::ReservedPrefix(":wat::my-thing".into())),
            ("UnnamespacedName", MacroErrorKind::UnnamespacedName("x".into())),
            ("DottedName", MacroErrorKind::DottedName(":user::x.y".into())),
            ("MalformedDefmacro", MacroErrorKind::MalformedDefmacro { reason: "missing name".into() }),
            ("ArityMismatch", MacroErrorKind::ArityMismatch { name: "my-macro".into(), expected: 2, got: 3 }),
            ("ArityTooFew", MacroErrorKind::ArityTooFew { name: "my-macro".into(), minimum: 1, got: 0 }),
            ("UnboundMacroParam", MacroErrorKind::UnboundMacroParam { name: "x".into() }),
            ("SpliceNotSequence", MacroErrorKind::SpliceNotSequence { name: "items".into(), got: "String" }),
            ("ExpansionDepthExceeded", MacroErrorKind::ExpansionDepthExceeded { limit: 64 }),
            ("MalformedTemplate", MacroErrorKind::MalformedTemplate { reason: "unexpected form".into() }),
            ("RefusedInMacro", MacroErrorKind::RefusedInMacro { head: ":wat::kernel::println".into() }),
            ("ExpandOnlyOutsideMacro", MacroErrorKind::ExpandOnlyOutsideMacro { head: ":wat::kernel::internal-only".into() }),
            ("ProgramBodyIntroducesName", MacroErrorKind::ProgramBodyIntroducesName {
                macro_name: "my-loop".into(), binder: "i".into(),
            }),
            ("ProgramBodyEvalFailed", MacroErrorKind::ProgramBodyEvalFailed {
                macro_name: "my-macro".into(),
                cause: Box::new(MacroError {
                    span: Span::new(Arc::new("inner.wat".to_string()), 3, 1),
                    kind: MacroErrorKind::MalformedTemplate { reason: "bad form".into() },
                }),
            }),
            ("MacroEvalRuntimeFailed", MacroErrorKind::MacroEvalRuntimeFailed {
                cause: Box::new(RuntimeError::new(
                    Span::new(Arc::new("rt.wat".to_string()), 7, 3),
                    RuntimeErrorKind::UnboundSymbol("foo".into()),
                )),
            }),
        ]
    }

    /// Scan `wat/macro-errors.wat`'s OWN source for every top-level
    /// `(:wat::core::defrecord :wat::macro::<Name> ...)` form.
    fn declared_macro_kind_names() -> BTreeSet<String> {
        let src = include_str!("../../wat/macro-errors.wat");
        let forms = wat_reader::parse_all_with_file(src, "wat/macro-errors.wat")
            .expect("wat/macro-errors.wat must parse");
        let mut names = BTreeSet::new();
        for form in &forms {
            let wat_reader::WatAST::List(items, _) = form else { continue };
            let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
            if head.as_str() != ":wat::core::defrecord" {
                continue;
            }
            let Some(wat_reader::WatAST::Keyword(name, _)) = items.get(1) else { continue };
            let Some(bare) = name.as_str().strip_prefix(":wat::macro::") else { continue };
            names.insert(bare.to_string());
        }
        names
    }

    /// G-list — the declaration is the list. The set of tags `error_edn()`
    /// produces for `all_variants()` must equal the set of `defrecord
    /// :wat::macro::*` names in `wat/macro-errors.wat`.
    ///
    /// Mutation (recorded in the strike report): add a stray
    /// `(:wat::core::defrecord :wat::macro::Bogus [])` to
    /// `wat/macro-errors.wat` — RED (declared has an extra name `produced`
    /// never names).
    #[test]
    fn g_list_declaration_is_the_list() {
        let produced: BTreeSet<String> = all_variants().iter().map(|(name, _)| name.to_string()).collect();
        let declared = declared_macro_kind_names();
        assert_eq!(
            produced, declared,
            "declared `:wat::macro::<Kind>` records in wat/macro-errors.wat must equal the \
             MacroErrorKind tags error_edn() produces"
        );
    }

    /// G-strict — every declared kind decodes typed, including the two
    /// nested-cause variants (`ProgramBodyEvalFailed`, self-referential to
    /// this very taxonomy; `MacroEvalRuntimeFailed`, whose `RuntimeErrorKind`
    /// cause was already fully declared in step 3a).
    ///
    /// Mutation (recorded in the strike report): comment out one
    /// `wat_record_from!` line in `src/types.rs` (e.g. `:wat::macro::
    /// ArityMismatch`) — RED, and the assertion message names `ArityMismatch`.
    #[test]
    fn g_strict_every_declared_kind_decodes_typed() {
        let types = TypeEnv::with_builtins();
        for (name, kind) in all_variants() {
            let err = MacroError { span: s(), kind };
            let wire = wat_edn::write(&err.error_edn());
            let decoded = decode_trusted_wire(&wire, Some(&types), None);
            assert!(
                decoded.is_ok(),
                "{name}: decode_trusted_wire(error_edn()) must succeed as a typed record; got {decoded:?}"
            );
        }
    }
}


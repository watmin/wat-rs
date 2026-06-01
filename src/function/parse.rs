//! # fn-form signature parsers
//!
//! ## Why this module exists
//!
//! Stone 241.18a — moved from `src/runtime.rs` (`parse_fn_signature`) and
//! `src/check.rs` (`parse_fn_signature_for_check`, `parse_fn_signature_for_check_diag`)
//! into the dedicated `src/function/` namespaced home per
//! `feedback_namespaced_home_vigilia_gate` REMARKABLE bar.
//!
//! These parsers share a single concern: consuming the canonical fn-form
//! signature layout `([name <- :T ...] -> :Ret)` and producing the parsed
//! triple `(param_names, param_types, ret_type)`. They differ only in their
//! error-handling contract:
//!
//! - `parse_fn_signature` — eval-tier; hard `RuntimeError` on malformed input.
//! - `parse_fn_signature_for_check` — check-tier silent; `Err(())` on mismatch.
//!
//! `parse_fn_signature_for_check_diag` — diagnostic check-tier — lives in
//! `infer.rs` because `infer_fn` is its only caller (keeping caller and callee
//! co-located per solvere).
//!
//! All three route argspec-triples through `crate::argspec::parse_argspec_triples`
//! (the canonical parser established at Stone 241.1-241.3).
//!
//! ## Arity contract
//!
//! Parsers accept exactly 3 args (after body and metadata are stripped by callers):
//!   `[ARGS-VECTOR, ->, :RET-TYPE]`
//! Body is NOT a parser concern — callers synthesize body independently.
//! Arity is enforced by the type (`&[WatAST; 3]`) — a wrong-length slice is
//! rejected at the call site, not inside the prefix. `ArityMismatch` as a
//! runtime variant is structurally impossible (Stone 243.4.1).

use crate::argspec::{parse_argspec_triples, ArgSpec, ArgSpecErrorKind, ParseOptions};
use crate::ast::WatAST;
use crate::function::FN_HEAD;
use crate::runtime::RuntimeError;
use crate::span::Span;
use crate::types::{parse_type_expr_with_span, TypeErrorKind, TypeExpr};

/// Step-location type for `parse_fn_signature_prefix` — Pattern A
/// (CONFORMARE.md §"The pattern").
///
/// `ParseStep` is the outer struct; span is STRUCTURALLY required — a
/// spanless parse-step error is uncompilable. `ParseStepKind` carries
/// variant-specific data only (no per-variant span fields).
///
/// The three outer parsers (`parse_fn_signature`, `parse_fn_signature_for_check`,
/// `parse_fn_signature_for_check_diag`) map this type to their tier's error
/// contract; the prefix itself does NOT produce error messages.
///
/// WHY pub(in crate::function): ParseStep is a fn-form parser intermediary
/// shared by parse.rs and infer.rs; pub(crate) would expose an internal
/// ladder type to the whole substrate; private would prevent infer.rs access.
pub(in crate::function) struct ParseStep {
    pub span: Span,
    pub kind: ParseStepKind,
}

/// Variant-specific data for `ParseStep` (Pattern A kind enum).
///
/// No span fields per variant — span lives on the outer `ParseStep` struct.
/// `ArityMismatch` has been eliminated: arity is now type-impossible via
/// `&[WatAST; 3]` (Stone 243.4.1 / CONFORMARE.md worked example).
pub(in crate::function) enum ParseStepKind {
    /// The first slot was not a `WatAST::Vector`. Carries the variant name
    /// of the form that was found instead.
    ArgsVecNotVector { found_variant: &'static str },
    /// The second slot was not the bare symbol `->`. Carries the variant name
    /// of the form that was found instead.
    ArrowMissing { found_variant: &'static str },
    /// The third slot was not a keyword. Carries the variant name of the form
    /// that was found instead.
    RetTypeNotKeyword { found_variant: &'static str },
    /// The return-type keyword failed `parse_type_expr_with_span`. Wraps the
    /// span-free `TypeErrorKind` — the outer `ParseStep.span` carries the
    /// location; storing `Box<TypeErrorKind>` avoids the double-stamp that
    /// `TypeError.to_string()` (which embeds the span in its Display) would
    /// produce when later placed into a `RuntimeError { span: step.span, .. }`.
    BadRetType(Box<TypeErrorKind>),
    /// The argspec triples failed. Wraps the span-free `ArgSpecErrorKind` —
    /// the outer `ParseStep.span` carries the location; `ArgSpecErrorKind::reason()`
    /// produces the span-free message.
    ArgSpecFailed(Box<ArgSpecErrorKind>),
}

impl ParseStepKind {
    /// Span-free human reason for this parse-step failure. Both tier mappers
    /// (eval RuntimeError, check CheckError) render through this — one source of truth.
    pub(in crate::function) fn reason(&self) -> String {
        match self {
            ParseStepKind::ArgsVecNotVector { found_variant } =>
                format!("fn signature: expected a vector `[name <- :T ...]` as the args-vector; got {found_variant}"),
            ParseStepKind::ArrowMissing { found_variant } =>
                format!("fn signature: expected `->` between args-vector and return type; got {found_variant}"),
            ParseStepKind::RetTypeNotKeyword { found_variant } =>
                format!("fn signature: expected a return-type keyword after `->` (e.g. `:wat::core::i64`); got {found_variant}"),
            ParseStepKind::BadRetType(k) => format!("invalid return type: {k}"),
            ParseStepKind::ArgSpecFailed(k) => k.reason(),
        }
    }
}

/// Shared structural prefix: Vector destructure + arrow match
/// + ret_type parse + argspec parse.
///
/// Returns `Err(ParseStep)` at the first structural mismatch; the three
/// outer parsers map the step to their tier's error contract.
///
/// Callers pass a fixed 3-element array `&[WatAST; 3]`: `[ARGS-VECTOR, ->, :RET-TYPE]`.
/// Arity is type-guaranteed — no runtime arity check required.
///
/// `parse_fn_signature_for_check_diag` in `src/function/infer.rs` routes through
/// this prefix and maps each `ParseStep` variant to its diagnostic tier contract.
pub(in crate::function) fn parse_fn_signature_prefix(
    sig: &[WatAST; 3],
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ParseStep> {
    let (args_vec, args_vec_span) = match &sig[0] {
        WatAST::Vector(items, span) => (items.as_slice(), span),
        other => {
            return Err(ParseStep {
                span: other.span().clone(),
                kind: ParseStepKind::ArgsVecNotVector {
                    found_variant: other.variant_name(),
                },
            });
        }
    };
    if !sig[1].is_bare_symbol("->") {
        return Err(ParseStep {
            span: sig[1].span().clone(),
            kind: ParseStepKind::ArrowMissing {
                found_variant: sig[1].variant_name(),
            },
        });
    }
    let ret_type: TypeExpr = match &sig[2] {
        WatAST::Keyword(k, span) => {
            parse_type_expr_with_span(k, span).map_err(|te| ParseStep {
                span: te.span,
                kind: ParseStepKind::BadRetType(Box::new(te.kind)),
            })?
        }
        other => {
            return Err(ParseStep {
                span: other.span().clone(),
                kind: ParseStepKind::RetTypeNotKeyword {
                    found_variant: other.variant_name(),
                },
            });
        }
    };
    let argspec: ArgSpec = parse_argspec_triples(
        args_vec,
        FN_HEAD,
        args_vec_span,
        ParseOptions { allow_rest_binder: false },
    )
    .map_err(|ae| ParseStep {
        span: ae.span,
        kind: ParseStepKind::ArgSpecFailed(Box::new(ae.kind)),
    })?;
    let (params, param_types): (Vec<String>, Vec<TypeExpr>) =
        argspec.fixed_params.into_iter().unzip();
    Ok((params, param_types, ret_type))
}

/// Arc 167 — flat-shape fn signature parser (eval tier).
///
/// Consumes the canonical fn-form signature layout (3 elements after body is
/// stripped by caller):
///
///   `[ARGS-VECTOR, ->, :RET-TYPE]`
///
/// `ARGS-VECTOR` is a `WatAST::Vector` whose body is flat triples
/// `name <- :T name <- :T ...` (empty vector → zero-arity fn). The
/// `<-` token reads as "consumes" — input direction; the sibling
/// `->` reads as "produces" — output direction. Arrows-as-duals.
///
/// Per 058-029, every parameter is typed and the return type is
/// required. No "untyped fn" exists in wat. This parser rejects
/// malformed flat-shape signatures with location-bearing errors.
///
/// Caller synthesizes body independently; parser sees only the 3-element
/// signature prefix. Moved from `src/runtime.rs` at Stone 241.18a.
// rune:excusare(OPEN-DEFERRAL → 243.7a) — clippy is correct (RuntimeError is large-by-value); the fix is the type-level boxing retrofit in Stone 243.7a (named, open, in-reach), not a per-site change. Struck the moment 243.7a ships.
#[allow(clippy::result_large_err)]
pub(crate) fn parse_fn_signature(
    args: &[WatAST; 3],
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), RuntimeError> {
    parse_fn_signature_prefix(args).map_err(|step| RuntimeError::MalformedForm {
        head: FN_HEAD.into(),
        reason: step.kind.reason(),
        span: step.span,
    })
}

/// Arc 167 — mirror of `parse_fn_signature` for the check pass.
///
/// Consumes the flat-shape fn-form signature layout (3 elements after body is
/// stripped by caller):
///
///   `[ARGS-VECTOR, ->, :RET-TYPE]`
///
/// Returns (names, types, ret). This is the SILENT CLASSIFIER: it answers
/// "is this a well-formed fn-shape?" for the `:ensure :fn` validator, NOT the
/// DIAGNOSTIC parser (`parse_fn_signature_for_check_diag` in `infer.rs`,
/// which surfaces each `ParseStep` as a `CheckError::MalformedForm` whose reason
/// comes from `ParseStepKind::reason()` — for the argspec case, that delegates to
/// `ArgSpecErrorKind::reason()`).
///
/// rune:sequi(reclassified-by-caller) — the `ArgSpecError` detail is
/// intentionally discarded to `()`. The sole caller (`:ensure :fn` validation
/// in `check.rs`) re-surfaces a coarser `CheckError::EnsureFnInvalid`
/// ("malformed :fn signature …") on the `Err(())` arm — that message IS the
/// intended UX for this path. Threading the sub-step detail through would
/// impair the deliberately-coarse `:ensure` diagnostic, so the discard is the
/// fix, not a defect. (The fine-grained path is A3, which keeps the detail.)
///
/// Moved from `src/check.rs` at Stone 241.18a.
pub(crate) fn parse_fn_signature_for_check(
    args: &[WatAST; 3],
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ()> {
    parse_fn_signature_prefix(args).map_err(|_| ())
}

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

use crate::argspec::{parse_argspec_triples, ArgSpec, ArgSpecError, ParseOptions};
use crate::ast::WatAST;
use crate::runtime::{ast_variant_name, RuntimeError};
use crate::span::Span;
use crate::types::{parse_type_expr_with_span, TypeError, TypeExpr};

/// Step-location enum for `parse_fn_signature_prefix`.
///
/// Each variant names the exact structural position that failed, carrying
/// enough context for each tier's error contract. The three outer parsers
/// (`parse_fn_signature`, `parse_fn_signature_for_check`,
/// `parse_fn_signature_for_check_diag`) map this enum to their tier's
/// error type; the prefix itself does NOT produce error messages.
pub(in crate::function) enum ParseStep {
    ArityMismatch { actual: usize },
    ArgsVecNotVector { found_variant: &'static str, span: Span },
    ArrowMissing { span: Span },
    RetTypeNotKeyword { span: Span },
    BadRetType(TypeError),
    ArgSpecFailed(ArgSpecError),
}

/// Shared structural prefix: arity gate + Vector destructure + arrow match
/// + ret_type parse + argspec parse.
///
/// Returns `Err(ParseStep)` at the first structural mismatch; the three
/// outer parsers map the step to their tier's error contract.
///
/// Callers pass a 3-element slice: `[ARGS-VECTOR, ->, :RET-TYPE]`.
///
/// `parse_fn_signature_for_check_diag` in `src/function/infer.rs` routes through
/// this prefix and maps each `ParseStep` variant to its diagnostic tier contract.
pub(in crate::function) fn parse_fn_signature_prefix(
    sig: &[WatAST],
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ParseStep> {
    if sig.len() != 3 {
        return Err(ParseStep::ArityMismatch { actual: sig.len() });
    }
    let (args_vec, args_vec_span) = match &sig[0] {
        WatAST::Vector(items, span) => (items.as_slice(), span),
        other => {
            return Err(ParseStep::ArgsVecNotVector {
                found_variant: ast_variant_name(other),
                span: other.span().clone(),
            });
        }
    };
    match &sig[1] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        other => {
            return Err(ParseStep::ArrowMissing {
                span: other.span().clone(),
            });
        }
    }
    let ret_type: TypeExpr = match &sig[2] {
        WatAST::Keyword(k, span) => parse_type_expr_with_span(k, span).map_err(ParseStep::BadRetType)?,
        other => {
            return Err(ParseStep::RetTypeNotKeyword {
                span: other.span().clone(),
            });
        }
    };
    let argspec: ArgSpec = parse_argspec_triples(
        args_vec,
        ":wat::core::fn",
        args_vec_span,
        ParseOptions { allow_rest_binder: false },
    )
    .map_err(ParseStep::ArgSpecFailed)?;
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
pub(crate) fn parse_fn_signature(
    args: &[WatAST],
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), RuntimeError> {
    parse_fn_signature_prefix(args).map_err(|step| match step {
        ParseStep::ArityMismatch { actual } => RuntimeError::MalformedForm {
            head: ":wat::core::fn".into(),
            reason: format!(
                "expected (:wat::core::fn [name <- :T ...] -> :Ret body); got {} args in signature prefix",
                actual
            ),
            span: Span::unknown(),
        },
        ParseStep::ArgsVecNotVector { found_variant, span } => RuntimeError::MalformedForm {
            head: ":wat::core::fn".into(),
            reason: format!(
                "fn signature must be a vector `[name <- :T ...]`; got {}",
                found_variant
            ),
            span,
        },
        ParseStep::ArrowMissing { span } => RuntimeError::MalformedForm {
            head: ":wat::core::fn".into(),
            reason: "fn signature missing `->` between args-vector and return type".into(),
            span,
        },
        ParseStep::RetTypeNotKeyword { span } => RuntimeError::MalformedForm {
            head: ":wat::core::fn".into(),
            reason: "fn signature missing return-type keyword after `->`".into(),
            span,
        },
        ParseStep::BadRetType(e) => {
            // WHY: each TypeError variant carries its own span field; extract via match
            let span = match &e {
                TypeError::MalformedTypeExpr { span, .. } => span.clone(),
                TypeError::AnyBanned { span, .. } => span.clone(),
                TypeError::InnerColonInCompoundArg { span, .. } => span.clone(),
                TypeError::AliasArityMismatch { span, .. } => span.clone(),
                TypeError::DuplicateType { span, .. } => span.clone(),
                TypeError::ReservedPrefix { span, .. } => span.clone(),
                TypeError::MalformedDecl { span, .. } => span.clone(),
                TypeError::MalformedName { span, .. } => span.clone(),
                TypeError::MalformedField { span, .. } => span.clone(),
                TypeError::MalformedVariant { span, .. } => span.clone(),
                TypeError::CyclicAlias { span, .. } => span.clone(),
                TypeError::CyclicUnion { span, .. } => span.clone(),
                TypeError::EmptyUnion { span, .. } => span.clone(),
                TypeError::SingleMemberUnion { span, .. } => span.clone(),
                TypeError::InvalidUnionMember { span, .. } => span.clone(),
                TypeError::CyclicSubtype { .. } => Span::unknown(),
            };
            RuntimeError::MalformedForm {
                head: ":wat::core::fn".into(),
                reason: e.to_string(),
                span,
            }
        },
        ParseStep::ArgSpecFailed(e) => {
            // WHY: map_err closure cannot use ?; convert explicitly via From impl
            // from the argspec home.
            RuntimeError::from(e)
        }
    })
}

/// Arc 167 — mirror of `parse_fn_signature` for the check pass.
///
/// Consumes the flat-shape fn-form signature layout (3 elements after body is
/// stripped by caller):
///
///   `[ARGS-VECTOR, ->, :RET-TYPE]`
///
/// Returns (names, types, ret). Errors are silenced. The `:ensure :fn`
/// call site doesn't produce user-facing diagnostics for malformed fn signatures;
/// if the fn is malformed, runtime parsing catches it and the checker simply
/// returns None.
///
/// Moved from `src/check.rs` at Stone 241.18a.
pub(crate) fn parse_fn_signature_for_check(
    args: &[WatAST],
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ()> {
    parse_fn_signature_prefix(args).map_err(|_| ())
}

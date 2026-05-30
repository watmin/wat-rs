//! # fn-form type inferencer
//!
//! ## Why this module exists
//!
//! Stone 241.18a — moved from `src/check.rs` into the dedicated
//! `src/function/` namespaced home per `feedback_namespaced_home_vigilia_gate`
//! REMARKABLE bar.
//!
//! `infer_fn` is the type-checker / type-inferencer for `:wat::core::fn`
//! expressions. It peels optional binding-level metadata, synthesizes the
//! implicit-do body, routes through `parse_fn_signature_for_check_diag` for
//! the signature, then checks the body against the declared return type under
//! the extended local environment.
//!
//! `parse_fn_signature_for_check_diag` lives here (not in parse.rs) because
//! `infer_fn` is its only caller — keeping caller and callee co-located
//! (solvere: no cross-module reach for single-caller private helpers).

use crate::ast::WatAST;
use crate::check::{
    apply_subst, format_type, infer, unify, CheckEnv, CheckError, CheckResult, InferCtx, Subst,
};
use crate::function::metadata::peel_metadata_preamble;
use crate::function::parse::{parse_fn_signature_prefix, ParseStep};
use crate::runtime::synthesize_fn_body;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// Returns `Some` on success. Returns `None` on any structural failure:
/// silently for ArityMismatch / ArgsVecNotVector / BadRetType (shape rejected
/// without diagnostic emission); after pushing a CheckError into `errors` for
/// ArrowMissing / RetTypeNotKeyword / ArgSpecFailed. Callers must drain
/// `errors` regardless of whether `None` accompanied a diagnostic push.
///
/// Accepts a 3-element slice: `[ARGS-VECTOR, ->, :RET-TYPE]`. Body is NOT
/// a parser concern — caller synthesizes body independently.
///
/// Moved from `src/check.rs` (via `src/function/parse.rs`) at Stone 241.18a.
/// Relocated to infer.rs at Stone 241.18a R0-remediation (C5) because
/// `infer_fn` is its only caller.
fn parse_fn_signature_for_check_diag(
    args: &[WatAST],
    errors: &mut Vec<CheckError>,
) -> Option<(Vec<String>, Vec<TypeExpr>, TypeExpr)> {
    match parse_fn_signature_prefix(args) {
        Ok((params, param_types, ret_type)) => Some((params, param_types, ret_type)),
        // Silent tiers: shape doesn't match canonical layout; caller falls through to None.
        Err(ParseStep::ArityMismatch { .. }) => None,
        Err(ParseStep::ArgsVecNotVector { .. }) => None,
        Err(ParseStep::BadRetType(_)) => None,
        // Diagnostic tiers: push CheckError so the user sees a clear error.
        Err(ParseStep::ArrowMissing { span }) => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::fn".into(),
                reason: "fn signature missing `->` between args-vector and return type".into(),
                span,
                remedies: vec![],
            });
            None
        }
        Err(ParseStep::RetTypeNotKeyword { span }) => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::fn".into(),
                reason: "fn signature missing return-type keyword after `->`".into(),
                span,
                remedies: vec![],
            });
            None
        }
        Err(ParseStep::ArgSpecFailed(e)) => {
            errors.push(e.into());
            None
        }
    }
}

/// Arc 155 retired `:wat::core::lambda`; arc 162 renamed this function
/// from `infer_lambda` to `infer_fn` to mirror the user-facing rename.
/// `:wat::core::lambda` has NO check arm — walker `BareLegacyLambda`
/// (src/check.rs) fires a fatal diagnostic at check time on any
/// user-source `:wat::core::lambda` form. Nothing routes lambda here.
/// This function is reached only via the `:wat::core::fn` check arm
/// (src/check.rs — the only active entry point).
///
/// A fn expression's type is `:wat::core::Fn(<param types>) -> <return type>`.
/// The signature is mandatory per 058-029 — every param and the
/// return are annotated. The body is checked against the declared
/// return type (same discipline as `check_function_body`).
///
/// Moved from `src/check.rs` at Stone 241.18a.
pub(crate) fn infer_fn(
    args: &[WatAST],
    env: &CheckEnv,
    outer_locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    let mut diagnostics: Vec<CheckError> = Vec::new();
    // Arc 167 — flat-shape fn signature consumer; arc 168 —
    // implicit-do body. Canonical form (3+ args after metadata peel):
    //   ARGS-VECTOR `->` :RET-TYPE  body1 body2 ... bodyN
    // Empty body is legal — the form's type is `:wat::core::nil`
    // (constrains the declared return type to `:wat::core::nil`).
    //
    // Stone 241.6 — fn-embedded metadata: defn macro expands
    // `(defn :name {meta} [args] -> :ret body)` to
    // `(def :name (fn {meta} [args] -> :ret body))`. The metadata
    // at args[0] is binding-level; peel it off so the type-checker
    // sees the real signature at args[1..]. The metadata was already
    // stored in binding_metadata by try_parse_fn_shape_def at
    // register_defines time.
    // Note: sister sequence in `src/function/eval.rs` (eval_fn).
    let sig_args = peel_metadata_preamble(args);
    if sig_args.len() < 3 {
        // HARVEST (236.2): silent-by-intent — malformed fn form with < 3 args;
        // parse won't even call check for badly-formed fn. Return fresh placeholder.
        return CheckResult::ok(fresh.fresh());
    }
    let body_ast = synthesize_fn_body(&sig_args[3..]);
    let (param_names, param_types, ret_type) =
        match parse_fn_signature_for_check_diag(&sig_args[..3], &mut diagnostics) {
            Some(parsed) => parsed,
            None => {
                // HARVEST (236.2): silent-by-intent — fn signature malformed; diag parser
                // already pushed errors into diagnostics if it could; return fresh placeholder.
                if diagnostics.is_empty() {
                    return CheckResult::ok(fresh.fresh());
                }
                return CheckResult::errs(diagnostics);
            }
        };

    // Check body against declared return type under extended locals.
    let mut body_locals = outer_locals.clone();
    for (name, ty) in param_names.iter().zip(param_types.iter()) {
        body_locals.insert(name.clone(), ty.clone());
    }
    // Push this fn's declared return type onto the enclosing-ret
    // stack so `try` inside the body propagates to the fn's
    // boundary (matches Rust's `?`-operator scoping — short-circuits
    // the innermost fn or closure, not the outer function).
    fresh.push_enclosing_ret(ret_type.clone());
    let body_ty = infer(&body_ast, env, &body_locals, fresh, subst).drain_errors_into(&mut diagnostics);
    fresh.pop_enclosing_ret();
    if let Some(body_ty) = body_ty {
        let body_span = body_ast.span();
        if unify(&body_ty, &ret_type, subst, env.types()).is_err() {
            // WHY: anonymous fn-form has no name; label by body span so users locate it in source
            diagnostics.push(CheckError::ReturnTypeMismatch {
                function: format!("<fn@{}>", body_span),
                expected: format_type(&apply_subst(&ret_type, subst)),
                got: format_type(&apply_subst(&body_ty, subst)),
                span: body_span.clone(),
                remedies: vec![],
            });
        }
    }

    let ty = TypeExpr::Fn {
        args: param_types,
        ret: Box::new(ret_type),
    };
    if diagnostics.is_empty() {
        CheckResult::ok(ty)
    } else {
        CheckResult::partial_with(ty, diagnostics)
    }
}

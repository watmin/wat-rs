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
use crate::function::parse::{parse_fn_signature_prefix, ParseStepKind};
use crate::function::FN_HEAD;
use crate::runtime::synthesize_fn_body;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// Outcome of fn-signature diagnostic parsing — makes the silent-vs-diagnostic
/// distinction structural (no `errors.is_empty()` side-channel inference).
enum SigParse {
    /// Parsed cleanly.
    Parsed(Vec<String>, Vec<TypeExpr>, TypeExpr),
    /// Outer form is not fn-shaped at all (ArgsVecNotVector) — silent: caller
    /// returns a fresh placeholder, no diagnostic.
    SilentReject,
    /// Fn-shaped but malformed — carries the diagnostic to surface.
    Diagnosed(CheckError),
}

fn parse_fn_signature_for_check_diag(args: &[WatAST; 3]) -> SigParse {
    match parse_fn_signature_prefix(args) {
        Ok((p, t, r)) => SigParse::Parsed(p, t, r),
        Err(step) if matches!(step.kind, ParseStepKind::ArgsVecNotVector { .. }) =>
            SigParse::SilentReject,
        Err(step) => SigParse::Diagnosed(CheckError::MalformedForm {
            head: FN_HEAD.into(),
            reason: step.kind.reason(),
            span: step.span,
            remedies: vec![],
        }),
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
    let mut errors: Vec<CheckError> = Vec::new();
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
    // sees the real signature (args[1..] when metadata is present; args
    // unchanged otherwise). The metadata was already stored in
    // binding_metadata by try_parse_fn_shape_def at register-defines time.
    // Note: sister sequence in `src/function/eval.rs` (eval_fn).
    let sig_args = peel_metadata_preamble(args);
    if sig_args.len() < 3 {
        // HARVEST (236.2): silent-by-intent — malformed fn form with < 3 args;
        // parse won't even call check for badly-formed fn. Return fresh placeholder.
        return CheckResult::ok(fresh.fresh());
    }
    let body_ast = synthesize_fn_body(&sig_args[3..]);
    // Safety: sig_args.len() >= 3 gated above; try_into on a 3-element prefix
    // cannot fail. The type guarantee eliminates the ArityMismatch class.
    let sig3: &[WatAST; 3] = sig_args[..3].try_into().expect("len >= 3 gated above");
    let (param_names, param_types, ret_type) = match parse_fn_signature_for_check_diag(sig3) {
        SigParse::Parsed(p, t, r) => (p, t, r),
        SigParse::SilentReject => {
            // Not fn-shaped at all — silent-by-intent; return a fresh placeholder.
            return CheckResult::ok(fresh.fresh());
        }
        SigParse::Diagnosed(err) => return CheckResult::err(err),
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
    let body_ty = infer(&body_ast, env, &body_locals, fresh, subst).drain_errors_into(&mut errors);
    fresh.pop_enclosing_ret();
    if let Some(body_ty) = body_ty {
        let body_span = body_ast.span();
        if unify(&body_ty, &ret_type, subst, env.types()).is_err() {
            // WHY: location rendered once via span_prefix in Display; human label carries no span
            errors.push(CheckError::ReturnTypeMismatch {
                function: ":anonymous".to_string(),
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
    if errors.is_empty() {
        CheckResult::ok(ty)
    } else {
        CheckResult::partial_with(ty, errors)
    }
}

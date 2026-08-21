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
    apply_subst, assignable, format_type, infer, CheckEnv, CheckError, CheckErrorKind, CheckResult, InferCtx, Subst,
};
use crate::function::metadata::{peel_metadata_preamble, peel_type_binder};
use crate::argspec::ParseOptions;
use crate::function::parse::{parse_fn_signature_prefix, ParseStepKind};
use crate::function::FN_HEAD;
use crate::runtime::synthesize_fn_body;
use crate::types::TypeExpr;
use crate::scope::Identifier;
use std::collections::HashMap;

/// Outcome of fn-signature diagnostic parsing — makes the silent-vs-diagnostic
/// distinction structural (no `errors.is_empty()` side-channel inference).
enum SigParse {
    /// Parsed cleanly. Carries fixed-param (names, types), return type, and
    /// optional rest-binder (name, type) from `& name <- :T`.
    Parsed(Vec<String>, Vec<TypeExpr>, TypeExpr, Option<(Identifier, TypeExpr)>),
    /// Outer form is not fn-shaped at all (ArgsVecNotVector) — silent: caller
    /// returns a fresh placeholder, no diagnostic.
    SilentReject,
    /// Fn-shaped but malformed — carries the diagnostic to surface.
    Diagnosed(CheckError),
}

fn parse_fn_signature_for_check_diag(args: &[WatAST; 3]) -> SigParse {
    // Arc 150 — allow rest-binders (`& name <- :T`) in fn-form signatures so
    // that variadic `defn` expansions (which land as `def + fn` forms at the
    // check pass) are accepted. Strict callers (`parse_fn_signature_for_check`
    // for `:ensure :fn` validation) still pass `allow_rest_binder: false`.
    match parse_fn_signature_prefix(args, ParseOptions { allow_rest_binder: true }) {
        Ok(sig) => {
            // CHECK tier — key by env_key (Stone 249.5e): mirrors the runtime Environment.
            let p = sig.params.iter().map(|id| crate::scope::resolution::env_key(id).into_owned()).collect();
            SigParse::Parsed(p, sig.param_types, sig.ret_type, sig.rest)
        }
        Err(step) if matches!(step.kind, ParseStepKind::ArgsVecNotVector { .. }) =>
            SigParse::SilentReject,
        Err(step) => SigParse::Diagnosed(CheckError {
            span: step.span,
            kind: CheckErrorKind::MalformedForm {
                head: FN_HEAD.into(),
                reason: step.kind.reason(),
                remedies: vec![],
            },
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
    // Arc 109 gamma-i — peel an optional `:- [T U ...]` type-param binder,
    // immediately after metadata and before the args-vector. Sister
    // sequence in `src/function/eval.rs` (eval_fn).
    let (binder, sig_args) = peel_type_binder(sig_args);
    if sig_args.len() < 3 {
        // HARVEST (236.2): silent-by-intent — malformed fn form with < 3 args;
        // parse won't even call check for badly-formed fn. Return fresh placeholder.
        return CheckResult::ok(fresh.fresh());
    }
    let body_ast = synthesize_fn_body(&sig_args[3..]);
    // Safety: sig_args.len() >= 3 gated above; try_into on a 3-element prefix
    // cannot fail. The type guarantee eliminates the ArityMismatch class.
    let sig3: &[WatAST; 3] = sig_args[..3].try_into().expect("len >= 3 gated above");
    let (param_names, mut param_types, mut ret_type, mut rest_param) = match parse_fn_signature_for_check_diag(sig3) {
        SigParse::Parsed(p, t, r, rest) => (p, t, r, rest),
        SigParse::SilentReject => {
            // Not fn-shaped at all — silent-by-intent; return a fresh placeholder.
            return CheckResult::ok(fresh.fresh());
        }
        SigParse::Diagnosed(err) => return CheckResult::err(err),
    };

    // Arc 109 gamma-i — generalize the binder's names into FRESH type
    // variables for THIS check pass. `infer_fn` runs exactly once per
    // `fn` AST node; the fresh Vars minted here let this ONE fn's body be
    // checked soundly (x and y both `:- [T]` must agree on T within a
    // single application). This is NOT let-polymorphism: it does not make
    // separate call sites of the SAME let-bound value re-instantiate
    // independently — that decision belongs to whatever resolves a Symbol
    // reference to its type (`locals.get` in `crate::check::infer`'s
    // Symbol arm), which stores one fixed `TypeExpr` per binding and is
    // check.rs, out of this stone's declared blast radius.
    if let Some(binder_names) = &binder {
        let mapping: HashMap<String, TypeExpr> = binder_names
            .iter()
            .map(|tp| (tp.clone(), fresh.fresh()))
            .collect();
        param_types = param_types.iter().map(|t| crate::check::rename(t, &mapping)).collect();
        ret_type = crate::check::rename(&ret_type, &mapping);
        rest_param = rest_param.map(|(ident, ty)| (ident, crate::check::rename(&ty, &mapping)));
    }

    // Check body against declared return type under extended locals.
    let mut body_locals = outer_locals.clone();
    for (name, ty) in param_names.iter().zip(param_types.iter()) {
        body_locals.insert(name.clone(), ty.clone());
    }
    // Arc 150 — variadic fn-forms: bind the rest-param name in body locals
    // with the declared Vector<T> type so uses of the rest-binder inside the
    // body (e.g. as the `xs` in `:wat::core::foldl ... xs`) type-check correctly.
    if let Some((rest_ident, rest_ty)) = rest_param {
        let rest_name = crate::scope::resolution::env_key(&rest_ident).into_owned();
        body_locals.insert(rest_name, rest_ty);
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
        // Arc 258 cascade — use `assignable` instead of bare `unify` so that a
        // specifically-typed record (e.g. :myapp::Voltage) satisfies a declared
        // return of :wat::core::Record via the is_subtype hierarchy.
        if !assignable(&body_ty, &ret_type, subst, env) {
            // WHY: location rendered once via span_prefix in Display; human label carries no span
            errors.push(CheckError {
                span: body_span.clone(),
                kind: CheckErrorKind::ReturnTypeMismatch {
                    function: ":anonymous".to_string(),
                    expected: format_type(&apply_subst(&ret_type, subst)),
                    got: format_type(&apply_subst(&body_ty, subst)),
                    remedies: vec![],
                },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::Identifier;
    use crate::types::TypeEnv;

    fn make_env_fresh_subst() -> (TypeEnv, InferCtx, Subst) {
        (TypeEnv::default(), InferCtx::default(), HashMap::new())
    }

    // ─── Line 109: infer_fn with fewer than 3 args after metadata peel ─────────

    /// When `sig_args.len() < 3` (line 109), `infer_fn` silently returns a
    /// fresh type variable rather than producing a diagnostic — the malformed fn
    /// form is ill-shaped in a way that an earlier pass should have caught.
    /// Asserts the result is `ok` (no errors) with a `Var(_)` placeholder.
    #[test]
    fn infer_fn_fewer_than_3_args_returns_fresh_placeholder() {
        let (types, mut fresh, mut subst) = make_env_fresh_subst();
        let check_env = crate::check::CheckEnv::with_builtins_and_types(&types);
        let locals: HashMap<String, TypeExpr> = HashMap::new();
        // Only 1 element — too few to be a valid fn-form signature.
        let args = &[WatAST::nil()];
        let result = infer_fn(args, &check_env, &locals, &mut fresh, &mut subst);
        assert!(result.is_ok(), "expected ok result for < 3 args; got errors: {:?}", result.errors());
        assert!(
            matches!(result.value(), Some(TypeExpr::Var(_))),
            "expected Var placeholder for < 3 args; got: {:?}",
            result.value()
        );
    }

    // ─── Lines 57 + 119: SilentReject path in parse_fn_signature_for_check_diag ─

    /// When `sig[0]` is not a `WatAST::Vector`, `parse_fn_signature_for_check_diag`
    /// returns `SigParse::SilentReject` (line 57), which causes `infer_fn` to
    /// silently return a fresh placeholder (line 119). This path is intentionally
    /// silent — the non-fn-shaped form is handled by other checker arms.
    #[test]
    fn infer_fn_non_vector_args_returns_silent_placeholder() {
        let (types, mut fresh, mut subst) = make_env_fresh_subst();
        let check_env = crate::check::CheckEnv::with_builtins_and_types(&types);
        let locals: HashMap<String, TypeExpr> = HashMap::new();
        let span = crate::rust_caller_span!();
        // sig[0] is a Keyword, not a Vector → ArgsVecNotVector → SilentReject.
        let args = &[
            WatAST::Keyword(":not-a-vec".into(), span.clone()),
            WatAST::Symbol(Identifier::bare("->"), span.clone()),
            WatAST::Keyword(":wat::core::i64".into(), span.clone()),
        ];
        let result = infer_fn(args, &check_env, &locals, &mut fresh, &mut subst);
        assert!(result.is_ok(), "expected ok (silent) result; got errors: {:?}", result.errors());
        assert!(
            matches!(result.value(), Some(TypeExpr::Var(_))),
            "expected Var placeholder for SilentReject; got: {:?}",
            result.value()
        );
    }
}

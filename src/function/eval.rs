//! # fn-form evaluator
//!
//! ## Why this module exists
//!
//! Stone 241.18a — moved from `src/runtime.rs` into the dedicated
//! `src/function/` namespaced home per `feedback_namespaced_home_vigilia_gate`
//! REMARKABLE bar.
//!
//! `eval_fn` is the runtime evaluator for `:wat::core::fn` expressions.
//! It peels optional binding-level metadata, synthesizes the implicit-do body,
//! and routes through `parse_fn_signature_with_rest` to produce a `Function` value.

use crate::ast::WatAST;
use crate::function::metadata::peel_metadata_preamble;
use crate::function::parse::{parse_fn_signature_with_rest, ParsedFnSignature};
use crate::function::FN_HEAD;
use crate::runtime::{Environment, Function, RuntimeError, RuntimeErrorKind, Value, synthesize_fn_body};
use crate::span::Span;
use std::sync::Arc;

/// Arc 155 retired `:wat::core::lambda`; arc 162 renamed this function
/// from `eval_lambda` to `eval_fn` to mirror the user-facing rename.
/// `:wat::core::lambda` has NO dispatch arm — walker `BareLegacyLambda`
/// (src/check.rs) fires a fatal diagnostic at check time on any
/// user-source `:wat::core::lambda` form. Nothing routes lambda here at
/// runtime. This function is reached only via the `:wat::core::fn`
/// dispatch arm (src/runtime.rs — the only active entry point).
///
/// Moved from `src/runtime.rs` at Stone 241.18a.
pub(crate) fn eval_fn(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
) -> Result<Value, RuntimeError> {
    // Stone 241.6 — fn-embedded metadata peel. The defn macro expands
    // `(defn :name {meta} [args] -> :ret body)` to
    // `(def :name (fn {meta} [args] -> :ret body))`. The metadata at
    // args[0] is binding-level; peel it off so eval_fn sees the real sig.
    // The metadata was already stored in binding_metadata at register_defines
    // time via try_parse_fn_shape_def's fn-embedded metadata path.
    // Note: sister sequence in `src/function/infer.rs` (infer_fn).
    let sig_args = peel_metadata_preamble(args);
    if sig_args.len() < 3 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: FN_HEAD.into(),
            reason: format!("expected [name <- :T ...] -> :Ret body ...; got {} element(s)", sig_args.len())
        }));
    }
    let body = synthesize_fn_body(&sig_args[3..]);
    // Safety: sig_args.len() >= 3 gated above; try_into on a 3-element prefix
    // cannot fail. The type guarantee eliminates the ArityMismatch class.
    let sig3: &[WatAST; 3] = sig_args[..3].try_into().expect("len >= 3 gated above");
    // Arc 150 — parse with rest-binder support so variadic fn-forms (from
    // variadic `defn` expansion) produce a Function with rest_param set.
    // Non-variadic forms produce rest = None — strict behavior unchanged.
    let ParsedFnSignature { params, param_types, ret_type, rest } = parse_fn_signature_with_rest(sig3)?;
    let (rest_param, rest_param_type) = match rest {
        // `rest_param` is a lookup key (never re-emitted as a binder node), so
        // flatten it; `params` stay whole. Arc 170.
        Some((name, ty)) => (Some(crate::scope::env_key(&name).into_owned()), Some(ty)),
        None => (None, None),
    };
    Ok(Value::wat__core__fn(Arc::new(Function {
        name: None,
        params,
        type_params: Vec::new(),
        param_types,
        ret_type,
        rest_param,
        rest_param_type,
        body: crate::value::FunctionBody::Wat(Arc::new(body)),
        closed_env: Some(env.clone()),
        rete: None,
        synthesized_for: None,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lines 44-47: `eval_fn` returns `MalformedForm` when the fn-form has
    /// fewer than 3 elements after the metadata peel. In production the parser
    /// always emits at least 3 (args-vec, `->`, ret-type), but directly-
    /// constructed AST can carry fewer — the guard is the production safety net.
    #[test]
    fn eval_fn_fewer_than_3_args_returns_malformed_form() {
        let span = crate::rust_caller_span!();
        let env = Environment::new();
        // Only 1 arg after the fn head has been stripped by the caller — too few.
        let args = &[WatAST::Vector(vec![], span.clone())];
        let err = eval_fn(args, &span, &env).unwrap_err();
        let reason = match err.kind() {
            RuntimeErrorKind::MalformedForm { reason, .. } => reason,
            _ => panic!("expected MalformedForm, got {:?}", err),
        };
        assert_eq!(reason, "expected [name <- :T ...] -> :Ret body ...; got 1 element(s)");
    }
}

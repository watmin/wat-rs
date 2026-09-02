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
//!
//! ## Arc 109 addendum — `defclause` call dispatch joins this file
//!
//! A `defclause` is the fn-form's multi-clause shape, so its evaluators —
//! `eval_call_to_defclause` (evaluate args, delegate), `select_defclause_clause`
//! (arity + type + `:guard` selection, no body eval — the door both the ordinary call
//! path and `eval_tail`'s clause arm share), `eval_call_to_defclause_with_vals` (select,
//! run the body, check `:ensure`) — moved verbatim out of `src/runtime.rs` here (arc 109
//! the defclause-into-function-home stone): evaluation joins evaluation, per this home's
//! ACT split. `value_matches_type_by_name`/`val_type_path`, the runtime type-matcher
//! `select_defclause_clause` dispatches through, live in the new `subsume.rs` sibling —
//! kept apart because they are consulted by BOTH the reachability wall in `parse.rs` and
//! the selector here, not because either role owns them.
//!
//! All three moved items are bumped from private to `pub(crate)`: none carries
//! `#[wat_intrinsic]` (`eval_call_to_defclause`/`eval_call_to_defclause_with_vals`/
//! `select_defclause_clause` are dispatched by explicit calls in `runtime.rs`'s own eval
//! spine, not through the intrinsic registry), so the bump is required for that
//! cross-module call, not a signature change.

use crate::ast::WatAST;
use crate::function::metadata::{peel_metadata_preamble, peel_type_binder};
use crate::function::parse::{parse_fn_signature_with_rest, ParsedFnSignature};
use crate::function::subsume::{val_type_path, value_matches_type_by_name};
use crate::function::FN_HEAD;
use crate::span::Span;
use crate::value::{
    ClauseAttempt, ClauseFailureReason, ClauseSet, Environment, EvalBreak, Function, RuntimeError,
    RuntimeErrorKind, SymbolTable, TrackedValue, Value, ValueSnapshot,
};
use std::sync::Arc;

// `eval_inner`, `apply_function`, and `synthesize_fn_body` are genuinely defined in
// `crate::runtime` (not a facade re-export of a `crate::value` type — see STOP-2) and stay
// there; none is one of this stone's 12 items.
use crate::runtime::{apply_function, eval_inner, synthesize_fn_body};

/// Arc 155 retired `:wat::core::lambda`; arc 162 renamed this function
/// from `eval_lambda` to `eval_fn` to mirror the user-facing rename.
/// `:wat::core::lambda` has NO dispatch arm — walker `BareLegacyLambda`
/// (src/check.rs) fires a fatal diagnostic at check time on any
/// user-source `:wat::core::lambda` form. Nothing routes lambda here at
/// runtime. This function is reached only via the `:wat::core::fn`
/// dispatch arm (src/runtime.rs — the only active entry point).
///
/// Moved from `src/runtime.rs` at Stone 241.18a.
///
/// Arc 255 Stone the-eval-door — no longer carries `#[wat_special_form_impl(role = eval)]`
/// directly: this fn's signature (three params, `Result<Value, RuntimeError>`) does not fit
/// the canonical `NativeHandler` shape the registry's `role = eval` pointer requires (STOP-3 —
/// its signature does not change). `eval_fn_form` (`src/intrinsic/special/fn_form.rs`) is the
/// thin delegate that carries the annotation instead.
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
    // Arc 109 gamma-i — peel an optional `:- [T U ...]` type-param binder,
    // immediately after metadata and before the args-vector.
    let (binder, sig_args) = peel_type_binder(sig_args);
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
        type_params: binder.unwrap_or_default(),
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

/// Stone 237.2 — dispatch: eval a call to a defclause-bound name.
///
/// Implements first-match-wins arity+type dispatch. Evaluates all args first,
/// then matches clauses in order. Type matching uses structural `value_matches_type`
/// — same dispatch discipline as `eval_dispatch_call_with_vals`.
pub(crate) fn eval_call_to_defclause(
    cs: Arc<ClauseSet>,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Evaluate all args eagerly.
    let vals: Vec<Value> = args
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    eval_call_to_defclause_with_vals(cs, vals, list_span, sym)
}

/// Core dispatch logic for defclause calls (args already evaluated).
///
/// Stone 237.3: extended with :guard evaluation (before body) and :ensure
/// post-condition check (after body). First-match-wins: arity + type +
/// guard must all pass before the body executes.
/// Clause-TCO stone — SELECTION ONLY: arity + runtime type match + `:guard` + arg binding.
///
/// Extracted from [`eval_call_to_defclause_with_vals`] so the TAIL path can pick a clause
/// WITHOUT evaluating its body. ONE DOOR: both the ordinary call path and `eval_tail`'s clause
/// arm select through here — duplicating this loop would be the "N ways to do a thing" defect
/// this substrate keeps deleting.
///
/// Returns the winning clause's index plus the scope its args are bound into. `:guard` is
/// evaluated HERE because a guard runs in clause-arg scope and therefore decides SELECTION;
/// `:ensure` is deliberately NOT here — it is a POST-condition and belongs to the body path.
pub(crate) fn select_defclause_clause(
    cs: &Arc<ClauseSet>,
    vals: &[Value],
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<(usize, Environment), EvalBreak> {
    let called_arity = vals.len();
    // Stone 237.4 — structured per-clause failure reasons (replaces Vec<String>).
    let mut attempted: Vec<ClauseAttempt> = Vec::new();

    for (clause_idx, clause) in cs.clauses.iter().enumerate() {
        // Pre-compute declared_arg_types for diagnostic use.
        let declared_arg_types: Vec<String> = clause
            .args
            .fixed_params
            .iter()
            .map(|(_, t)| crate::check::format_type(t))
            .collect();
        let declared_arity = clause.args.fixed_params.len();

        // 1. Arity match.
        // Stone 241.5 — variadic-min: when rest_param is present, caller must
        // supply AT LEAST the fixed args; strict equality preserved otherwise.
        let fixed_arity = declared_arity;
        let has_rest = clause.args.rest_param.is_some();
        let arity_ok = if has_rest {
            called_arity >= fixed_arity
        } else {
            called_arity == fixed_arity
        };
        if !arity_ok {
            attempted.push(ClauseAttempt {
                clause_index: clause_idx,
                declared_arity,
                declared_arg_types,
                failure_reason: ClauseFailureReason::ArityMismatch {
                    expected: fixed_arity,
                    got: called_arity,
                },
            });
            continue;
        }

        // 2. Type match: each actual value must match the declared arg type.
        //    Record the first failing position for ArgTypeMismatch.
        let type_mismatch: Option<(usize, String, String)> = clause
            .args
            .fixed_params
            .iter()
            .zip(vals.iter())
            .enumerate()
            .find_map(|(pos, ((_, ty), val))| {
                if value_matches_type_by_name(val, ty, sym) {
                    None
                } else {
                    Some((
                        pos,
                        crate::check::format_type(ty),
                        val_type_path(val).to_string(),
                    ))
                }
            });

        if let Some((pos, expected, got)) = type_mismatch {
            attempted.push(ClauseAttempt {
                clause_index: clause_idx,
                declared_arity,
                declared_arg_types,
                failure_reason: ClauseFailureReason::ArgTypeMismatch {
                    position: pos,
                    expected,
                    got,
                },
            });
            continue;
        }

        // 2.5 (S3 Stone 241.5) — Rest-binder element type check.
        // When rest_param is present, extract T from (Vector :- [T]) and check
        // each trailing value against T.
        if let Some((_rest_name, rest_ty)) = &clause.args.rest_param {
            let elem_ty = match rest_ty {
                crate::types::TypeExpr::Parametric { head, args }
                    if head == "wat::core::Vector" && args.len() == 1 =>
                {
                    &args[0]
                }
                _ => {
                    // Defensive: parser should enforce (Vector :- [T]); if not, fail clause.
                    attempted.push(ClauseAttempt {
                        clause_index: clause_idx,
                        declared_arity,
                        declared_arg_types,
                        failure_reason: ClauseFailureReason::ArgTypeMismatch {
                            position: fixed_arity,
                            expected: "(Vector :- [T])".to_string(),
                            got: crate::check::format_type(rest_ty),
                        },
                    });
                    continue;
                }
            };
            let rest_type_mismatch =
                vals[fixed_arity..]
                    .iter()
                    .enumerate()
                    .find_map(|(rest_pos, val)| {
                        if value_matches_type_by_name(val, elem_ty, sym) {
                            None
                        } else {
                            Some((
                                fixed_arity + rest_pos,
                                crate::check::format_type(elem_ty),
                                val_type_path(val).to_string(),
                            ))
                        }
                    });
            if let Some((pos, expected, got)) = rest_type_mismatch {
                attempted.push(ClauseAttempt {
                    clause_index: clause_idx,
                    declared_arity,
                    declared_arg_types,
                    failure_reason: ClauseFailureReason::ArgTypeMismatch {
                        position: pos,
                        expected,
                        got,
                    },
                });
                continue;
            }
        }

        // 3. Bind clause args into a child scope (needed for :guard eval).
        let mut scope = Environment::new();
        for ((param_name_ident, _), val) in clause.args.fixed_params.iter().zip(vals.iter()) {
            let span = list_span.clone();
            scope = scope
                .child()
                .bind(
                    crate::scope::env_key(param_name_ident),
                    span,
                    TrackedValue::from(val.clone()),
                )
                .build();
        }

        // 3.5 (S4 Stone 241.5) — Bind rest values as Value::Vec in scope.
        // Collect trailing vals into a wat::core::Vector and bind to rest_param.name.
        if let Some((rest_name_ident, _rest_ty)) = &clause.args.rest_param {
            let rest_vals: Vec<Value> = vals[fixed_arity..].to_vec();
            let rest_vec = Value::Vec(Arc::new(rest_vals));
            scope = scope
                .child()
                .bind(
                    crate::scope::env_key(rest_name_ident),
                    list_span.clone(),
                    TrackedValue::from(rest_vec),
                )
                .build();
        }

        // 4. Stone 237.3 — :guard evaluation (before body).
        if let Some(guard_ast) = &clause.guard {
            let guard_result = eval_inner(guard_ast, &scope, sym).map(|tv| tv.value_owned())?;
            match &guard_result {
                Value::bool(true) => {
                    // Guard passes — continue to body.
                }
                Value::bool(false) => {
                    // Guard false → record GuardFalse attempt; try next clause.
                    attempted.push(ClauseAttempt {
                        clause_index: clause_idx,
                        declared_arity,
                        declared_arg_types,
                        failure_reason: ClauseFailureReason::GuardFalse,
                    });
                    continue;
                }
                other => {
                    // Non-bool from guard — type-checker should have caught this.
                    // Defensive: treat non-true as GuardFalse skip.
                    let _ = other;
                    attempted.push(ClauseAttempt {
                        clause_index: clause_idx,
                        declared_arity,
                        declared_arg_types,
                        failure_reason: ClauseFailureReason::GuardFalse,
                    });
                    continue;
                }
            }
        }

        // 5. Selected — the caller decides whether to run the body or tail-call it.
        return Ok((clause_idx, scope));
    }

    // No clause matched (arity + type + guard all fell through).
    let called_args: Vec<ValueSnapshot> = vals.iter().map(ValueSnapshot::of).collect();
    Err(RuntimeError::new(
        list_span.clone(),
        RuntimeErrorKind::NoMatchingClause {
            name: cs.name.clone(),
            called_arity,
            called_args,
            attempted_clauses: Box::new(attempted),
        },
    )
    .into())
}

/// Core dispatch for defclause calls (args already evaluated): select, run the body,
/// then check `:ensure`. Selection lives in [`select_defclause_clause`].
pub(crate) fn eval_call_to_defclause_with_vals(
    cs: Arc<ClauseSet>,
    vals: Vec<Value>,
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let (clause_idx, scope) = select_defclause_clause(&cs, &vals, list_span, sym)?;
    let clause = &cs.clauses[clause_idx];

    // Body evaluation.
    //
    // Clause-TCO stone — a clause with NO `:ensure` runs through `apply_function`, which
    // evaluates the body in TAIL position and owns the trampoline that catches
    // `EvalSignal::TailCall`. That is what actually makes clause recursion flat: without it the
    // body is evaluated by `eval_inner`, so an `if`/`match` inside it never reaches `eval_tail`,
    // the recursive call is an ordinary call, and the stack grows until it SIGSEGVs. Measured:
    // adding only the `eval_tail` arm did NOT fix the 200k probe — this line is the other half.
    //
    // ⚠ An `:ensure` clause keeps the direct `eval_inner` path: a post-condition runs AFTER the
    // body and needs the frame it returns into, which a tail call abandons.
    if clause.ensure_fn.is_none() {
        if let Some(f) = &clause.func {
            return apply_function(f.clone(), vals, sym, list_span.clone()).map_err(Into::into);
        }
    }
    let result = eval_inner(&clause.body, &scope, sym).map(|tv| tv.value_owned())?;

        // 6. Stone 237.3 / 237.4 — :ensure post-condition check (after body).
        if let Some(ensure_ast) = &clause.ensure_fn {
            // Capture spans and snapshot for rich diagnostics (Stone 237.4).
            let ensure_expr_snapshot = format!("{:?}", ensure_ast);
            let body_span = clause.body.span().clone();
            let ensure_span = ensure_ast.span().clone();

            // Evaluate the :ensure :fn form to get a callable.
            let ensure_fn_val = eval_inner(ensure_ast, &scope, sym).map(|tv| tv.value_owned())?;
            let ensure_result = match ensure_fn_val {
                Value::wat__core__fn(func) => {
                    apply_function(func, vec![result.clone()], sym, list_span.clone())?
                }
                other => {
                    // Type-checker should have caught non-fn :ensure. Defensive.
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: format!("defclause {}/clause#{} :ensure", cs.name, clause_idx),
                            expected: "wat::core::fn",
                            got: Box::new(ValueSnapshot::of(&other)),
                        },
                    )
                    .into());
                }
            };
            match &ensure_result {
                Value::bool(true) => {
                    // Postcondition passes — return result.
                }
                Value::bool(false) => {
                    return Err(RuntimeError::new(
                        body_span,
                        RuntimeErrorKind::PostconditionFailed {
                            defclause_name: cs.name.clone(),
                            clause_index: clause_idx,
                            ensure_expr_snapshot,
                            returned_value: Box::new(ValueSnapshot::of(&result)),
                            ensure_span: Box::new(ensure_span),
                        },
                    )
                    .into());
                }
                other => {
                    // Non-bool from ensure — type-checker should have caught.
                    // Defensive: treat as postcondition failure.
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: format!(
                                "defclause {}/clause#{} :ensure result",
                                cs.name, clause_idx
                            ),
                            expected: "wat::core::bool",
                            got: Box::new(ValueSnapshot::of(other)),
                        },
                    )
                    .into());
                }
            }
        }

    Ok(result)
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

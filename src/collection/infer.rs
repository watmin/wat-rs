//! Check-side inference intrinsics for the collection dispatch home.
//!
//! These 4 functions are the **projective intrinsic** check arms — each one
//! performs type-level computation (projecting type variables out of the
//! container's parametric type into the return or a peer argument) that a
//! monomorphic `defclause` cannot express. Declared in `infer_list`
//! (src/check.rs) via redirect arms; implementations live here.
//!
//! See `src/collection/mod.rs` and `docs/DISPATCH.md` for the full doctrine.

use crate::ast::WatAST;
use crate::check::{
    apply_subst, format_type, infer, reduce, unify, CheckEnv, CheckError, CheckErrorKind,
    CheckResult, InferCtx, Subst,
};
use crate::span::Span;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// Type-check `(:wat::core::contains? coll elem)` — arc 237 Stone 237.7b-ii.
///
/// Custom inference arm (Tier B): extracts the collection's element/key type
/// and unifies arg1 against it so wrong-element calls are rejected at check time.
/// Accepted collection shapes:
/// - `Vector<T>` → arg1 must unify with T; returns `bool`.
/// - `HashSet<T>` → arg1 must unify with T; returns `bool`.
/// - `HashMap<K,V>` → arg1 must unify with **K** (contains? on HashMap is contains-key?); returns `bool`.
///
/// All other shapes produce a teaching TypeMismatch. Plain ∀ scheme is insufficient
/// because element-typing must be enforced (probe contains_q_wrong_element_rejected_at_check).
pub(crate) fn infer_contains(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::contains?";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let bool_ty = TypeExpr::Path(":wat::core::bool".into());
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(),
            expected: 2,
            got: args.len()
        } });
        return CheckResult::partial_with(bool_ty, local_errors);
    }
    // Infer arg0 (the collection).
    let arg0_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    // Infer arg1 (the element/key) regardless of arg0 outcome so we always
    // surface all errors (mirrors infer_positional_accessor).
    let arg1_ty = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = arg0_ty {
        let reduced = reduce(&coll_ty, subst, env.types());
        // Extract the expected element/key type from the collection shape.
        let elem_ty_opt: Option<TypeExpr> = match &reduced {
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::Vector" => {
                targs.first().map(|t| apply_subst(t, subst))
            }
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::HashSet" => {
                targs.first().map(|t| apply_subst(t, subst))
            }
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::HashMap" => {
                // contains? on HashMap checks the KEY, not the value.
                targs.first().map(|k| apply_subst(k, subst))
            }
            // Arc-278-0a — PersistentMap: contains? checks the KEY, same as HashMap.
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentMap" => {
                targs.first().map(|k| apply_subst(k, subst))
            }
            // Arc-278-0b — PersistentVector: contains? checks element membership, same as Vector.
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentVector" => {
                targs.first().map(|t| apply_subst(t, subst))
            }
            // Unresolved type variable — e.g., returned by `from-holon` which has a
            // generic return type. Cannot prove non-collection without more context;
            // skip element-type check and let the runtime enforce. The runtime will
            // fire a teaching error if a non-collection is actually passed at runtime.
            // POLICY: unresolved-Var defers to the runtime backstop by design,
            // uniformly across all four collection intrinsics (infer_contains /
            // infer_conj / infer_get / infer_assoc). Each sibling carries a matching
            // Var arm that cites this comment as the policy source.
            TypeExpr::Var(_) => None,
            _ => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, HashSet<T>, HashMap<K,V>, PersistentMap<K,V>, or PersistentVector<T>".into(),
                    got: format_type(&reduced)
                } });
                None
            }
        };

        // If we extracted an element type and inferred arg1, unify them.
        if let (Some(elem_ty), Some(arg1)) = (elem_ty_opt, arg1_ty) {
            if unify(&arg1, &elem_ty, subst, env.types()).is_err() {
                local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#2".into(),
                    expected: format_type(&elem_ty),
                    got: format_type(&apply_subst(&arg1, subst))
                } });
            }
        }
    }

    if local_errors.is_empty() {
        CheckResult::ok(bool_ty)
    } else {
        CheckResult::partial_with(bool_ty, local_errors)
    }
}

/// Type-check `(:wat::core::conj coll elem)` — arc 237 Stone 237.7b-iii.
///
/// Custom inference arm (Tier B): extracts the collection's element type
/// and unifies arg1 against it so wrong-element calls are rejected at check time.
/// Accepted collection shapes (2 only — HashMap uses `assoc`, not `conj`):
/// - `Vector<T>` → arg1 must unify with T; returns `Vector<T>` (type-preserving).
/// - `HashSet<T>` → arg1 must unify with T; returns `HashSet<T>` (type-preserving).
///
/// All other shapes (including HashMap) produce a teaching TypeMismatch.
/// Plain ∀ scheme is insufficient: element-typing must be enforced AND the
/// return is the collection type, not bool (probe conj_vector_preserves_collection_type).
pub(crate) fn infer_conj(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::conj";
    let mut local_errors: Vec<CheckError> = Vec::new();
    // Fallback type if we can't determine the collection type (arity error, etc.).
    // Uses fresh.fresh() (the infer_assoc pattern) so we propagate a free type variable
    // rather than a confident-but-wrong :bool when arg0 inference fails.
    let fallback_ty = fresh.fresh();
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(),
            expected: 2,
            got: args.len()
        } });
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // Infer arg0 (the collection).
    let arg0_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    // Infer arg1 (the element) regardless of arg0 outcome so we always
    // surface all errors (mirrors infer_contains).
    let arg1_ty = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = arg0_ty {
        let reduced = reduce(&coll_ty, subst, env.types());
        // Extract the expected element type from the collection shape.
        // Two arms only — HashMap is assoc's territory.
        let elem_ty_opt: Option<TypeExpr> = match &reduced {
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::Vector" => {
                targs.first().map(|t| apply_subst(t, subst))
            }
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::HashSet" => {
                targs.first().map(|t| apply_subst(t, subst))
            }
            // Arc-278-0b — PersistentVector: conj appends element; returns PersistentVector<T>.
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentVector" => {
                targs.first().map(|t| apply_subst(t, subst))
            }
            // Unresolved type variable — defers to the runtime backstop by design,
            // uniformly across the four collection intrinsics (see infer_contains).
            TypeExpr::Var(_) => None,
            _ => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, HashSet<T>, or PersistentVector<T>".into(),
                    got: format_type(&reduced)
                } });
                None
            }
        };

        // If we extracted an element type and inferred arg1, unify them.
        if let (Some(elem_ty), Some(arg1)) = (elem_ty_opt, arg1_ty) {
            if unify(&arg1, &elem_ty, subst, env.types()).is_err() {
                local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#2".into(),
                    expected: format_type(&elem_ty),
                    got: format_type(&apply_subst(&arg1, subst))
                } });
            }
        }

        // Type-preserving return: return the matched collection type (not bool).
        let ret_ty = apply_subst(&coll_ty, subst);
        return if local_errors.is_empty() {
            CheckResult::ok(ret_ty)
        } else {
            CheckResult::partial_with(ret_ty, local_errors)
        };
    }

    if local_errors.is_empty() {
        CheckResult::ok(fallback_ty)
    } else {
        CheckResult::partial_with(fallback_ty, local_errors)
    }
}

/// Type-check `(:wat::core::get coll key-or-index)` — arc 237 Stone 237.7b-iv.
///
/// Custom inference arm (Tier B): extracts the return element type from the
/// collection shape and wraps it in `Option<_>`. Two arms only — NO HashSet
/// (HashSet has no positional get; that's `contains?`'s territory).
///
/// - `Vector<T>`   → arg1 must unify with **`i64`** (the index — NOT the element type T).
///   Returns `Option<T>`. Load-bearing twist: arg1 is i64, independent of T.
/// - `HashMap<K,V>` → arg1 must unify with **`K`** (the key — NOT the value V).
///   Returns `Option<V>`.
///
/// All other shapes produce a teaching TypeMismatch.
pub(crate) fn infer_get(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::get";
    let mut local_errors: Vec<CheckError> = Vec::new();
    // Fallback type used for arity-error only; success path always returns Option<_>.
    let fallback_ty = TypeExpr::Parametric {
        head: "wat::core::Option".into(),
        args: vec![fresh.fresh()],
    };
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(),
            expected: 2,
            got: args.len()
        } });
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // Infer arg0 (the collection).
    let arg0_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    // Infer arg1 (the index / key) regardless of arg0 outcome so we always
    // surface all errors (mirrors infer_contains / infer_conj).
    let arg1_ty = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = arg0_ty {
        let reduced = reduce(&coll_ty, subst, env.types());
        // Match collection shape; extract (expected_arg1_type, return_element_type).
        // NO HashSet arm — HashSet has no get.
        let shape_opt: Option<(TypeExpr, TypeExpr)> = match &reduced {
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::Vector" => {
                // arg1 is the INDEX (i64), independent of the element type T.
                let elem_ty = targs.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh());
                let idx_ty = TypeExpr::Path(":wat::core::i64".into());
                Some((idx_ty, elem_ty))
            }
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::HashMap" => {
                // arg1 is the KEY (K); return wraps VALUE (V).
                let key_ty = targs.first().map(|k| apply_subst(k, subst)).unwrap_or_else(|| fresh.fresh());
                let val_ty = targs.get(1).map(|v| apply_subst(v, subst)).unwrap_or_else(|| fresh.fresh());
                Some((key_ty, val_ty))
            }
            // Arc-278-0a — PersistentMap: same K→V get semantics as HashMap.
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentMap" => {
                let key_ty = targs.first().map(|k| apply_subst(k, subst)).unwrap_or_else(|| fresh.fresh());
                let val_ty = targs.get(1).map(|v| apply_subst(v, subst)).unwrap_or_else(|| fresh.fresh());
                Some((key_ty, val_ty))
            }
            // Arc-278-0b — PersistentVector: same i64→Option<T> get semantics as std Vector.
            // arg1 is the INDEX (i64), independent of the element type T.
            // Returns Option<T> — None on out-of-bounds, Some(elem) on hit (safe, never raises).
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentVector" => {
                let elem_ty = targs.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh());
                let idx_ty = TypeExpr::Path(":wat::core::i64".into());
                Some((idx_ty, elem_ty))
            }
            // Unresolved type variable — defers to the runtime backstop by design,
            // uniformly across the four collection intrinsics (see infer_contains).
            TypeExpr::Var(_) => None,
            _ => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, HashMap<K,V>, PersistentMap<K,V>, or PersistentVector<T>".into(),
                    got: format_type(&reduced)
                } });
                None
            }
        };

        if let Some((expected_arg1_ty, elem_ty)) = shape_opt {
            // Unify arg1 against the expected type (i64 for Vector, K for HashMap).
            if let Some(arg1) = arg1_ty {
                if unify(&arg1, &expected_arg1_ty, subst, env.types()).is_err() {
                    local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                        callee: OP.into(),
                        param: "#2".into(),
                        expected: format_type(&expected_arg1_ty),
                        got: format_type(&apply_subst(&arg1, subst))
                    } });
                }
            }
            // Return Option<element> — the load-bearing precision this custom arm exists for.
            let ret_ty = TypeExpr::Parametric {
                head: "wat::core::Option".into(),
                args: vec![apply_subst(&elem_ty, subst)],
            };
            return if local_errors.is_empty() {
                CheckResult::ok(ret_ty)
            } else {
                CheckResult::partial_with(ret_ty, local_errors)
            };
        }
    }

    if local_errors.is_empty() {
        CheckResult::ok(fallback_ty)
    } else {
        CheckResult::partial_with(fallback_ty, local_errors)
    }
}

/// Type-check `(:wat::core::assoc coll key new-value)` — arc 237 Stone 237.7c.
///
/// Records-doctrine slice: promotes the surface name from a HashMap-only alias to a
/// polymorphic intrinsic with a custom inference arm. Two arms:
///   `HashMap<K,V> + K + V → HashMap<K,V>`   (type-preserving; arg2 unifies with V, NOT K)
///   `:wat::Record + :keyword + ∀T → :wat::Record`  (arg2 free; flavor preserved at runtime)
pub(crate) fn infer_assoc(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::assoc";
    let mut local_errors: Vec<CheckError> = Vec::new();
    // Fallback type used for arity-error only; success paths return the collection type.
    let fallback_ty = fresh.fresh();
    if args.len() != 3 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(),
            expected: 3,
            got: args.len()
        } });
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // Infer arg0 (the collection).
    let arg0_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    // Infer arg1 (the key / field name) — always, so we surface all errors.
    let arg1_ty = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    // Infer arg2 (the new value) — always; free ∀T for Record arm, V for HashMap arm.
    let arg2_ty = infer(&args[2], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = arg0_ty {
        let reduced = reduce(&coll_ty, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::HashMap" => {
                // HashMap<K,V>: arg1 unifies with K, arg2 unifies with V.
                // Return is type-preserving: HashMap<K,V> (unchanged).
                let key_ty = targs.first().map(|k| apply_subst(k, subst)).unwrap_or_else(|| fresh.fresh());
                let val_ty = targs.get(1).map(|v| apply_subst(v, subst)).unwrap_or_else(|| fresh.fresh());
                // Unify arg1 against K.
                if let Some(arg1) = arg1_ty {
                    if unify(&arg1, &key_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#2".into(),
                            expected: format_type(&key_ty),
                            got: format_type(&apply_subst(&arg1, subst))
                        } });
                    }
                }
                // Unify arg2 against V (NOT K — the K-vs-V trap).
                if let Some(arg2) = arg2_ty {
                    if unify(&arg2, &val_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[2].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#3".into(),
                            expected: format_type(&val_ty),
                            got: format_type(&apply_subst(&arg2, subst))
                        } });
                    }
                }
                // Return type-preserving HashMap<K,V>.
                let ret_ty = apply_subst(&coll_ty, subst);
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            // Arc-278-0a — PersistentMap<K,V>: same K+V unification as HashMap; returns PersistentMap<K,V>.
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentMap" => {
                let key_ty = targs.first().map(|k| apply_subst(k, subst)).unwrap_or_else(|| fresh.fresh());
                let val_ty = targs.get(1).map(|v| apply_subst(v, subst)).unwrap_or_else(|| fresh.fresh());
                if let Some(arg1) = arg1_ty {
                    if unify(&arg1, &key_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#2".into(),
                            expected: format_type(&key_ty),
                            got: format_type(&apply_subst(&arg1, subst))
                        } });
                    }
                }
                if let Some(arg2) = arg2_ty {
                    if unify(&arg2, &val_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[2].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#3".into(),
                            expected: format_type(&val_ty),
                            got: format_type(&apply_subst(&arg2, subst))
                        } });
                    }
                }
                // Return type-preserving PersistentMap<K,V>.
                let ret_ty = apply_subst(&coll_ty, subst);
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            TypeExpr::Path(p)
                if crate::types::is_subtype(p, ":wat::Record", env.types())
                    || crate::types::is_subtype(p, ":wat::holon::Record", env.types()) =>
            {
                // Record (base :wat::Record, holonic :wat::holon::Record, or any
                // specifically-typed subtype like :myapp::Voltage):
                // arg1 must be :keyword; arg2 is free ∀T — DO NOT unify.
                // Arc 258 cascade — accept all record subtypes here so assoc on
                // specifically-typed records type-checks without a TypeMismatch.
                // Flavor is preserved at runtime by eval_record_assoc.
                let keyword_ty = TypeExpr::Path(":wat::core::keyword".into());
                if let Some(arg1) = arg1_ty {
                    if unify(&arg1, &keyword_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#2".into(),
                            expected: format_type(&keyword_ty),
                            got: format_type(&apply_subst(&arg1, subst))
                        } });
                    }
                }
                // arg2 is free ∀T — no unification. Flavor preserved at runtime via eval_record_assoc.
                // (arg2_ty was inferred above to surface any parse errors; no unification follows.)
                // Return the concrete record type (type-preserving for specifically-typed records).
                let ret_ty = reduced.clone();
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            // Unresolved type variable — defers to the runtime backstop by design,
            // uniformly across the four collection intrinsics (see infer_contains).
            TypeExpr::Var(_) => {}
            _ => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "HashMap<K,V> or :wat::Record".into(),
                    got: format_type(&reduced)
                } });
            }
        }
    }

    if local_errors.is_empty() {
        CheckResult::ok(fallback_ty)
    } else {
        CheckResult::partial_with(fallback_ty, local_errors)
    }
}

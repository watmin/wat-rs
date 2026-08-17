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
    apply_subst, assignable, format_type, infer, reduce, unify, CheckEnv, CheckError,
    CheckErrorKind, CheckResult, InferCtx, Subst,
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
            // seq-1b — List: element membership, same scan as Vector
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::List" => {
                targs.first().map(|t| apply_subst(t, subst))
            }
            // seq-1b — Tuple: scan over Value; element check uses PartialEq
            // Tuple is heterogeneous so we accept any Value as the element — no unification
            // to enforce (heterogeneous). Return None so the unify block is skipped.
            TypeExpr::Tuple(_) => {
                // Tuple contains? is valid but the elem type is Value (top); no unification
                // to enforce (heterogeneous). Return None so the unify block is skipped.
                None
            }
            // seq-1b — WatAstList: element membership (child form scan)
            TypeExpr::Path(p) if p == ":wat::WatAST" => {
                // WatAstList: contains? compares a WatAST child; arg1 must be :wat::WatAST
                Some(TypeExpr::Path(":wat::WatAST".into()))
            }
            _ => {
                // Arc-278-A2 — Check if this is a Record subtype before rejecting.
                // Record subtypes come as TypeExpr::Path classified by MapContainer::of_type.
                use crate::collection::map_container::MapContainer;
                if MapContainer::of_type(&reduced, env.types()) == Some(MapContainer::Record) {
                    // Record: contains? tests field existence by keyword name. Arg1 must be a keyword.
                    Some(TypeExpr::Path(":wat::core::keyword".into()))
                } else {
                    local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                        callee: OP.into(),
                        param: "#1".into(),
                        expected: "Vector<T>, HashSet<T>, HashMap<K,V>, PersistentMap<K,V>, PersistentVector<T>, List<T>, Tuple, WatAstList, or :wat::core::Record".into(),
                        got: format_type(&reduced)
                    } });
                    None
                }
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
        // Extract the expected element type from the collection shape via the registry.
        // StreamContainer::of_type + has_append() is the single source of truth — no
        // hand-rolled per-container arms here. HashMap is assoc's territory (not a StreamContainer).
        let elem_ty_opt: Option<TypeExpr> = match crate::collection::seq_container::StreamContainer::of_type(&reduced) {
            Some(container) if container.has_append() => {
                // All has_append containers are parametric with element type T as first arg.
                match &reduced {
                    TypeExpr::Parametric { args: targs, .. } => {
                        targs.first().map(|t| apply_subst(t, subst))
                    }
                    // Bare path form without type param — no element type available.
                    _ => None,
                }
            }
            // ∅ N/A: container exists but does not support append (Tuple, WatAstList).
            Some(_) => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, HashSet<T>, PersistentVector<T>, or List<T>".into(),
                    got: format_type(&reduced)
                } });
                None
            }
            // Not a sequence container at all (or unresolved type variable).
            None => {
                if matches!(reduced, TypeExpr::Var(_)) {
                    // Unresolved type variable — defer to the runtime backstop by design,
                    // uniformly across the four collection intrinsics (see infer_contains).
                    None
                } else {
                    local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                        callee: OP.into(),
                        param: "#1".into(),
                        expected: "Vector<T>, HashSet<T>, PersistentVector<T>, or List<T>".into(),
                        got: format_type(&reduced)
                    } });
                    None
                }
            }
        };

        // Adding an element to an immutable/persistent collection is an UP-CAST: the element
        // must be assignable-to (a subtype of) the declared element type, not type-EQUAL. `unify`
        // (invariant equality) runs FIRST — it preserves inference (concretizes a fresh elem_ty
        // from the element, binds vars) — and only when it fails do we fall to the directional
        // `assignable` up-cast. This is what lets a `Peer'<Never, O>` timer (arc 278 Stone 2 —
        // `after`'s honest uninhabited send-type) conj into a service's `selectables` vec
        // `Vector<Peer'<Reply, O>>` (I-slot: `Never <: Reply`, the R7 bottom). unify-first keeps
        // the join semantics; the assignable-fallback only ever ADDS acceptance (sound up-cast).
        if let (Some(elem_ty), Some(arg1)) = (elem_ty_opt, arg1_ty) {
            if unify(&arg1, &elem_ty, subst, env.types()).is_err()
                && !assignable(&arg1, &elem_ty, subst, env)
            {
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
            // seq-1b — List: index i64 → Option<T>
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::List" => {
                let elem_ty = targs.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh());
                let idx_ty = TypeExpr::Path(":wat::core::i64".into());
                Some((idx_ty, elem_ty))
            }
            // seq-1b — WatAstList: index i64 → Option<WatAST>
            TypeExpr::Path(p) if p == ":wat::WatAST" => {
                let elem_ty = TypeExpr::Path(":wat::WatAST".into());
                let idx_ty = TypeExpr::Path(":wat::core::i64".into());
                Some((idx_ty, elem_ty))
            }
            // seq-1b — HashSet: element membership-as-lookup; arg1 is T, return Option<T>
            TypeExpr::Parametric { head, args: targs } if head == "wat::core::HashSet" => {
                let elem_ty = targs.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh());
                Some((elem_ty.clone(), elem_ty))
            }
            _ => {
                // Arc-278-A2 — Check if this is a Record subtype before rejecting.
                // Record subtypes come as TypeExpr::Path classified by MapContainer::of_type.
                use crate::collection::map_container::MapContainer;
                if MapContainer::of_type(&reduced, env.types()) == Some(MapContainer::Record) {
                    // Record: key is keyword; return element is :wat::core::Value (universal top).
                    // Precise per-field-type projection on a literal keyword is a future refinement.
                    let keyword_ty = TypeExpr::Path(":wat::core::keyword".into());
                    let val_ty = TypeExpr::Path(":wat::core::Value".into());
                    Some((keyword_ty, val_ty))
                } else {
                    local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                        callee: OP.into(),
                        param: "#1".into(),
                        expected: "Vector<T>, HashMap<K,V>, PersistentMap<K,V>, PersistentVector<T>, List<T>, WatAstList, HashSet<T>, or :wat::core::Record".into(),
                        got: format_type(&reduced)
                    } });
                    None
                }
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
///   `:wat::core::Record + :keyword + ∀T → :wat::core::Record`  (arg2 free; flavor preserved at runtime)
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
        use crate::collection::map_container::MapContainer;
        match MapContainer::of_type(&reduced, env.types()) {
            Some(m) if m.can_assoc() => match m {   // exhaustive over MapContainer, no `_`
                MapContainer::HashMap | MapContainer::PersistentMap => {
                    // HashMap<K,V> / PersistentMap<K,V>: arg1 unifies with K, arg2 unifies with V.
                    // Return is type-preserving (unchanged). `reduced` is Parametric here; extract targs.
                    let targs = match &reduced {
                        TypeExpr::Parametric { args: ta, .. } => ta,
                        _ => unreachable!("of_type classified HashMap/PersistentMap → must be Parametric"),
                    };
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
                    // Return type-preserving collection<K,V>.
                    let ret_ty = apply_subst(&coll_ty, subst);
                    return if local_errors.is_empty() {
                        CheckResult::ok(ret_ty)
                    } else {
                        CheckResult::partial_with(ret_ty, local_errors)
                    };
                }
                MapContainer::Record => {
                    // Record (base :wat::core::Record, holonic :wat::holon::Record, or any
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
            },
            Some(_) => {
                // can_assoc()==false (none today; the slot for future non-assoc-capable map members).
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "HashMap<K,V> or :wat::core::Record".into(),
                    got: format_type(&reduced)
                } });
            }
            None => {
                // Unresolved type variable — defers to the runtime backstop by design,
                // uniformly across the four collection intrinsics (see infer_contains).
                // Any other shape (non-map, non-Var) → teaching TypeMismatch.
                match &reduced {
                    TypeExpr::Var(_) => {} // defers to runtime backstop
                    _ => {
                        local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#1".into(),
                            expected: "HashMap<K,V> or :wat::core::Record".into(),
                            got: format_type(&reduced)
                        } });
                    }
                }
            }
        }
    }

    if local_errors.is_empty() {
        CheckResult::ok(fallback_ty)
    } else {
        CheckResult::partial_with(fallback_ty, local_errors)
    }
}

// ─── Arc-278-0d — 8 projective infer arms for transform ops ───────────────────
//
// Each arm accepts `Vector<T>` OR `PersistentVector<T>` and projects the return
// type-preservingly.  The static Vec-only TypeSchemes for these ops are RETIRED
// (check.rs:17963-18073); these arms are the single source of truth, mirroring
// what infer_conj/infer_get/infer_assoc do for their ops.
//
// Common shape: extract the collection arg's parametric element type, accept
// the two container heads, emit a teaching TypeMismatch for anything else.

/// Extract `(container_head, elem_ty)` from a reduced TypeExpr that satisfies the
/// caller-supplied capability gate.
///
/// Arc-278 strike 3 — classification now delegates to the registry:
/// `StreamContainer::of_type` classifies the shape, the `cap` gate selects the
/// capability-capable subset. Adding a new container = one change to
/// `seq_container.rs`; this function needs no edit.
///
/// `cap` is the capability predicate to apply:
/// - Pass `StreamContainer::mappable` for map/filter/foldl/foldr (order-agnostic element transform).
/// - Pass `StreamContainer::ordered` for reverse/take/drop/concat (order-dependent sequence ops).
///
/// Returns `None` for a `Var` (caller defers to the runtime backstop) or for
/// any shape the registry doesn't classify as passing `cap` (caller emits TypeMismatch).
fn extract_seq_elem(
    reduced: &TypeExpr,
    subst: &mut Subst,
    fresh: &mut InferCtx,
    cap: fn(crate::collection::seq_container::StreamContainer) -> bool,
) -> Option<(&'static str, TypeExpr)> {
    use crate::collection::seq_container::StreamContainer;

    // Unresolved type variable — defer to the runtime backstop (same policy as
    // infer_contains/conj/get/assoc; see infer_contains for the authoritative comment).
    if matches!(reduced, TypeExpr::Var(_)) {
        return None;
    }

    let container = StreamContainer::of_type(reduced)?;
    if !cap(container) {
        return None;
    }

    // Derive the canonical head string and element type from the classified shape.
    // - Parametric forms carry the head and targs directly on the TypeExpr.
    // - Bare Path forms (arc-278-0d.1) carry no targs → fresh element type.
    //   Bare is valid where element type is genuinely heterogeneous or left open
    //   (e.g. un-parameterized params); fn-unification constrains it from the
    //   reducer/predicate's element param, exactly as the empty-args parametric case does.
    match (container, reduced) {
        (StreamContainer::Vector, TypeExpr::Parametric { args: targs, .. }) => {
            let elem_ty = targs.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh());
            Some(("wat::core::Vector", elem_ty))
        }
        (StreamContainer::PersistentVector, TypeExpr::Parametric { args: targs, .. }) => {
            let elem_ty = targs.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh());
            Some(("wat::core::PersistentVector", elem_ty))
        }
        (StreamContainer::Vector, TypeExpr::Path(_)) => Some(("wat::core::Vector", fresh.fresh())),
        (StreamContainer::PersistentVector, TypeExpr::Path(_)) => Some(("wat::core::PersistentVector", fresh.fresh())),
        (StreamContainer::List, TypeExpr::Parametric { args: targs, .. }) => {
            let elem_ty = targs.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh());
            Some(("wat::core::List", elem_ty))
        }
        (StreamContainer::List, TypeExpr::Path(_)) => Some(("wat::core::List", fresh.fresh())),
        // No other containers pass the cap gate today.
        _ => None,
    }
}

/// Reconstruct a container type given the head name and an element type.
fn seq_ty(coll_head: &str, elem_ty: TypeExpr) -> TypeExpr {
    TypeExpr::Parametric {
        head: coll_head.to_string(),
        args: vec![elem_ty],
    }
}

/// Arc 118.2a — shared input classification for the now-LAZY `map`/`take`/`drop` (the Rust
/// intrinsics that stay native for the bootstrap-circularity reason documented on
/// `crate::stream::NativeLazyCell` / `eval_vec_map`). Accepts `Vector<T>` | `List<T>` |
/// `PersistentVector<T>` | `Stream<T>` — a FIXED set, independent of the `mappable()`/
/// `ordered()` capability tables (those stay exactly as they were, still gating `foldl`/
/// `foldr`/`reverse`/`concat`, which this arc does not touch).
///
/// Returns `None` for a `Var` (caller defers to the runtime backstop, same policy as
/// `extract_seq_elem`) or for any other shape (caller emits `TypeMismatch`).
///
/// **This IS the `Seqable` set — and, as of stone 118.3-B, wat CAN spell it.** Clojure has
/// exactly one `filter` (and `map`/`reduce`/…) because it calls `seq`, a universal coercion every
/// collection implements, and walks an `ISeq`. This hardcoded four-head match is still the ONLY
/// place the concept exists in the *checker's own* lazy-input classification — that has not
/// changed, and this fn is unaffected by 118.3-B — but the three blockers once cited here for why
/// a `.wat` program itself could never name "any of these four containers" as a parameter type
/// are now ALL refuted, measured against the disk (not reasoned):
/// 1. ~~no `defsurface` `:nature` admits a builtin container~~ — REFUTED: `:nature
///    :wat::core::Struct` + `extend-type` on a builtin, both bare and parametric, type-check and
///    run today (`wat-scripts/scratch-pad/probe-seqable-is-spellable-today.wat`,
///    `probe-seqable-parametric-all-four.wat`).
/// 2. ~~no builtin satisfies any surface today~~ — REFUTED twice over: a bare surface over
///    `Vector`/`PersistentVector` (`SCORE-293.4d`, 2026-06-28), and — the stone-118.3-B fix,
///    `src/check.rs`'s `(Parametric actual, Parametric expected)` arm, ~14858 — a PARAMETRIC
///    surface (`Seqable<T>`) over all four: `Vector`, `PersistentVector`, `List`, `Stream`.
/// 3. ~~wat has no ad-hoc unions~~ — DISSOLVED, not refuted: it was never a union. It is N
///    `extend-type`s of ONE surface — Clojure's `ISeq`.
///
/// Minting `:wat::core::Seqable` in the stdlib, extending these four containers, and pointing
/// `join`/`map`/`filter` at it is the NEXT stone (118.3-B's brief, "out of scope" section) — this
/// function's hand-rolled four-head match is exactly what that stone would delete. See
/// `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/MEASURED-118.3-B-is-a-string-compare-not-a-mechanism.md`
/// for the full diagnosis, and `docs/arc/2026/04/109-kill-std/NOTE-seqable-has-no-name-in-wat.md`
/// for the original writeup (the twelve `<verb>-stream` twins that exist only because this type
/// wasn't wired up yet, and why arc 278's native route is this note's PRECONDITION, not a
/// competing fix — it collapses the set's ~30 hand-rolled re-spellings down to this one function,
/// which is what made naming it tractable).
fn extract_lazyable_elem(reduced: &TypeExpr, subst: &mut Subst, fresh: &mut InferCtx) -> Option<TypeExpr> {
    match reduced {
        TypeExpr::Parametric { head, args }
            if head == "wat::core::Vector"
                || head == "wat::core::List"
                || head == "wat::core::PersistentVector"
                || head == "wat::stream::Stream" =>
        {
            Some(args.first().map(|t| apply_subst(t, subst)).unwrap_or_else(|| fresh.fresh()))
        }
        TypeExpr::Path(p)
            if p == ":wat::core::Vector"
                || p == ":wat::core::List"
                || p == ":wat::core::PersistentVector"
                || p == ":wat::stream::Stream" =>
        {
            Some(fresh.fresh())
        }
        _ => None,
    }
}

/// Type-check `(:wat::core::map f xs)` — arc 118.2a (was arc 278 stone 0d, eager).
///
/// LAZY now: `Seqable<T> × fn(T)->U → Stream<U>`, where `Seqable ∈ {Vector, List,
/// PersistentVector, Stream}`. The return is ALWAYS a `Stream<U>` — the input container kind
/// is no longer preserved (that was the eager contract; the lazy one is uniform, matching
/// `crate::stream::value_as_stream`'s runtime normalization in `eval_vec_map`).
pub(crate) fn infer_map(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::map";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![fresh.fresh()] };
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 2, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // fn-first: arg[0] is the mapping function, arg[1] is the collection.
    let fn_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let coll_ty_opt = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_lazyable_elem(&reduced, subst, fresh) {
            Some(elem_ty) => {
                // Unify arg[0] against fn(T)->U; U is the fresh output element type.
                let u_var = fresh.fresh();
                let expected_fn_ty = TypeExpr::Fn {
                    args: vec![elem_ty],
                    ret: Box::new(u_var.clone()),
                };
                if let Some(f_ty) = fn_ty {
                    if unify(&f_ty, &expected_fn_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#1".into(),
                            expected: format_type(&expected_fn_ty),
                            got: format_type(&apply_subst(&f_ty, subst))
                        }});
                    }
                }
                // Return Stream<U> — ALWAYS a Stream now (the lazy flip); U is the fn's output.
                let ret_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![apply_subst(&u_var, subst)] };
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {
                // Unresolved collection type — still check the fn arg if possible.
                // Return a fresh var; the runtime backstop enforces the rest.
            }
            None => {
                local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#2".into(),
                    expected: "Vector<T>, PersistentVector<T>, List<T>, or Stream<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::filter pred xs)` — Arc-278 DESIGN-STONE seq-traversal-one-door,
/// Strike 2a. `filter` is NATIVE now (`eval_filter`, `src/collection/transform.rs`),
/// superseding the five wat `defclause` arms it used to have (`wat/seq.wat`).
///
/// LAZY: `Seqable<T> × fn(T)->bool → Stream<T>` — the SAME `Seqable` set as `map`/`take`/
/// `drop`/`seqable->stream` (see [`extract_lazyable_elem`]'s doc), because `filter`'s runtime
/// composes through `seqable->stream`'s exact per-container normalization. Mirrors
/// [`infer_map`] in every particular except the callee's return type is pinned to `bool`
/// (never a fresh `U`) and the element type `T` is preserved on the way out (filter narrows a
/// stream, it does not transform its elements).
pub(crate) fn infer_filter(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::filter";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![fresh.fresh()] };
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 2, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // pred-first: arg[0] is the predicate, arg[1] is the collection.
    let fn_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let coll_ty_opt = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_lazyable_elem(&reduced, subst, fresh) {
            Some(elem_ty) => {
                // pred must be fn(T) -> bool — T from the collection, bool fixed (not a fresh U).
                let bool_ty = TypeExpr::Path(":wat::core::bool".into());
                let expected_fn_ty = TypeExpr::Fn {
                    args: vec![elem_ty.clone()],
                    ret: Box::new(bool_ty),
                };
                if let Some(f_ty) = fn_ty {
                    if unify(&f_ty, &expected_fn_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#1".into(),
                            expected: format_type(&expected_fn_ty),
                            got: format_type(&apply_subst(&f_ty, subst))
                        }});
                    }
                }
                // Return Stream<T> — T is preserved (filter narrows, never transforms).
                let ret_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![apply_subst(&elem_ty, subst)] };
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {
                // Unresolved collection type — still check the fn arg if possible.
                // Return a fresh var; the runtime backstop enforces the rest.
            }
            None => {
                local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#2".into(),
                    expected: "Vector<T>, PersistentVector<T>, List<T>, or Stream<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::foldl f init xs)` — arc 278 stone 0d.
///
/// Projective: `fn(Acc,T)->Acc × Acc × C<T> → Acc`.
/// The collection arg is arg[2] (fn-first, init-second); result is the accumulator type.
pub(crate) fn infer_foldl(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::foldl";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = fresh.fresh();
    if args.len() != 3 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 3, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // fn-first: arg[0]=f, arg[1]=init (Acc), arg[2]=collection C<T>.
    let fn_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let init_ty_opt = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let coll_ty_opt = infer(&args[2], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_seq_elem(&reduced, subst, fresh, crate::collection::seq_container::StreamContainer::mappable) {
            Some((_coll_head, elem_ty)) => {
                // Accumulator type: unify a fresh Acc var against init's inferred type.
                let acc_var = fresh.fresh();
                if let Some(init_ty) = init_ty_opt {
                    if unify(&init_ty, &acc_var, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#2".into(),
                            expected: format_type(&acc_var),
                            got: format_type(&apply_subst(&init_ty, subst))
                        }});
                    }
                }
                // f must be fn(Acc, T) -> Acc.
                let acc_ty = apply_subst(&acc_var, subst);
                let expected_fn_ty = TypeExpr::Fn {
                    args: vec![acc_ty.clone(), elem_ty],
                    ret: Box::new(acc_ty.clone()),
                };
                if let Some(f_ty) = fn_ty {
                    if unify(&f_ty, &expected_fn_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#1".into(),
                            expected: format_type(&expected_fn_ty),
                            got: format_type(&apply_subst(&f_ty, subst))
                        }});
                    }
                }
                // Return type is the accumulator.
                let ret_ty = apply_subst(&acc_var, subst);
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {}
            None => {
                local_errors.push(CheckError { span: args[2].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#3".into(),
                    expected: "Vector<T>, PersistentVector<T>, or List<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::foldr f init xs)` — arc 278 stone 0d.
///
/// Projective: `fn(T,Acc)->Acc × Acc × C<T> → Acc`.
/// Same layout as foldl but fold function argument order is (T, Acc) → Acc instead of (Acc, T) → Acc.
pub(crate) fn infer_foldr(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::foldr";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = fresh.fresh();
    if args.len() != 3 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 3, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // fn-first: arg[0]=f, arg[1]=init (Acc), arg[2]=collection C<T>.
    let fn_ty = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let init_ty_opt = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let coll_ty_opt = infer(&args[2], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_seq_elem(&reduced, subst, fresh, crate::collection::seq_container::StreamContainer::mappable) {
            Some((_coll_head, elem_ty)) => {
                // Accumulator type: unify a fresh Acc var against init's inferred type.
                let acc_var = fresh.fresh();
                if let Some(init_ty) = init_ty_opt {
                    if unify(&init_ty, &acc_var, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#2".into(),
                            expected: format_type(&acc_var),
                            got: format_type(&apply_subst(&init_ty, subst))
                        }});
                    }
                }
                // f must be fn(T, Acc) -> Acc  — note T comes first, unlike foldl.
                let acc_ty = apply_subst(&acc_var, subst);
                let expected_fn_ty = TypeExpr::Fn {
                    args: vec![elem_ty, acc_ty.clone()],
                    ret: Box::new(acc_ty.clone()),
                };
                if let Some(f_ty) = fn_ty {
                    if unify(&f_ty, &expected_fn_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#1".into(),
                            expected: format_type(&expected_fn_ty),
                            got: format_type(&apply_subst(&f_ty, subst))
                        }});
                    }
                }
                let ret_ty = apply_subst(&acc_var, subst);
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {}
            None => {
                local_errors.push(CheckError { span: args[2].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#3".into(),
                    expected: "Vector<T>, PersistentVector<T>, or List<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::reverse xs)` — arc 278 stone 0d.
///
/// Projective: `C<T> → C<T>` — both container kind and element type are fully preserved.
pub(crate) fn infer_reverse(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::reverse";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = fresh.fresh();
    if args.len() != 1 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 1, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    let coll_ty_opt = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_seq_elem(&reduced, subst, fresh, crate::collection::seq_container::StreamContainer::ordered) {
            Some((coll_head, elem_ty)) => {
                // C<T> → C<T>: return the same container type unchanged.
                let ret_ty = seq_ty(coll_head, apply_subst(&elem_ty, subst));
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {}
            None => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, PersistentVector<T>, or List<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::take xs n)` — arc 118.2a (was arc 278 stone 0d, eager).
///
/// LAZY now: `Seqable<T> × i64 → Stream<T>` — always returns a `Stream<T>` (see
/// `extract_lazyable_elem`'s doc for the accepted `Seqable` set and why the input
/// classification no longer routes through `ordered()`).
pub(crate) fn infer_take(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::take";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![fresh.fresh()] };
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 2, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // Receiver-first: arg[0] is the collection, arg[1] is the count (i64).
    let coll_ty_opt = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let n_ty = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let i64_ty = TypeExpr::Path(":wat::core::i64".into());

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_lazyable_elem(&reduced, subst, fresh) {
            Some(elem_ty) => {
                // Verify the count argument is i64.
                if let Some(n) = n_ty {
                    if unify(&n, &i64_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#2".into(),
                            expected: format_type(&i64_ty),
                            got: format_type(&apply_subst(&n, subst))
                        }});
                    }
                }
                // Return Stream<T> — ALWAYS a Stream now (the lazy flip).
                let ret_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![apply_subst(&elem_ty, subst)] };
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {}
            None => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, PersistentVector<T>, List<T>, or Stream<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::drop xs n)` — arc 118.2a (was arc 278 stone 0d, eager).
///
/// LAZY now: `Seqable<T> × i64 → Stream<T>` — always returns a `Stream<T>`, still lazy
/// beyond the drop point (see [`crate::collection::transform::eval_vec_drop`]'s doc).
pub(crate) fn infer_drop(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::drop";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![fresh.fresh()] };
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 2, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    // Receiver-first: arg[0] is the collection, arg[1] is the count (i64).
    let coll_ty_opt = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let n_ty = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let i64_ty = TypeExpr::Path(":wat::core::i64".into());

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_lazyable_elem(&reduced, subst, fresh) {
            Some(elem_ty) => {
                // Verify the count argument is i64.
                if let Some(n) = n_ty {
                    if unify(&n, &i64_ty, subst, env.types()).is_err() {
                        local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                            callee: OP.into(),
                            param: "#2".into(),
                            expected: format_type(&i64_ty),
                            got: format_type(&apply_subst(&n, subst))
                        }});
                    }
                }
                // Return Stream<T> — ALWAYS a Stream now (the lazy flip).
                let ret_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![apply_subst(&elem_ty, subst)] };
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {}
            None => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, PersistentVector<T>, List<T>, or Stream<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::seqable->stream coll)` — Arc-278 DESIGN-STONE
/// seq-traversal-one-door, Strike 1 (native now; was a wat `defclause`, `wat/seq.wat`).
///
/// `Seqable<T> → Stream<T>` — the private eager→lazy normalizer every stateful lazy
/// transformer (`keep`, `keep-indexed`, `take-nth`, `dedupe`, `distinct`, `map-indexed`)
/// delegates through. Accepts the same `Seqable` set as `map`/`take`/`drop` (see
/// `extract_lazyable_elem`'s doc): Vector<T> | List<T> | PersistentVector<T> | Stream<T>.
pub(crate) fn infer_seqable_to_stream(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::seqable->stream";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![fresh.fresh()] };
    if args.len() != 1 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 1, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    let coll_ty_opt = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(coll_ty) = coll_ty_opt {
        let reduced = reduce(&coll_ty, subst, env.types());
        match extract_lazyable_elem(&reduced, subst, fresh) {
            Some(elem_ty) => {
                let ret_ty = TypeExpr::Parametric { head: "wat::stream::Stream".into(), args: vec![apply_subst(&elem_ty, subst)] };
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(reduced, TypeExpr::Var(_)) => {}
            None => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, PersistentVector<T>, List<T>, or Stream<T>".into(),
                    got: format_type(&reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::concat a b)` — arc 278 stone 0d.
///
/// Projective: `C<T> × C<T> → C<T>` — same-kind-only; mixed Vector+PersistentVector → TypeMismatch.
/// This mirrors the runtime 0c shipped: `vector_concat_inner` rejects mixed kinds.
///
/// CONCAT PATH: `concat` is a defalias for `:wat::core::Vector/concat` (core.wat:44).
/// The alias synthesizes a Function whose scheme is `[Vec<T>, Vec<T>] → Vec<T>`.
/// At check time that scheme rejects PersistentVector.  This custom arm intercepts
/// `:wat::core::concat` in the keyword-head match BEFORE the alias scheme is consulted,
/// enabling honest polymorphism over both container kinds.
pub(crate) fn infer_concat(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::concat";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = fresh.fresh();
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 2, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }
    let a_ty_opt = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let b_ty_opt = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(a_ty) = a_ty_opt {
        let a_reduced = reduce(&a_ty, subst, env.types());
        match extract_seq_elem(&a_reduced, subst, fresh, crate::collection::seq_container::StreamContainer::ordered) {
            Some((coll_head_a, elem_ty_a)) => {
                // arg[1] must be the same container kind with the same element type.
                if let Some(b_ty) = b_ty_opt {
                    let b_reduced = reduce(&b_ty, subst, env.types());
                    match extract_seq_elem(&b_reduced, subst, fresh, crate::collection::seq_container::StreamContainer::ordered) {
                        Some((coll_head_b, elem_ty_b)) => {
                            // Same-kind check: Vector+PersistentVector is a TypeMismatch.
                            if coll_head_a != coll_head_b {
                                local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                                    callee: OP.into(),
                                    param: "#2".into(),
                                    expected: format_type(&a_reduced),
                                    got: format_type(&b_reduced)
                                }});
                            } else {
                                // Same kind: unify element types.
                                if unify(&elem_ty_b, &elem_ty_a, subst, env.types()).is_err() {
                                    local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                                        callee: OP.into(),
                                        param: "#2".into(),
                                        expected: format_type(&a_reduced),
                                        got: format_type(&b_reduced)
                                    }});
                                }
                            }
                        }
                        None if matches!(b_reduced, TypeExpr::Var(_)) => {}
                        None => {
                            local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                                callee: OP.into(),
                                param: "#2".into(),
                                expected: "Vector<T>, PersistentVector<T>, or List<T>".into(),
                                got: format_type(&b_reduced)
                            }});
                        }
                    }
                }
                // Return C<T> — the container kind from arg[0] is preserved.
                let ret_ty = seq_ty(coll_head_a, apply_subst(&elem_ty_a, subst));
                return if local_errors.is_empty() {
                    CheckResult::ok(ret_ty)
                } else {
                    CheckResult::partial_with(ret_ty, local_errors)
                };
            }
            None if matches!(a_reduced, TypeExpr::Var(_)) => {}
            None => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>, PersistentVector<T>, or List<T>".into(),
                    got: format_type(&a_reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

/// Type-check `(:wat::core::PersistentVector/concat to from)` —
/// DESIGN-STONE-into-pv-from-vector.md.
///
/// The per-Type sibling of `Vector/concat`, MINTED rather than widening `infer_concat`'s
/// same-kind-only contract above (that gate stays exactly as-is; `Vector+PersistentVector`
/// through the general `concat`/`Vector/concat` surface is still, correctly, a TypeMismatch).
///
/// Two accepted shapes — NOT symmetric, deliberately:
///   `PersistentVector<T> × Vector<T>            -> PersistentVector<T>`
///   `PersistentVector<T> × PersistentVector<T>  -> PersistentVector<T>`
///
/// arg1 (`to`, the receiver) MUST reduce to `PersistentVector<T>` specifically — this is the
/// param whose kind the result is pinned to (DESIGN row 2: the receiver's kind is preserved).
/// arg2 (`from`) is the one position with dual coverage: Vector<T> OR PersistentVector<T>,
/// never List<T>/Stream<T>/HashSet<T> (`into`'s existing `(PersistentVector<T>, Stream<T>)`
/// clause already owns the Stream case; nothing here widens it).
/// Arc 278 — `:wat::core::Vector/extend :: ∀T. Vector<T> × (Vector<T> | PersistentVector<T>) -> Vector<T>`.
///
/// The mirror of [`infer_persistentvector_concat`]: destination fixes the result kind (Vector
/// here), source accepts EITHER ordered kind. A single static `TypeScheme` cannot express the
/// dual-shape second argument, which is why this is a custom arm rather than a registration.
pub(crate) fn infer_vector_extend(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::Vector/extend";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = seq_ty("wat::core::Vector", fresh.fresh());
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 2, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }

    let a_ty_opt = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let b_ty_opt = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(a_ty) = a_ty_opt {
        let a_reduced = reduce(&a_ty, subst, env.types());
        match extract_seq_elem(&a_reduced, subst, fresh, crate::collection::seq_container::StreamContainer::ordered) {
            Some(("wat::core::Vector", elem_ty_a)) => {
                if let Some(b_ty) = b_ty_opt {
                    let b_reduced = reduce(&b_ty, subst, env.types());
                    match extract_seq_elem(&b_reduced, subst, fresh, crate::collection::seq_container::StreamContainer::ordered) {
                        Some((coll_head_b, elem_ty_b))
                            if coll_head_b == "wat::core::Vector" || coll_head_b == "wat::core::PersistentVector" =>
                        {
                            if unify(&elem_ty_b, &elem_ty_a, subst, env.types()).is_err() {
                                local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                                    callee: OP.into(),
                                    param: "#2".into(),
                                    expected: format_type(&a_reduced),
                                    got: format_type(&b_reduced)
                                }});
                            }
                        }
                        None if matches!(b_reduced, TypeExpr::Var(_)) => {}
                        _ => {
                            local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                                callee: OP.into(),
                                param: "#2".into(),
                                expected: "Vector<T> or PersistentVector<T>".into(),
                                got: format_type(&b_reduced)
                            }});
                        }
                    }
                }
                let ret_ty = seq_ty("wat::core::Vector", apply_subst(&elem_ty_a, subst));
                return if local_errors.is_empty() { CheckResult::ok(ret_ty) } else { CheckResult::partial_with(ret_ty, local_errors) };
            }
            None if matches!(a_reduced, TypeExpr::Var(_)) => {}
            _ => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "Vector<T>".into(),
                    got: format_type(&a_reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}

pub(crate) fn infer_persistentvector_concat(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    const OP: &str = ":wat::core::PersistentVector/concat";
    let mut local_errors: Vec<CheckError> = Vec::new();
    let fallback_ty = seq_ty("wat::core::PersistentVector", fresh.fresh());
    if args.len() != 2 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: OP.into(), expected: 2, got: args.len()
        }});
        return CheckResult::partial_with(fallback_ty, local_errors);
    }

    let a_ty_opt = infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    let b_ty_opt = infer(&args[1], env, locals, fresh, subst).drain_errors_into(&mut local_errors);

    if let Some(a_ty) = a_ty_opt {
        let a_reduced = reduce(&a_ty, subst, env.types());
        match extract_seq_elem(&a_reduced, subst, fresh, crate::collection::seq_container::StreamContainer::ordered) {
            Some(("wat::core::PersistentVector", elem_ty_a)) => {
                if let Some(b_ty) = b_ty_opt {
                    let b_reduced = reduce(&b_ty, subst, env.types());
                    match extract_seq_elem(&b_reduced, subst, fresh, crate::collection::seq_container::StreamContainer::ordered) {
                        // The one deliberate divergence from infer_concat: arg2 accepts
                        // EITHER Vector OR PersistentVector, not just a matching kind.
                        Some((coll_head_b, elem_ty_b))
                            if coll_head_b == "wat::core::Vector" || coll_head_b == "wat::core::PersistentVector" =>
                        {
                            if unify(&elem_ty_b, &elem_ty_a, subst, env.types()).is_err() {
                                local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                                    callee: OP.into(),
                                    param: "#2".into(),
                                    expected: format_type(&a_reduced),
                                    got: format_type(&b_reduced)
                                }});
                            }
                        }
                        None if matches!(b_reduced, TypeExpr::Var(_)) => {}
                        _ => {
                            local_errors.push(CheckError { span: args[1].span().clone(), kind: CheckErrorKind::TypeMismatch {
                                callee: OP.into(),
                                param: "#2".into(),
                                expected: "Vector<T> or PersistentVector<T>".into(),
                                got: format_type(&b_reduced)
                            }});
                        }
                    }
                }
                let ret_ty = seq_ty("wat::core::PersistentVector", apply_subst(&elem_ty_a, subst));
                return if local_errors.is_empty() { CheckResult::ok(ret_ty) } else { CheckResult::partial_with(ret_ty, local_errors) };
            }
            None if matches!(a_reduced, TypeExpr::Var(_)) => {}
            _ => {
                local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                    callee: OP.into(),
                    param: "#1".into(),
                    expected: "PersistentVector<T>".into(),
                    got: format_type(&a_reduced)
                }});
            }
        }
    }
    if local_errors.is_empty() { CheckResult::ok(fallback_ty) } else { CheckResult::partial_with(fallback_ty, local_errors) }
}


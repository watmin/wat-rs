//! Special-form doc entry for `:wat::core::ann-form` — arc 255 Stone 1a-zeta, the last three of
//! the special-form table. `eval_ann_form`/`eval_ann_form_tail` (`src/runtime.rs`) already fit
//! their canonical `NativeHandler`/`TailHandler` shapes exactly and are annotated in place; the
//! check arm did NOT exist as a named fn (it was inline in `check.rs`'s big `match k.as_str()`),
//! so `infer_ann_form` below is that arm's body, moved here verbatim (STOP-3), the same
//! extraction `quote.rs`'s `infer_quote` performed for `:wat::core::quote`.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckError, CheckErrorKind, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// A checked, type-erased identity: infer `<expr>`'s type `S`, require `S` assignable to the
/// ascribed `<type>`, and return `<expr>`'s own value unchanged — `<type>` is never evaluated,
/// only parsed and unified against at check time; the runtime slot it occupies is ERASED.
///
/// **Category ground —** `ann-form`'s WHOLE contract — is `<expr>`'s static type assignable to
/// `<type>`? — is discharged entirely by `infer_ann_form` (this file) at check time: a
/// `TypeMismatch` there refuses the call site before evaluation ever runs. At runtime,
/// `eval_ann_form` (`src/runtime.rs:6175`) is bare identity on `<expr>`'s value —
/// `eval_inner(&args[0], env, sym).map(|tv| tv.value_owned())` — `<type>` (`args[1]`) is never
/// touched by it at all, exactly the shape `identity.rs`'s `require-wire-address` doc argues for
/// `:CheckGate`'s first real member: *"Its body is bare identity ... the ENTIRE contract ... is
/// discharged ... at check time."* `ann-form` is `:CheckGate`'s second real member.
/// `CheckGate`.
///
/// **Purity ground —** `eval_ann_form` calls `eval_inner` on `args[0]` unconditionally — the
/// only sub-form it ever evaluates; `args[1]` (the type slot) is read only by
/// `parse_type_node` at CHECK time (this file's `infer_ann_form`), never by `eval_ann_form` at
/// runtime at all. `ann-form` adds no effect of its own; it is pure exactly when `<expr>` is,
/// the same sentence `Purity::Preserving` was minted with for `if`/`and`/`do`. `Preserving`.
///
/// **Determinism ground —** the same `<expr>`, evaluated in the same environment, always
/// produces the same result if `<expr>` itself is deterministic; `eval_ann_form` consults
/// nothing else (no clock, no entropy, no scope-hygiene counter). `Preserving`.
///
/// **Totality ground —** `eval_ann_form`'s only fallible path of its own is the
/// `args.len() != 2` arity guard — its own doc calls this "belt-and-suspenders" (the checker
/// already enforces arity 2 before runtime), the same "malformed signature is outside
/// totality's domain" carve-out `if`'s/`quote`'s own grounds use. Past that guard, total exactly
/// when `<expr>` is. `Preserving`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `is_expand_time_legal` residue hand-list —
/// grepped in full for this stone — has ZERO occurrences of `"ann-form"` anywhere. Unlike
/// `do`/`stream::lazy` (both literally present in that list), `ann-form` is named nowhere: pre-
/// registration, `is_expand_time_legal(":wat::core::ann-form")` returns `false`
/// (`lookup_entry` is `None`, and the hand-list doesn't contain it either), so
/// `validate_pure_total` currently REFUSES any macro body containing `ann-form`
/// (`RefusedInMacro`) — not a deliberate ruling: `ann-form` needs no runtime-only state (no IO,
/// spawn, clock, or entropy — `:RuntimeOnly`'s own disqualifying criteria), and `eval_ann_form`
/// evaluates a real sub-form (`<expr>`) unconditionally at its own call site, the identical
/// shape `and`/`if`/`do`'s own `Preserving` grounds argue. Registering `Preserving` here is
/// grounded directly in that code and, as a side effect, closes exactly the kind of "false
/// refusal" gap `wat/runtime-meta.wat`'s own `ExpandTime` prose warns about ("a false refusal
/// only surfaces when some macro body happens to call the verb") — reported as a finding, not
/// silently absorbed. `Preserving`.
///
/// @added 1.0.0
/// @Category CheckGate
/// @Purity Preserving
/// @Determinism Preserving
/// @Totality Preserving
/// @ExpandTime Preserving
/// @syntax (:wat::core::ann-form <expr> <type>)
/// @ret :T `<expr>`'s own value, unchanged; `<type>` is erased at runtime
/// @example (:wat::core::ann-form 7 :wat::core::i64) #=> 7
#[wat_special_form(":wat::core::ann-form")]
pub(crate) struct AnnForm;

/// Arc 255 Stone 1a-zeta — the `role = check` pointer for `:wat::core::ann-form`. The inline arm
/// at `check.rs`'s `":wat::core::ann-form"` match key stays untouched in its OWN LOGIC (STOP-3:
/// extracted verbatim), moved wholesale to this named fn so the registry's `role = check`
/// annotation names real, reachable code rather than a fn nothing calls — the same wiring
/// `:wat::core::quote` got at Stone 1a-gamma-i. `infer_component_against`/`assignable`/
/// `apply_subst`/`format_type` are `check.rs` helpers reused verbatim
/// (`infer_component_against` widened `fn` -> `pub(crate) fn` this stone so this file can call
/// it; the other three were already `pub(crate)`/`pub`).
#[wat_special_form_impl(":wat::core::ann-form", role = check)]
pub(crate) fn infer_ann_form(
    _head: &str,
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    let mut local_errors: Vec<CheckError> = Vec::new();
    if args.len() != 2 {
        local_errors.push(CheckError {
            span: head_span.clone(),
            kind: CheckErrorKind::ArityMismatch {
                callee: ":wat::core::ann-form".into(),
                expected: 2,
                got: args.len(),
            },
        });
        return CheckResult::errs(local_errors);
    }
    // Parse the type slot — accepts Keyword, Symbol (wat.type/X), or
    // parametric List ((wat.type/Vector i64) etc).
    let ascribed_ty = match crate::types::parse_type_node(&args[1]) {
        Ok(t) => t,
        Err(te) => {
            local_errors.push(CheckError {
                span: te.span().clone(),
                kind: CheckErrorKind::MalformedForm {
                    head: ":wat::core::ann-form".into(),
                    reason: format!("type slot failed to parse: {}", te.kind()),
                    remedies: vec![],
                },
            });
            return CheckResult::errs(local_errors);
        }
    };
    // Arc-check-literal-elems (generalized, this strike): a parametric-
    // compound literal/ctor-call (`[...]` / `{...}` / `#{...}` /
    // `(:wat::core::Tuple ...)`) ascribed to a known matching expected
    // type up-casts its components against the expected component
    // type(s) (via infer_component_against -> check_compound_against_expected)
    // instead of inferring the whole compound bottom-up and requiring
    // `assignable` on the result. Non-compound / non-matching-shape exprs
    // fall back to plain bottom-up infer inside the same helper, so the
    // `assignable` check below still applies uniformly to both cases.
    let expr_ty = crate::check::infer_component_against(
        &args[0],
        &ascribed_ty,
        env,
        locals,
        fresh,
        subst,
        &mut local_errors,
    );
    // Require expr's type S assignable to the ascribed type T.
    if let Some(s) = expr_ty {
        if !crate::check::assignable(&s, &ascribed_ty, subst, env) {
            local_errors.push(CheckError {
                span: args[0].span().clone(),
                kind: CheckErrorKind::TypeMismatch {
                    callee: ":wat::core::ann-form".into(),
                    param: "expr".into(),
                    expected: crate::check::format_type(&crate::check::apply_subst(&ascribed_ty, subst)),
                    got: crate::check::format_type(&crate::check::apply_subst(&s, subst)),
                },
            });
        }
    }
    let ty = crate::check::apply_subst(&ascribed_ty, subst);
    if local_errors.is_empty() {
        CheckResult::ok(ty)
    } else {
        CheckResult::partial_with(ty, local_errors)
    }
}

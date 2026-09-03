//! Special-form doc entry for `:wat::core::macroexpand` — arc 255 Stone 1a-gamma-i, the six
//! homoiconic verbs that really evaluate. `macroexpand`'s sibling `:wat::core::macroexpand-1`
//! (`macroexpand_1.rs`) shares this row's `role = check` pointer (`infer_macroexpand`, below,
//! stacked with two `#[wat_special_form_impl]` attributes — legal because `role = check` emits
//! SOURCE ONLY, no dispatch shim; see `[[NOTE-role-eval-cannot-stack-and-the-error-does-not-
//! say-so]]` for why `role = eval` could NOT take the same shortcut, which is why each keeps its
//! own separate eval fn in `src/reflect/expand.rs`).
//!
//! ⛔ STOP-1 measured and NOT triggered — write it down so the next self does not re-litigate
//! it. The brief flagged `macroexpand` as the likely `@Purity Effectful` candidate ("expansion
//! can run arbitrary macro bodies"). Measured instead: `src/macros/eval.rs`'s module doc names
//! its own mechanism — `macro_eval(form, env, sym)` ALWAYS runs `validate_pure_total(form)?`
//! BEFORE any call reaches `crate::runtime::eval` (grepped: exactly one call site to the real
//! evaluator inside `src/macros/`, and it is gated). DEFAULT-DENY: a macro body can only invoke
//! the pure-total allow-listed subset — an effectful head is refused before it ever runs, not
//! silently admitted. `macroexpand`/`macroexpand-1` cannot leak an effect through the bodies
//! they expand, by construction, so `Effectful` would be dishonest in the OTHER direction — a
//! declared effect the fence structurally cannot produce.
//!
//! ★ What macro expansion DOES leak — measured by RUNNING it twice, not read — is
//! **nondeterminism**: `expand_macro_call` (`src/macros/expand.rs:942`) draws a fresh `ScopeId`
//! from a process-global monotonic counter (`fresh_scope()`) on every call, and `walk_template`
//! tags every template-LOCAL identifier (never a spliced argument) with it for hygiene. Two
//! `macroexpand-1` calls on the textually IDENTICAL quoted form, in the same run, produced
//! `#wat.ast/ScopedSymbol {:name "tmp" :scopes [1069]}` and `#wat.ast/ScopedSymbol {:name "tmp"
//! :scopes [1070]}` for a macro binding a template-local `tmp` — `=` on the two results is `false`
//! (`wat-scripts/scratch-pad/255-1a-gamma-i-macroexpand-hygiene-determinism.wat`). This is
//! EXACTLY `:wat::core::fresh-symbol`'s own documented shape (`src/intrinsic/ast.rs:359` — same
//! counter, same mechanism, ruled `@Purity Pure` / `@Determinism Nondeterministic` for the
//! identical reason: "no I/O, no observable side effect... the result cannot be pinned across
//! calls, only its shape can"). `Nondeterministic`, not `Effectful` — and NOT the naive
//! `Deterministic` a first read would assume.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{
    apply_subst, assignable, format_type, CheckEnv, CheckError, CheckErrorKind, CheckResult,
    InferCtx, Subst,
};
use crate::span::Span;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// Fully expand `<form>` to fixpoint: apply `:wat::core::macroexpand-1` repeatedly until the AST
/// stops changing, bounded by `EXPANSION_DEPTH_LIMIT` to catch non-terminating macro cycles.
/// Arc 030.
///
/// **Category ground —** the program transforming its own syntax via its own macro layer is
/// self-reference at the syntax level — `:Reflection`'s own prose, the same family this stone's
/// other five rows share. `Reflection`.
///
/// **Purity ground —** see this file's module doc (the STOP-1 finding): `eval_macroexpand`
/// (`src/reflect/expand.rs:86`) evaluates its ONE argument via ordinary call-by-value
/// (`eval_inner`, exactly once, unconditionally — the same shape `write-forms`/`struct->form`
/// already cover, NOT the selective/`Preserving` shape `quasiquote` has), then repeatedly calls
/// `expand_once`, whose only path to the real evaluator (`macro_eval`) is fenced to the
/// pure-total allow-list. No effect can leak through either the argument evaluation (ordinary,
/// not attributed to this form) or the expansion loop (structurally denied by the fence). `Pure`.
///
/// **Determinism ground — measured by RUNNING it twice, `Nondeterministic`:** see the module
/// doc's probe. Any macro whose body introduces a template-local identifier (a `let`-bound temp,
/// a generated helper name — an ordinary, common macro-authoring shape) causes
/// `walk_template`'s hygiene tagging to mint a fresh, monotonically-increasing `ScopeId` on every
/// expansion, so the SAME input form does not reliably produce the SAME output value across
/// calls. The identical shape `:wat::core::fresh-symbol`'s own `Nondeterministic` ruling already
/// covers, same counter. `Nondeterministic`.
///
/// **Totality ground — measured, `Partial`:** `eval_macroexpand`'s own body raises
/// `MacroExpansionFailed` when the fixpoint loop's own `expand_once` call fails, and separately
/// synthesizes an `ExpansionDepthExceeded` cause when `EXPANSION_DEPTH_LIMIT` iterations pass
/// without reaching a fixpoint (a non-terminating macro-rewrite cycle) — a real, non-arity
/// failure mode the type checker cannot rule out ahead of time, because whether `<form>` names a
/// real, correctly-arity'd, terminating macro invocation is a RUNTIME fact about arbitrary
/// `:wat::WatAST` data, not a static one. `Partial`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `is_expand_time_legal` allow-list does NOT
/// carry `":wat::core::macroexpand"` (measured by grep — zero hits) — a macro body that calls
/// `macroexpand` on itself is refused by `validate_pure_total` before expansion runs.
/// `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Reflection
/// @Purity Pure
/// @Determinism Nondeterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::macroexpand <form>)
/// @ret :wat::WatAST `<form>` expanded to fixpoint — every macro call, and every macro call inside the result, applied until nothing changes
/// @example-norun (:wat::core::macroexpand (:wat::core::quote (:some::macro-with-a-template-local-binder arg))) #=> the fully-expanded form, with any template-introduced identifier carrying a freshly-minted `ScopeId` on every call — never guaranteed equal to a prior call's result for the same input
#[wat_special_form(":wat::core::macroexpand")]
pub(crate) struct Macroexpand;

/// Arc 255 Stone 1a-gamma-i — the shared `role = check` pointer for BOTH
/// `:wat::core::macroexpand` AND `:wat::core::macroexpand-1` (same stacking shape
/// `infer_boolean_shortcircuit`/`infer_config_set_bool` use for `and`/`or` and the two config
/// setters — one fn, two FQDNs, because both forms are checked by this exact same body,
/// `check.rs`'s own inline arm having always matched on `"macroexpand-1" | "macroexpand"`
/// together). The inline arm at `check.rs:3850` stays untouched in its OWN LOGIC (STOP-3:
/// extracted verbatim, `k.clone()` renamed to `head.to_string()` — the unavoidable mechanical
/// consequence of the match-scoped `k` binding not existing outside the match arm, not a
/// behaviour change), moved wholesale to this named fn so the registry's `role = check`
/// annotation names real, reachable code rather than a fn nothing calls.
#[wat_special_form_impl(":wat::core::macroexpand", role = check)]
#[wat_special_form_impl(":wat::core::macroexpand-1", role = check)]
pub(crate) fn infer_macroexpand(
    head: &str,
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    // Arc 030: macro debugging primitives.
    // (:wat::core::macroexpand{-1}? <wat::WatAST>) -> :wat::WatAST
    let mut local_errors: Vec<CheckError> = Vec::new();
    if args.len() != 1 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: head.to_string(),
            expected: 1,
            got: args.len()
        } });
        let ty = TypeExpr::Path(":wat::WatAST".into());
        return if local_errors.is_empty() { CheckResult::ok(ty) } else { CheckResult::partial_with(ty, local_errors) };
    }
    if let Some(arg_ty) = crate::check::infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors) {
        let expected = TypeExpr::Path(":wat::WatAST".into());
        if !assignable(&arg_ty, &expected, subst, env) {
            local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
                callee: head.to_string(),
                param: "#1".into(),
                expected: format_type(&apply_subst(&expected, subst)),
                got: format_type(&apply_subst(&arg_ty, subst))
            } });
        }
    }
    let ty = TypeExpr::Path(":wat::WatAST".into());
    if local_errors.is_empty() { CheckResult::ok(ty) } else { CheckResult::partial_with(ty, local_errors) }
}

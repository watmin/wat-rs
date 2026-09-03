//! Special-form doc entry for `:wat::core::struct->form` — arc 255 Stone 1a-gamma-i, the six
//! homoiconic verbs that really evaluate. Unlike `quote`/`quasiquote`/`forms` (which capture or
//! walk SYNTAX), this row reads a live VALUE's structure and re-renders it as syntax — the
//! inverse direction, still `:Reflection`.
//!
//! The `role = eval` pointer is `eval_struct_to_form` itself (`src/reflect/render.rs`), annotated
//! in place — its signature already fits the canonical `NativeHandler` shape, unlike `quote`'s or
//! `forms`' (see those rows' own docs for why THEY need a thin delegate here instead).

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckError, CheckErrorKind, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// Lift a struct VALUE to its constructor-call FORM: `∀T. T → :wat::WatAST`. Evaluates
/// `<struct-value>` (ordinary call-by-value), then — if the result is a `Value::Aggregate` with
/// `nature == Struct` — renders `:class::Foo'` applied to each field's own `value_to_watast`
/// rendering; any other shape (including a non-Struct Aggregate) is a runtime `TypeMismatch`.
///
/// **Category ground —** the program converting its own internal runtime representation (a
/// live struct value) back into its own syntax is self-reference at the syntax level — the same
/// `:Reflection` this stone's other five rows share (`src/reflect/`'s own module name). Not
/// `:Transform` (`:Transform`'s prose is "the SAME value in another form"; this returns a
/// DIFFERENT value — a syntax tree describing how to reconstruct the input, not the input
/// re-expressed).
///
/// **Purity ground —** `eval_struct_to_form` (`src/reflect/render.rs:36`) evaluates its ONE
/// argument via ordinary call-by-value (`eval_inner(&args[0], env, sym)?`) exactly once,
/// unconditionally, then processes the resulting VALUE — the identical shape `write-forms`'s own
/// `Pure` ruling already covers ("the one arg is evaluated by ordinary call-by-value — not
/// itself an effect. Past that, the body only runs a pure structural transform"). Contrast
/// `quasiquote` and `if`/`match`/`and`: those SELECT which of several possible sub-forms actually
/// run (an untaken `if` branch, a template's un-unquoted literal parts, never evaluated at all),
/// which is why THEIR purity is `Preserving`. `struct->form` has no untaken alternative — its one
/// argument is always fully evaluated, the same as any ordinary function's argument — so the
/// evaluation of that argument is not attributed to `struct->form`'s own purity, exactly the
/// convention `write-forms` already set. Past the argument, the body reads the resulting
/// Aggregate's fields and renders them: no IO, no ambient state, no re-invocation of any
/// caller-supplied callable. `Pure`.
///
/// **Determinism ground —** the same struct value always renders to the same `:wat::WatAST` —
/// `value_to_watast` is a pure structural walk with no clock, entropy, or scope-hygiene counter.
/// `Deterministic`.
///
/// **Totality ground — measured, `Partial`:** two real, non-arity failure modes that the type
/// checker CANNOT rule out ahead of time, because `check.rs`'s own comment says so directly —
/// "the arg's type is inferred for context but not constrained (the runtime errors if T isn't a
/// Struct)": (1) the evaluated argument is not a Struct-natured Aggregate →
/// `RuntimeErrorKind::TypeMismatch` (`render.rs:61`); (2) a field's own value has no
/// `value_to_watast` rendering (e.g. a live resource handle) → that fallible sub-call's own
/// error propagates (`render.rs:80`, the `?`). Unlike the arity check the other five rows on
/// this stone carve out of Totality's domain (pre-empted by the STATIC checker, since arity is
/// syntactically visible), whether a ∀T argument's RUNTIME value happens to be a Struct — or
/// whether every one of its fields happens to be renderable — is not a static fact the checker
/// can decide, so both stay inside Totality's domain. `Partial`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `is_expand_time_legal` allow-list carries
/// `":wat::core::struct->form"` verbatim (measured by grep) — legal inside a `defmacro` body
/// during expansion despite being `Partial`; a partial verb can still be expand-time-legal
/// (`ExpandTime`'s own prose: "a partial or nondeterministic verb can be perfectly legal here").
/// `Legal`.
///
/// @added 1.0.0
/// @Category Reflection
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime Legal
/// @syntax (:wat::core::struct->form <struct-value>)
/// @ret :wat::WatAST a constructor-call form (`:class::Foo' field1 field2 ...`) that reconstructs `<struct-value>`
/// @example (:wat::core::ast-kind (:wat::core::struct->form (:wat::holon::CapacityExceeded :cost 7 :budget 3))) #=> "list"
#[wat_special_form(":wat::core::struct->form")]
pub(crate) struct StructToForm;

/// Arc 255 Stone 1a-gamma-i — the `role = check` pointer for `:wat::core::struct->form`. The
/// inline arm at `check.rs:3364` stays untouched in its OWN LOGIC (STOP-3: extracted verbatim),
/// moved wholesale to this named fn so the registry's `role = check` annotation names real,
/// reachable code rather than a fn nothing calls — the same wiring `:wat::core::use!` got last
/// stone.
#[wat_special_form_impl(":wat::core::struct->form", role = check)]
pub(crate) fn infer_struct_to_form(
    _head: &str,
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    // Arc 091 slice 8 — lift a struct VALUE to its
    // constructor-call FORM. ∀T. T → :wat::WatAST. The
    // arg's type is inferred for context but not
    // constrained (the runtime errors if T isn't a
    // Struct). Return type is :wat::WatAST.
    let mut local_errors: Vec<CheckError> = Vec::new();
    if args.len() != 1 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: ":wat::core::struct->form".into(),
            expected: 1,
            got: args.len()
        } });
    } else {
        let _ = crate::check::infer(&args[0], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
    }
    let ty = TypeExpr::Path(":wat::WatAST".into());
    if local_errors.is_empty() { CheckResult::ok(ty) } else { CheckResult::partial_with(ty, local_errors) }
}

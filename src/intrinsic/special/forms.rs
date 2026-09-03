//! Special-form doc entry for `:wat::core::forms` — arc 255 Stone 1a-gamma-i, the six
//! homoiconic verbs that really evaluate. Variadic sibling of `:wat::core::quote`
//! (`quote.rs`) — see that row for the shape this stone registers all six against.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};
use std::collections::HashMap;

/// Capture N forms as unevaluated `:wat::WatAST` values in one call:
/// `(:wat::core::forms f1 f2 ... fn)` → `(:wat::core::Vector :- [wat::WatAST])`, one entry per
/// form (including the empty case, `(:wat::core::forms)` #=> `[]`). Semantically equivalent to
/// `(vec :wat::WatAST (quote f1) (quote f2) ... (quote fn))` without the per-form quote
/// ceremony — building program-as-data payloads for `:wat::kernel::run-sandboxed-ast`,
/// `:wat::eval-ast!`, or any consumer of AST sequences.
///
/// **Category ground —** identical to `:wat::core::quote`'s row (`quote.rs`): the program
/// capturing its own unevaluated syntax as data is self-reference at the syntax level —
/// `:Reflection`'s own prose. `Reflection`.
///
/// **Purity ground —** measured directly: `eval_forms` (`src/reflect/match.rs:377`) never calls
/// `eval`/`eval_inner` on any of `args` — each is wrapped as `Value::wat__WatAST(Arc::new(a.
/// clone()))` and collected. Evaluates NOTHING AT ALL, the identical shape `quote`'s own `Pure`
/// ruling argues (contrast `struct->form`/`macroexpand`, which fully evaluate their one
/// argument, and `quasiquote`, which evaluates its unquoted sub-forms). Unconditionally free of
/// effect regardless of what any argument contains. `Pure`.
///
/// **Determinism ground —** the same N forms always produce a structurally identical Vector of
/// `Value::wat__WatAST` — no clock, no entropy, no scope-hygiene counter (that mechanism lives
/// only in the macro-expansion walker, never called here). `Deterministic`.
///
/// **Totality ground —** `eval_forms`'s own doc comment states it outright: "infallible — no
/// error path (always `Ok`)" — no match, no unwrap, no fallible sub-call, defined
/// unconditionally for every arity including zero. The strongest form of `Total` on this
/// stone's six rows — unlike `quote`, `forms` does not even carry a redundant arity check to be
/// carved out of the domain, because every arity is legal. `Total`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `is_expand_time_legal` allow-list carries
/// `":wat::core::forms"` verbatim (measured by grep) — unconditionally legal inside a `defmacro`
/// body during expansion. `Legal`.
///
/// @added 1.0.0
/// @Category Reflection
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Total
/// @ExpandTime Legal
/// @syntax (:wat::core::forms <form>*)
/// @ret (:wat::core::Vector :- [:wat::WatAST]) one unevaluated `:wat::WatAST` per positional argument, in order; empty Vector for zero arguments
/// @example (:wat::core::length (:wat::core::forms 1 2 3)) #=> 3
#[wat_special_form(":wat::core::forms")]
pub(crate) struct Forms;

/// Arc 255 Stone 1a-gamma-i — the `role = eval` pointer for `:wat::core::forms`. `eval_forms`
/// (`src/reflect/match.rs:377`) does not fit the canonical `NativeHandler` shape: two params
/// (`args`, `list_span`, no `env`/`sym`) — its own module doc states the signature is
/// "unchanged; only the location moved" from its `src/runtime.rs` birthplace (arc 109 reflect
/// stone), so STOP-3 forbids reshaping it here. This thin delegate takes the full four
/// `NativeHandler` params, ignores `env`/`sym` (`forms` never evaluates anything, so there is
/// nothing for either to do), and forwards to the untouched `eval_forms`.
#[wat_special_form_impl(":wat::core::forms", role = eval)]
fn eval_forms_form(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::reflect::r#match::eval_forms(args, list_span)
}

/// Arc 255 Stone 1a-gamma-i — the `role = check` pointer for `:wat::core::forms`. The inline arm
/// at `check.rs:3353` stays untouched in its OWN LOGIC (STOP-3: extracted verbatim), moved
/// wholesale to this named fn so the registry's `role = check` annotation names real, reachable
/// code rather than a fn nothing calls — the same wiring `:wat::core::use!` got last stone.
#[wat_special_form_impl(":wat::core::forms", role = check)]
pub(crate) fn infer_forms(
    _head: &str,
    _args: &[WatAST],
    _head_span: &Span, // rune:lint(unused-span) — infallible — no error path: `forms` captures every positional arg as DATA and returns a fixed `Vector<WatAST>` for any arity, including zero, so there is no `CheckResult::errs` for a span to locate.
    _env: &CheckEnv,
    _locals: &HashMap<String, TypeExpr>,
    _fresh: &mut InferCtx,
    _subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    // Variadic sibling of quote. Every positional arg is
    // DATA, captured as `:wat::WatAST`. The checker does
    // not recurse into any of them. Return type is
    // `(:wat::core::Vector :- [wat::WatAST])` regardless of arity (including
    // zero, which produces an empty Vec).
    CheckResult::ok(TypeExpr::Parametric {
        head: "wat::core::Vector".into(),
        args: vec![TypeExpr::Path(":wat::WatAST".into())],
    })
}

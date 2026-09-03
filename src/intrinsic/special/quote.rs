//! Special-form doc entry for `:wat::core::quote` — arc 255 Stone 1a-gamma-i, the six
//! homoiconic verbs that really evaluate. A fourth shape, distinct from the three
//! `DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md` taught: `quote`
//! reaches the evaluator and returns a real value — it is not `Unevaluated` and not a no-op.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckErrorKind, CheckError, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};
use std::collections::HashMap;

/// Capture `<expr>` as an unevaluated `:wat::WatAST` value — the mechanism that places a wat
/// program into the algebra as data. The inner form is NEVER evaluated: no side effect fires, no
/// function is called, regardless of what `<expr>` contains. This is homoiconicity's entry
/// point — quote is how programs become data without running.
///
/// **Category ground —** the program capturing its OWN unevaluated syntax as a first-class
/// value is self-reference at the syntax level, exactly `:Reflection`'s own prose ("the program
/// interrogating ITSELF") — the same family `src/reflect/`'s module name already claims for
/// `struct->form`/`forms`/`macroexpand`, siblings of this row. Not `:Transform` (that pole
/// returns the SAME value in ANOTHER form — `quote` returns the argument's own SYNTAX, never
/// evaluated into a value in the first place, so there is no "same value" on the input side to
/// re-express). `Reflection`.
///
/// **Purity ground —** measured directly: `eval_quote` (`src/runtime.rs:5990`) never calls
/// `eval`/`eval_inner`/`step_list` on `args[0]` — it wraps the raw `WatAST` node in
/// `Value::wat__WatAST` and returns. Unlike `quasiquote` (which evaluates its UNQUOTED
/// sub-forms) or `struct->form`/`macroexpand` (which fully evaluate their one argument via
/// ordinary call-by-value, the same shape `write-forms`'s own `Pure` ruling already covers),
/// `quote` evaluates NOTHING AT ALL — the same "evaluates none of its sub-forms" shape `fn`'s own
/// `Legal`/`Pure` ruling argues (`fn_form.rs`). Unconditionally free of effect, regardless of
/// what `<expr>` contains. `Pure`.
///
/// **Determinism ground —** the same `<expr>` always produces a structurally identical
/// `Value::wat__WatAST` — no clock, no entropy, no scope-hygiene counter (that mechanism lives
/// only in the MACRO-EXPANSION walker, `src/macros/expand.rs`'s `walk_template`, which `quote`
/// never calls). `Deterministic`.
///
/// **Totality ground —** `eval_quote`'s only fallible path is an `ArityMismatch` for
/// `args.len() != 1` — a shape the type checker (`infer_quote`, this file) already refuses
/// before evaluation reaches this form, the same "malformed signature is outside totality's
/// domain" carve-out `fn`'s own `Total` ruling argues. Past that gate, construction
/// (`Value::wat__WatAST(Arc::new(args[0].clone()))`) cannot fail. `Total`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `validate_pure_total` special-cases `quote`'s
/// head unconditionally (`if head == ":wat::core::quote" ... { return Ok(()); }`) — never
/// refused, and never recurses into `<expr>` either (it is data, not code, even inside a macro
/// body). Legal regardless of what a macro quotes. `Legal`.
///
/// @added 1.0.0
/// @Category Reflection
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Total
/// @ExpandTime Legal
/// @syntax (:wat::core::quote <expr>)
/// @ret :wat::WatAST `<expr>`'s own unevaluated syntax, wrapped as a first-class value
/// @example (:wat::core::ast-kind (:wat::core::quote (f x))) #=> "list"
#[wat_special_form(":wat::core::quote")]
pub(crate) struct Quote;

/// Arc 255 Stone 1a-gamma-i — the `role = eval` pointer for `:wat::core::quote`. `eval_quote`
/// (`src/runtime.rs:5990`) does not fit the canonical `NativeHandler` shape: two params
/// (`args`, `list_span`, no `env`/`sym`) — the same asymmetry `fn_form.rs`'s `eval_fn_form`
/// hit for `eval_fn`. STOP-3 forbids reshaping it: it has a second caller
/// (`src/intrinsic/holon/atom.rs`'s `eval_holon_literal`, which shares `eval_quote` deliberately
/// — its own doc says so) whose 2-arg call site would also need touching. This thin delegate
/// takes the full four `NativeHandler` params, ignores `env`/`sym` (`quote` never evaluates
/// anything, so there is nothing for either to do), and forwards to the untouched `eval_quote`.
#[wat_special_form_impl(":wat::core::quote", role = eval)]
fn eval_quote_form(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_quote(args, list_span)
}

/// Arc 255 Stone 1a-gamma-i — the `role = check` pointer for `:wat::core::quote`. The inline arm
/// at `check.rs:3289` stays untouched in its OWN LOGIC (STOP-3: extracted verbatim), moved
/// wholesale to this named fn so the registry's `role = check` annotation names real, reachable
/// code rather than a fn nothing calls — the same wiring `:wat::core::use!` got last stone.
#[wat_special_form_impl(":wat::core::quote", role = check)]
pub(crate) fn infer_quote(
    _head: &str,
    args: &[WatAST],
    head_span: &Span,
    _env: &CheckEnv,
    _locals: &HashMap<String, TypeExpr>,
    _fresh: &mut InferCtx,
    _subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    // Quote captures an unevaluated AST. The argument is
    // DATA, not an expression — the type checker does not
    // recurse into it. Return type is `:wat::WatAST`.
    let mut local_errors: Vec<CheckError> = Vec::new();
    if args.len() != 1 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: ":wat::core::quote".into(),
            expected: 1,
            got: args.len()
        } });
    }
    let ty = TypeExpr::Path(":wat::WatAST".into());
    if local_errors.is_empty() { CheckResult::ok(ty) } else { CheckResult::partial_with(ty, local_errors) }
}

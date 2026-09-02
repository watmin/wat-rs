//! Special-form doc entry for `:wat::core::fn` — arc 255.SF, the-membership-gap-gets-a-ratchet.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// Construct a closure over the enclosing environment: bind each declared `<param>` to its `:T`,
/// close over the current scope, and defer `<body>` entirely — none of it runs until the
/// resulting value is later CALLED. `fn` must see its own unevaluated params/body forms to build
/// the closure from them (not their values), which is why it is a special form and not an
/// ordinary function, the same reason `if`/`match`/`let` are.
///
/// **Purity ground —** unlike `if`/`match`/`let`, `fn` evaluates NONE of its sub-forms when it
/// itself runs — the params and body are captured whole, not run, and the only other input is
/// the enclosing environment, which is cloned (a cheap `Arc`/handle copy), never mutated or
/// read for effect. Building the closure is unconditionally free of effect, regardless of what
/// the body will later do when the closure is called — a DIFFERENT evaluation event, at a
/// different call site. `Pure`.
///
/// **Determinism ground —** the same params, body, and environment always produce a
/// structurally identical closure; nothing sampled, timed, or otherwise variable enters
/// construction. `Deterministic`.
///
/// **Totality ground —** given the well-formed `[<param> <- :T ...] -> :RetType <body>+` shape,
/// closure construction always succeeds; its only failure mode is a malformed signature
/// (arity/shape), which is outside totality's domain the same way `if`'s own fixed-arity check
/// is (see `if`'s own doc, `control_flow.rs`). `Total`.
///
/// **Expand-time ground —** because `fn` evaluates none of its sub-forms at its own call site,
/// it needs no runtime-only state (no clock, no spawn, no submitted-form evaluation) to build
/// the closure value — legal UNCONDITIONALLY, not merely inherited from an evaluated sub-form.
/// This is why `fn` is `Legal` rather than `Preserving`: `Preserving` (see `if`/`match`) names a
/// form whose own legality IS its sub-forms', and `fn` has no such dependency to preserve —
/// nothing of its body runs while it is itself being constructed, at expand time or otherwise.
/// `Legal`.
///
/// @added 1.0.0
/// @Category Binding
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Total
/// @ExpandTime Legal
/// @syntax (:wat::core::fn [<param> <- :T ...] -> :RetType <body>+)
/// @ret :wat::core::Fn the constructed closure, callable with the declared parameter and return types
/// @example ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) 7) #=> 7
#[wat_special_form(":wat::core::fn")]
pub(crate) struct Fn;

/// Arc 255 Stone the-eval-door — the `role = eval` pointer for `:wat::core::fn`. `eval_fn`
/// (`src/function/eval.rs`) does not fit the canonical `NativeHandler` shape: three params
/// (no `sym`) and `Result<Value, RuntimeError>` rather than `Result<Value, EvalBreak>` (STOP-3
/// forbids reshaping it — it has three callers' worth of history). This thin delegate takes the
/// full four `NativeHandler` params, ignores `sym` (`fn` construction never consults the symbol
/// table — it captures the enclosing `Environment`, not a symbol binding), and converts the
/// error via `.map_err(Into::into)` — the same idiom `src/intrinsic/kernel/stdio.rs`'s handlers
/// already use to bridge a `RuntimeError`-returning inner fn into `EvalBreak`.
#[wat_special_form_impl(":wat::core::fn", role = eval)]
fn eval_fn_form(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::function::eval_fn(args, list_span, env).map_err(Into::into)
}

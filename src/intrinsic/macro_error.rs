//! `:wat::core::macro-error` — arc 255 Stone the-registry-answers-first-wave-3.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-registry-answers-first-wave-3.md`.
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-registry-answers-first-wave-3.md`.
//!
//! One verb, its own home — same shape as `program.rs`/`bytes.rs`/`char.rs` (a single verb with
//! no namespace-mate in this registry, given its own file rather than stretched into an
//! unrelated neighbour's framing). `macro-error` is the ONE of this wave's five with no
//! pre-existing named fn to delegate to — arc 258 Stone 258.2b's "first-class macro-abort" body
//! lived INLINE in `runtime.rs`'s `dispatch_keyword_head` match arm. The brief's own words for
//! this case: "the smallest honest treatment: a named fn it can delegate to, or a delegate
//! carrying the body as-is." This is the latter — the body below is that arm, moved verbatim
//! (parameter shape only adapted to the `#[wat_intrinsic]` shim's per-arg signature; the logic is
//! unchanged, byte for byte).

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

/// `(:wat::core::macro-error msg) -> !` — arc 258 Stone 258.2b, first-class macro-abort.
///
/// Evaluates its one String argument and UNCONDITIONALLY returns
/// `Err(EvalBreak::Diagnostic(RuntimeErrorKind::MacroAbort { message }))` (or `TypeMismatch` if
/// `msg` doesn't evaluate to a String) so the macro engine (`macro_eval_pre_validated`,
/// `src/macros/eval.rs`) wraps it into a clean `MacroError` without "runtime::eval failed:"
/// prefix noise. Macro-body-only — legal only where a `defmacro` body's `validate_pure_total`
/// walk admits it (the blessed allow-list, `src/macros/eval.rs`). Homed here with its real (1)
/// arity declared; the hand-rolled `require_one_arg` arity/eval is unchanged inside the body.
///
/// **Purity ground:** the one arg is evaluated by ordinary call-by-value (not itself an effect).
/// Past that, the body only classifies the already-evaluated value and builds an error payload —
/// no `eval_inner`/`apply_function` on caller-supplied code beyond that one argument evaluation,
/// no IO, no ambient state. Pure ∧ Deterministic.
///
/// **Totality ground — RULED, not transcribed, against
/// `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`:** the body UNCONDITIONALLY
/// returns `Err(EvalBreak::Diagnostic(Box::new(RuntimeError::new(.., RuntimeErrorKind::MacroAbort
/// { .. }))))` on every input — never `Ok`. The decisive line is which `EvalBreak` VARIANT that
/// is, not the word "signal" this verb's own arc-258 doc comment happens to use for it:
/// `EvalBreak::Diagnostic`'s own doc (`src/value/signal.rs:70-72`) says plainly "carries a source
/// location and surfaces to user code as an error" — the SAME variant an ordinary
/// `TypeMismatch`/`ArityMismatch` raise uses, and the opposite of `Option/try`/`Result/try`'s
/// `EvalBreak::Signal(EvalSignal::OptionPropagate | TryPropagate(_))`
/// (`src/value/signal.rs:78-81`: "Caught at function boundaries; never surfaces to user code"),
/// which `apply_function` catches and repackages as the ENCLOSING function's own
/// checker-guaranteed `Option`/`Result` return — a real value the caller `match`es. This verb's
/// `Diagnostic` is caught nowhere at the wat-value level: it unwinds past every enclosing wat
/// form and is caught only by `macro_eval_pre_validated` (`src/macros/eval.rs:109-116`), which
/// matches on `e.kind()` and repackages it as a Rust `MacroError` — a macro-EXPANSION-time
/// (compile-time) failure, never a `Value` any wat code receives or branches on. Confirmed
/// empirically against the pre-stone binary: a direct call passes `--check` (exit 0) and raises
/// at run (`RuntimeError`/`MacroAbort`, exit 1) — the same "passes check, raises at run"
/// signature every other `Partial` raise in this registry has. `try` and this verb share a
/// family resemblance (both called "propagation"/"abort" informally) and land on OPPOSITE
/// verdicts, exactly the trap the brief named: the body's Rust TYPE decides, not the family.
/// `Partial`.
///
/// **Expand-time ground — `ExpandOnly`, not `Legal` (arc 255 Stone
/// expand-only-the-missing-pole):** `Legal` means *also* callable at expand time, alongside a
/// real runtime call site — that was the closest available coordinate before this stone, and it
/// understated what the disk already asserted twice: this doc comment's own header, above,
/// "Macro-body-only — legal only where a `defmacro` body's `validate_pure_total` walk admits it"
/// (`macro_error.rs:28`), and `src/value/signal.rs:529`'s `MacroAbort` variant doc, "Macro-body-
/// only: evaluated at expand time (step 4), never post-expansion." Both say ONLY, and `Legal` has no pole
/// for ONLY — `macro-error` has no runtime call site at all; its only legitimate caller is a
/// `defmacro` body during expansion. `ExpandOnly` is `RuntimeOnly`'s mirror and is the coordinate
/// that says exactly that.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    ExpandOnly
/// @Category      ControlFlow
/// @arg     msg :wat::core::String the message raised as the macro-abort's `MacroAbort.message`
/// @ret     :wat::core::nil never returns — always `Err`, on every input
/// @example-norun (:wat::core::macro-error "malformed template") #=> always raises; not a runnable example
/// @see     :wat::core::Option/expect
#[wat_intrinsic(":wat::core::macro-error")]
pub(crate) fn eval_macro_error(
    msg: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::macro-error";
    let v = match crate::edn::render::require_one_arg(OP, std::slice::from_ref(msg), env, sym, list_span)
    {
        Ok(v) => v,
        Err(e) => return Err(EvalBreak::Diagnostic(Box::new(e))),
    };
    let message = match &v {
        Value::String(s) => (**s).clone(),
        other => {
            return Err(EvalBreak::Diagnostic(Box::new(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::String",
                    got: Box::new(crate::runtime::ValueSnapshot::of(other)),
                },
            ))))
        }
    };
    Err(EvalBreak::Diagnostic(Box::new(RuntimeError::new(
        list_span.clone(),
        RuntimeErrorKind::MacroAbort { message },
    ))))
}

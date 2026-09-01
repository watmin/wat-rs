//! Arc 109 Stone — the map's last two items: `option`. The Option-side third of
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-last-two-map-items.md`'s
//! four destinations — its edge is `src/intrinsic/option.rs`, whose three thin
//! `#[wat_intrinsic]` delegates (`eval_option_expect` / `eval_some_ctor` /
//! `eval_option_try`) called straight into these bodies while they still lived
//! in `src/runtime.rs`'s megafile. A DIRECTORY (`mod.rs`), not a top-level
//! `src/option.rs` file — the DESIGN's contract decision: a directory is the
//! partition line the eventual crate migration consumes, and a new top-level
//! `.rs` is a step that campaign exists to undo.
//!
//! Three items, moved verbatim: `eval_some_ctor` (the `Some` tagged
//! constructor), `eval_option_expect` (`Option/expect`'s panic-on-`:None`
//! body), `eval_option_try` (`Option/try`'s propagate-on-`:None` body).
//! `eval_option_expect`'s own panic machinery — `expect_panic` /
//! `extract_panics` — is NOT here: both are shared with `eval_result_expect`
//! (`src/result/mod.rs`), so they live in `src/assertion.rs`, which already
//! owns the `AssertionPayload` type they build. Bodies verbatim; only the
//! visibility keyword and the `crate::assertion::` import path changed.
//!
//! Sibling: `src/result/mod.rs` (the `Result`-side four).

use crate::ast::WatAST;
use crate::assertion::expect_panic;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    EvalBreak, EvalSignal, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use std::sync::Arc;

/// `(Some <expr>)` — tagged constructor of the built-in `(:Option :- [T])`
/// enum (058-030). Reserved bare identifier; users cannot shadow it.
/// Arity 1. Evaluates `expr` and wraps it in `Value::Option(Some(_))`.
///
/// The dual is `:None` (keyword literal, nullary) handled directly in
/// [`eval`]. Together they are the only way to produce `Value::Option`;
/// callers consume via `(:wat::core::match ...)`.
///
/// Arc 255 Stone A-2-ii-b-1 — `pub(crate)` so `src/intrinsic/option.rs`'s thin
/// `#[wat_intrinsic]` delegate can call straight into this unchanged body.
pub(crate) fn eval_some_ctor(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: "Some".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    Ok(Value::Option(Arc::new(Some(v))))
}

/// `(:wat::core::Option/try <option-expr>)` — Arc 109 slice 1j. The
/// Option-side mirror of `Result/try`: unwrap a `(:Option :- [T])` to its
/// inner `T`, or short-circuit the enclosing Option-returning function with
/// `:None`.
///
/// Semantics on the inner Option:
/// - `(Some v)` — evaluates to `v`; execution continues.
/// - `:None` — raises [`EvalSignal::OptionPropagate`]. The walker
///   unwinds through `let` / `match` / `if` / any nested form until
///   it reaches the innermost enclosing [`apply_function`], which
///   catches the signal and packages it as the function's own
///   `Value::Option(Arc::new(None))` return value.
///
/// The type checker (see `crate::check::infer_option_try`) guarantees
/// the enclosing function returns `(:Option :- [_])`. The dispatcher
/// assumes this invariant and does not re-verify at runtime.
///
/// Arc 255 Stone — the option/result siblings — `pub(crate)` so
/// `src/intrinsic/option.rs`'s thin `#[wat_intrinsic]` delegate can call straight into this
/// unchanged body.
pub(crate) fn eval_option_try(
    op: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: op.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    match v {
        Value::Option(o) => match std::sync::Arc::try_unwrap(o) {
            Ok(Some(inner)) => Ok(inner),
            Ok(None) => Err(EvalBreak::Signal(EvalSignal::OptionPropagate)),
            Err(shared) => match &*shared {
                Some(inner) => Ok(inner.clone()),
                None => Err(EvalBreak::Signal(EvalSignal::OptionPropagate)),
            },
        },
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "(Option :- [T])",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// `(:wat::core::option::expect -> :T <opt> <msg>)` — the
/// panic-on-:None companion to `:wat::core::try`'s propagation form.
/// Arc 108.
///
/// `args[0]` is the `->` symbol; `args[1]` is the declared arm-result
/// type keyword `:T`; `args[2]` is the opt-expr (must evaluate to
/// `(:Option :- [T])`); `args[3]` is the msg-expr (must evaluate to
/// `:String`). Type declared at HEAD position before any value
/// producer — see `infer_option_expect` in check.rs for the
/// rationale.
///
/// On `Some(v)` returns `v`. On `:None` evaluates the msg, snapshots
/// the wat call stack, builds an `AssertionPayload` with the
/// opt-expression's span as `location`, and `panic_any`s. Caught by
/// the substrate's catch_unwind in run-sandboxed-ast / by Rust's
/// default panic handler outside a sandbox.
///
/// Arc 255 Stone A-2-ii-b-0 — `pub(crate)` so `src/intrinsic/option.rs`'s thin
/// `#[wat_intrinsic]` delegate can call straight into this unchanged body.
pub(crate) fn eval_option_expect(
    op: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: op.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let opt = eval_inner(&args[0], env, sym)?.value_owned();
    match opt {
        Value::Option(o) => match std::sync::Arc::try_unwrap(o) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => expect_panic(op, &args[1], env, sym, args[0].span().clone(), None),
            Err(shared) => match &*shared {
                Some(v) => Ok(v.clone()),
                None => expect_panic(op, &args[1], env, sym, args[0].span().clone(), None),
            },
        },
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "(Option :- [T])",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

//! Arc 109 Stone — the map's last two items: `result`. The Result-side fourth
//! of `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-last-two-map-items.md`'s
//! four destinations — its edge is `src/intrinsic/result.rs`, whose four thin
//! `#[wat_intrinsic]` delegates (`eval_ok_ctor` / `eval_err_ctor` /
//! `eval_result_expect` / `eval_result_try`) called straight into these
//! bodies while they still lived in `src/runtime.rs`'s megafile. A DIRECTORY
//! (`mod.rs`), not a top-level `src/result.rs` file — the DESIGN's contract
//! decision: a directory is the partition line the eventual crate migration
//! consumes, and a new top-level `.rs` is a step that campaign exists to
//! undo.
//!
//! Four items, moved verbatim: `eval_ok_ctor` (the `Ok` tagged constructor),
//! `eval_err_ctor` (the `Err` tagged constructor), `eval_result_expect`
//! (`Result/expect`'s panic-on-`Err` body), `eval_try`
//! (`Result/try`'s propagate-on-`Err` body — pre-slice-1j spelling
//! `:wat::core::try`). `eval_result_expect`'s own panic machinery —
//! `expect_panic` / `extract_panics` — is NOT here: both are shared with
//! `eval_option_expect` (`src/option/mod.rs`), so they live in
//! `src/assertion.rs`, which already owns the `AssertionPayload` type they
//! build. Bodies verbatim; only the visibility keyword and the
//! `crate::assertion::` import path changed.
//!
//! Sibling: `src/option/mod.rs` (the `Option`-side three).

use crate::ast::WatAST;
use crate::assertion::{expect_panic, extract_panics};
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    EvalBreak, EvalSignal, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use std::sync::Arc;

/// `(Ok <expr>)` — tagged constructor for the built-in `(:Result :- [T E])`
/// enum. Reserved bare identifier. Arity 1. Evaluates `expr` and wraps
/// in `Value::Result(Ok(_))`.
///
/// Arc 255 Stone A-2-ii-b-1 — `pub(crate)` so `src/intrinsic/result.rs`'s thin
/// `#[wat_intrinsic]` delegate can call straight into this unchanged body.
pub(crate) fn eval_ok_ctor(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: "Ok".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    Ok(Value::Result(Arc::new(Ok(v))))
}

/// `(Err <expr>)` — tagged constructor for the built-in `(:Result :- [T E])`
/// enum. Reserved bare identifier. Arity 1. Evaluates `expr` and wraps
/// in `Value::Result(Err(_))`.
///
/// Arc 255 Stone A-2-ii-b-1 — `pub(crate)` so `src/intrinsic/result.rs`'s thin
/// `#[wat_intrinsic]` delegate can call straight into this unchanged body.
pub(crate) fn eval_err_ctor(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: "Err".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    Ok(Value::Result(Arc::new(Err(v))))
}

/// `(:wat::core::Result/try <result-expr>)` — unwrap a `(:Result :- [T E])`
/// to its inner `T`, or short-circuit the enclosing Result-returning
/// function with `Err(e)`.
///
/// Pre-slice-1j spelling: `:wat::core::try`. The dispatcher passes the
/// user-typed head string in `op` so error messages reflect the form
/// the user wrote.
///
/// Semantics on the inner Result:
/// - `(Ok v)` — evaluates to `v`; execution continues.
/// - `(Err e)` — raises [`EvalSignal::TryPropagate(e)`]. The walker
///   unwinds through `let` / `match` / `if` / any nested form until it
///   reaches the innermost enclosing [`apply_function`], which catches
///   the signal and packages it as the function's own `Err(e)` return
///   value.
///
/// The type checker guarantees the enclosing function is Result-typed
/// and that the propagated `E` matches. This dispatcher assumes both
/// and does not re-verify at runtime.
///
/// Type error (not a checker guarantee — the runtime still guards):
/// arg is not a `Value::Result`. Caller surfaces `TypeMismatch`.
///
/// Arc 255 Stone — the option/result siblings — `pub(crate)` so
/// `src/intrinsic/result.rs`'s thin `#[wat_intrinsic]` delegate (`eval_result_try`) can call
/// straight into this unchanged body.
pub(crate) fn eval_try(
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
        Value::Result(r) => match std::sync::Arc::try_unwrap(r) {
            Ok(std::result::Result::Ok(ok)) => Ok(ok),
            Ok(std::result::Result::Err(e)) => {
                Err(EvalBreak::Signal(EvalSignal::TryPropagate(Box::new(e))))
            }
            Err(shared) => match &*shared {
                std::result::Result::Ok(ok) => Ok(ok.clone()),
                std::result::Result::Err(e) => Err(EvalBreak::Signal(EvalSignal::TryPropagate(
                    Box::new(e.clone()),
                ))),
            },
        },
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "(Result :- [T E])",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// `(:wat::core::result::expect -> :T <res> <msg>)` — the panic-on-Err
/// sibling of `option::expect`. Arc 108.
///
/// Arc 255 Stone — the option/result siblings — `pub(crate)` so
/// `src/intrinsic/result.rs`'s thin `#[wat_intrinsic]` delegate can call straight into this
/// unchanged body.
pub(crate) fn eval_result_expect(
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
    let res = eval_inner(&args[0], env, sym)?.value_owned();
    match res {
        Value::Result(r) => match std::sync::Arc::try_unwrap(r) {
            Ok(std::result::Result::Ok(ok)) => Ok(ok),
            Ok(std::result::Result::Err(e)) => {
                let chain = extract_panics(&e);
                expect_panic(op, &args[1], env, sym, args[0].span().clone(), chain)
            }
            Err(shared) => match &*shared {
                std::result::Result::Ok(ok) => Ok(ok.clone()),
                std::result::Result::Err(e) => {
                    let chain = extract_panics(e);
                    expect_panic(op, &args[1], env, sym, args[0].span().clone(), chain)
                }
            },
        },
        other => Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "(Result :- [T E])",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

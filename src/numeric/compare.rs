//! Arc 109 Stone 1 — the numeric tower's comparison concern.
//!
//! Split by CONCERN, never by TYPE (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-numeric-home.md`). Today this is one
//! function (f64's NaN-correct ordering primitive — i64/bigint/rational ordering is not a
//! numeric-tower leaf; it routes through `eval_compare` / `src/value/numeric_order.rs`,
//! neither of which move in this stone), moved verbatim out of `src/runtime.rs` (arc 109
//! Stone 1). Behaviour is unchanged; only the location moved.
//!
//! `src/intrinsic/f64.rs` is the EDGE; this module is the IMPL. The edge must never be
//! referenced from here — see STOP-2 in the stone's brief.

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::runtime::eval_inner;

/// Stone 237.8b — NaN-correct f64 ordering primitive.
///
/// Uses direct f64 IEEE 754 comparison predicates rather than routing through
/// `values_compare` (which maps NaN→Equal via `unwrap_or`). IEEE 754 guarantees:
/// - any ordered comparison involving NaN returns false
/// - `a < NaN` = false, `NaN < a` = false, `NaN <= NaN` = false, etc.
///
/// This is correct per the DESIGN gate `gate_4b_f64_nan_ordering`.
pub(crate) fn eval_f64_compare<F: Fn(f64, f64) -> bool>(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    pred: F,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: head.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let a_span = args[0].span().clone();
    let a = eval_inner(&args[0], env, sym)?.value_owned();
    let b = eval_inner(&args[1], env, sym)?.value_owned();
    match (a, b) {
        (Value::f64(x), Value::f64(y)) => Ok(Value::bool(pred(x, y))),
        (other, _) => Err(RuntimeError::new(
            a_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "f64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

//! Arc 109 Stone 1 — the numeric tower's arithmetic concern.
//!
//! Split by CONCERN, never by TYPE (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-numeric-home.md`): the tower is growing
//! from today's 5 numeric types toward ~16 (Rust's full integer/float set plus this
//! substrate's `BigInt`/`Rational`), and a per-type layout would multiply into one file per
//! type. This file holds every arithmetic leaf — the AST-door `eval_*_arith` wrappers and the
//! pre-evaluated `arith_*_*_inner` value-door twins — for i64/f64/bigint/rational, moved
//! verbatim out of `src/runtime.rs` (arc 109 Stone 1, "the numeric home"). Behaviour is
//! unchanged; only the location moved. The promotion lattice that makes adding a type a
//! linear edit (rather than a new per-pair match) is stone 2's work, not this one's.
//!
//! `src/intrinsic/{i64,f64,bigint,rational}.rs` are the EDGE (registration + delegation);
//! this module is the IMPL. The edge must never be referenced from here — see STOP-2 in the
//! stone's brief.

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::runtime::{collapse_bigrational, eval_inner, to_bigrational, I64ArithErr};
use num_bigint::BigInt;
use num_rational::BigRational;

/// Integer arith: `:wat::i64::{+,-,*,/}`. Strictly i64 × i64 →
/// i64. No promotion; a f64 arg is a type error.
///
/// `pub(crate)` — arc 255 Stone A-i: `intrinsic/i64.rs`'s `:wat::i64::*`
/// registered handlers call this SAME fn (not a second copy) so both
/// spellings share one arity/type-check/dispatch path.
pub(crate) fn eval_i64_arith<F>(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(i64, i64, &Span) -> Result<i64, EvalBreak>,
{
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
    let b_span = args[1].span().clone();
    let a = eval_inner(&args[0], env, sym)?;
    let b = eval_inner(&args[1], env, sym)?;
    match (a.value(), b.value()) {
        (Value::i64(x), Value::i64(y)) => Ok(Value::i64(op(*x, *y, &b_span)?)),
        (other, _) if !matches!(other, Value::i64(_)) => Err(RuntimeError::new(
            a_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(a.value())),
            },
        )
        .into()),
        (_, _) => Err(RuntimeError::new(
            b_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(b.value())),
            },
        )
        .into()),
    }
}

/// Bigint arith: `:wat::core::bigint::{+,-,*,/}`. Strictly bigint × bigint.
/// No promotion; an i64 arg is a type error — callers promote explicitly via
/// `:wat::i64::to-bigint` (the `wat/core.wat` contagion arms do this).
/// Arc 300 stone C1 — arbitrary precision: `op` never sees an overflow case
/// (contrast `eval_i64_arith`'s wrapping semantics). `op` returns a `Value`
/// directly (not a `BigInt`) because `/` can collapse to EITHER
/// `Value::wat__core__BigInt` (divisible) or `Value::wat__core__Rational`
/// (else) — a single-output-type shape can't express that.
pub(crate) fn eval_bigint_arith<F>(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(&BigInt, &BigInt, &Span) -> Result<Value, EvalBreak>,
{
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
    let b_span = args[1].span().clone();
    let a = eval_inner(&args[0], env, sym)?;
    let b = eval_inner(&args[1], env, sym)?;
    match (a.value(), b.value()) {
        (Value::wat__core__BigInt(x), Value::wat__core__BigInt(y)) => op(x, y, &b_span),
        (other, _) if !matches!(other, Value::wat__core__BigInt(_)) => Err(RuntimeError::new(
            a_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "bigint",
                got: Box::new(ValueSnapshot::of(a.value())),
            },
        )
        .into()),
        (_, _) => Err(RuntimeError::new(
            b_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "bigint",
                got: Box::new(ValueSnapshot::of(b.value())),
            },
        )
        .into()),
    }
}

/// Rational arith: `:wat::core::rational::{+,-,*,/}`. Modeled on
/// `eval_bigint_arith` above, one type over — but EVERY op here can
/// COLLAPSE (contrast bigint, where only `/` collapses): `op` returns the
/// raw `BigRational` result and this wrapper applies [`collapse_bigrational`]
/// uniformly. Operands are coerced via [`to_bigrational`] (rational or
/// bigint; i64 is still a type error — promote explicitly via
/// `:wat::i64::to-rational`).
pub(crate) fn eval_rational_arith<F>(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(&BigRational, &BigRational, &Span) -> Result<BigRational, EvalBreak>,
{
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
    let b_span = args[1].span().clone();
    let a = eval_inner(&args[0], env, sym)?;
    let b = eval_inner(&args[1], env, sym)?;
    match (to_bigrational(a.value()), to_bigrational(b.value())) {
        (Some(x), Some(y)) => {
            let r = op(&x, &y, &b_span)?;
            Ok(collapse_bigrational(r))
        }
        (None, _) => Err(RuntimeError::new(
            a_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "rational",
                got: Box::new(ValueSnapshot::of(a.value())),
            },
        )
        .into()),
        (_, None) => Err(RuntimeError::new(
            b_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "rational",
                got: Box::new(ValueSnapshot::of(b.value())),
            },
        )
        .into()),
    }
}

/// Float arith: `:wat::f64::{+,-,*,/}`. Strictly f64 × f64 →
/// f64. No promotion; an i64 arg is a type error.
pub(crate) fn eval_f64_arith<F>(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(f64, f64, &Span) -> Result<f64, EvalBreak>,
{
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
    let b_span = args[1].span().clone();
    let a = eval_inner(&args[0], env, sym)?.value_owned();
    let b = eval_inner(&args[1], env, sym)?.value_owned();
    match (a, b) {
        (Value::f64(x), Value::f64(y)) => Ok(Value::f64(op(x, y, &b_span)?)),
        (other, _) if !matches!(other, Value::f64(_)) => Err(RuntimeError::new(
            a_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "f64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
        (_, other) => Err(RuntimeError::new(
            b_span,
            RuntimeErrorKind::TypeMismatch {
                op: head.into(),
                expected: "f64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

// Arc 255 Stone Q-2 — gained `span: &Span`. This helper is called ONLY by the 19
// value-door twins (`src/intrinsic/{i64,f64,bigint,rational}.rs`), never by the AST
// door's `eval_i64_arith`/`i64_add_op` (STOP-3: those keep their own spans untouched).
// Every error below now carries the caller's real span instead of a synthesized one.
pub(crate) fn arith_i64_i64_inner<F>(
    impl_name: &str,
    vals: &[Value],
    span: &Span,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(i64, i64) -> Result<i64, I64ArithErr>,
{
    let a = vals.first().expect("arity-checked");
    let b = vals.get(1).expect("arity-checked");
    match (a, b) {
        (Value::i64(x), Value::i64(y)) => match op(*x, *y) {
            Ok(r) => Ok(Value::i64(r)),
            Err(I64ArithErr::DivByZero) => {
                Err(RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into())
            }
            Err(I64ArithErr::Overflow(a, b)) => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::IntegerOverflow {
                    op: impl_name.into(),
                    a,
                    b,
                },
            )
            .into()),
        },
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: impl_name.into(),
                expected: "(i64, i64)",
                got: Box::new(ValueSnapshot::of(a)),
            },
        )
        .into()),
    }
}

// Arc 255 Stone Q-2 — gained `span: &Span`; see `arith_i64_i64_inner`'s comment above.
pub(crate) fn arith_f64_f64_inner<F>(
    impl_name: &str,
    vals: &[Value],
    span: &Span,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(f64, f64) -> Result<f64, ()>,
{
    let a = vals.first().expect("arity-checked");
    let b = vals.get(1).expect("arity-checked");
    match (a, b) {
        (Value::f64(x), Value::f64(y)) => match op(*x, *y) {
            Ok(r) => Ok(Value::f64(r)),
            Err(()) => {
                Err(RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into())
            }
        },
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: impl_name.into(),
                expected: "(f64, f64)",
                got: Box::new(ValueSnapshot::of(a)),
            },
        )
        .into()),
    }
}

/// Arc 300 stone C1 — bigint substrate-addressed arithmetic leaf, mirroring
/// `arith_i64_i64_inner`/`arith_f64_f64_inner` above. `op` returns a `Value`
/// directly (not a `BigInt`) so `/` can produce EITHER
/// `Value::wat__core__BigInt` or `Value::wat__core__Rational` — same
/// two-output-type reason as `eval_bigint_arith`.
// Arc 255 Stone Q-2 — gained `span: &Span`; see `arith_i64_i64_inner`'s comment above.
pub(crate) fn arith_bigint_bigint_inner<F>(
    impl_name: &str,
    vals: &[Value],
    span: &Span,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(&BigInt, &BigInt) -> Result<Value, ()>,
{
    let a = vals.first().expect("arity-checked");
    let b = vals.get(1).expect("arity-checked");
    match (a, b) {
        (Value::wat__core__BigInt(x), Value::wat__core__BigInt(y)) => match op(x, y) {
            Ok(v) => Ok(v),
            Err(()) => {
                Err(RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into())
            }
        },
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: impl_name.into(),
                expected: "(bigint, bigint)",
                got: Box::new(ValueSnapshot::of(a)),
            },
        )
        .into()),
    }
}

/// Arc 300 stone C2 — rational substrate-addressed arithmetic leaf, mirroring
/// `arith_bigint_bigint_inner` above. `op` returns the raw `BigRational`
/// (not a `Value`) because EVERY rational op collapses (contrast bigint,
/// where only `/` does) — this helper applies `collapse_bigrational`
/// uniformly after `op`. Operands are coerced via `to_bigrational` (rational
/// or bigint — see its doc for why bigint is accepted here too).
// Arc 255 Stone Q-2 — gained `span: &Span`; see `arith_i64_i64_inner`'s comment above.
pub(crate) fn arith_rational_rational_inner<F>(
    impl_name: &str,
    vals: &[Value],
    span: &Span,
    op: F,
) -> Result<Value, EvalBreak>
where
    F: Fn(&BigRational, &BigRational) -> Result<BigRational, ()>,
{
    let a = vals.first().expect("arity-checked");
    let b = vals.get(1).expect("arity-checked");
    match (to_bigrational(a), to_bigrational(b)) {
        (Some(x), Some(y)) => match op(&x, &y) {
            Ok(r) => Ok(collapse_bigrational(r)),
            Err(()) => {
                Err(RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into())
            }
        },
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: impl_name.into(),
                expected: "(rational, rational)",
                got: Box::new(ValueSnapshot::of(a)),
            },
        )
        .into()),
    }
}

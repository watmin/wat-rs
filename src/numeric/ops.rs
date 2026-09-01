//! Arc 109 Stone 1 — the numeric tower's type-specific-operation concern.
//!
//! Split by CONCERN, never by TYPE (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-numeric-home.md`). Unlike `arith.rs` /
//! `convert.rs` / `compare.rs`, these operations do NOT cross the tower — `rational::numerator`
//! /`rational::denominator` are rational-only accessors, and `f64::round`/`f64::unary`/
//! `f64::clamp` are f64-only. They live in their own file rather than mixed into a tower-wide
//! mechanism because each is a per-type surface, not a per-concern one. Moved verbatim out of
//! `src/runtime.rs` (arc 109 Stone 1). Behaviour is unchanged; only the location moved.
//!
//! `src/intrinsic/{f64,rational}.rs` are the EDGE; this module is the IMPL. The edge must
//! never be referenced from here — see STOP-2 in the stone's brief.

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::runtime::{bigint_component_to_value, eval_inner, eval_one_arg};

/// `:wat::core::rational/numerator` — slash-form accessor (cf `Uuid/version`).
///
/// Arc 255 Stone D — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::rational/numerator` literal through Stone C) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_rational_numerator(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let r = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "rational",
        |v| match v {
            Value::wat__core__Rational(r) => Ok(r),
            other => Err(other),
        },
    )?;
    Ok(bigint_component_to_value(r.numer().clone()))
}

/// `:wat::core::rational/denominator` — slash-form accessor (cf `Uuid/version`).
///
/// Arc 255 Stone D — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::rational/denominator` literal through Stone C) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_rational_denominator(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let r = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "rational",
        |v| match v {
            Value::wat__core__Rational(r) => Ok(r),
            other => Err(other),
        },
    )?;
    Ok(bigint_component_to_value(r.denom().clone()))
}

/// Arc 019 — `(:wat::f64::round v digits)`. Rounds `v` to
/// `digits` decimal places using round-half-away-from-zero (wraps
/// Rust's `f64::round()` after scaling). `digits=0` rounds to the
/// nearest integer; `digits=2` rounds to two decimals. Negative
/// `digits` is rejected as MalformedForm — "round to nearest 10"
/// has no load-bearing use case and feels like asking for a
/// divide-by-zero answer; if a real caller surfaces, a future
/// arc extends. NaN and ±∞ pass through unchanged.
///
/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::f64::round` `const OP` through Stone B) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_f64_round(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
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
    let arg0_span = args[0].span().clone();
    let arg1_span = args[1].span().clone();
    let v = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::f64(x) => x,
        other => {
            return Err(RuntimeError::new(
                arg0_span,
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "f64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let digits = match eval_inner(&args[1], env, sym)?.value_owned() {
        Value::i64(d) => d,
        other => {
            return Err(RuntimeError::new(
                arg1_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "i64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    if digits < 0 {
        return Err(RuntimeError::new(arg1_span, RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!(
                "`digits` must be non-negative; got {}. Negative digits (round to nearest 10 / 100 / ...) has no load-bearing use case today",
                digits
            )
        }).into());
    }
    let factor = 10f64.powi(digits as i32);
    Ok(Value::f64((v * factor).round() / factor))
}

/// Arc 046 — strict-f64 unary helper for the `:wat::core::f64`
/// namespace primitives. Mirrors `eval_math_unary`
/// (`:wat::math::*` namespace, arc 255 Stone HOME-9) but takes the full op name as a
/// string and rejects `i64` arguments — the `:wat::core::f64`
/// family is consistently strict (matches `eval_f64_arith`'s
/// `f64::+/-/*//` discipline), while `:wat::math::*` permits
/// `i64 -> f64` promotion for ergonomic transcendental calls.
pub(crate) fn eval_f64_unary(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
    f: fn(f64) -> f64,
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
    let arg_span = args[0].span().clone();
    let v = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::f64(x) => x,
        other => {
            return Err(RuntimeError::new(
                arg_span,
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "f64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::f64(f(v)))
}

/// Arc 046 — `(:wat::f64::clamp v lo hi)`. Bounds `v` into
/// `[lo, hi]` via `f64::clamp`. Strict-f64 (no `i64` promotion);
/// matches the `:wat::f64` family discipline. Rust's
/// `f64::clamp` panics if `lo > hi` or either is NaN — we surface
/// that as a `MalformedForm` rather than letting it propagate as
/// a panic, since wat-side errors should be catchable.
///
/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::f64::clamp` `const OP` through Stone B) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_f64_clamp(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: op.into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }
    let mut vs = [0.0_f64; 3];
    for (i, slot) in vs.iter_mut().enumerate() {
        let arg_span = args[i].span().clone();
        *slot = match eval_inner(&args[i], env, sym)?.value_owned() {
            Value::f64(x) => x,
            other => {
                return Err(RuntimeError::new(
                    arg_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: op.into(),
                        expected: "f64",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        };
    }
    let [v, lo, hi] = vs;
    if lo.is_nan() || hi.is_nan() || lo > hi {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "lo must be ≤ hi and neither may be NaN; got lo={}, hi={}",
                    lo, hi
                ),
            },
        )
        .into());
    }
    Ok(Value::f64(v.clamp(lo, hi)))
}

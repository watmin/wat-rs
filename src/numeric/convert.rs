//! Arc 109 Stone 1 — the numeric tower's conversion concern.
//!
//! Split by CONCERN, never by TYPE (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-numeric-home.md`). Every named
//! `:wat::<type>::to-<other>` cast plus the `:u8` range-checked cast, moved verbatim out of
//! `src/runtime.rs` (arc 109 Stone 1). Behaviour is unchanged; only the location moved. Today
//! this is nine numeric pairs written by hand for four types — stone 2 is what turns adding a
//! type into a linear edit here, not this move.
//!
//! `src/intrinsic/{i64,f64,bigint,rational}.rs` are the EDGE; this module is the IMPL. The
//! edge must never be referenced from here — see STOP-2 in the stone's brief.

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::runtime::{eval_inner, eval_one_arg};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::ToPrimitive;
use std::sync::Arc;

/// `:wat::i64::to-rational` — infallible promotion (mirrors C1's
/// `i64::to-bigint`). Used by the `wat/core.wat` `+ - * /` defclauses'
/// i64⊕rational contagion arms.
///
/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::i64::to-rational` literal through Stone B) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_i64_to_rational(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let n = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "i64",
        |v| match v {
            Value::i64(n) => Ok(n),
            other => Err(other),
        },
    )?;
    Ok(Value::wat__core__Rational(Box::new(
        BigRational::from_integer(BigInt::from(n)),
    )))
}

/// `:wat::core::bigint::to-rational` — infallible promotion. Used by the
/// `wat/core.wat` `+ - * /` defclauses' bigint⊕rational contagion arms.
///
/// Arc 255 Stone D — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::bigint::to-rational` literal through Stone C) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_bigint_to_rational(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let n = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "bigint",
        |v| match v {
            Value::wat__core__BigInt(n) => Ok(n),
            other => Err(other),
        },
    )?;
    Ok(Value::wat__core__Rational(Box::new(
        BigRational::from_integer(*n),
    )))
}

/// `:wat::core::rational::to-f64` — `BigRational::to_f64` via num-traits
/// `ToPrimitive` (mirrors `eval_bigint_to_f64`'s posture). Also the float-
/// contagion path (`rational ⊕ f64 → f64`) used by the `core.wat` defclauses.
///
/// Arc 255 Stone D — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::rational::to-f64` literal through Stone C) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_rational_to_f64(
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
    let x = r.to_f64().unwrap_or_else(|| {
        use num_traits::Signed;
        if r.is_negative() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    });
    Ok(Value::f64(x))
}

/// `:wat::core::u8 <i64-expr>` — range-checked cast from `:i64` to
/// `:u8`. Arc 008 slice 1. Rejects values outside 0..=255 at runtime
/// with a MalformedForm describing the offending value. The argument
/// type is enforced statically; this primitive only runs if the
/// checker saw an `:i64` at the call site.
pub(crate) fn eval_u8_cast(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::u8".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let arg_span = args[0].span().clone();
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    match v {
        Value::i64(n) => {
            if !(0..=255).contains(&n) {
                return Err(RuntimeError::new(
                    arg_span,
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::u8".into(),
                        reason: format!("value {} out of :u8 range 0..=255", n),
                    },
                )
                .into());
            }
            Ok(Value::u8(n as u8))
        }
        other => Err(RuntimeError::new(
            arg_span,
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::u8".into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::i64::to-string` literal through Stone B) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_i64_to_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let n = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "i64",
        |v| match v {
            Value::i64(n) => Ok(n),
            other => Err(other),
        },
    )?;
    Ok(Value::String(Arc::new(n.to_string())))
}

/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::i64::to-f64` literal through Stone B) so a raised error names
/// whichever spelling the caller actually used.
pub(crate) fn eval_i64_to_f64(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let n = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "i64",
        |v| match v {
            Value::i64(n) => Ok(n),
            other => Err(other),
        },
    )?;
    Ok(Value::f64(n as f64))
}

/// `:wat::i64::to-bigint` — infallible promotion (arbitrary precision
/// never loses i64 range). Arc 300 stone C1: used by the `wat/core.wat`
/// `+ - * /` defclauses' i64⊕bigint contagion arms.
///
/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::i64::to-bigint` literal through Stone B) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_i64_to_bigint(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let n = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "i64",
        |v| match v {
            Value::i64(n) => Ok(n),
            other => Err(other),
        },
    )?;
    Ok(Value::wat__core__BigInt(Box::new(BigInt::from(n))))
}

/// `:wat::core::bigint::to-f64` — lossy beyond f64's 53-bit mantissa (same
/// posture as `:wat::i64::to-f64`). Arc 300 stone C1.
///
/// Arc 255 Stone D — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::bigint::to-f64` literal through Stone C) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_bigint_to_f64(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let n = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "bigint",
        |v| match v {
            Value::wat__core__BigInt(n) => Ok(n),
            other => Err(other),
        },
    )?;
    let x = n.to_f64().unwrap_or_else(|| {
        if *n < BigInt::from(0) {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    });
    Ok(Value::f64(x))
}

/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::f64::to-string` literal through Stone B) so a raised error
/// names whichever spelling the caller actually used.
pub(crate) fn eval_f64_to_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let f = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "f64",
        |v| match v {
            Value::f64(f) => Ok(f),
            other => Err(other),
        },
    )?;
    Ok(Value::String(Arc::new(format!("{}", f))))
}

/// Arc 255 Stone C — `op` is now a caller-supplied parameter (was a hardcoded
/// `:wat::core::f64::to-i64` literal through Stone B) so a raised error names
/// whichever spelling the caller actually used.
pub(crate) fn eval_f64_to_i64(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    op: &str,
) -> Result<Value, EvalBreak> {
    let f = eval_one_arg(
        op,
        args,
        list_span,
        env,
        sym,
        "f64",
        |v| match v {
            Value::f64(f) => Ok(f),
            other => Err(other),
        },
    )?;
    let result = if f.is_finite() && f >= (i64::MIN as f64) && f <= (i64::MAX as f64) {
        Some(Value::i64(f as i64))
    } else {
        None
    };
    Ok(Value::Option(Arc::new(result)))
}

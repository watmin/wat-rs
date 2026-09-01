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
use crate::runtime::eval_inner;
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

// ─── Arc 255 Stone A-i — the SHARED i64 op fns ─────────────────────────────
//
// Named, `pub(crate)` op fns for `+ - * / mod quot rem`, factored out of what
// through Stone B were inline closures on BOTH the OLD `:wat::core::i64::*`
// dispatch arm (retired at Stone C — `runtime.rs` no longer calls these
// directly) and `intrinsic/i64.rs`'s `:wat::i64::*` registered handlers,
// which are now these fns' only caller. One implementation of the
// overflow/division contract, never two — the brief's STOP-1 concern.
// `head` is a parameter (not a captured closure variable) so the
// `IntegerOverflow`/`DivisionByZero` error's `op` field always names
// whichever spelling the caller actually used.

pub(crate) fn i64_add_op(head: &str, a: i64, b: i64, b_span: &Span) -> Result<i64, EvalBreak> {
    a.checked_add(b).ok_or_else(|| {
        RuntimeError::new(
            b_span.clone(),
            RuntimeErrorKind::IntegerOverflow { op: head.into(), a, b },
        )
        .into()
    })
}

pub(crate) fn i64_sub_op(head: &str, a: i64, b: i64, b_span: &Span) -> Result<i64, EvalBreak> {
    a.checked_sub(b).ok_or_else(|| {
        RuntimeError::new(
            b_span.clone(),
            RuntimeErrorKind::IntegerOverflow { op: head.into(), a, b },
        )
        .into()
    })
}

pub(crate) fn i64_mul_op(head: &str, a: i64, b: i64, b_span: &Span) -> Result<i64, EvalBreak> {
    a.checked_mul(b).ok_or_else(|| {
        RuntimeError::new(
            b_span.clone(),
            RuntimeErrorKind::IntegerOverflow { op: head.into(), a, b },
        )
        .into()
    })
}

/// `/` — truncating division; `i64::MIN / -1` is the one division-overflow
/// edge (`checked_div` returns `None` here since `b != 0` was already ruled
/// out above).
pub(crate) fn i64_div_op(head: &str, a: i64, b: i64, b_span: &Span) -> Result<i64, EvalBreak> {
    if b == 0 {
        Err(RuntimeError::new(b_span.clone(), RuntimeErrorKind::DivisionByZero).into())
    } else {
        a.checked_div(b).ok_or_else(|| {
            RuntimeError::new(
                b_span.clone(),
                RuntimeErrorKind::IntegerOverflow { op: head.into(), a, b },
            )
            .into()
        })
    }
}

/// `quot` — truncate toward zero, same as `/` (clj's mod/rem/quot trio).
pub(crate) fn i64_quot_op(head: &str, a: i64, b: i64, b_span: &Span) -> Result<i64, EvalBreak> {
    if b == 0 {
        Err(RuntimeError::new(b_span.clone(), RuntimeErrorKind::DivisionByZero).into())
    } else {
        a.checked_div(b).ok_or_else(|| {
            RuntimeError::new(
                b_span.clone(),
                RuntimeErrorKind::IntegerOverflow { op: head.into(), a, b },
            )
            .into()
        })
    }
}

/// `rem` — sign of the DIVIDEND. Never overflows (`|remainder| < |divisor|`);
/// `head` is unused (no `IntegerOverflow` branch exists for `rem`).
pub(crate) fn i64_rem_op(_head: &str, a: i64, b: i64, b_span: &Span) -> Result<i64, EvalBreak> {
    if b == 0 {
        Err(RuntimeError::new(b_span.clone(), RuntimeErrorKind::DivisionByZero).into())
    } else {
        // i64::MIN rem -1 is mathematically 0 but `checked_rem` returns `None`
        // (would need the same overflowing quotient as `/`) — clj-faithful
        // special-case rather than IntegerOverflow (`rem` itself never
        // overflows: |remainder| < |divisor|).
        Ok(a.checked_rem(b).unwrap_or(0))
    }
}

/// `mod` — sign of the DIVISOR, floored (adjust `rem`'s result by `+ b` when
/// the remainder is nonzero and disagrees in sign with the divisor). Never
/// overflows; `head` unused, same reason as `rem`.
pub(crate) fn i64_mod_op(_head: &str, a: i64, b: i64, b_span: &Span) -> Result<i64, EvalBreak> {
    if b == 0 {
        Err(RuntimeError::new(b_span.clone(), RuntimeErrorKind::DivisionByZero).into())
    } else {
        // Same MIN/-1 special-case as `rem` above (mod(MIN,-1) = 0).
        let r = a.checked_rem(b).unwrap_or(0);
        Ok(if r != 0 && (r < 0) != (b < 0) { r + b } else { r })
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

/// `:wat::core::bigint::/`'s op fn: divisible → `bigint` quotient;
/// otherwise → `:wat::core::rational` (reduced via `BigRational::new`,
/// REUSING Stone B's rational representation — no new rational impl).
/// Division by zero is a clean runtime error, never a panic (BigInt's `Div`
/// would otherwise panic on zero divisor like a primitive integer divide).
pub(crate) fn bigint_div(a: &BigInt, b: &BigInt, b_span: &Span) -> Result<Value, EvalBreak> {
    use num_traits::Zero;
    if b.is_zero() {
        return Err(RuntimeError::new(b_span.clone(), RuntimeErrorKind::DivisionByZero).into());
    }
    let (q, r) = (a / b, a % b);
    if r.is_zero() {
        Ok(Value::wat__core__BigInt(Box::new(q)))
    } else {
        Ok(Value::wat__core__Rational(Box::new(
            num_rational::BigRational::new(a.clone(), b.clone()),
        )))
    }
}

/// Coerce a `Value` to a `BigRational` for rational arithmetic — accepts
/// `:wat::core::rational` directly AND `:wat::core::bigint` (self-promoted
/// via `BigRational::from_integer`). Arc 300 stone C2: the bigint acceptance
/// is NOT strictness relaxation for its own sake — it is what lets
/// `wat/core.wat`'s N-ary rational fold reuse the SAME raw `rational::{+,-,*,/}`
/// intrinsic as its fold step (mirroring i64/f64/bigint's fold shape exactly)
/// even after an intermediate step COLLAPSES to bigint (see [`collapse_bigrational`]).
/// An i64 arg is still a type error — callers promote i64 explicitly via
/// `:wat::i64::to-rational` (mirrors C1's i64::to-bigint contagion pattern).
pub(crate) fn to_bigrational(v: &Value) -> Option<BigRational> {
    match v {
        Value::wat__core__Rational(r) => Some((**r).clone()),
        Value::wat__core__BigInt(n) => Some(BigRational::from_integer((**n).clone())),
        _ => None,
    }
}

/// The pinned C2 collapse: a `BigRational` arithmetic result that reduces to
/// a whole number (`is_integer()`) becomes `:wat::core::bigint` (C1's type,
/// reused as the collapse target — never a new "integer-valued rational"
/// representation); otherwise it stays `:wat::core::rational`. Inverse of
/// C1's `bigint::/` → rational collapse (that one collapses DOWN on failure
/// to divide evenly; this one collapses UP whenever the ratio happens to
/// reduce to an integer).
pub(crate) fn collapse_bigrational(r: BigRational) -> Value {
    if r.is_integer() {
        Value::wat__core__BigInt(Box::new(r.to_integer()))
    } else {
        Value::wat__core__Rational(Box::new(r))
    }
}

/// `:wat::core::rational::/`'s op fn: division by zero is a clean runtime
/// error, never a panic (mirrors `bigint_div`'s zero-divisor guard).
pub(crate) fn rational_div(a: &BigRational, b: &BigRational, b_span: &Span) -> Result<BigRational, EvalBreak> {
    use num_traits::Zero;
    if b.is_zero() {
        return Err(RuntimeError::new(b_span.clone(), RuntimeErrorKind::DivisionByZero).into());
    }
    Ok(a / b)
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

// Arc 255 Stone A-ii — the six `eval_f64_arith` op bodies, lifted out of the
// inline closures the OLD `:wat::core::f64::{+,-,*,/,max,min}` dispatch arms
// used to carry through Stone B; those arms are retired at Stone C, so the
// `:wat::f64::*` home (`src/intrinsic/f64.rs`) is now these fns' only caller.
// Unlike `i64_add_op` and
// friends, these take NO `head` param and cannot fail: f64 arithmetic has no
// overflow/division-by-zero error path (IEEE 754 gives ±Inf/NaN instead), so
// there is no `:op` to attribute an error to.
pub(crate) fn f64_add_op(
    a: f64,
    b: f64,
    _span: &Span, // rune:lint(unused-span) — infallible — no error path: IEEE 754 f64 addition is TOTAL (yields ±Inf/NaN, never an error), so there is no RuntimeError for this span to locate
) -> Result<f64, EvalBreak> {
    Ok(a + b)
}
pub(crate) fn f64_sub_op(
    a: f64,
    b: f64,
    _span: &Span, // rune:lint(unused-span) — infallible — no error path: IEEE 754 f64 subtraction is TOTAL (yields ±Inf/NaN, never an error), so there is no RuntimeError for this span to locate
) -> Result<f64, EvalBreak> {
    Ok(a - b)
}
pub(crate) fn f64_mul_op(
    a: f64,
    b: f64,
    _span: &Span, // rune:lint(unused-span) — infallible — no error path: IEEE 754 f64 multiplication is TOTAL (yields ±Inf/NaN, never an error), so there is no RuntimeError for this span to locate
) -> Result<f64, EvalBreak> {
    Ok(a * b)
}
pub(crate) fn f64_div_op(
    a: f64,
    b: f64,
    _span: &Span, // rune:lint(unused-span) — infallible — no error path: IEEE 754 f64 division is TOTAL (yields ±Inf/NaN, never an error), so there is no RuntimeError for this span to locate
) -> Result<f64, EvalBreak> {
    Ok(a / b)
}
pub(crate) fn f64_max_op(
    a: f64,
    b: f64,
    _span: &Span, // rune:lint(unused-span) — infallible — no error path: IEEE 754 f64 maximum is TOTAL (yields ±Inf/NaN, never an error), so there is no RuntimeError for this span to locate
) -> Result<f64, EvalBreak> {
    Ok(a.max(b))
}
pub(crate) fn f64_min_op(
    a: f64,
    b: f64,
    _span: &Span, // rune:lint(unused-span) — infallible — no error path: IEEE 754 f64 minimum is TOTAL (yields ±Inf/NaN, never an error), so there is no RuntimeError for this span to locate
) -> Result<f64, EvalBreak> {
    Ok(a.min(b))
}

/// Arc 148 slice 4 — Value-level arithmetic leaves used by
/// `dispatch_substrate_impl` when a per-Type leaf is addressed directly.
/// Two same-type helpers remain (i64-i64, f64-f64).
///
/// arc 237 Stone 237.8a — mixed-type helpers (i64-f64, f64-i64)
/// DELETED under THE DECISION (`feedback_no_implicit_coercion`).
///
/// The `Err(())` channel signals divide-by-zero (the helper translates
/// to `RuntimeError::DivisionByZero` with a synthesized span — the
/// dispatch path doesn't have argument spans available, so the span
/// is unknown).
///
/// Arc 300 stone C3 — the i64 leaves (only) enrich this to
/// [`I64ArithErr`], a small kind distinguishing divide-by-zero from
/// `checked_*` overflow (`None`); `f64`/`bigint`/`rational` are
/// unaffected and keep the plain `Result<T, ()>` divide-by-zero-only
/// channel.
// Arc 255 Stone N — widened `pub(crate)` (both were private): the 19
// arithmetic verbs' `value_handler` adapters (`src/intrinsic/{i64,f64,
// bigint,rational}.rs`) call these SAME fns — the exact op this table's own
// arms already used — from a different module, so the registry can serve
// `apply` with no new arithmetic implementation.
pub(crate) enum I64ArithErr {
    DivByZero,
    /// `checked_add/sub/mul/div` returned `None` — carries the operands
    /// so the error names the exact overflowing expression.
    Overflow(i64, i64),
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

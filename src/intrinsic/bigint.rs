//! `:wat::bigint::*` intrinsics — arc 255 Stone D, the bigint home.
//!
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md`.
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-D-bigint-and-rational.md`.
//!
//! The 6 bigint ops (`+ - * / to-f64 to-rational`), registered under their
//! new top-level home `:wat::bigint::*` — mirrors Stone A-i's i64 home
//! (`src/intrinsic/i64.rs`) exactly: same self-contained shape, same "share
//! the implementation, duplicate only the trivial predicate" rule.
//!
//! **Self-contained, no separate namespace-home file.** The bigint
//! arithmetic already lives in `runtime.rs` (`eval_bigint_arith`,
//! `bigint_div`, `eval_bigint_to_f64`, `eval_bigint_to_rational` — arc 300
//! stone C1) — nothing here duplicates it. `eval_bigint_arith` is generic
//! over a closure `F: Fn(&BigInt, &BigInt, &Span) -> Result<Value, EvalBreak>`
//! and so cannot itself carry `#[wat_intrinsic]`; every handler below is a
//! thin shim that supplies the SAME closure the old `:wat::core::bigint::*`
//! dispatch arm in `runtime.rs` supplied, and passes its own new spelling as
//! the `op`/`head` parameter so an error names whichever spelling the caller
//! actually used (same posture Stone C gave the i64/f64 converters).
//!
//! ## The old spelling still works through Phase 2
//!
//! Through Phase 2 of this stone, `:wat::core::bigint::*` keeps working
//! exactly as before (its arm in `runtime.rs`'s `dispatch_keyword_head_value`
//! is untouched) — this module ADDS a second address for the same 6
//! behaviors, deleting nothing. Phase 3 retires the old half.
//!
//! `:wat::core::+` — the polymorphic generic — is untouched; only the
//! per-type spelling moves.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

// ─── binary arithmetic: + - * / ─────────────────────────────────────────────
//
// Each handler clones its two `&WatAST` args into a 2-element array and
// forwards to `crate::runtime::eval_bigint_arith` — the EXACT arity-check /
// type-check / dispatch fn the old `:wat::core::bigint::*` arm calls — with
// the SAME closure that arm supplies. No arithmetic is re-implemented here.

/// `(:wat::bigint::+ a b)` → the sum of `a` and `b`, arbitrary precision.
/// Never overflows.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::bigint the left addend
/// @arg     b :wat::core::bigint the right addend
/// @ret     :wat::core::bigint the sum of `a` and `b`
/// @example (:wat::bigint::+ (:wat::i64::to-bigint 1) (:wat::i64::to-bigint 2)) #=> (:wat::i64::to-bigint 3)
#[wat_intrinsic(":wat::bigint::+", value = eval_bigint_add_value)]
pub(crate) fn eval_bigint_add(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::bigint::+";
    crate::runtime::eval_bigint_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _| {
        Ok(Value::wat__core__BigInt(Box::new(x + y)))
    })
}

// Arc 255 Stone N — value-level twin, for `dispatch_substrate_impl`'s
// registry-first door (`src/runtime.rs`). Same `arith_bigint_bigint_inner`-
// based implementation that fn's own `:wat::bigint::+` arm already used
// before this stone — see `i64.rs`'s `eval_i64_add_value` comment for why
// this is deliberately not merged with `eval_bigint_arith` above.
fn eval_bigint_add_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_bigint_bigint_inner(":wat::bigint::+", vals, span, |a, b| {
        Ok(Value::wat__core__BigInt(Box::new(a + b)))
    })
}

/// `(:wat::bigint::- a b)` → `a` minus `b`, arbitrary precision. Never
/// overflows.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::bigint the minuend
/// @arg     b :wat::core::bigint the subtrahend
/// @ret     :wat::core::bigint `a` minus `b`
/// @example (:wat::bigint::- (:wat::i64::to-bigint 5) (:wat::i64::to-bigint 3)) #=> (:wat::i64::to-bigint 2)
#[wat_intrinsic(":wat::bigint::-", value = eval_bigint_sub_value)]
pub(crate) fn eval_bigint_sub(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::bigint::-";
    crate::runtime::eval_bigint_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _| {
        Ok(Value::wat__core__BigInt(Box::new(x - y)))
    })
}

// Arc 255 Stone N — value-level twin; see `eval_bigint_add_value`'s comment above.
fn eval_bigint_sub_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_bigint_bigint_inner(":wat::bigint::-", vals, span, |a, b| {
        Ok(Value::wat__core__BigInt(Box::new(a - b)))
    })
}

/// `(:wat::bigint::* a b)` → `a` times `b`, arbitrary precision. Never
/// overflows.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::bigint the first factor
/// @arg     b :wat::core::bigint the second factor
/// @ret     :wat::core::bigint `a` times `b`
/// @example (:wat::bigint::* (:wat::i64::to-bigint 3) (:wat::i64::to-bigint 4)) #=> (:wat::i64::to-bigint 12)
#[wat_intrinsic(":wat::bigint::*", value = eval_bigint_mul_value)]
pub(crate) fn eval_bigint_mul(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::bigint::*";
    crate::runtime::eval_bigint_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _| {
        Ok(Value::wat__core__BigInt(Box::new(x * y)))
    })
}

// Arc 255 Stone N — value-level twin; see `eval_bigint_add_value`'s comment above.
fn eval_bigint_mul_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_bigint_bigint_inner(":wat::bigint::*", vals, span, |a, b| {
        Ok(Value::wat__core__BigInt(Box::new(a * b)))
    })
}

/// `(:wat::bigint::/ a b)` → `a` divided by `b`. Divisible → `:wat::core::bigint`
/// quotient; otherwise → `:wat::core::rational` (reduced), reusing the
/// rational representation rather than truncating. `b = 0` raises
/// `DivisionByZero` — never a panic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::bigint the dividend
/// @arg     b :wat::core::bigint the divisor
/// @ret     :wat::core::bigint `a` divided by `b` (bigint if divisible, rational otherwise)
/// @example (:wat::bigint::/ (:wat::i64::to-bigint 6) (:wat::i64::to-bigint 2)) #=> (:wat::i64::to-bigint 3)
#[wat_intrinsic(":wat::bigint::/", value = eval_bigint_div_value)]
pub(crate) fn eval_bigint_div_intrinsic(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::bigint::/";
    crate::runtime::eval_bigint_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::bigint_div)
}

// Arc 255 Stone N — value-level twin; see `eval_bigint_add_value`'s comment
// above. Body copied verbatim from `dispatch_substrate_impl`'s own
// `:wat::bigint::/` arm (`src/runtime.rs`) — NOT the direct path's
// `crate::runtime::bigint_div` (a different fn, `(&BigInt,&BigInt,&Span) ->
// Result<Value, EvalBreak>`, incompatible with `arith_bigint_bigint_inner`'s
// `Fn(&BigInt,&BigInt) -> Result<Value, ()>`).
fn eval_bigint_div_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_bigint_bigint_inner(":wat::bigint::/", vals, span, |a, b| {
        use num_traits::Zero;
        if b.is_zero() {
            return Err(());
        }
        let (q, r) = (a / b, a % b);
        if r.is_zero() {
            Ok(Value::wat__core__BigInt(Box::new(q)))
        } else {
            Ok(Value::wat__core__Rational(Box::new(
                num_rational::BigRational::new(a.clone(), b.clone()),
            )))
        }
    })
}

// ─── unary conversions: to-f64 to-rational ─────────────────────────────────
//
// `crate::runtime::eval_bigint_to_{f64,rational}` already take
// `args: &[WatAST]` + an `op: &str`; `std::slice::from_ref(n)` hands them a
// length-1 view of our single `&WatAST` param with no clone at all.

/// `(:wat::bigint::to-f64 n)` → `n` cast to `:wat::core::f64`. Lossy beyond
/// f64's 53-bit mantissa; never fails.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     n :wat::core::bigint the bigint to cast
/// @ret     :wat::core::f64 `n`, cast to f64
/// @example (:wat::bigint::to-f64 (:wat::i64::to-bigint 5)) #=> 5.0
#[wat_intrinsic(":wat::bigint::to-f64")]
pub(crate) fn eval_bigint_to_f64_intrinsic(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_bigint_to_f64(std::slice::from_ref(n), span, env, sym, ":wat::bigint::to-f64")
}

/// `(:wat::bigint::to-rational n)` → `n` promoted to `:wat::core::rational`.
/// Infallible.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     n :wat::core::bigint the bigint to promote
/// @ret     :wat::core::rational `n`, promoted to rational
/// @example (:wat::bigint::to-rational (:wat::i64::to-bigint 5)) #=> (:wat::bigint::to-rational (:wat::i64::to-bigint 5))
#[wat_intrinsic(":wat::bigint::to-rational")]
pub(crate) fn eval_bigint_to_rational_intrinsic(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_bigint_to_rational(std::slice::from_ref(n), span, env, sym, ":wat::bigint::to-rational")
}

//! `:wat::rational::*` intrinsics — arc 255 Stone D, the rational home.
//!
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md`.
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-D-bigint-and-rational.md`.
//!
//! The 7 rational ops (`+ - * / to-f64 numerator denominator`), registered
//! under their new top-level home `:wat::rational::*`. Two of these
//! (`numerator`/`denominator`) also change SHAPE, not just address: the old
//! home used a slash-form accessor (`:wat::core::rational/numerator`, cf
//! `Uuid/version`); the new home spells them as ordinary `::` verbs
//! (`:wat::rational::numerator`) — the recorded `:wat::core::Uuid/v4 ->
//! :wat::uuid::v4` precedent this stone's brief names.
//!
//! **Self-contained, no separate namespace-home file.** The rational
//! arithmetic already lives in `runtime.rs` (`eval_rational_arith`,
//! `rational_div`, `eval_rational_to_f64`, `eval_rational_numerator`,
//! `eval_rational_denominator` — arc 300 stone C2) — nothing here duplicates
//! it. `eval_rational_arith` is generic over a closure
//! `F: Fn(&BigRational, &BigRational, &Span) -> Result<BigRational, EvalBreak>`
//! and so cannot itself carry `#[wat_intrinsic]`; every handler below is a
//! thin shim that supplies the SAME closure the old
//! `:wat::core::rational::*` dispatch arm supplied, and passes its own new
//! spelling as the `op`/`head` parameter so an error names whichever
//! spelling the caller actually used (same posture Stone C gave the i64/f64
//! converters).
//!
//! ## The old spelling still works through Phase 2
//!
//! Through Phase 2 of this stone, `:wat::core::rational::*` /
//! `:wat::core::rational/*` keep working exactly as before (their arms in
//! `runtime.rs`'s `dispatch_keyword_head_value` are untouched) — this module
//! ADDS a second address for the same 7 behaviors, deleting nothing. Phase 3
//! retires the old half.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

// ─── binary arithmetic: + - * / ─────────────────────────────────────────────
//
// Each handler clones its two `&WatAST` args into a 2-element array and
// forwards to `crate::numeric::arith::eval_rational_arith` — the EXACT arity-check /
// type-check / dispatch fn the old `:wat::core::rational::*` arm calls —
// with the SAME closure that arm supplies (every op COLLAPSES via
// `collapse_bigrational` inside `eval_rational_arith` itself). No arithmetic
// is re-implemented here.

/// `(:wat::rational::+ a b)` → the sum of `a` and `b`. Collapses to
/// `:wat::core::bigint` when the result is integer-valued.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::rational the left addend
/// @arg     b :wat::core::rational the right addend
/// @ret     :wat::core::rational the sum of `a` and `b`
/// @example (:wat::rational::+ (:wat::i64::to-rational 1) (:wat::i64::to-rational 2)) #=> (:wat::i64::to-rational 3)
#[wat_intrinsic(":wat::rational::+", value = eval_rational_add_value)]
pub(crate) fn eval_rational_add(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rational::+";
    crate::numeric::arith::eval_rational_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _| {
        Ok(x + y)
    })
}

// Arc 255 Stone N — value-level twin, for `dispatch_substrate_impl`'s
// registry-first door (`src/runtime.rs`). Same `arith_rational_rational_inner`
// -based implementation that fn's own `:wat::rational::+` arm already used
// before this stone — see `i64.rs`'s `eval_i64_add_value` comment for why
// this is deliberately not merged with `eval_rational_arith` above.
fn eval_rational_add_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_rational_rational_inner(":wat::rational::+", vals, span, |a, b| Ok(a + b))
}

/// `(:wat::rational::- a b)` → `a` minus `b`. Collapses to `:wat::core::bigint`
/// when the result is integer-valued.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::rational the minuend
/// @arg     b :wat::core::rational the subtrahend
/// @ret     :wat::core::rational `a` minus `b`
/// @example (:wat::rational::- (:wat::i64::to-rational 5) (:wat::i64::to-rational 3)) #=> (:wat::i64::to-rational 2)
#[wat_intrinsic(":wat::rational::-", value = eval_rational_sub_value)]
pub(crate) fn eval_rational_sub(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rational::-";
    crate::numeric::arith::eval_rational_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _| {
        Ok(x - y)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_rational_add_value`'s comment above.
fn eval_rational_sub_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_rational_rational_inner(":wat::rational::-", vals, span, |a, b| Ok(a - b))
}

/// `(:wat::rational::* a b)` → `a` times `b`. Collapses to `:wat::core::bigint`
/// when the result is integer-valued.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::rational the first factor
/// @arg     b :wat::core::rational the second factor
/// @ret     :wat::core::rational `a` times `b`
/// @example (:wat::rational::* (:wat::i64::to-rational 3) (:wat::i64::to-rational 4)) #=> (:wat::i64::to-rational 12)
#[wat_intrinsic(":wat::rational::*", value = eval_rational_mul_value)]
pub(crate) fn eval_rational_mul(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rational::*";
    crate::numeric::arith::eval_rational_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _| {
        Ok(x * y)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_rational_add_value`'s comment above.
fn eval_rational_mul_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_rational_rational_inner(":wat::rational::*", vals, span, |a, b| Ok(a * b))
}

/// `(:wat::rational::/ a b)` → `a` divided by `b`. `b = 0` raises
/// `DivisionByZero`. Collapses to `:wat::core::bigint` when the result is
/// integer-valued.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::rational the dividend
/// @arg     b :wat::core::rational the divisor
/// @ret     :wat::core::rational `a` divided by `b`
/// @example (:wat::rational::/ (:wat::i64::to-rational 6) (:wat::i64::to-rational 2)) #=> (:wat::i64::to-rational 3)
#[wat_intrinsic(":wat::rational::/", value = eval_rational_div_value)]
pub(crate) fn eval_rational_div_intrinsic(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rational::/";
    crate::numeric::arith::eval_rational_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::rational_div)
}

// Arc 255 Stone N — value-level twin; see `eval_rational_add_value`'s
// comment above. Body copied verbatim from `dispatch_substrate_impl`'s own
// `:wat::rational::/` arm (`src/runtime.rs`) — NOT the direct path's
// `crate::runtime::rational_div` (incompatible signature, same reason as
// `bigint.rs`'s `eval_bigint_div_value`).
fn eval_rational_div_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_rational_rational_inner(":wat::rational::/", vals, span, |a, b| {
        use num_traits::Zero;
        if b.is_zero() {
            return Err(());
        }
        Ok(a / b)
    })
}

// ─── to-f64 + accessors: to-f64 numerator denominator ─────────────────────
//
// `crate::runtime::eval_rational_{to_f64,numerator,denominator}` already
// take `args: &[WatAST]` + an `op: &str`; `std::slice::from_ref(n)` hands
// them a length-1 view of our single `&WatAST` param with no clone at all.

/// `(:wat::rational::to-f64 n)` → `n` cast to `:wat::core::f64`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     n :wat::core::rational the rational to cast
/// @ret     :wat::core::f64 `n`, cast to f64
/// @example (:wat::rational::to-f64 (:wat::i64::to-rational 5)) #=> 5.0
#[wat_intrinsic(":wat::rational::to-f64")]
pub(crate) fn eval_rational_to_f64_intrinsic(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::numeric::convert::eval_rational_to_f64(std::slice::from_ref(n), span, env, sym, ":wat::rational::to-f64")
}

/// `(:wat::rational::numerator n)` → the numerator of `n`. Renders as
/// `:wat::core::i64` when it fits; `:wat::core::bigint` otherwise (never
/// silently truncated).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     n :wat::core::rational the rational to inspect
/// @ret     :wat::core::i64 the numerator of `n`
/// @example (:wat::rational::numerator (:wat::i64::to-rational 5)) #=> 5
#[wat_intrinsic(":wat::rational::numerator")]
pub(crate) fn eval_rational_numerator_intrinsic(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::numeric::ops::eval_rational_numerator(std::slice::from_ref(n), span, env, sym, ":wat::rational::numerator")
}

/// `(:wat::rational::denominator n)` → the denominator of `n`. Renders as
/// `:wat::core::i64` when it fits; `:wat::core::bigint` otherwise (never
/// silently truncated).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     n :wat::core::rational the rational to inspect
/// @ret     :wat::core::i64 the denominator of `n`
/// @example (:wat::rational::denominator (:wat::i64::to-rational 5)) #=> 1
#[wat_intrinsic(":wat::rational::denominator")]
pub(crate) fn eval_rational_denominator_intrinsic(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::numeric::ops::eval_rational_denominator(std::slice::from_ref(n), span, env, sym, ":wat::rational::denominator")
}

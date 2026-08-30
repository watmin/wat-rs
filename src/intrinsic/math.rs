//! `:wat::math::*` intrinsics — arc 255 Stone HOME-10, the math home.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-HOME-10-math-stat-seq-get-actual-homes.md`.
//!
//! The 6 math ops (`ln exp sqrt sin cos pi`), registered under their already-final
//! home `:wat::math::*` (HOME-9 renamed them off the dead `:wat::std::` namespace;
//! this stone only moves the dispatch arm into a `#[wat_intrinsic]` handler —
//! nothing is renamed, no corpus file is touched).
//!
//! **Self-contained, no separate namespace-home file.** Measured (the drawing
//! commit, `93c5aef52`): `eval_math_unary` is arity-check -> unwrap -> `f(x)` ->
//! rewrap, where `f` is a Rust std fn (`f64::ln`, `f64::sqrt`, …) — squarely in
//! the shim-only band the existing homes (`uuid`, `bytes`, `char`) already
//! occupy, not the two-layer case HOME-8's `src/holon/` split earned. Every
//! handler below is a thin shim over `crate::runtime::eval_math_unary` /
//! `eval_math_pi` — the SAME functions the old `:wat::math::*` dispatch arms
//! called — passing its own spelling through so an error names whichever
//! spelling the caller actually used.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::math::ln x)` → the natural logarithm of `x`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     x :wat::core::f64 the value to take the natural log of
/// @ret     :wat::core::f64 the natural logarithm of `x`
/// @example (:wat::math::ln 1.0) #=> 0.0
#[wat_intrinsic(":wat::math::ln")]
pub(crate) fn eval_math_ln_intrinsic(
    x: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_math_unary(std::slice::from_ref(x), env, sym, "ln", f64::ln, span)
}

/// `(:wat::math::exp x)` → `e` raised to the power `x`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     x :wat::core::f64 the exponent
/// @ret     :wat::core::f64 `e` raised to the power `x`
/// @example (:wat::math::exp 0.0) #=> 1.0
#[wat_intrinsic(":wat::math::exp")]
pub(crate) fn eval_math_exp_intrinsic(
    x: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_math_unary(std::slice::from_ref(x), env, sym, "exp", f64::exp, span)
}

/// `(:wat::math::sqrt x)` → the non-negative square root of `x`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     x :wat::core::f64 the value to take the square root of
/// @ret     :wat::core::f64 the non-negative square root of `x`
/// @example (:wat::math::sqrt 16.0) #=> 4.0
#[wat_intrinsic(":wat::math::sqrt")]
pub(crate) fn eval_math_sqrt_intrinsic(
    x: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_math_unary(std::slice::from_ref(x), env, sym, "sqrt", f64::sqrt, span)
}

/// `(:wat::math::sin x)` → the sine of `x` (radians).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     x :wat::core::f64 the angle in radians
/// @ret     :wat::core::f64 the sine of `x`
/// @example (:wat::math::sin 0.0) #=> 0.0
#[wat_intrinsic(":wat::math::sin")]
pub(crate) fn eval_math_sin_intrinsic(
    x: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_math_unary(std::slice::from_ref(x), env, sym, "sin", f64::sin, span)
}

/// `(:wat::math::cos x)` → the cosine of `x` (radians).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @arg     x :wat::core::f64 the angle in radians
/// @ret     :wat::core::f64 the cosine of `x`
/// @example (:wat::math::cos 0.0) #=> 1.0
#[wat_intrinsic(":wat::math::cos")]
pub(crate) fn eval_math_cos_intrinsic(
    x: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_math_unary(std::slice::from_ref(x), env, sym, "cos", f64::cos, span)
}

/// `(:wat::math::pi)` — the mathematical constant π as `:wat::core::f64`.
/// Nullary. Backing: `std::f64::consts::PI`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Arithmetic
/// @ret     :wat::core::f64 the mathematical constant π
/// @example (:wat::math::pi) #=> 3.141592653589793
#[wat_intrinsic(":wat::math::pi")]
pub(crate) fn eval_math_pi_intrinsic(span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::eval_math_pi(&[], span)
}

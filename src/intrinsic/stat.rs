//! `:wat::stat::*` intrinsics — arc 255 Stone HOME-10, the stat home.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-HOME-10-math-stat-seq-get-actual-homes.md`.
//!
//! The 3 stat ops (`mean variance stddev`), registered under their already-final
//! home `:wat::stat::*` (HOME-9 renamed them off the dead `:wat::std::` namespace;
//! this stone only moves the dispatch arm into a `#[wat_intrinsic]` handler —
//! nothing is renamed, no corpus file is touched).
//!
//! **Self-contained, no separate namespace-home file.** Measured (the drawing
//! commit, `93c5aef52`): the "real arithmetic" here is `let mut sum = 0.0; sum
//! += x` inside a body that is mostly arity/type-checking — squarely in the
//! shim-only band, not the two-layer case. Every handler below is a thin shim
//! over `crate::runtime::eval_stat_{mean,variance,stddev}` — the SAME functions
//! the old `:wat::stat::*` dispatch arms called.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::stat::mean xs)` → the population mean of `xs`. `None` on empty
/// input — matches `f64::min-of`/`f64::max-of`'s reduction-empty convention.
///
/// Surfaced by holon-lab-trading arc 026 slice 9 (Hurst's R/S analysis) and
/// slice 4 (Bollinger's RollingStddev). Universal enough to live in core
/// stdlib.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     xs (:wat::core::Vector :- [:wat::core::f64]) the values to average
/// @ret     (:wat::core::Option :- [:wat::core::f64]) `Some` the population mean of `xs`, or `None` on empty input
/// @example (:wat::stat::mean (:wat::core::Vector :- [:wat::core::f64] 2.0 4.0)) #=> (:wat::core::Some 3.0)
#[wat_intrinsic(":wat::stat::mean")]
pub(crate) fn eval_stat_mean_intrinsic(
    xs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_stat_mean(std::slice::from_ref(xs), env, sym, span)
}

/// `(:wat::stat::variance xs)` → the population variance of `xs` (divides by
/// `n`; matches numpy's default `ddof=0`). `None` on empty input;
/// single-point input returns `Some(0.0)` (no spread).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     xs (:wat::core::Vector :- [:wat::core::f64]) the values to measure the spread of
/// @ret     (:wat::core::Option :- [:wat::core::f64]) `Some` the population variance of `xs`, or `None` on empty input
/// @example (:wat::stat::variance (:wat::core::Vector :- [:wat::core::f64] 2.0 4.0)) #=> (:wat::core::Some 1.0)
#[wat_intrinsic(":wat::stat::variance")]
pub(crate) fn eval_stat_variance_intrinsic(
    xs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_stat_variance(std::slice::from_ref(xs), env, sym, span)
}

/// `(:wat::stat::stddev xs)` → the square root of the population variance of
/// `xs`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     xs (:wat::core::Vector :- [:wat::core::f64]) the values to measure the spread of
/// @ret     (:wat::core::Option :- [:wat::core::f64]) `Some` the population standard deviation of `xs`, or `None` on empty input
/// @example (:wat::stat::stddev (:wat::core::Vector :- [:wat::core::f64] 2.0 4.0)) #=> (:wat::core::Some 1.0)
#[wat_intrinsic(":wat::stat::stddev")]
pub(crate) fn eval_stat_stddev_intrinsic(
    xs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_stat_stddev(std::slice::from_ref(xs), env, sym, span)
}

//! `:wat::i64::*` intrinsics — arc 255 Stone A-i, the i64 home.
//!
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md`.
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-i-the-i64-home.md`.
//!
//! The 17 i64 ops (`+ - * / < <= > >= = not= mod quot rem to-bigint to-f64
//! to-rational to-string`), registered under their new top-level home
//! `:wat::i64::*` — `:wat::i64::+` renders, once the surface flips, as
//! `wat.i64/+`, a two-segment namespace+name; `:wat::core::i64::+` buries the
//! type one level too deep (see the design's "why the destination" section).
//!
//! **Self-contained, no separate namespace-home file** — follows `bytes.rs`'s
//! shape, not `string.rs`'s two-home split. The i64 arithmetic lives in
//! `runtime.rs` (`eval_i64_arith`, `eval_compare`, the seven named
//! `i64_*_op` fns, and the four `eval_i64_to_*` scalar-conversion fns) —
//! nothing here duplicates it. Every handler below is a thin
//! `#[wat_intrinsic]` shim that calls straight into those `pub(crate)` fns.
//!
//! ## Arc 255 Stone C — the old spelling is retired
//!
//! Through Stone B, `:wat::core::i64::*` kept working exactly as before —
//! this module ADDED a second address for the same 17 behaviors, deleting
//! nothing. Stone C deletes the OLD half: `dispatch_keyword_head_value`'s
//! per-type i64 arms in `runtime.rs` are gone, `dispatch_substrate_impl` /
//! `arith_i64_i64_inner`'s match keys are the new spelling directly, and
//! `:wat::core::i64::*` is now a `RETIREMENT_TABLE` hit
//! (`src/remedy/retirement.rs`) — a check-time error naming this module's
//! spelling as the remedy, not a silent fallthrough. The shared `i64_*_op`
//! fns this module's handlers call are untouched; only their OLD callers in
//! `runtime.rs` are gone.
//!
//! `:wat::core::+` — the polymorphic generic — is untouched; only the
//! per-type spelling moved (enabling arc 256's generic defclause later).

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::numeric::arith::I64ArithErr;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

// ─── binary arithmetic: + - * / mod quot rem ───────────────────────────────
//
// Each handler clones its two `&WatAST` args into a 2-element array and
// forwards to `crate::numeric::arith::eval_i64_arith` — the EXACT arity-check /
// type-check / dispatch fn the old `:wat::i64::*` arm calls — with a
// closure that forwards straight to the shared named op fn
// (`crate::numeric::arith::i64_add_op` etc). No arithmetic is re-implemented here.

/// `(:wat::i64::+ a b)` → the sum of `a` and `b`, strict i64 (no promotion
/// from f64). Overflow raises a distinct `RuntimeErrorKind::IntegerOverflow`
/// — never wrapped, never conflated with `DivisionByZero` — via the SAME
/// shared op fn `:wat::i64::+` calls.
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping; `mod`/`rem`/`quot`
/// route through the same `eval_i64_arith` dispatch as `/` — div-by-zero is a deterministic
/// located abort (`RuntimeErrorKind::DivisionByZero`), never a panic, same as `/`. Safe to
/// evaluate while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the left addend
/// @arg     b :wat::core::i64 the right addend
/// @ret     :wat::core::i64 the sum of `a` and `b`
/// @example (:wat::i64::+ 1 2) #=> 3
#[wat_intrinsic(":wat::i64::+", value = eval_i64_add_value)]
pub(crate) fn eval_i64_add(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::+";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, b_span| {
        crate::numeric::arith::i64_add_op(OP, x, y, b_span)
    })
}

// Arc 255 Stone N — value-level twin, for `dispatch_substrate_impl`'s
// registry-first door (`src/runtime.rs`, `:wat::core::apply`'s substrate
// fallback). NOT the same Rust fn as `eval_i64_add` above — `apply` hands
// already-evaluated `Value`s with no arg-level `Span`s, so it goes through
// `arith_i64_i64_inner` (the SAME fn `dispatch_substrate_impl`'s own
// `:wat::i64::+` arm already called before this stone; error spans are
// synthesized there, real argument spans here — a pre-existing difference
// this stone does not change). See BRIEF-STONE-N's "two parallel
// implementations" note: this is deliberately NOT merged into
// `eval_i64_arith`/`i64_add_op` above, which would drop apply's ability to
// ever gain real spans and would risk widening today's `apply`-only
// synthesized-span behavior onto the direct path instead.
fn eval_i64_add_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::+", vals, span, |a, b| {
        a.checked_add(b).ok_or(I64ArithErr::Overflow(a, b))
    })
}

/// `(:wat::i64::- a b)` → `a` minus `b`, strict i64. Overflow raises
/// `RuntimeErrorKind::IntegerOverflow`, same shared op fn as `:wat::i64::-`.
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping; `mod`/`rem`/`quot`
/// route through the same `eval_i64_arith` dispatch as `/` — div-by-zero is a deterministic
/// located abort (`RuntimeErrorKind::DivisionByZero`), never a panic, same as `/`. Safe to
/// evaluate while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the minuend
/// @arg     b :wat::core::i64 the subtrahend
/// @ret     :wat::core::i64 `a` minus `b`
/// @example (:wat::i64::- 5 3) #=> 2
#[wat_intrinsic(":wat::i64::-", value = eval_i64_sub_value)]
pub(crate) fn eval_i64_sub(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::-";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, b_span| {
        crate::numeric::arith::i64_sub_op(OP, x, y, b_span)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_i64_add_value`'s comment
// above for why this is `arith_i64_i64_inner`, not `eval_i64_arith`.
fn eval_i64_sub_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::-", vals, span, |a, b| {
        a.checked_sub(b).ok_or(I64ArithErr::Overflow(a, b))
    })
}

/// `(:wat::i64::* a b)` → `a` times `b`, strict i64. Overflow raises
/// `RuntimeErrorKind::IntegerOverflow`, same shared op fn as `:wat::i64::*`.
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping; `mod`/`rem`/`quot`
/// route through the same `eval_i64_arith` dispatch as `/` — div-by-zero is a deterministic
/// located abort (`RuntimeErrorKind::DivisionByZero`), never a panic, same as `/`. Safe to
/// evaluate while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the first factor
/// @arg     b :wat::core::i64 the second factor
/// @ret     :wat::core::i64 `a` times `b`
/// @example (:wat::i64::* 3 4) #=> 12
#[wat_intrinsic(":wat::i64::*", value = eval_i64_mul_value)]
pub(crate) fn eval_i64_mul(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::*";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, b_span| {
        crate::numeric::arith::i64_mul_op(OP, x, y, b_span)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_i64_add_value`'s comment
// above for why this is `arith_i64_i64_inner`, not `eval_i64_arith`.
fn eval_i64_mul_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::*", vals, span, |a, b| {
        a.checked_mul(b).ok_or(I64ArithErr::Overflow(a, b))
    })
}

/// `(:wat::i64::/ a b)` → `a` divided by `b`, truncating toward zero. `b = 0`
/// raises `DivisionByZero`; `i64::MIN / -1` raises `IntegerOverflow` — the
/// one division-overflow edge. Same shared op fn as `:wat::i64::/`.
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping; `mod`/`rem`/`quot`
/// route through the same `eval_i64_arith` dispatch as `/` — div-by-zero is a deterministic
/// located abort (`RuntimeErrorKind::DivisionByZero`), never a panic, same as `/`. Safe to
/// evaluate while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the dividend
/// @arg     b :wat::core::i64 the divisor
/// @ret     :wat::core::i64 `a` divided by `b`, truncated toward zero
/// @example (:wat::i64::/ 6 2) #=> 3
#[wat_intrinsic(":wat::i64::/", value = eval_i64_div_value)]
pub(crate) fn eval_i64_div(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::/";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, b_span| {
        crate::numeric::arith::i64_div_op(OP, x, y, b_span)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_i64_add_value`'s comment
// above for why this is `arith_i64_i64_inner`, not `eval_i64_arith`.
fn eval_i64_div_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::/", vals, span, |a, b| {
        if b == 0 {
            Err(I64ArithErr::DivByZero)
        } else {
            a.checked_div(b).ok_or(I64ArithErr::Overflow(a, b))
        }
    })
}

/// `(:wat::i64::mod a b)` → `a` modulo `b`, floored — sign follows the
/// DIVISOR (clj's `mod`). `b = 0` raises `DivisionByZero`. Same shared op fn
/// as `:wat::i64::mod`.
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping; `mod`/`rem`/`quot`
/// route through the same `eval_i64_arith` dispatch as `/` — div-by-zero is a deterministic
/// located abort (`RuntimeErrorKind::DivisionByZero`), never a panic, same as `/`. Safe to
/// evaluate while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the dividend
/// @arg     b :wat::core::i64 the divisor
/// @ret     :wat::core::i64 `a` modulo `b`, sign of `b`
/// @example (:wat::i64::mod -7 3) #=> 2
#[wat_intrinsic(":wat::i64::mod", value = eval_i64_mod_value)]
pub(crate) fn eval_i64_mod(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::mod";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, b_span| {
        crate::numeric::arith::i64_mod_op(OP, x, y, b_span)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_i64_add_value`'s comment
// above for why this is `arith_i64_i64_inner`, not `eval_i64_arith`.
fn eval_i64_mod_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::mod", vals, span, |a, b| {
        if b == 0 {
            Err(I64ArithErr::DivByZero)
        } else {
            let r = a.checked_rem(b).unwrap_or(0);
            Ok(if r != 0 && (r < 0) != (b < 0) {
                r + b
            } else {
                r
            })
        }
    })
}

/// `(:wat::i64::quot a b)` → `a` divided by `b`, truncated toward zero
/// (clj's `quot`). `b = 0` raises `DivisionByZero`. Same shared op fn as
/// `:wat::i64::quot`.
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping; `mod`/`rem`/`quot`
/// route through the same `eval_i64_arith` dispatch as `/` — div-by-zero is a deterministic
/// located abort (`RuntimeErrorKind::DivisionByZero`), never a panic, same as `/`. Safe to
/// evaluate while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the dividend
/// @arg     b :wat::core::i64 the divisor
/// @ret     :wat::core::i64 `a` divided by `b`, truncated toward zero
/// @example (:wat::i64::quot -7 3) #=> -2
#[wat_intrinsic(":wat::i64::quot", value = eval_i64_quot_value)]
pub(crate) fn eval_i64_quot(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::quot";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, b_span| {
        crate::numeric::arith::i64_quot_op(OP, x, y, b_span)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_i64_add_value`'s comment
// above for why this is `arith_i64_i64_inner`, not `eval_i64_arith`.
fn eval_i64_quot_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::quot", vals, span, |a, b| {
        if b == 0 {
            Err(I64ArithErr::DivByZero)
        } else {
            a.checked_div(b).ok_or(I64ArithErr::Overflow(a, b))
        }
    })
}

/// `(:wat::i64::rem a b)` → the remainder of `a` divided by `b` — sign
/// follows the DIVIDEND (clj's `rem`). `b = 0` raises `DivisionByZero`. Same
/// shared op fn as `:wat::i64::rem`.
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping; `mod`/`rem`/`quot`
/// route through the same `eval_i64_arith` dispatch as `/` — div-by-zero is a deterministic
/// located abort (`RuntimeErrorKind::DivisionByZero`), never a panic, same as `/`. Safe to
/// evaluate while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the dividend
/// @arg     b :wat::core::i64 the divisor
/// @ret     :wat::core::i64 the remainder of `a` divided by `b`, sign of `a`
/// @example (:wat::i64::rem -7 3) #=> -1
#[wat_intrinsic(":wat::i64::rem", value = eval_i64_rem_value)]
pub(crate) fn eval_i64_rem(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::rem";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, b_span| {
        crate::numeric::arith::i64_rem_op(OP, x, y, b_span)
    })
}

// Arc 255 Stone N — value-level twin; see `eval_i64_add_value`'s comment
// above for why this is `arith_i64_i64_inner`, not `eval_i64_arith`.
fn eval_i64_rem_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::rem", vals, span, |a, b| {
        if b == 0 {
            Err(I64ArithErr::DivByZero)
        } else {
            Ok(a.checked_rem(b).unwrap_or(0))
        }
    })
}

// ─── comparisons: < <= > >= = not= ─────────────────────────────────────────
//
// `crate::runtime::eval_compare` is the SAME engine `:wat::i64::{<,<=,
// >,>=,=,not=}` calls (NaN-aware `numeric_order` first, `values_compare`
// fallback). Each predicate closure below is trivial (an `Ordering` compare,
// not an arithmetic contract) and is duplicated the same way the existing
// `:wat::core::<` / `:wat::i64::<` pair already duplicates it in
// runtime.rs — there is no algorithm here to share beyond the engine itself.

/// `(:wat::i64::< a b)` → whether `a` is less than `b`.
///
/// **Totality ground —** comparisons never overflow (only `+`/`-`/`*`/`/` do), so the
/// output is defined for every pair of i64 inputs. Grouped with `i64::>`/`i64::>=`/`i64::<=`
/// in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's,
/// made by reading the implementation.
///
/// **Expand-time ground —** integer comparison: pure, deterministic, total over `i64`. Safe
/// to evaluate at macro-expansion time. Ruling relocated from `macros/eval.rs`'s expand-time
/// allow-list (arc 255 expand-T4a), from its "Integer comparison" group; the verdict is that
/// list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::bool true iff `a` is less than `b`
/// @example (:wat::i64::< 1 2) #=> true
#[wat_intrinsic(":wat::i64::<")]
pub(crate) fn eval_i64_lt(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::<";
    crate::runtime::eval_compare(OP, &[a.clone(), b.clone()], span, env, sym, |o| {
        o == std::cmp::Ordering::Less
    })
}

/// `(:wat::i64::<= a b)` → whether `a` is less than or equal to `b`.
///
/// **Totality ground —** comparisons never overflow (only `+`/`-`/`*`/`/` do), so the
/// output is defined for every pair of i64 inputs. Grouped with `i64::>`/`i64::<`/`i64::>=`
/// in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's,
/// made by reading the implementation.
///
/// **Expand-time ground —** integer comparison: pure, deterministic, total over `i64`. Safe
/// to evaluate at macro-expansion time. Ruling relocated from `macros/eval.rs`'s expand-time
/// allow-list (arc 255 expand-T4a), from its "Integer comparison" group; the verdict is that
/// list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::bool true iff `a` is less than or equal to `b`
/// @example (:wat::i64::<= 2 2) #=> true
#[wat_intrinsic(":wat::i64::<=")]
pub(crate) fn eval_i64_lte(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::<=";
    crate::runtime::eval_compare(OP, &[a.clone(), b.clone()], span, env, sym, |o| {
        o != std::cmp::Ordering::Greater
    })
}

/// `(:wat::i64::> a b)` → whether `a` is greater than `b`.
///
/// **Totality ground —** comparisons never overflow (only `+`/`-`/`*`/`/` do), so the
/// output is defined for every pair of i64 inputs. Grouped with `i64::<`/`i64::>=`/`i64::<=`
/// in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's,
/// made by reading the implementation.
///
/// **Expand-time ground —** integer comparison: pure, deterministic, total over `i64`. Safe
/// to evaluate at macro-expansion time. Ruling relocated from `macros/eval.rs`'s expand-time
/// allow-list (arc 255 expand-T4a), from its "Integer comparison" group; the verdict is that
/// list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::bool true iff `a` is greater than `b`
/// @example (:wat::i64::> 3 2) #=> true
#[wat_intrinsic(":wat::i64::>")]
pub(crate) fn eval_i64_gt(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::>";
    crate::runtime::eval_compare(OP, &[a.clone(), b.clone()], span, env, sym, |o| {
        o == std::cmp::Ordering::Greater
    })
}

/// `(:wat::i64::>= a b)` → whether `a` is greater than or equal to `b`.
///
/// **Totality ground —** comparisons never overflow (only `+`/`-`/`*`/`/` do), so the
/// output is defined for every pair of i64 inputs. Grouped with `i64::>`/`i64::<`/`i64::<=`
/// in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's,
/// made by reading the implementation.
///
/// **Expand-time ground —** integer comparison: pure, deterministic, total over `i64`. Safe
/// to evaluate at macro-expansion time. Ruling relocated from `macros/eval.rs`'s expand-time
/// allow-list (arc 255 expand-T4a), from its "Integer comparison" group; the verdict is that
/// list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::bool true iff `a` is greater than or equal to `b`
/// @example (:wat::i64::>= 2 2) #=> true
#[wat_intrinsic(":wat::i64::>=")]
pub(crate) fn eval_i64_gte(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::>=";
    crate::runtime::eval_compare(OP, &[a.clone(), b.clone()], span, env, sym, |o| {
        o != std::cmp::Ordering::Less
    })
}

/// `(:wat::i64::= a b)` → whether `a` equals `b`.
///
/// **Totality ground —** an equality compare over i64 never raises, same class as the
/// ordering family above (per-type-equality-restored, 2026-08-05). Grouped with
/// `i64::not=` in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is
/// that list's, made by reading the implementation.
///
/// **Expand-time ground —** comparison over two same-category scalars; gap closure, not a
/// rename (builder ruling 2026-08-26: "if we're missing logical stuff, we add it - we are
/// cleaning up months of hacking"). Pure, and at least as total as `i64::/` (whose
/// division-by-zero is blessed above as a deterministic located abort). Ruling relocated from
/// `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a); the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::bool true iff `a` equals `b`
/// @example (:wat::i64::= 2 2) #=> true
#[wat_intrinsic(":wat::i64::=")]
pub(crate) fn eval_i64_eq(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::=";
    crate::runtime::eval_compare(OP, &[a.clone(), b.clone()], span, env, sym, |o| {
        o == std::cmp::Ordering::Equal
    })
}

/// `(:wat::i64::not= a b)` → whether `a` does not equal `b`.
///
/// **Totality ground —** an equality compare over i64 never raises, same class as the
/// ordering family above (per-type-equality-restored, 2026-08-05). Grouped with `i64::=`
/// in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's,
/// made by reading the implementation.
///
/// **Expand-time ground —** comparison over two same-category scalars; gap closure, not a
/// rename (builder ruling 2026-08-26: "if we're missing logical stuff, we add it - we are
/// cleaning up months of hacking"). Pure, and at least as total as `i64::/` (whose
/// division-by-zero is blessed above as a deterministic located abort). Ruling relocated from
/// `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a); the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::bool true iff `a` does not equal `b`
/// @example (:wat::i64::not= 2 3) #=> true
#[wat_intrinsic(":wat::i64::not=")]
pub(crate) fn eval_i64_not_eq(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::not=";
    crate::runtime::eval_compare(OP, &[a.clone(), b.clone()], span, env, sym, |o| {
        o != std::cmp::Ordering::Equal
    })
}

// ─── unary conversions: to-bigint to-f64 to-rational to-string ────────────
//
// `crate::runtime::eval_i64_to_{bigint,f64,rational,string}` already take
// `args: &[WatAST]`; `std::slice::from_ref(n)` hands them a length-1 view of
// our single `&WatAST` param with no clone at all.

/// `(:wat::i64::to-bigint n)` → `n` promoted to arbitrary-precision
/// `:wat::core::bigint`. Infallible (bigint always holds i64's full range).
///
/// **Expand-time ground —** `i64 -> BigInt`, a widening; no domain failure. Gap closure, not
/// a rename (builder ruling 2026-08-26: "if we're missing logical stuff, we add it - we are
/// cleaning up months of hacking"). Ruling relocated from `macros/eval.rs`'s expand-time
/// allow-list (arc 255 expand-T4a); the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     n :wat::core::i64 the i64 to promote
/// @ret     :wat::core::bigint `n`, promoted to bigint
/// @example (:wat::i64::to-bigint 5) #=> (:wat::i64::to-bigint 5)
#[wat_intrinsic(":wat::i64::to-bigint")]
pub(crate) fn eval_i64_to_bigint(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::numeric::convert::eval_i64_to_bigint(std::slice::from_ref(n), span, env, sym, ":wat::i64::to-bigint")
}

/// `(:wat::i64::to-f64 n)` → `n` cast to `:wat::core::f64`. Lossy beyond
/// f64's 53-bit mantissa; never fails (no NaN/Inf can result from a finite
/// i64).
///
/// **Totality ground —** a total, lossy-but-never-raising conversion: `i64::MAX` ≈
/// 9.2e18 is nowhere near f64's overflow boundary ≈1.8e308, so the result is always
/// finite, never ±Inf. From `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the
/// verdict is that list's, made by reading the implementation.
///
/// **Expand-time ground —** scalar conversions: pure, deterministic value-to-value
/// conversion. Safe to evaluate while a `defmacro` body is being expanded. Ruling relocated
/// from `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a), from its "Scalar
/// conversions" group (whose header note explains the list also carries this verb's
/// dual-spelled sibling, so either surface spelling is legal in a macro body); the verdict is
/// that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     n :wat::core::i64 the i64 to cast
/// @ret     :wat::core::f64 `n`, cast to f64
/// @example (:wat::i64::to-f64 5) #=> 5.0
#[wat_intrinsic(":wat::i64::to-f64")]
pub(crate) fn eval_i64_to_f64(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::numeric::convert::eval_i64_to_f64(std::slice::from_ref(n), span, env, sym, ":wat::i64::to-f64")
}

/// `(:wat::i64::to-rational n)` → `n` promoted to `:wat::core::rational`.
/// Infallible.
///
/// **Expand-time ground —** `i64 -> Rational` (`n/1`); no domain failure. Gap closure, not a
/// rename (builder ruling 2026-08-26: "if we're missing logical stuff, we add it - we are
/// cleaning up months of hacking"). Ruling relocated from `macros/eval.rs`'s expand-time
/// allow-list (arc 255 expand-T4a); the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     n :wat::core::i64 the i64 to promote
/// @ret     :wat::core::rational `n`, promoted to rational
/// @example (:wat::i64::to-rational 5) #=> (:wat::i64::to-rational 5)
#[wat_intrinsic(":wat::i64::to-rational")]
pub(crate) fn eval_i64_to_rational(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::numeric::convert::eval_i64_to_rational(std::slice::from_ref(n), span, env, sym, ":wat::i64::to-rational")
}

// ─── bitwise: bit-and bit-or bit-xor bit-not, and the three shifts ─────────
//
// Arc the-little-wat — the-little-wat's F-134 measured the absence of these at 27x on byte-field
// extraction and named it the gate on packet-shaped work. They are Clojure's seven, with
// Clojure's semantics, and Clojure already draws the line this module needs: `(+ Long/MAX_VALUE 1)`
// THROWS while `(bit-shift-left 1 63)` returns a bit pattern and does not. Arc 300 Stone C3 copied
// the first half of that design on purpose; this is the second half.
//
// **These are the first TOTAL i64 binary ops.** `+ - * / mod rem quot` can all raise; a bit
// operation cannot -- shifting a bit off the top is not overflow, it is what a shift is. So the
// closures below always return `Ok`, and `I64ArithErr` is never constructed.
//
// Shift counts are masked to the low six bits, which is what the JVM does (`(bit-shift-left 1 64)`
// is `1`, not `0`) and, conveniently, also what x86-64 does for a 64-bit operand -- so a compiler
// emitting a bare `shl`/`sar`/`shr` matches Clojure without any extra work.

/// `(:wat::i64::bit-and a b)` → the bitwise AND of `a` and `b` (clj's `bit-and`).
/// Total: no overflow, no error.
///
/// **Expand-time ground —** bitwise: pure, total, no error channel at all. Strictly safer
/// than the arithmetic group already on the expand-time allow-list (arc 255 expand-T4a).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::i64 the bitwise AND
/// @example (:wat::i64::bit-and -1 255) #=> 255
#[wat_intrinsic(":wat::i64::bit-and", value = eval_i64_bit_and_value)]
pub(crate) fn eval_i64_bit_and(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::bit-and";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _b_span| Ok(x & y))
}

fn eval_i64_bit_and_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::bit-and", vals, span, |a, b| Ok(a & b))
}

/// `(:wat::i64::bit-or a b)` → the bitwise OR of `a` and `b` (clj's `bit-or`). Total: no overflow, no error.
///
/// **Expand-time ground —** bitwise: pure, total, no error channel at all. Strictly safer
/// than the arithmetic group already on the expand-time allow-list (arc 255 expand-T4a).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::i64 the bitwise OR of `a` and `b`
/// @example (:wat::i64::bit-or -256 15) #=> -241
#[wat_intrinsic(":wat::i64::bit-or", value = eval_i64_bit_or_value)]
pub(crate) fn eval_i64_bit_or(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::bit-or";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _b_span| Ok(x | y))
}

fn eval_i64_bit_or_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::bit-or", vals, span, |a, b| Ok(a | b))
}

/// `(:wat::i64::bit-xor a b)` → the bitwise XOR of `a` and `b` (clj's `bit-xor`). Total: no overflow, no error.
///
/// **Expand-time ground —** bitwise: pure, total, no error channel at all. Strictly safer
/// than the arithmetic group already on the expand-time allow-list (arc 255 expand-T4a).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::i64 the bitwise XOR of `a` and `b`
/// @example (:wat::i64::bit-xor -1 -1) #=> 0
#[wat_intrinsic(":wat::i64::bit-xor", value = eval_i64_bit_xor_value)]
pub(crate) fn eval_i64_bit_xor(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::bit-xor";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _b_span| Ok(x ^ y))
}

fn eval_i64_bit_xor_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::bit-xor", vals, span, |a, b| Ok(a ^ b))
}

/// `(:wat::i64::bit-shift-left a b)` → `a` shifted left by `b` bits, the count masked to six bits (clj's `bit-shift-left`). Total: no overflow, no error.
///
/// **Expand-time ground —** bitwise: pure, total, no error channel at all. Strictly safer
/// than the arithmetic group already on the expand-time allow-list (arc 255 expand-T4a).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::i64 `a` shifted left by `b` bits, the count masked to six bits
/// @example (:wat::i64::bit-shift-left 1 63) #=> -9223372036854775808
#[wat_intrinsic(":wat::i64::bit-shift-left", value = eval_i64_bit_shift_left_value)]
pub(crate) fn eval_i64_bit_shift_left(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::bit-shift-left";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _b_span| Ok(x.wrapping_shl(y as u32)))
}

fn eval_i64_bit_shift_left_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::bit-shift-left", vals, span, |a, b| Ok(a.wrapping_shl(b as u32)))
}

/// `(:wat::i64::bit-shift-right a b)` → `a` shifted right by `b` bits, sign-extending, the count masked to six bits (clj's `bit-shift-right`). Total: no overflow, no error.
///
/// **Expand-time ground —** bitwise: pure, total, no error channel at all. Strictly safer
/// than the arithmetic group already on the expand-time allow-list (arc 255 expand-T4a).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::i64 `a` shifted right by `b` bits, sign-extending, the count masked to six bits
/// @example (:wat::i64::bit-shift-right -7 1) #=> -4
#[wat_intrinsic(":wat::i64::bit-shift-right", value = eval_i64_bit_shift_right_value)]
pub(crate) fn eval_i64_bit_shift_right(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::bit-shift-right";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _b_span| Ok(x.wrapping_shr(y as u32)))
}

fn eval_i64_bit_shift_right_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::bit-shift-right", vals, span, |a, b| Ok(a.wrapping_shr(b as u32)))
}

/// `(:wat::i64::unsigned-bit-shift-right a b)` → `a` shifted right by `b` bits, zero-filling, the count masked to six bits (clj's `unsigned-bit-shift-right`). Total: no overflow, no error.
///
/// **Expand-time ground —** bitwise: pure, total, no error channel at all. Strictly safer
/// than the arithmetic group already on the expand-time allow-list (arc 255 expand-T4a).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the left operand
/// @arg     b :wat::core::i64 the right operand
/// @ret     :wat::core::i64 `a` shifted right by `b` bits, zero-filling, the count masked to six bits
/// @example (:wat::i64::unsigned-bit-shift-right -1 60) #=> 15
#[wat_intrinsic(":wat::i64::unsigned-bit-shift-right", value = eval_i64_unsigned_bit_shift_right_value)]
pub(crate) fn eval_i64_unsigned_bit_shift_right(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::unsigned-bit-shift-right";
    crate::numeric::arith::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, |x, y, _b_span| Ok(((x as u64).wrapping_shr(y as u32)) as i64))
}

fn eval_i64_unsigned_bit_shift_right_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::numeric::arith::arith_i64_i64_inner(":wat::i64::unsigned-bit-shift-right", vals, span, |a, b| Ok(((a as u64).wrapping_shr(b as u32)) as i64))
}

/// `(:wat::i64::bit-not a)` → the bitwise complement of `a` (clj's `bit-not`).
/// Total: no overflow, no error.
///
/// **Expand-time ground —** bitwise: pure, total, no error channel at all. Strictly safer
/// than the arithmetic group already on the expand-time allow-list (arc 255 expand-T4a).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Arithmetic
/// @arg     a :wat::core::i64 the operand
/// @ret     :wat::core::i64 the bitwise complement
/// @example (:wat::i64::bit-not 0) #=> -1
#[wat_intrinsic(":wat::i64::bit-not")]
pub(crate) fn eval_i64_bit_not(
    a: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::i64::bit-not";
    // One operand, evaluated ONCE -- see `eval_i64_unary`. No value-level twin, for the same
    // reason `:wat::i64::to-string` has none: the value door's i64 helper is two-operand.
    crate::numeric::arith::eval_i64_unary(OP, std::slice::from_ref(a), span, env, sym, |x| !x)
}

/// `(:wat::i64::to-string n)` → the base-10 rendering of `n`.
///
/// **Totality ground —** verified by reading `eval_i64_to_string` (`n.to_string()`): a
/// well-typed i64 converts to a String with no domain restriction whatsoever, same
/// reasoning as `i64::to-f64` above. One of the `i64::to-string`/`f64::to-string`/
/// `bool::to-string` trio in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the
/// verdict is that list's, made by reading the implementation.
///
/// **Expand-time ground —** scalar conversions: pure, deterministic value-to-value
/// conversion. Safe to evaluate while a `defmacro` body is being expanded. Ruling relocated
/// from `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a), from its "Scalar
/// conversions" group (whose header note explains the list also carries this verb's
/// dual-spelled sibling, so either surface spelling is legal in a macro body); the verdict is
/// that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     n :wat::core::i64 the i64 to render
/// @ret     :wat::core::String the base-10 rendering of `n`
/// @example (:wat::i64::to-string 42) #=> "42"
#[wat_intrinsic(":wat::i64::to-string")]
pub(crate) fn eval_i64_to_string(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::numeric::convert::eval_i64_to_string(std::slice::from_ref(n), span, env, sym, ":wat::i64::to-string")
}

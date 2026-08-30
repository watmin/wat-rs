//! `:wat::f64::*` intrinsics — arc 255 Stone A-ii, the f64 home.
//!
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md`.
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-ii-the-f64-home.md`.
//!
//! The 19 f64 ops (`+ - * / < <= > >= = not= max min abs round clamp to-i64
//! to-string max-of min-of`), registered under their new top-level home
//! `:wat::f64::*` — mirrors Stone A-i's i64 home (`src/intrinsic/i64.rs`)
//! exactly: same self-contained shape (no separate namespace-home split),
//! same "share the implementation, duplicate only the trivial predicate"
//! rule.
//!
//! **Self-contained, no separate namespace-home file** — the f64 arithmetic
//! and conversions ALREADY live in `runtime.rs` (`eval_f64_arith`,
//! `eval_f64_compare`, `eval_f64_unary`, `eval_f64_clamp`, `eval_f64_round`,
//! `eval_f64_to_i64`, `eval_f64_to_string`) — nothing here duplicates them.
//! Every handler below is a thin `#[wat_intrinsic]` shim that calls straight
//! into those (now `pub(crate)`) fns.
//!
//! ## Arc 255 Stone C — the old spelling is retired
//!
//! Through Stone B, `:wat::core::f64::*` kept working exactly as before —
//! this module ADDED a second address for the same 19 behaviors, deleting
//! nothing; its dispatch arm in `runtime.rs` (`dispatch_keyword_head_value`'s
//! giant match) and the separate substrate-addressed table
//! (`dispatch_substrate_impl` / `arith_f64_f64_inner`) carried the OLD arm's
//! six inline arithmetic closures (`+ - * / max min`) factored into six named
//! `f64_*_op` fns, so both spellings shared ONE implementation of each op.
//! Stone C deletes the OLD half: those `runtime.rs` arms are gone,
//! `dispatch_substrate_impl`'s match keys are the new spelling directly, and
//! `:wat::core::f64::*` is now a `RETIREMENT_TABLE` hit
//! (`src/remedy/retirement.rs`) — a check-time error naming this module's
//! spelling as the remedy. The shared `f64_*_op` fns this module's handlers
//! call are untouched; only their OLD callers in `runtime.rs` are gone.
//!
//! `:wat::core::+` — the polymorphic generic — is untouched; only the
//! per-type spelling moved.
//!
//! ## The shape i64 did not have to solve
//!
//! **`max-of` / `min-of` are VARIADIC here, unlike the retired
//! `:wat::core::f64::*` spelling's single-`Vector`-argument shape** (the OLD
//! `:wat::core::f64::max-of` took exactly ONE `(Vector :- [f64])` and reduced
//! its elements via `eval_f64_reduce`, `runtime.rs`). The brief's live
//! examples for the variadic shape (`intrinsic/list.rs`'s `:wat::core::List`,
//! `intrinsic/string.rs`'s `:wat::string::concat`) both take bare args
//! directly, not a pre-constructed collection — so `:wat::f64::max-of`
//! follows THAT convention: `(:wat::f64::max-of 1.0 2.0 3.0)`, no `Vector`
//! wrapper. The two spellings had genuinely different calling conventions,
//! not just different names — but never duplicated the float contract: both
//! reduced with the literal same Rust fn value, `f64::max` / `f64::min` (see
//! `f64_variadic_reduce` below — same fn pointer, not a re-implementation of
//! "what does max mean for two f64s, including NaN"). What differed between
//! them was genuinely-different plumbing (how the elements are gathered),
//! the same class of duplication Stone A-i already accepted for
//! `eval_compare`'s `|o| o == Ordering::Less` predicates — a shape mismatch
//! in the harness, not two copies of an algorithm.
//!
//! Zero elements → `None` (never an error) — the OLD reduce's own documented
//! rationale ("max/min of an empty set are undefined") carried over verbatim
//! to the variadic form.
//!
//! `eval_f64_round` / `eval_f64_clamp` / `eval_f64_to_i64` /
//! `eval_f64_to_string` took their op name as an internal hardcoded
//! `const OP` through Stone B — harmless while both spellings resolved to the
//! same name's error text, exactly like i64's `eval_i64_to_{string,f64,
//! bigint,rational}` did. Stone C makes `op` a caller-supplied PARAMETER on
//! all eight of these fns (the four here, the four in `i64.rs`) instead: a
//! TypeMismatch/MalformedForm raised through `:wat::f64::round` now reports
//! `:op ":wat::f64::round"`, the spelling the caller actually used — the old
//! constant would otherwise have named a retired spelling that no longer
//! resolves to anything.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};

// ─── binary arithmetic: + - * / ─────────────────────────────────────────────
//
// Each handler clones its two `&WatAST` args into a 2-element array and
// forwards to `crate::runtime::eval_f64_arith` — the EXACT arity-check /
// type-check / dispatch fn the old `:wat::f64::*` arm calls — with the
// SAME named op fn (`crate::runtime::f64_add_op` etc.) the old arm now also
// calls. No arithmetic is re-implemented here.

/// `(:wat::f64::+ a b)` → the sum of `a` and `b`, strict f64 (no promotion
/// from i64). IEEE 754 throughout: never raises — a finite result overflows
/// to `±Inf`, never an error — via the SAME shared op fn `:wat::f64::+`
/// calls.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::f64 the left addend
/// @arg     b :wat::core::f64 the right addend
/// @ret     :wat::core::f64 the sum of `a` and `b`
/// @example (:wat::f64::+ 1.0 2.0) #=> 3.0
#[wat_intrinsic(":wat::f64::+", value = eval_f64_add_value)]
pub(crate) fn eval_f64_add(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::+";
    crate::runtime::eval_f64_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::f64_add_op)
}

// Arc 255 Stone N — value-level twin, for `dispatch_substrate_impl`'s
// registry-first door (`src/runtime.rs`). Same `arith_f64_f64_inner`-based
// implementation `dispatch_substrate_impl`'s own `:wat::f64::+` arm already
// used before this stone — see `i64.rs`'s `eval_i64_add_value` comment for
// why this is deliberately not merged with `eval_f64_arith` above.
fn eval_f64_add_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_f64_f64_inner(":wat::f64::+", vals, span, |a, b| Ok(a + b))
}

/// `(:wat::f64::- a b)` → `a` minus `b`, strict f64. Same shared op fn as
/// `:wat::f64::-`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::f64 the minuend
/// @arg     b :wat::core::f64 the subtrahend
/// @ret     :wat::core::f64 `a` minus `b`
/// @example (:wat::f64::- 5.0 3.0) #=> 2.0
#[wat_intrinsic(":wat::f64::-", value = eval_f64_sub_value)]
pub(crate) fn eval_f64_sub(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::-";
    crate::runtime::eval_f64_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::f64_sub_op)
}

// Arc 255 Stone N — value-level twin; see `eval_f64_add_value`'s comment above.
fn eval_f64_sub_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_f64_f64_inner(":wat::f64::-", vals, span, |a, b| Ok(a - b))
}

/// `(:wat::f64::* a b)` → `a` times `b`, strict f64. Same shared op fn as
/// `:wat::f64::*`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::f64 the first factor
/// @arg     b :wat::core::f64 the second factor
/// @ret     :wat::core::f64 `a` times `b`
/// @example (:wat::f64::* 3.0 4.0) #=> 12.0
#[wat_intrinsic(":wat::f64::*", value = eval_f64_mul_value)]
pub(crate) fn eval_f64_mul(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::*";
    crate::runtime::eval_f64_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::f64_mul_op)
}

// Arc 255 Stone N — value-level twin; see `eval_f64_add_value`'s comment above.
fn eval_f64_mul_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_f64_f64_inner(":wat::f64::*", vals, span, |a, b| Ok(a * b))
}

/// `(:wat::f64::/ a b)` → `a` divided by `b`. IEEE 754 division: `b = 0.0`
/// produces `±Inf` or `NaN`, never a runtime error (only `:wat::i64::/`
/// raises on division by zero). Same shared op fn as `:wat::f64::/`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::f64 the dividend
/// @arg     b :wat::core::f64 the divisor
/// @ret     :wat::core::f64 `a` divided by `b`
/// @example (:wat::f64::/ 6.0 2.0) #=> 3.0
#[wat_intrinsic(":wat::f64::/", value = eval_f64_div_value)]
pub(crate) fn eval_f64_div(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::/";
    crate::runtime::eval_f64_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::f64_div_op)
}

// Arc 255 Stone N — value-level twin; see `eval_f64_add_value`'s comment above.
fn eval_f64_div_value(vals: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::arith_f64_f64_inner(":wat::f64::/", vals, span, |a, b| Ok(a / b))
}

// ─── max / min (binary) ─────────────────────────────────────────────────────
//
// NOT to be confused with the variadic `max-of` / `min-of` below — these two
// take exactly two f64 args, same as `+ - * /`, via the same `eval_f64_arith`
// engine.

/// `(:wat::f64::max a b)` → the larger of `a` and `b` (`f64::max`, so a NaN
/// operand loses to a non-NaN one — IEEE 754 `maxNum`). Same shared op fn as
/// `:wat::f64::max`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::f64 the larger of `a` and `b`
/// @example (:wat::f64::max 1.0 2.0) #=> 2.0
#[wat_intrinsic(":wat::f64::max")]
pub(crate) fn eval_f64_max(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::max";
    crate::runtime::eval_f64_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::f64_max_op)
}

/// `(:wat::f64::min a b)` → the smaller of `a` and `b` (`f64::min`, IEEE 754
/// `minNum`). Same shared op fn as `:wat::f64::min`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::f64 the smaller of `a` and `b`
/// @example (:wat::f64::min 1.0 2.0) #=> 1.0
#[wat_intrinsic(":wat::f64::min")]
pub(crate) fn eval_f64_min(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::min";
    crate::runtime::eval_f64_arith(OP, &[a.clone(), b.clone()], span, env, sym, crate::runtime::f64_min_op)
}

// ─── comparisons: < <= > >= = not= ─────────────────────────────────────────
//
// `crate::runtime::eval_f64_compare` is the SAME engine
// `:wat::f64::{<,<=,>,>=,=,not=}` calls (already NaN-correct — IEEE 754
// falls out of a bare `a < b` / `a == b` with no special-casing). Each
// predicate closure below is trivial (not an algorithm) and is duplicated
// the same way `src/intrinsic/i64.rs` duplicates `eval_compare`'s Ordering
// predicates — there is no algorithm here to share beyond the engine itself.

/// `(:wat::f64::< a b)` → whether `a` is less than `b` (NaN-correct: any
/// comparison against NaN is `false`).
///
/// **Totality ground —** a comparison, not an arithmetic op: `eval_f64_compare` returns a
/// `bool` for any two f64 inputs including NaN/±Inf (IEEE says `NaN > x` is `false`, never
/// a raise), so the output itself can never be the undefined thing this axis polices
/// (BRIEF-the-f64-surface-is-a-stub.md Part A, added beside `f64::>`). Grouped with
/// `f64::>`/`f64::<=`/`f64::>=` in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a);
/// the verdict is that list's, made by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::bool true iff `a` is less than `b`
/// @example (:wat::f64::< 1.0 2.0) #=> true
#[wat_intrinsic(":wat::f64::<")]
pub(crate) fn eval_f64_lt(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::<";
    crate::runtime::eval_f64_compare(OP, &[a.clone(), b.clone()], span, env, sym, |a, b| a < b)
}

/// `(:wat::f64::<= a b)` → whether `a` is less than or equal to `b`.
///
/// **Totality ground —** a comparison whose output is a bool, never itself the undefined
/// value; `eval_f64_compare` is NaN-correct (`NaN > 1.0` is `false`, not a raise) — there is
/// no input on which it fails to produce an ordinary bool (BRIEF-the-f64-surface-is-a-
/// stub.md Part A). Grouped with `f64::>`/`f64::<`/`f64::>=` in `rete/purity.rs`'s `total`
/// sub-list (arc 255 total-T4a); the verdict is that list's, made by reading the
/// implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::bool true iff `a` is less than or equal to `b`
/// @example (:wat::f64::<= 2.0 2.0) #=> true
#[wat_intrinsic(":wat::f64::<=")]
pub(crate) fn eval_f64_lte(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::<=";
    crate::runtime::eval_f64_compare(OP, &[a.clone(), b.clone()], span, env, sym, |a, b| a <= b)
}

/// `(:wat::f64::> a b)` → whether `a` is greater than `b`.
///
/// **Totality ground —** a comparison, not an arithmetic op: `eval_f64_compare` returns a
/// `bool` for any two f64 inputs including NaN/±Inf (IEEE says `NaN > x` is `false`, never
/// a raise), so the output itself can never be the undefined thing this axis polices — same
/// shape as the `coincident?`/`presence?` predicates. Grouped with `f64::<`/`f64::<=`/
/// `f64::>=` in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that
/// list's, made by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::bool true iff `a` is greater than `b`
/// @example (:wat::f64::> 3.0 2.0) #=> true
#[wat_intrinsic(":wat::f64::>")]
pub(crate) fn eval_f64_gt(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::>";
    crate::runtime::eval_f64_compare(OP, &[a.clone(), b.clone()], span, env, sym, |a, b| a > b)
}

/// `(:wat::f64::>= a b)` → whether `a` is greater than or equal to `b`.
///
/// **Totality ground —** a comparison whose output is a bool, never itself the undefined
/// value; `eval_f64_compare` is NaN-correct (`NaN > 1.0` is `false`, not a raise) — there is
/// no input on which it fails to produce an ordinary bool (BRIEF-the-f64-surface-is-a-
/// stub.md Part A). Grouped with `f64::>`/`f64::<`/`f64::<=` in `rete/purity.rs`'s `total`
/// sub-list (arc 255 total-T4a); the verdict is that list's, made by reading the
/// implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::bool true iff `a` is greater than or equal to `b`
/// @example (:wat::f64::>= 2.0 2.0) #=> true
#[wat_intrinsic(":wat::f64::>=")]
pub(crate) fn eval_f64_gte(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::>=";
    crate::runtime::eval_f64_compare(OP, &[a.clone(), b.clone()], span, env, sym, |a, b| a >= b)
}

/// `(:wat::f64::= a b)` → whether `a` equals `b`. IEEE 754 equality: `NaN =
/// NaN` is `false`, falls out for free — not special-cased.
///
/// **Totality ground —** a comparison, not arithmetic — `eval_f64_compare` returns a `bool`
/// for any two f64 inputs including NaN/±Inf (never raises), the same reasoning `f64::>`
/// uses (per-type-equality-restored, 2026-08-05). Grouped with `f64::not=` in
/// `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's, made
/// by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::bool true iff `a` equals `b`
/// @example (:wat::f64::= 2.0 2.0) #=> true
#[wat_intrinsic(":wat::f64::=")]
pub(crate) fn eval_f64_eq(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::=";
    crate::runtime::eval_f64_compare(OP, &[a.clone(), b.clone()], span, env, sym, |a, b| a == b)
}

/// `(:wat::f64::not= a b)` → whether `a` does not equal `b`.
///
/// **Totality ground —** a comparison, not arithmetic — `eval_f64_compare` returns a `bool`
/// for any two f64 inputs including NaN/±Inf (never raises), the same reasoning `f64::>`
/// uses (per-type-equality-restored, 2026-08-05). Grouped with `f64::=` in
/// `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's, made
/// by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     a :wat::core::f64 the left operand
/// @arg     b :wat::core::f64 the right operand
/// @ret     :wat::core::bool true iff `a` does not equal `b`
/// @example (:wat::f64::not= 2.0 3.0) #=> true
#[wat_intrinsic(":wat::f64::not=")]
pub(crate) fn eval_f64_not_eq(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::not=";
    crate::runtime::eval_f64_compare(OP, &[a.clone(), b.clone()], span, env, sym, |a, b| a != b)
}

// ─── unary: abs round to-i64 to-string ─────────────────────────────────────

/// `(:wat::f64::abs n)` → the absolute value of `n`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     n :wat::core::f64 the f64 to take the absolute value of
/// @ret     :wat::core::f64 the absolute value of `n`
/// @example (:wat::f64::abs -3.5) #=> 3.5
#[wat_intrinsic(":wat::f64::abs")]
pub(crate) fn eval_f64_abs(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_f64_unary(std::slice::from_ref(n), span, env, sym, ":wat::f64::abs", f64::abs)
}

/// `(:wat::f64::round v digits)` → `v` rounded to `digits` decimal places,
/// round-half-away-from-zero. `digits` must be non-negative. Delegates to
/// the SAME `crate::runtime::eval_f64_round` as `:wat::f64::round`;
/// its `:op` in any raised error names the OLD spelling regardless of which
/// name the caller used (see this module's header) — a pre-existing
/// property of the shared fn, not new here.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     v :wat::core::f64 the value to round
/// @arg     digits :wat::core::i64 the number of decimal places, non-negative
/// @ret     :wat::core::f64 `v` rounded to `digits` decimal places
/// @example (:wat::f64::round 1.5 0) #=> 2.0
#[wat_intrinsic(":wat::f64::round")]
pub(crate) fn eval_f64_round(
    v: &WatAST,
    digits: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_f64_round(&[v.clone(), digits.clone()], span, env, sym, ":wat::f64::round")
}

/// `(:wat::f64::to-i64 n)` → `(Some n)` truncated to `:wat::core::i64` when
/// `n` is finite and in i64 range, `None` otherwise (NaN, ±Inf, or
/// out-of-range). Same shared op fn as `:wat::f64::to-i64`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     n :wat::core::f64 the f64 to truncate
/// @ret     (:wat::core::Option :- [:wat::core::i64]) `Some(n as i64)` when in range, `None` otherwise
/// @example (:wat::f64::to-i64 3.75) #=> (:wat::f64::to-i64 3.75)
#[wat_intrinsic(":wat::f64::to-i64")]
pub(crate) fn eval_f64_to_i64(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_f64_to_i64(std::slice::from_ref(n), span, env, sym, ":wat::f64::to-i64")
}

/// `(:wat::f64::to-string n)` → the rendering of `n`. Same shared op fn as
/// `:wat::f64::to-string`.
///
/// **Totality ground —** verified by reading `eval_f64_to_string` (`format!("{}", f)`,
/// defined for NaN/±Inf/-0.0 too): a well-typed f64 converts to a String with no domain
/// restriction whatsoever, same reasoning as `i64::to-f64`/`i64::to-string`. One of the
/// `i64::to-string`/`f64::to-string`/`bool::to-string` trio in `rete/purity.rs`'s `total`
/// sub-list (arc 255 total-T4a); the verdict is that list's, made by reading the
/// implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     n :wat::core::f64 the f64 to render
/// @ret     :wat::core::String the rendering of `n`
/// @example (:wat::f64::to-string 2.5) #=> "2.5"
#[wat_intrinsic(":wat::f64::to-string")]
pub(crate) fn eval_f64_to_string(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_f64_to_string(std::slice::from_ref(n), span, env, sym, ":wat::f64::to-string")
}

// ─── ternary: clamp ─────────────────────────────────────────────────────────
//
// TERNARY SHAPE CHOICE: three fixed `&WatAST` params (`Exact(3)`), not the
// slice form `eval_f64_clamp` itself takes. `clamp` has three semantically
// distinct, named positions (v/lo/hi) — exactly the shape `@arg` triples
// already document for every OTHER op in this file — so `Exact(3)` keeps the
// delegation honest with the reflection surface (named args, not an
// anonymous slice) and gets its arity check for free from the macro, same as
// every binary op above. The slice form remains `eval_f64_clamp`'s own
// signature; this handler just assembles the 3-element array it expects,
// same pattern the binary ops above use for `eval_f64_arith`'s 2-element one.

/// `(:wat::f64::clamp v lo hi)` → `v` bounded into `[lo, hi]`. `lo > hi` or
/// either NaN raises `MalformedForm`. Same shared op fn as
/// `:wat::f64::clamp` (see this module's header re: its `:op`
/// attribution).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     v :wat::core::f64 the value to bound
/// @arg     lo :wat::core::f64 the lower bound
/// @arg     hi :wat::core::f64 the upper bound
/// @ret     :wat::core::f64 `v`, bounded into `[lo, hi]`
/// @example (:wat::f64::clamp 5.0 -1.0 1.0) #=> 1.0
#[wat_intrinsic(":wat::f64::clamp")]
pub(crate) fn eval_f64_clamp(
    v: &WatAST,
    lo: &WatAST,
    hi: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_f64_clamp(&[v.clone(), lo.clone(), hi.clone()], span, env, sym, ":wat::f64::clamp")
}

// ─── variadic: max-of min-of ────────────────────────────────────────────────
//
// See this module's header for why these are variadic here (bare args) while
// their `:wat::f64::*` twins take a single `Vector` — and why that is
// a calling-convention difference, not a duplicated float contract: both
// reduce with the literal same `f64::max` / `f64::min` fn pointer.

/// Shared reduction core for `max-of` / `min-of`'s variadic form. Evaluates
/// each arg, requires f64, and folds with `fold` — the exact same fn pointer
/// (`f64::max` / `f64::min`) `eval_f64_reduce`'s dispatch call passes for the
/// `:wat::f64::*` spelling. Zero args → `None` (empty extremum is
/// undefined, never an error — `eval_f64_reduce`'s own documented contract,
/// carried over verbatim).
fn f64_variadic_reduce(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    fold: fn(f64, f64) -> f64,
) -> Result<Value, EvalBreak> {
    let mut acc: Option<f64> = None;
    for a in args {
        let a_span = a.span().clone();
        match crate::runtime::eval_inner(a, env, sym)?.value_owned() {
            Value::f64(x) => acc = Some(match acc {
                Some(cur) => fold(cur, x),
                None => x,
            }),
            other => {
                return Err(RuntimeError::new(
                    a_span,
                    RuntimeErrorKind::TypeMismatch {
                        op: op.into(),
                        expected: "f64",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        }
    }
    Ok(Value::Option(std::sync::Arc::new(acc.map(Value::f64))))
}

/// `(:wat::f64::max-of v1 v2 ...)` → `(Some max)` of the given f64s, or
/// `None` for zero args. Variadic (bare args — contrast
/// `:wat::f64::max-of`, which takes ONE `(Vector :- [f64])`; see this
/// module's header).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     args… :wat::core::f64 the values to reduce
/// @ret     (:wat::core::Option :- [:wat::core::f64]) the maximum, or `None` if no args given
/// @example (:wat::f64::max-of 1.0 2.0 3.0) #=> (:wat::f64::max-of 1.0 2.0 3.0)
#[wat_intrinsic(":wat::f64::max-of")]
pub(crate) fn eval_f64_max_of(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — no own error path; every error is per-element, carrying that element's own span (see f64_variadic_reduce)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::max-of";
    f64_variadic_reduce(OP, args, env, sym, f64::max)
}

/// `(:wat::f64::min-of v1 v2 ...)` → `(Some min)` of the given f64s, or
/// `None` for zero args. Variadic (bare args — contrast
/// `:wat::f64::min-of`, which takes ONE `(Vector :- [f64])`; see this
/// module's header).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
/// @arg     args… :wat::core::f64 the values to reduce
/// @ret     (:wat::core::Option :- [:wat::core::f64]) the minimum, or `None` if no args given
/// @example (:wat::f64::min-of 1.0 2.0 3.0) #=> (:wat::f64::min-of 1.0 2.0 3.0)
#[wat_intrinsic(":wat::f64::min-of")]
pub(crate) fn eval_f64_min_of(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — no own error path; every error is per-element, carrying that element's own span (see f64_variadic_reduce)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::f64::min-of";
    f64_variadic_reduce(OP, args, env, sym, f64::min)
}

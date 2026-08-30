//! `:wat::seq::*` intrinsics — arc 255 Stone HOME-10, the seq home.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-HOME-10-math-stat-seq-get-actual-homes.md`.
//!
//! The 3 seq ops (`zip window remove-at`), registered under their already-final
//! home `:wat::seq::*` (HOME-9 renamed them off the dead `:wat::std::list::`
//! namespace and made them Seqable-generic in the same motion; this stone only
//! moves the dispatch arm into a `#[wat_intrinsic]` handler — nothing is
//! renamed, no semantics changed, no corpus file is touched).
//!
//! **Self-contained, no separate namespace-home file.** Measured (the drawing
//! commit, `93c5aef52`): each of these is a thin shim over
//! `require_seqable_vec` — squarely in the shim-only band, not the two-layer
//! case. Every handler below forwards to
//! `crate::collection::transform::eval_seq_{zip,window,remove_at}` — the SAME
//! functions the old `:wat::seq::*` dispatch arms called, passing its own
//! spelling through as `op` so an error names whichever spelling the caller
//! actually used.
//!
//! `zip`/`window`/`remove-at` accept the full `Seqable :- [T]` surface
//! (`Vector`, `PersistentVector`, `List`, `Stream`) for their collection
//! argument(s), not just `Value::Vec` — the same widening `check.rs`'s custom
//! `infer_zip`/`infer_window`/`infer_remove_at` arms already declare (no static
//! `TypeScheme` is registered for these three; a Seqable-generic input with a
//! container-shaped return cannot be expressed by one). `@arg`'s
//! `(:wat::core::Seqable :- [T])` spelling mirrors `:wat::string::join`'s
//! `pieces` parameter (`src/intrinsic/string.rs`), the existing precedent for a
//! registered Seqable-generic argument.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::seq::zip xs ys)` → `Vector<Tuple<T,U>>`. Short-circuits at the
/// shorter input's length (matches Rust's `xs.iter().zip(ys)`). `xs` and `ys`
/// are each independently any member of the `Seqable :- [T]` surface.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     xs (:wat::core::Seqable :- [T]) the left sequence
/// @arg     ys (:wat::core::Seqable :- [U]) the right sequence
/// @ret     (:wat::core::Vector :- [(:wat::core::Tuple :- [T U])]) pairs of corresponding elements, truncated to the shorter input
/// @example (:wat::seq::zip (:wat::core::Vector :- [:wat::core::i64] 1 2 3) (:wat::core::Vector :- [:wat::core::i64] 4 5 6)) #=> (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple 1 4) (:wat::core::Tuple 2 5) (:wat::core::Tuple 3 6))
#[wat_intrinsic(":wat::seq::zip")]
pub(crate) fn eval_seq_zip_intrinsic(
    xs: &WatAST,
    ys: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::seq::zip";
    crate::collection::transform::eval_seq_zip(OP, &[xs.clone(), ys.clone()], span, env, sym)
}

/// `(:wat::seq::window xs n)` → `Vector<Vector<T>>`. Sliding window of size
/// `n`; maps to Rust's `slice.windows(n)` (Clojure's `partition`). `n <= 0` or
/// `n > xs`'s length returns an empty Vector. `xs` accepts any member of the
/// `Seqable :- [T]` surface.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     xs (:wat::core::Seqable :- [T]) the sequence to window over
/// @arg     n :wat::core::i64 the window size
/// @ret     (:wat::core::Vector :- [(:wat::core::Vector :- [T])]) every contiguous window of size `n`
/// @example (:wat::seq::window (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4) 2) #=> (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::Vector :- [:wat::core::i64] 1 2) (:wat::core::Vector :- [:wat::core::i64] 2 3) (:wat::core::Vector :- [:wat::core::i64] 3 4))
#[wat_intrinsic(":wat::seq::window")]
pub(crate) fn eval_seq_window_intrinsic(
    xs: &WatAST,
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::seq::window";
    crate::collection::transform::eval_seq_window(OP, &[xs.clone(), n.clone()], span, env, sym)
}

/// `(:wat::seq::remove-at xs i)` → `Vector<T>`, a new Vector with the element
/// at index `i` removed. Out-of-range or negative `i` returns `xs` unchanged
/// (no error). NOT a duplicate of `:wat::core::remove` (drops by predicate);
/// this drops by index. `xs` accepts any member of the `Seqable :- [T]`
/// surface.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     xs (:wat::core::Seqable :- [T]) the sequence to remove from
/// @arg     i :wat::core::i64 the index to remove
/// @ret     (:wat::core::Vector :- [T]) `xs` with the element at `i` removed, or `xs` unchanged if `i` is out of range
/// @example (:wat::seq::remove-at (:wat::core::Vector :- [:wat::core::i64] 1 2 3) 1) #=> (:wat::core::Vector :- [:wat::core::i64] 1 3)
#[wat_intrinsic(":wat::seq::remove-at")]
pub(crate) fn eval_seq_remove_at_intrinsic(
    xs: &WatAST,
    i: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::seq::remove-at";
    crate::collection::transform::eval_seq_remove_at(OP, &[xs.clone(), i.clone()], span, env, sym)
}

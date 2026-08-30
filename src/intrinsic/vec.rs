//! `:wat::vec::*` intrinsics — arc 255 Stone E-ii (the vectors get their
//! homes), REGISTRY half of the two-home split `intrinsic/string.rs`
//! established.
//!
//! The 7 Rust-implemented `:wat::vec::*` verbs, off the `:wat::core::`
//! junk-drawer (`:wat::core::Vector/*`) onto their own top-level namespace —
//! **the UNMARKED name goes to `PersistentVector` (`intrinsic/vector.rs`),
//! not here**: the builder's move to a persistent-backed default is
//! "probably a week or two" out, so plain `Vector` takes the flavor-marked
//! home now and never moves again once the swap lands (`:wat::vector::`
//! already IS what the default will be called). See
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-ii-the-vectors-get-their-homes.md`.
//!
//! ⚠ Measured against the actual corpus + `collection/eval.rs`, this family
//! carries SEVEN verbs, not the six the brief names — `empty?` is real
//! (`vector_empty_q_inner`, 10 corpus call sites) and was simply omitted
//! from the brief's list. `concat · conj · contains? · empty? · extend ·
//! get · length`. `extend` exists ONLY here — the `PersistentVector`
//! (`intrinsic/vector.rs`) verb set has no `extend` twin; the two verb sets
//! are NOT symmetric.
//!
//! **Two homes** (same split as the string carve): this file is the REGISTRY
//! home — dispatch shim + `///` preamble only. The algorithms these handlers
//! call (`vector_length_inner`, `vector_get_inner`, …) live in
//! `src/collection/eval.rs`, the NAMESPACE home — already the algorithm's
//! home before this stone, untouched by it (name-only rename; handler bodies
//! do not move).
//!
//! Both the old `:wat::core::Vector/*` spelling and this new one are LIVE
//! during Phase 1/2 of this stone (register, then move the corpus by
//! codemod); Phase 3 retires the old spelling, leaving this file as the ONLY
//! dispatch path — reached via `crate::intrinsic::registry().lookup`,
//! consulted BEFORE `runtime.rs`'s literal match
//! (`DESIGN-STONE-255.1c-guard-hoist.md`).
//!
//! `concat`/`extend` are FINGERPRINT schemes here (same-kind: `Vector<T> ×
//! Vector<T> -> Vector<T>`) — `extend`'s dual shape (arg2 also accepts a
//! `PersistentVector<T>`) is handled by the custom `infer_list` arm
//! (`check.rs`, calling `collection::infer::infer_vector_extend`), which
//! always intercepts before this doc's fingerprint would be consulted.

use wat_macros::wat_intrinsic;

use crate::value::{EvalBreak, Value};

// ─── the 7 verbs ────────────────────────────────────────────────────────────
//
// arc 255 Stone O-iv-b — migrated to ALGEBRA. Each pair here was, before this stone, a
// hand-written AST shell PLUS a hand-written value twin (Stone N, named via the `value =` attribute)
// that each called the same `*_inner` fn, the value twin guarded only by `.expect
// ("arity-checked")` naming a check that happened on the OTHER door. One declaration now feeds
// both doors; the arity check is generated, and true on the door that raises it. See
// `src/intrinsic/vector.rs` (the worked example, O-iii).

/// `(:wat::vec::length v)` → the number of elements in `v`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     v (:wat::core::Vector :- [T]) the vector probed
/// @ret     :wat::core::i64 the number of elements in `v`
/// @example (:wat::vec::length (:wat::core::Vector)) #=> 0
/// @example (:wat::vec::length (:wat::core::Vector 1 2 3)) #=> 3
/// @see     :wat::vec::empty?
#[wat_intrinsic(":wat::vec::length")]
pub(crate) fn vector_length(v: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::vector_length_inner(v)
}

/// `(:wat::vec::empty? v)` → whether `v` has zero elements.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     v (:wat::core::Vector :- [T]) the vector probed
/// @ret     :wat::core::bool true iff `v` has zero elements
/// @example (:wat::vec::empty? (:wat::core::Vector)) #=> true
/// @example (:wat::vec::empty? (:wat::core::Vector 1)) #=> false
/// @see     :wat::vec::length
#[wat_intrinsic(":wat::vec::empty?")]
pub(crate) fn vector_empty_q(v: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::vector_empty_q_inner(v)
}

/// `(:wat::vec::contains? v item)` → whether `item` occurs as an element of
/// `v`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     v (:wat::core::Vector :- [T]) the vector probed
/// @arg     item :T the candidate element
/// @ret     :wat::core::bool true iff `item` occurs in `v`
/// @example (:wat::vec::contains? (:wat::core::Vector 1 2 3) 2) #=> true
/// @example (:wat::vec::contains? (:wat::core::Vector 1 2 3) 9) #=> false
/// @see     :wat::vec::get
#[wat_intrinsic(":wat::vec::contains?")]
pub(crate) fn vector_contains_q(v: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::vector_contains_q_inner(v, item)
}

/// `(:wat::vec::get v i)` → `Some` of the element at index `i` in `v`, or
/// `None` on an out-of-range index. Safe: never raises on OOB.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     v (:wat::core::Vector :- [T]) the vector probed
/// @arg     i :wat::core::i64 the index looked up
/// @ret     (:wat::core::Option :- [T]) `Some` the element at `i`, or `None` on OOB
/// @example (:wat::vec::get (:wat::core::Vector 1 2 3) 0) #=> (:wat::core::Some 1)
/// @example (:wat::vec::get (:wat::core::Vector 1 2 3) 9) #=> :None
/// @see     :wat::vec::contains?
#[wat_intrinsic(":wat::vec::get")]
pub(crate) fn vector_get(v: &Value, i: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::vector_get_inner(v, i)
}

/// `(:wat::vec::conj v item)` → a NEW `Vector` with `item` appended; the
/// original `v` is UNCHANGED.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     v (:wat::core::Vector :- [T]) the vector transformed
/// @arg     item :T the element appended
/// @ret     (:wat::core::Vector :- [T]) `v` with `item` appended
/// @example (:wat::vec::length (:wat::vec::conj (:wat::core::Vector) 1)) #=> 1
/// @see     :wat::vec::concat
#[wat_intrinsic(":wat::vec::conj")]
pub(crate) fn vector_conj(v: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::vector_conj_inner(v, item)
}

/// `(:wat::vec::concat left right)` → a NEW `Vector` holding every element
/// of `left` followed by every element of `right` (`left`/`right`
/// unchanged). Same-kind only: both sides must be `Vector` (a `Vector` ×
/// `PersistentVector` mix is a `TypeMismatch` — see `:wat::vec::extend` for
/// the widened-source sibling).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     left (:wat::core::Vector :- [T]) the left half
/// @arg     right (:wat::core::Vector :- [T]) the right half
/// @ret     (:wat::core::Vector :- [T]) `left` followed by `right`
/// @example (:wat::vec::length (:wat::vec::concat (:wat::core::Vector 1) (:wat::core::Vector 2))) #=> 2
/// @see     :wat::vec::extend
#[wat_intrinsic(":wat::vec::concat")]
pub(crate) fn vector_concat(left: &Value, right: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::vector_concat_inner(left, right)
}

/// `(:wat::vec::extend to from)` — arc 278: a `Vector` extended by every
/// element of `from`, in ONE build. `to` MUST be a `Vector`; `from` accepts
/// EITHER a `Vector` or a `PersistentVector` (the dual shape a single static
/// scheme cannot express — the check-time custom arm, not this fingerprint
/// doc, is what actually admits a `PersistentVector` `from`). Returns a NEW
/// `Vector` (`to`/`from` unchanged).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     to (:wat::core::Vector :- [T]) the receiver; its kind is preserved
/// @arg     from (:wat::core::Vector :- [T]) the elements appended (a `PersistentVector<T>` is also accepted at check time)
/// @ret     (:wat::core::Vector :- [T]) `to` with every element of `from` appended
/// @example (:wat::vec::length (:wat::vec::extend (:wat::core::Vector 1) (:wat::core::Vector 2 3))) #=> 3
/// @see     :wat::vec::concat
#[wat_intrinsic(":wat::vec::extend")]
pub(crate) fn vector_extend(to: &Value, from: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::vector_extend_inner(to, from)
}

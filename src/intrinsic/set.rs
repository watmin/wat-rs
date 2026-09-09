//! `:wat::set::*` intrinsics — persistent set over `rpds::HashTrieSetSync`.
//!
//! ★ WHY `PersistentSet` GETS THE UNMARKED `:wat::set::` NAME — recorded
//! in `src/intrinsic/map.rs` for PersistentMap and applied here: the
//! builder is moving to a persistent-backed default. Naming this family
//! `:wat::set::` NOW means it never moves again once that swap lands.
//! `:wat::hashset::` keeps the cloning `HashSet` under its marked name.
//!
//! Two homes: this file is the REGISTRY (dispatch shim + `///` preamble).
//! Algorithms live in `src/collection/eval.rs`.

use wat_macros::wat_intrinsic;

use crate::value::{EvalBreak, Value};

/// `(:wat::set::length s)` → the number of elements in `s`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     s (:wat::core::PersistentSet :- [T]) the set probed
/// @ret     :wat::core::i64 the number of elements in `s`
/// @example (:wat::set::length (:wat::core::PersistentSet :- [:i64])) #=> 0
/// @see     :wat::set::empty?
#[wat_intrinsic(":wat::set::length")]
pub(crate) fn persistentset_length(s: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentset_length_inner(s)
}

/// `(:wat::set::empty? s)` → whether `s` has zero elements.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     s (:wat::core::PersistentSet :- [T]) the set probed
/// @ret     :wat::core::bool true iff `s` has zero elements
/// @example (:wat::set::empty? (:wat::core::PersistentSet :- [:i64])) #=> true
/// @see     :wat::set::length
#[wat_intrinsic(":wat::set::empty?")]
pub(crate) fn persistentset_empty_q(s: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentset_empty_q_inner(s)
}

/// `(:wat::set::contains? s item)` → whether `item` is a member of `s`.
/// An unhashable `item` always returns `false` (it can never have been
/// inserted).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     s (:wat::core::PersistentSet :- [T]) the set probed
/// @arg     item :T the candidate element
/// @ret     :wat::core::bool true iff `item` is a member of `s`
/// @example (:wat::set::contains? (:wat::set::conj (:wat::core::PersistentSet :- [:i64]) 1) 1) #=> true
/// @see     :wat::set::conj
#[wat_intrinsic(":wat::set::contains?")]
pub(crate) fn persistentset_contains_q(s: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentset_contains_q_inner(s, item)
}

/// `(:wat::set::conj s item)` → a NEW `PersistentSet` with `item` inserted;
/// the original `s` is UNCHANGED and shares structure with the result.
/// Raises `TypeMismatch` if `item` is not a hashable value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     s (:wat::core::PersistentSet :- [T]) the set transformed
/// @arg     item :T the element inserted
/// @ret     (:wat::core::PersistentSet :- [T]) `s` with `item` inserted
/// @example (:wat::set::length (:wat::set::conj (:wat::core::PersistentSet :- [:i64]) 1)) #=> 1
/// @see     :wat::set::disj
#[wat_intrinsic(":wat::set::conj")]
pub(crate) fn persistentset_conj(s: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentset_conj_inner(s, item)
}

/// `(:wat::set::disj s item)` → a NEW `PersistentSet` with `item` removed;
/// the original `s` is UNCHANGED. An absent `item` returns `s` unchanged
/// (not an error).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     s (:wat::core::PersistentSet :- [T]) the set transformed
/// @arg     item :T the element removed
/// @ret     (:wat::core::PersistentSet :- [T]) `s` with `item` removed
/// @example (:wat::set::empty? (:wat::set::disj (:wat::set::conj (:wat::core::PersistentSet :- [:i64]) 1) 1)) #=> true
/// @see     :wat::set::conj
#[wat_intrinsic(":wat::set::disj")]
pub(crate) fn persistentset_disj(s: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentset_disj_inner(s, item)
}

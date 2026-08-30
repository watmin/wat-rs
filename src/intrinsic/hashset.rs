//! `:wat::hashset::*` intrinsics — arc 255 Stone E-iii (set + list get their
//! homes), REGISTRY half of the two-home split `intrinsic/string.rs`
//! established.
//!
//! The 4 Rust-implemented `:wat::hashset::*` verbs, off the `:wat::core::`
//! junk-drawer (`:wat::core::HashSet/*`) onto their own top-level namespace.
//!
//! ★ WHY `HashSet` TAKES THE MARKED `:wat::hashset::` NAME, NOT the unmarked
//! `:wat::set::` — measured from the backing type, not taste:
//! `Arc<HashSet<Value>>` is the **copy-on-write** flavor, the same side of
//! the axis as `HashMap` and `Vector`, not the structurally-shared
//! `rpds`-backed side `PersistentMap`/`PersistentVector` sit on. The builder
//! has ruled that a persistent-backed set is coming; `:wat::set::` must stay
//! FREE for that flavor once it lands, the same reason `:wat::map::` stayed
//! free for `PersistentMap` (`intrinsic/map.rs`) rather than being claimed by
//! `HashMap`. `hashset` names what it is — a `HashSet` — same shape as
//! `hashmap`. See
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-iii-set-and-list-get-their-homes.md`.
//!
//! **Two homes** (same split as the string carve): this file is the REGISTRY
//! home — dispatch shim + `///` preamble only. The algorithms these handlers
//! call (`hashset_length_inner`, `hashset_conj_inner`, …) live in
//! `src/collection/eval.rs`, the NAMESPACE home — already the algorithm's
//! home before this stone, untouched by it (name-only rename; handler bodies
//! do not move).
//!
//! Both the old `:wat::core::HashSet/*` spelling and this new one are LIVE
//! during Phase 1/2 of this stone (register, then move the corpus by
//! codemod); Phase 3 retires the old spelling, leaving this file as the
//! ONLY dispatch path — reached via `crate::intrinsic::registry().lookup`,
//! consulted BEFORE `runtime.rs`'s literal match
//! (`DESIGN-STONE-255.1c-guard-hoist.md`).
//!
//! ⚠ The bare TYPE constructor `:wat::core::HashSet` does NOT move (STOP-3 —
//! same rule as `:wat::core::List`, `intrinsic/list.rs`): only the 4
//! slash-verbs below are this stone's territory. `HashSet` also has no
//! direct-call `get` verb — its "get-by-equality" is `contains?` per arc 146
//! DESIGN audit table, and the generic `:wat::core::get` surface reaches
//! `hashset_get_inner` polymorphically (never through a `HashSet/get`
//! keyword) — measured against `runtime.rs`'s `dispatch_keyword_head_value`
//! and `dispatch_substrate_impl`, neither of which has such an arm.

use wat_macros::wat_intrinsic;

use crate::value::{EvalBreak, Value};

// ─── the 4 verbs ────────────────────────────────────────────────────────────
//
// arc 255 Stone O-iv-b — migrated to ALGEBRA. Each pair here was, before this stone, a
// hand-written AST shell PLUS a hand-written value twin (Stone N, named via the `value =` attribute)
// that each called the same `*_inner` fn, the value twin guarded only by `.expect
// ("arity-checked")` naming a check that happened on the OTHER door. One declaration now feeds
// both doors; the arity check is generated, and true on the door that raises it. See
// `src/intrinsic/vector.rs` (the worked example, O-iii).

/// `(:wat::hashset::length s)` → the number of elements in `s`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     s (:wat::core::HashSet :- [T]) the set probed
/// @ret     :wat::core::i64 the number of elements in `s`
/// @example (:wat::hashset::length (:wat::core::HashSet :- [:i64])) #=> 0
/// @example (:wat::hashset::length (:wat::core::HashSet :- [:i64] 1 2 3)) #=> 3
/// @see     :wat::hashset::empty?
#[wat_intrinsic(":wat::hashset::length")]
pub(crate) fn hashset_length(s: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashset_length_inner(s)
}

/// `(:wat::hashset::empty? s)` → whether `s` has zero elements.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     s (:wat::core::HashSet :- [T]) the set probed
/// @ret     :wat::core::bool true iff `s` has zero elements
/// @example (:wat::hashset::empty? (:wat::core::HashSet :- [:i64])) #=> true
/// @example (:wat::hashset::empty? (:wat::core::HashSet :- [:i64] 1)) #=> false
/// @see     :wat::hashset::length
#[wat_intrinsic(":wat::hashset::empty?")]
pub(crate) fn hashset_empty_q(s: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashset_empty_q_inner(s)
}

/// `(:wat::hashset::contains? s item)` → whether `item` is a member of `s`.
/// An unhashable `item` always returns `false` (it can never have been
/// inserted).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     s (:wat::core::HashSet :- [T]) the set probed
/// @arg     item :T the candidate element
/// @ret     :wat::core::bool true iff `item` is a member of `s`
/// @example (:wat::hashset::contains? (:wat::core::HashSet :- [:i64] 1 2 3) 2) #=> true
/// @example (:wat::hashset::contains? (:wat::core::HashSet :- [:i64] 1 2 3) 9) #=> false
/// @see     :wat::hashset::conj
#[wat_intrinsic(":wat::hashset::contains?")]
pub(crate) fn hashset_contains_q(s: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashset_contains_q_inner(s, item)
}

/// `(:wat::hashset::conj s item)` → a NEW `HashSet` with `item` inserted;
/// the original `s` is UNCHANGED. Raises `TypeMismatch` if `item` is not a
/// hashable value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s (:wat::core::HashSet :- [T]) the set transformed
/// @arg     item :T the element inserted
/// @ret     (:wat::core::HashSet :- [T]) `s` with `item` inserted
/// @example (:wat::hashset::length (:wat::hashset::conj (:wat::core::HashSet :- [:i64]) 1)) #=> 1
/// @see     :wat::hashset::contains?
#[wat_intrinsic(":wat::hashset::conj")]
pub(crate) fn hashset_conj(s: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashset_conj_inner(s, item)
}

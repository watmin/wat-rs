//! `:wat::vector::*` intrinsics — arc 255 Stone E-ii (the vectors get their
//! homes), REGISTRY half of the two-home split `intrinsic/string.rs`
//! established.
//!
//! The 6 Rust-implemented `:wat::vector::*` verbs, off the `:wat::core::`
//! junk-drawer (`:wat::core::PersistentVector/*`) onto their own top-level
//! namespace.
//!
//! ★ WHY `PersistentVector` GETS THE UNMARKED `:wat::vector::` NAME — same
//! reason Stone E-i gave `PersistentMap` the unmarked `:wat::map::` name: the
//! builder is moving to a persistent-backed default "probably a week or two"
//! out. Naming this family `:wat::vector::` NOW means it never moves again
//! once that swap lands — its name already IS what the default will be
//! called; only `:wat::vec::` (`intrinsic/vec.rs`, the plain `Vector`
//! family) moves later, once, as a prefix rename. `Both flavors survive`
//! this stone — nothing about which is the DEFAULT changes here. See
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-ii-the-vectors-get-their-homes.md`.
//!
//! ⚠ Measured against the actual corpus + `collection/eval.rs`, this family
//! carries SIX verbs, not the five the brief names — `empty?` is real
//! (`persistentvector_empty_q_inner`, 5 corpus call sites) and was simply
//! omitted from the brief's list. `concat · conj · contains? · empty? · get ·
//! length`. `extend` remains `Vector`-only (`intrinsic/vec.rs`) — the two
//! verb sets are NOT symmetric.
//!
//! **Two homes** (same split as the string carve): this file is the REGISTRY
//! home — dispatch shim + `///` preamble only. The algorithms these handlers
//! call (`persistentvector_length_inner`, `persistentvector_get_inner`, …)
//! live in `src/collection/eval.rs`, the NAMESPACE home — already the
//! algorithm's home before this stone, untouched by it (name-only rename;
//! handler bodies do not move).
//!
//! Both the old `:wat::core::PersistentVector/*` spelling and this new one
//! are LIVE during Phase 1/2 of this stone (register, then move the corpus
//! by codemod); Phase 3 retires the old spelling, leaving this file as the
//! ONLY dispatch path — reached via `crate::intrinsic::registry().lookup`,
//! consulted BEFORE `runtime.rs`'s literal match
//! (`DESIGN-STONE-255.1c-guard-hoist.md`).
//!
//! `concat` is a FINGERPRINT scheme here (same-kind: `PersistentVector<T> ×
//! PersistentVector<T> -> PersistentVector<T>`) — the dual shape (arg2 also
//! accepts a plain `Vector<T>`) is handled by the custom `infer_list` arm
//! (`check.rs`, calling `collection::infer::infer_persistentvector_concat`),
//! which always intercepts before this doc's fingerprint would be consulted
//! (`DESIGN-STONE-into-pv-from-vector.md`).

use wat_macros::wat_intrinsic;

use crate::value::{EvalBreak, Value};

// ─── the 6 verbs ────────────────────────────────────────────────────────────
//
// arc 255 Stone O-iii — migrated to ALGEBRA. Each handler's leading params are now `&Value`
// (not `&WatAST`), so `#[wat_intrinsic]` generates BOTH the AST door (the shim it always
// generated) and the value door (what `:wat::core::apply` reaches through
// `dispatch_substrate_impl`) from this ONE declaration, behind one arity check. The `env`/`sym`
// eval-the-arg step and the `_span: &Span // rune:lint(unused-span)` param both disappear —
// there is no span to justify and nothing left to hold it — because the macro now does that
// step itself, once, for both doors.

/// `(:wat::vector::length v)` → the number of elements in `v`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     v (:wat::core::PersistentVector :- [T]) the vector probed
/// @ret     :wat::core::i64 the number of elements in `v`
/// @example (:wat::vector::length (:wat::core::PersistentVector)) #=> 0
/// @example (:wat::vector::length (:wat::core::PersistentVector 1 2 3)) #=> 3
/// @see     :wat::vector::empty?
#[wat_intrinsic(":wat::vector::length")]
pub(crate) fn persistentvector_length(v: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentvector_length_inner(v)
}

/// `(:wat::vector::empty? v)` → whether `v` has zero elements.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     v (:wat::core::PersistentVector :- [T]) the vector probed
/// @ret     :wat::core::bool true iff `v` has zero elements
/// @example (:wat::vector::empty? (:wat::core::PersistentVector)) #=> true
/// @example (:wat::vector::empty? (:wat::core::PersistentVector 1)) #=> false
/// @see     :wat::vector::length
#[wat_intrinsic(":wat::vector::empty?")]
pub(crate) fn persistentvector_empty_q(v: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentvector_empty_q_inner(v)
}

/// `(:wat::vector::contains? v item)` → whether `item` occurs as an element
/// of `v`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     v (:wat::core::PersistentVector :- [T]) the vector probed
/// @arg     item :T the candidate element
/// @ret     :wat::core::bool true iff `item` occurs in `v`
/// @example (:wat::vector::contains? (:wat::core::PersistentVector 1 2 3) 2) #=> true
/// @example (:wat::vector::contains? (:wat::core::PersistentVector 1 2 3) 9) #=> false
/// @see     :wat::vector::get
#[wat_intrinsic(":wat::vector::contains?")]
pub(crate) fn persistentvector_contains_q(v: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentvector_contains_q_inner(v, item)
}

/// `(:wat::vector::get v i)` → `Some` of the element at index `i` in `v`, or
/// `None` on an out-of-range index. Safe: never raises on OOB (use
/// `(:wat::vector::contains? v i)`-style bounds logic to guard first).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     v (:wat::core::PersistentVector :- [T]) the vector probed
/// @arg     i :wat::core::i64 the index looked up
/// @ret     (:wat::core::Option :- [T]) `Some` the element at `i`, or `None` on OOB
/// @example (:wat::vector::get (:wat::core::PersistentVector 1 2 3) 0) #=> (:wat::core::Some 1)
/// @example (:wat::vector::get (:wat::core::PersistentVector 1 2 3) 9) #=> :None
/// @see     :wat::vector::contains?
#[wat_intrinsic(":wat::vector::get")]
pub(crate) fn persistentvector_get(v: &Value, i: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentvector_get_inner(v, i)
}

/// `(:wat::vector::conj v item)` → a NEW `PersistentVector` with `item`
/// appended; the original `v` is UNCHANGED.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     v (:wat::core::PersistentVector :- [T]) the vector transformed
/// @arg     item :T the element appended
/// @ret     (:wat::core::PersistentVector :- [T]) `v` with `item` appended
/// @example (:wat::vector::length (:wat::vector::conj (:wat::core::PersistentVector) 1)) #=> 1
/// @see     :wat::vector::concat
#[wat_intrinsic(":wat::vector::conj")]
pub(crate) fn persistentvector_conj(v: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentvector_conj_inner(v, item)
}

/// `(:wat::vector::concat to from)` — `DESIGN-STONE-into-pv-from-vector.md`.
/// Appends every element of `from` onto `to`, returning a NEW
/// `PersistentVector` (`to`/`from` unchanged). `to` MUST be a
/// `PersistentVector`; `from` accepts EITHER a `PersistentVector` or a plain
/// `Vector` (the dual shape a single static scheme cannot express — the
/// check-time custom arm, not this fingerprint doc, is what actually admits
/// a `Vector` `from`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     to (:wat::core::PersistentVector :- [T]) the receiver; its kind is preserved
/// @arg     from (:wat::core::PersistentVector :- [T]) the elements appended (a plain `Vector<T>` is also accepted at check time)
/// @ret     (:wat::core::PersistentVector :- [T]) `to` with every element of `from` appended
/// @example (:wat::vector::length (:wat::vector::concat (:wat::core::PersistentVector 1) (:wat::core::PersistentVector 2))) #=> 2
/// @see     :wat::vector::conj
///
/// arc 255 Stone O-iii — TWO fns became ONE. Before this stone this verb was written as a
/// hand-written AST shell (`eval_persistentvector_concat_home`) PLUS a hand-written value twin
/// (`eval_persistentvector_concat_home_value`, named via `value = <path>`) that each called
/// `persistentvector_concat_inner` and each opened with an unchecked index into `vals`, guarded
/// only by an `.expect` naming a check that happened on the OTHER door. One declaration now
/// feeds both doors; the arity check is generated, and true on the door that raises it.
#[wat_intrinsic(":wat::vector::concat")]
pub(crate) fn persistentvector_concat(to: &Value, from: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentvector_concat_inner(to, from)
}

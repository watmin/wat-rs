//! `:wat::hashmap::*` intrinsics — arc 255 Stone E-i (the maps get their
//! homes), REGISTRY half of the two-home split `intrinsic/string.rs`
//! established.
//!
//! The 8 Rust-implemented `:wat::hashmap::*` verbs, off the `:wat::core::`
//! junk-drawer (`:wat::core::HashMap/*`) onto their own top-level namespace —
//! **the UNMARKED name goes to `PersistentMap` (`intrinsic/map.rs`), not
//! here**: the builder's move to a persistent-backed default is "probably a
//! week or two" out, so `HashMap` takes the flavor-marked home now and never
//! moves again once the swap lands (`:wat::map::` already IS what the
//! default will be called). See
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-i-the-maps-get-their-homes.md`.
//!
//! **Two homes** (same split as the string carve): this file is the REGISTRY
//! home — dispatch shim + `///` preamble only. The algorithms these handlers
//! call (`hashmap_length_inner`, `hashmap_get_inner`, …) live in
//! `src/collection/eval.rs`, the NAMESPACE home — already the algorithm's
//! home before this stone, untouched by it (name-only rename; handler bodies
//! do not move).
//!
//! Both the old `:wat::core::HashMap/*` spelling and this new one are LIVE
//! during Phase 1/2 of this stone (register, then move the corpus by
//! codemod); Phase 3 retires the old spelling (`runtime.rs`'s literal match
//! arms + `check.rs`'s old-named type schemes + `rete/purity.rs`'s
//! classification + `macros/eval.rs`'s F5 gate), leaving this file as the
//! ONLY dispatch path — reached via `crate::intrinsic::registry().lookup`,
//! consulted BEFORE `runtime.rs`'s literal match
//! (`DESIGN-STONE-255.1c-guard-hoist.md`).

use wat_macros::wat_intrinsic;

use crate::value::{EvalBreak, Value};

// ─── the 8 verbs ────────────────────────────────────────────────────────────
//
// arc 255 Stone O-iv-b — migrated to ALGEBRA. Each pair here was, before this stone, a
// hand-written AST shell PLUS a hand-written value twin (Stone N, named via the `value =` attribute)
// that each called the same `*_inner` fn, the value twin guarded only by `.expect
// ("arity-checked")` naming a check that happened on the OTHER door. One declaration now feeds
// both doors; the arity check is generated, and true on the door that raises it. See
// `src/intrinsic/vector.rs` (the worked example, O-iii) and `map.rs` (this same stone, the
// new-door half).

/// `(:wat::hashmap::length m)` → the number of key/value entries in `m`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @ret     :wat::core::i64 the number of entries in `m`
/// @example (:wat::hashmap::length (:wat::core::HashMap)) #=> 0
/// @see     :wat::hashmap::empty?
#[wat_intrinsic(":wat::hashmap::length")]
pub(crate) fn hashmap_length(m: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_length_inner(m)
}

/// `(:wat::hashmap::empty? m)` → whether `m` has zero entries.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @ret     :wat::core::bool true iff `m` has zero entries
/// @example (:wat::hashmap::empty? (:wat::core::HashMap)) #=> true
/// @see     :wat::hashmap::length
#[wat_intrinsic(":wat::hashmap::empty?")]
pub(crate) fn hashmap_empty_q(m: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_empty_q_inner(m)
}

/// `(:wat::hashmap::contains-key? m k)` → whether `k` is a key in `m`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @arg     k :K the candidate key
/// @ret     :wat::core::bool true iff `k` occurs as a key in `m`
/// @example (:wat::hashmap::contains-key? (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1) "a") #=> true
/// @see     :wat::hashmap::get
#[wat_intrinsic(":wat::hashmap::contains-key?")]
pub(crate) fn hashmap_contains_key_q(m: &Value, k: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_contains_key_q_inner(m, k)
}

/// `(:wat::hashmap::get m k)` → `Some` of the value at key `k` in `m`, or
/// `None` on a miss.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @arg     k :K the key looked up
/// @ret     (:wat::core::Option :- [V]) `Some` the value at `k`, or `None` on a miss
/// @example (:wat::hashmap::get (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1) "a") #=> (:wat::core::Some 1)
/// @see     :wat::hashmap::contains-key?
#[wat_intrinsic(":wat::hashmap::get")]
pub(crate) fn hashmap_get(m: &Value, k: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_get_inner(m, k)
}

/// `(:wat::hashmap::assoc m k v)` → `m` with key `k` bound to value `v`
/// (inserted or overwritten).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     m (:wat::core::HashMap :- [K V]) the map transformed
/// @arg     k :K the key inserted or overwritten
/// @arg     v :V the value bound to `k`
/// @ret     (:wat::core::HashMap :- [K V]) `m` with `k` bound to `v`
/// @example (:wat::hashmap::length (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1)) #=> 1
/// @see     :wat::hashmap::dissoc
#[wat_intrinsic(":wat::hashmap::assoc")]
pub(crate) fn hashmap_assoc(m: &Value, k: &Value, v: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_assoc_inner(m, k, v)
}

/// `(:wat::hashmap::dissoc m k)` → `m` with key `k` removed (a no-op if `k`
/// is absent).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     m (:wat::core::HashMap :- [K V]) the map transformed
/// @arg     k :K the key removed
/// @ret     (:wat::core::HashMap :- [K V]) `m` with `k` removed
/// @example (:wat::hashmap::length (:wat::hashmap::dissoc (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1) "a")) #=> 0
/// @see     :wat::hashmap::assoc
#[wat_intrinsic(":wat::hashmap::dissoc")]
pub(crate) fn hashmap_dissoc(m: &Value, k: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_dissoc_inner(m, k)
}

/// `(:wat::hashmap::keys m)` → a `Vector` of `m`'s keys. Iteration ORDER is
/// NOT part of the contract (`Arc<std::HashMap>`'s default hasher is seeded
/// per process) — pure ∧ total, NOT deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     m (:wat::core::HashMap :- [K V]) the map projected
/// @ret     (:wat::core::Vector :- [K]) `m`'s keys, order unspecified
/// @example-norun (:wat::hashmap::length (:wat::hashmap::keys (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1))) #=> 1
/// @see     :wat::hashmap::values
#[wat_intrinsic(":wat::hashmap::keys")]
pub(crate) fn hashmap_keys(m: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_keys_inner(m)
}

/// `(:wat::hashmap::values m)` → a `Vector` of `m`'s values. Iteration ORDER
/// is NOT part of the contract, same as `:wat::hashmap::keys`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     m (:wat::core::HashMap :- [K V]) the map projected
/// @ret     (:wat::core::Vector :- [V]) `m`'s values, order unspecified
/// @example-norun (:wat::hashmap::length (:wat::hashmap::values (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1))) #=> 1
/// @see     :wat::hashmap::keys
#[wat_intrinsic(":wat::hashmap::values")]
pub(crate) fn hashmap_values(m: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::hashmap_values_inner(m)
}

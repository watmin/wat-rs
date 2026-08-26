//! `:wat::map::*` intrinsics — arc 255 Stone E-i (the maps get their homes),
//! REGISTRY half of the two-home split `intrinsic/string.rs` established.
//!
//! The 8 Rust-implemented `:wat::map::*` verbs, off the `:wat::core::`
//! junk-drawer (`:wat::core::PersistentMap/*`) onto their own top-level
//! namespace.
//!
//! ★ WHY `PersistentMap` GETS THE UNMARKED `:wat::map::` NAME — this is the
//! stone's whole point, not a style pick. The builder is moving to a
//! persistent-backed default "probably a week or two" out. Naming this
//! family `:wat::map::` NOW means it never moves again once that swap
//! lands — its name already IS what the default will be called; only
//! `:wat::hashmap::` (`intrinsic/hashmap.rs`) moves later, once, as a prefix
//! rename. `Both flavors survive` this stone — nothing about which is
//! the DEFAULT changes here. See
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-i-the-maps-get-their-homes.md`.
//!
//! **Two homes** (same split as the string carve): this file is the REGISTRY
//! home — dispatch shim + `///` preamble only. The algorithms these handlers
//! call (`persistentmap_length_inner`, `persistentmap_get_inner`, …) live in
//! `src/collection/eval.rs`, the NAMESPACE home — already the algorithm's
//! home before this stone, untouched by it (name-only rename; handler bodies
//! do not move).
//!
//! Both the old `:wat::core::PersistentMap/*` spelling and this new one are
//! LIVE during Phase 1/2 of this stone (register, then move the corpus by
//! codemod); Phase 3 retires the old spelling, leaving this file as the ONLY
//! dispatch path — reached via `crate::intrinsic::registry().lookup`,
//! consulted BEFORE `runtime.rs`'s literal match
//! (`DESIGN-STONE-255.1c-guard-hoist.md`).

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

// ─── the 8 verbs ────────────────────────────────────────────────────────────

/// `(:wat::map::length m)` → the number of key/value entries in `m`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map probed
/// @ret     :wat::core::i64 the number of entries in `m`
/// @example (:wat::map::length (:wat::core::PersistentMap)) #=> 0
/// @see     :wat::map::empty?
#[wat_intrinsic(":wat::map::length")]
pub(crate) fn eval_persistentmap_length_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — the only error (TypeMismatch) locates at `m`'s own eval, not this call's span
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_length_inner(&m)
}

/// `(:wat::map::empty? m)` → whether `m` has zero entries.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map probed
/// @ret     :wat::core::bool true iff `m` has zero entries
/// @example (:wat::map::empty? (:wat::core::PersistentMap)) #=> true
/// @see     :wat::map::length
#[wat_intrinsic(":wat::map::empty?")]
pub(crate) fn eval_persistentmap_empty_q_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_empty_q_inner(&m)
}

/// `(:wat::map::contains-key? m k)` → whether `k` is a key in `m`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map probed
/// @arg     k :K the candidate key
/// @ret     :wat::core::bool true iff `k` occurs as a key in `m`
/// @example (:wat::map::contains-key? (:wat::map::assoc (:wat::core::PersistentMap) "a" 1) "a") #=> true
/// @see     :wat::map::get
#[wat_intrinsic(":wat::map::contains-key?")]
pub(crate) fn eval_persistentmap_contains_key_q_home(
    m: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    let k = eval_inner(k, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_contains_key_q_inner(&m, &k)
}

/// `(:wat::map::get m k)` → `Some` of the value at key `k` in `m`, or `None`
/// on a miss.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map probed
/// @arg     k :K the key looked up
/// @ret     (:wat::core::Option :- [V]) `Some` the value at `k`, or `None` on a miss
/// @example (:wat::map::get (:wat::map::assoc (:wat::core::PersistentMap) "a" 1) "a") #=> (:wat::core::Some 1)
/// @see     :wat::map::contains-key?
#[wat_intrinsic(":wat::map::get")]
pub(crate) fn eval_persistentmap_get_home(
    m: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    let k = eval_inner(k, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_get_inner(&m, &k)
}

/// `(:wat::map::assoc m k v)` → `m` with key `k` bound to value `v` (inserted
/// or overwritten).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map transformed
/// @arg     k :K the key inserted or overwritten
/// @arg     v :V the value bound to `k`
/// @ret     (:wat::core::PersistentMap :- [K V]) `m` with `k` bound to `v`
/// @example (:wat::map::length (:wat::map::assoc (:wat::core::PersistentMap) "a" 1)) #=> 1
/// @see     :wat::map::dissoc
#[wat_intrinsic(":wat::map::assoc")]
pub(crate) fn eval_persistentmap_assoc_home(
    m: &WatAST,
    k: &WatAST,
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    let k = eval_inner(k, env, sym)?.value_owned();
    let v = eval_inner(v, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_assoc_inner(&m, &k, &v)
}

/// `(:wat::map::dissoc m k)` → `m` with key `k` removed (a no-op if `k` is
/// absent).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map transformed
/// @arg     k :K the key removed
/// @ret     (:wat::core::PersistentMap :- [K V]) `m` with `k` removed
/// @example (:wat::map::length (:wat::map::dissoc (:wat::map::assoc (:wat::core::PersistentMap) "a" 1) "a")) #=> 0
/// @see     :wat::map::assoc
#[wat_intrinsic(":wat::map::dissoc")]
pub(crate) fn eval_persistentmap_dissoc_home(
    m: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    let k = eval_inner(k, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_dissoc_inner(&m, &k)
}

/// `(:wat::map::keys m)` → a `Vector` of `m`'s keys. Iteration ORDER is NOT
/// part of the contract (`src/value/pmap.rs`: "the trie has no meaningful
/// order") — pure ∧ total, NOT deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Projection
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map projected
/// @ret     (:wat::core::Vector :- [K]) `m`'s keys, order unspecified
/// @example-norun (:wat::map::length (:wat::map::keys (:wat::map::assoc (:wat::core::PersistentMap) "a" 1))) #=> 1
/// @see     :wat::map::values
#[wat_intrinsic(":wat::map::keys")]
pub(crate) fn eval_persistentmap_keys_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_keys_inner(&m)
}

/// `(:wat::map::values m)` → a `Vector` of `m`'s values. Iteration ORDER is
/// NOT part of the contract, same as `:wat::map::keys`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Projection
/// @arg     m (:wat::core::PersistentMap :- [K V]) the map projected
/// @ret     (:wat::core::Vector :- [V]) `m`'s values, order unspecified
/// @example-norun (:wat::map::length (:wat::map::values (:wat::map::assoc (:wat::core::PersistentMap) "a" 1))) #=> 1
/// @see     :wat::map::keys
#[wat_intrinsic(":wat::map::values")]
pub(crate) fn eval_persistentmap_values_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_values_inner(&m)
}

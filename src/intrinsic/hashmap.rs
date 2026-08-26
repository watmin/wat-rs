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

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

// ─── the 8 verbs ────────────────────────────────────────────────────────────

/// `(:wat::hashmap::length m)` → the number of key/value entries in `m`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @ret     :wat::core::i64 the number of entries in `m`
/// @example (:wat::hashmap::length (:wat::core::HashMap)) #=> 0
/// @see     :wat::hashmap::empty?
#[wat_intrinsic(":wat::hashmap::length")]
pub(crate) fn eval_hashmap_length_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — the only error (TypeMismatch) locates at `m`'s own eval, not this call's span
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::hashmap_length_inner(&m)
}

/// `(:wat::hashmap::empty? m)` → whether `m` has zero entries.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @ret     :wat::core::bool true iff `m` has zero entries
/// @example (:wat::hashmap::empty? (:wat::core::HashMap)) #=> true
/// @see     :wat::hashmap::length
#[wat_intrinsic(":wat::hashmap::empty?")]
pub(crate) fn eval_hashmap_empty_q_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::hashmap_empty_q_inner(&m)
}

/// `(:wat::hashmap::contains-key? m k)` → whether `k` is a key in `m`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @arg     k :K the candidate key
/// @ret     :wat::core::bool true iff `k` occurs as a key in `m`
/// @example (:wat::hashmap::contains-key? (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1) "a") #=> true
/// @see     :wat::hashmap::get
#[wat_intrinsic(":wat::hashmap::contains-key?")]
pub(crate) fn eval_hashmap_contains_key_q_home(
    m: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    let k = eval_inner(k, env, sym)?.value_owned();
    crate::collection::eval::hashmap_contains_key_q_inner(&m, &k)
}

/// `(:wat::hashmap::get m k)` → `Some` of the value at key `k` in `m`, or
/// `None` on a miss.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     m (:wat::core::HashMap :- [K V]) the map probed
/// @arg     k :K the key looked up
/// @ret     (:wat::core::Option :- [V]) `Some` the value at `k`, or `None` on a miss
/// @example (:wat::hashmap::get (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1) "a") #=> (:wat::core::Some 1)
/// @see     :wat::hashmap::contains-key?
#[wat_intrinsic(":wat::hashmap::get")]
pub(crate) fn eval_hashmap_get_home(
    m: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    let k = eval_inner(k, env, sym)?.value_owned();
    crate::collection::eval::hashmap_get_inner(&m, &k)
}

/// `(:wat::hashmap::assoc m k v)` → `m` with key `k` bound to value `v`
/// (inserted or overwritten).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     m (:wat::core::HashMap :- [K V]) the map transformed
/// @arg     k :K the key inserted or overwritten
/// @arg     v :V the value bound to `k`
/// @ret     (:wat::core::HashMap :- [K V]) `m` with `k` bound to `v`
/// @example (:wat::hashmap::length (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1)) #=> 1
/// @see     :wat::hashmap::dissoc
#[wat_intrinsic(":wat::hashmap::assoc")]
pub(crate) fn eval_hashmap_assoc_home(
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
    crate::collection::eval::hashmap_assoc_inner(&m, &k, &v)
}

/// `(:wat::hashmap::dissoc m k)` → `m` with key `k` removed (a no-op if `k`
/// is absent).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     m (:wat::core::HashMap :- [K V]) the map transformed
/// @arg     k :K the key removed
/// @ret     (:wat::core::HashMap :- [K V]) `m` with `k` removed
/// @example (:wat::hashmap::length (:wat::hashmap::dissoc (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1) "a")) #=> 0
/// @see     :wat::hashmap::assoc
#[wat_intrinsic(":wat::hashmap::dissoc")]
pub(crate) fn eval_hashmap_dissoc_home(
    m: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    let k = eval_inner(k, env, sym)?.value_owned();
    crate::collection::eval::hashmap_dissoc_inner(&m, &k)
}

/// `(:wat::hashmap::keys m)` → a `Vector` of `m`'s keys. Iteration ORDER is
/// NOT part of the contract (`Arc<std::HashMap>`'s default hasher is seeded
/// per process) — pure ∧ total, NOT deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Projection
/// @arg     m (:wat::core::HashMap :- [K V]) the map projected
/// @ret     (:wat::core::Vector :- [K]) `m`'s keys, order unspecified
/// @example-norun (:wat::hashmap::length (:wat::hashmap::keys (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1))) #=> 1
/// @see     :wat::hashmap::values
#[wat_intrinsic(":wat::hashmap::keys")]
pub(crate) fn eval_hashmap_keys_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::hashmap_keys_inner(&m)
}

/// `(:wat::hashmap::values m)` → a `Vector` of `m`'s values. Iteration ORDER
/// is NOT part of the contract, same as `:wat::hashmap::keys`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Projection
/// @arg     m (:wat::core::HashMap :- [K V]) the map projected
/// @ret     (:wat::core::Vector :- [V]) `m`'s values, order unspecified
/// @example-norun (:wat::hashmap::length (:wat::hashmap::values (:wat::hashmap::assoc (:wat::core::HashMap) "a" 1))) #=> 1
/// @see     :wat::hashmap::keys
#[wat_intrinsic(":wat::hashmap::values")]
pub(crate) fn eval_hashmap_values_home(
    m: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span)
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::hashmap_values_inner(&m)
}

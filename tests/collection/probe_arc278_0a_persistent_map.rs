//! Arc 278 stone 0a — disconfirming probe: `:wat::core::PersistentMap` (RED at HEAD).
//!
//! The rete working memory's four memories are maps updated incrementally during fire. wat's std
//! `:wat::core::HashMap` is `Arc<std::HashMap>` — clone-on-write, O(n) per update (the "wasteful tree" at
//! the data-structure level). Stone 0a adds `:wat::core::PersistentMap` (rpds `HashTrieMapSync`):
//! structural sharing, O(log n) immutable updates — `assoc` returns a NEW map; the original is unchanged.
//!
//! RED at HEAD: `:wat::core::PersistentMap` is an unknown head → eval/check error → the `expect`s panic.
//! The probe COMPILES at HEAD (only public API + wat source strings); it fails at RUNTIME, on exactly the
//! gap. GREEN when stone 0a ships. Un-ignore then.
//!
//! Run: cargo test --release -p wat --test probe_arc278_0a_persistent_map -- --include-ignored

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each assertion's expr lives in a named zero-arg fn in the
// co-located `.wat` fixture, driven via `call_beside`. PersistentMap's per-type
// ops carry no registered TypeScheme (runtime-dispatched intrinsics, same class
// as `metadata-of`) — each defn annotates the documented/observed return shape.
#[test]
fn persistent_map_core_behavior() {
    // 1. ctor + length
    assert_eq!(
        call_beside(file!(), ":t::p1-ctor-length").expect("eval"),
        Value::i64(2),
        "PersistentMap ctor + length"
    );

    // 2. contains-key? hit + miss
    assert_eq!(
        call_beside(file!(), ":t::p2-contains-hit").expect("eval"),
        Value::bool(true),
        "contains-key? hit"
    );
    assert_eq!(
        call_beside(file!(), ":t::p3-contains-miss").expect("eval"),
        Value::bool(false),
        "contains-key? miss"
    );

    // 3. IMMUTABILITY / structural sharing — assoc returns a NEW map; the original is unchanged.
    assert_eq!(
        call_beside(file!(), ":t::p4-assoc-immutable-original").expect("eval"),
        Value::i64(1),
        "assoc must NOT mutate the original (structural sharing)"
    );
    assert_eq!(
        call_beside(file!(), ":t::p5-assoc-extended").expect("eval"),
        Value::i64(2),
        "assoc returns the extended map"
    );

    // 4. dissoc
    assert_eq!(
        call_beside(file!(), ":t::p6-dissoc-removes").expect("eval"),
        Value::bool(false),
        "dissoc removes the key"
    );

    // 5. LAYER-1 polymorphism — the GENERIC ops dispatch on PersistentMap ("a map is a map").
    assert_eq!(
        call_beside(file!(), ":t::p7-generic-contains").expect("eval"),
        Value::bool(true),
        "generic contains? on a PersistentMap"
    );
    assert_eq!(
        call_beside(file!(), ":t::p8-generic-assoc").expect("eval"),
        Value::i64(2),
        "generic assoc on a PersistentMap"
    );
}

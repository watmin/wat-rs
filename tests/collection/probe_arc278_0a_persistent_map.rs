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

use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

#[test]
fn persistent_map_core_behavior() {
    let world = startup_bare().expect("startup");

    let ev = |expr: &str| -> Value {
        let ast = wat::parse_one!(expr).expect("parse");
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval `{expr}` raised: {e:?}"))
            .value_owned()
    };

    // 1. ctor + length
    assert_eq!(
        ev("(:wat::core::PersistentMap/length (:wat::core::PersistentMap :a 1 :b 2))"),
        Value::i64(2),
        "PersistentMap ctor + length"
    );

    // 2. contains-key? hit + miss
    assert_eq!(
        ev("(:wat::core::PersistentMap/contains-key? (:wat::core::PersistentMap :a 1) :a)"),
        Value::bool(true),
        "contains-key? hit"
    );
    assert_eq!(
        ev("(:wat::core::PersistentMap/contains-key? (:wat::core::PersistentMap :a 1) :z)"),
        Value::bool(false),
        "contains-key? miss"
    );

    // 3. IMMUTABILITY / structural sharing — assoc returns a NEW map; the original is unchanged.
    assert_eq!(
        ev("(:wat::core::let [pm  (:wat::core::PersistentMap :a 1) \
                              _pm2 (:wat::core::PersistentMap/assoc pm :b 2)] \
              (:wat::core::PersistentMap/length pm))"),
        Value::i64(1),
        "assoc must NOT mutate the original (structural sharing)"
    );
    assert_eq!(
        ev("(:wat::core::PersistentMap/length \
              (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap :a 1) :b 2))"),
        Value::i64(2),
        "assoc returns the extended map"
    );

    // 4. dissoc
    assert_eq!(
        ev("(:wat::core::PersistentMap/contains-key? \
              (:wat::core::PersistentMap/dissoc (:wat::core::PersistentMap :a 1) :a) :a)"),
        Value::bool(false),
        "dissoc removes the key"
    );

    // 5. LAYER-1 polymorphism — the GENERIC ops dispatch on PersistentMap ("a map is a map").
    assert_eq!(
        ev("(:wat::core::contains? (:wat::core::PersistentMap :a 1) :a)"),
        Value::bool(true),
        "generic contains? on a PersistentMap"
    );
    assert_eq!(
        ev("(:wat::core::PersistentMap/length (:wat::core::assoc (:wat::core::PersistentMap :a 1) :b 2))"),
        Value::i64(2),
        "generic assoc on a PersistentMap"
    );
}

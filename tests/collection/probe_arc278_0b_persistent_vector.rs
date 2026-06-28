//! Arc 278 stone 0b — disconfirming probe: `:wat::core::PersistentVector` (RED at HEAD).
//!
//! The VECTOR mirror of stone 0a. The rete `Token` carries a `matches` provenance vector grown at each
//! join; std `Vector` is `Arc<Vec>` clone-on-write (O(n)/push). `:wat::core::PersistentVector` (rpds
//! `VectorSync`) gives O(log n) structural-sharing `conj` — a NEW vector; the original is unchanged.
//!
//! RED at HEAD: `:wat::core::PersistentVector` is an unknown head → eval/check error → the `expect`s panic.
//! Compiles at HEAD (public API + wat strings); fails at RUNTIME on exactly the gap. GREEN when 0b ships.
//!
//! Run: cargo test --release -p wat --test probe_arc278_0b_persistent_vector -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

#[test]
fn persistent_vector_core_behavior() {
    let world = startup_bare().expect("startup");

    let ev = |expr: &str| -> Value {
        let ast = wat::parse_one!(expr).expect("parse");
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval `{expr}` raised: {e:?}"))
            .value_owned()
    };

    // 1. ctor + length
    assert_eq!(
        ev("(:wat::core::PersistentVector/length (:wat::core::PersistentVector 10 20 30))"),
        Value::i64(3),
        "PersistentVector ctor + length"
    );

    // 2. get by index
    assert_eq!(
        ev("(:wat::core::PersistentVector/get (:wat::core::PersistentVector 10 20 30) 1)"),
        Value::Option(Arc::new(Some(Value::i64(20)))),
        "get by index"
    );

    // 3. IMMUTABILITY / structural sharing — conj returns a NEW vector; the original is unchanged.
    assert_eq!(
        ev("(:wat::core::let [pv  (:wat::core::PersistentVector 1 2) \
                              _pv2 (:wat::core::PersistentVector/conj pv 3)] \
              (:wat::core::PersistentVector/length pv))"),
        Value::i64(2),
        "conj must NOT mutate the original (structural sharing)"
    );
    assert_eq!(
        ev("(:wat::core::PersistentVector/length \
              (:wat::core::PersistentVector/conj (:wat::core::PersistentVector 1 2) 3))"),
        Value::i64(3),
        "conj returns the extended vector"
    );

    // 4. LAYER-1 polymorphism — the GENERIC ops dispatch on PersistentVector.
    assert_eq!(
        ev("(:wat::core::get (:wat::core::PersistentVector 10 20 30) 2)"),
        Value::Option(Arc::new(Some(Value::i64(30)))),
        "generic get on a PersistentVector"
    );
    assert_eq!(
        ev("(:wat::core::PersistentVector/length (:wat::core::conj (:wat::core::PersistentVector 1) 2))"),
        Value::i64(2),
        "generic conj on a PersistentVector"
    );
}

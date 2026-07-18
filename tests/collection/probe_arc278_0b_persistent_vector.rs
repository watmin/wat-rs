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
use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each assertion's expr lives in a named zero-arg fn in the
// co-located `.wat` fixture, driven via `call_beside`. PersistentVector's per-type
// ops carry no registered TypeScheme (runtime-dispatched intrinsics, same class as
// `metadata-of`) — each defn annotates the documented/observed return shape.
#[test]
fn persistent_vector_core_behavior() {
    // 1. ctor + length
    assert_eq!(
        call_beside(file!(), ":t::p1-ctor-length").expect("eval"),
        Value::i64(3),
        "PersistentVector ctor + length"
    );

    // 2. get by index
    assert_eq!(
        call_beside(file!(), ":t::p2-get-by-index").expect("eval"),
        Value::Option(Arc::new(Some(Value::i64(20)))),
        "get by index"
    );

    // 3. IMMUTABILITY / structural sharing — conj returns a NEW vector; the original is unchanged.
    assert_eq!(
        call_beside(file!(), ":t::p3-conj-immutable-original").expect("eval"),
        Value::i64(2),
        "conj must NOT mutate the original (structural sharing)"
    );
    assert_eq!(
        call_beside(file!(), ":t::p4-conj-extended").expect("eval"),
        Value::i64(3),
        "conj returns the extended vector"
    );

    // 4. LAYER-1 polymorphism — the GENERIC ops dispatch on PersistentVector.
    assert_eq!(
        call_beside(file!(), ":t::p5-generic-get").expect("eval"),
        Value::Option(Arc::new(Some(Value::i64(30)))),
        "generic get on a PersistentVector"
    );
    assert_eq!(
        call_beside(file!(), ":t::p6-generic-conj").expect("eval"),
        Value::i64(2),
        "generic conj on a PersistentVector"
    );
}

//! Arc 278 stone 0c — disconfirming probe: PersistentVector transform/sequence op parity. RED at HEAD.
//!
//! 0a/0b gave PersistentVector the accessor ops (length/get/conj/first/rest/…); PersistentMap is already at
//! full HashMap parity. The remaining miss: the TRANSFORM + SEQUENCE ops std `Vec` has —
//! `map`/`filter`/`foldl`/`foldr`/`concat`/`reverse`/`take`/`drop` — are std-`Vec`-only. This probe runs a
//! PersistentVector through all 8 and asserts each works. RED at HEAD: these don't dispatch on PersistentVector.
//! (Arc 118.B6b: `foldr` retired — its slot below is now `reduce` over `reverse`.)
//!
//! Arc 118.2a note: `map`/`filter`/`take`/`drop` flipped LAZY — they now ALWAYS return a `Stream<T>`
//! (never container-preserving). The four assertions below that used to check "still a
//! PersistentVector" directly now materialize via `(:wat::core::into (:wat::core::PersistentVector) …)`
//! first — the parity this probe proves (PersistentVector is accepted as INPUT to every op) still
//! holds; only the "and comes back out the same container kind" half is retired by design.
//! `foldl`/`reverse`/`concat` are untouched by 118.2a and keep their original assertions.
//!
//! Run: cargo test --release -p wat --test probe_arc278_0c_persistent_parity -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each assertion's expr lives in a named zero-arg fn in the
// co-located `.wat` fixture, driven via `call_beside_value` — no inline wat/format! driver.
#[test]
fn persistent_vector_transform_parity() {
    // foldl / reduce-over-reverse (fn-first; return the accumulator)
    assert_eq!(call_beside_value(file!(), ":t::p1-foldl").expect("eval"), Value::i64(6), "foldl over PersistentVector");
    assert_eq!(call_beside_value(file!(), ":t::p2-fold-reverse").expect("eval"), Value::i64(6), "reduce-over-reverse over PersistentVector");

    // map / filter (fn-first; arc 118.2a: LAZY, returns Stream<T> — materialize via `into`
    // (PersistentVector) to prove PersistentVector is accepted as input).
    assert_eq!(
        call_beside_value(file!(), ":t::p3-map").expect("eval"),
        Value::i64(3), "map accepts a PersistentVector, materializes back to one"
    );
    assert_eq!(
        call_beside_value(file!(), ":t::p4-filter").expect("eval"),
        Value::i64(2), "filter accepts a PersistentVector, materializes back to one"
    );

    // reverse (type-preserving; head after reverse == 3 — get returns Option<T>)
    assert_eq!(
        call_beside_value(file!(), ":t::p5-reverse").expect("eval"),
        Value::Option(Arc::new(Some(Value::i64(3)))), "reverse a PersistentVector"
    );

    // take / drop (coll-first; arc 118.2a: LAZY, returns Stream<T> — materialize via `into`).
    assert_eq!(
        call_beside_value(file!(), ":t::p6-take").expect("eval"),
        Value::i64(2), "take n accepts a PersistentVector, materializes back to one"
    );
    assert_eq!(
        call_beside_value(file!(), ":t::p7-drop").expect("eval"),
        Value::i64(2), "drop n accepts a PersistentVector, materializes back to one"
    );

    // concat (two PersistentVectors → a PersistentVector)
    assert_eq!(
        call_beside_value(file!(), ":t::p8-concat").expect("eval"),
        Value::i64(3), "concat two PersistentVectors"
    );
}

//! Arc 278 stone 0c — disconfirming probe: PersistentVector transform/sequence op parity. RED at HEAD.
//!
//! 0a/0b gave PersistentVector the accessor ops (length/get/conj/first/rest/…); PersistentMap is already at
//! full HashMap parity. The remaining miss: the TRANSFORM + SEQUENCE ops std `Vec` has —
//! `map`/`filter`/`foldl`/`foldr`/`concat`/`reverse`/`take`/`drop` — are std-`Vec`-only. This probe runs a
//! PersistentVector through all 8 and asserts each works. RED at HEAD: these don't dispatch on PersistentVector.
//!
//! Arc 118.2a note: `map`/`filter`/`take`/`drop` flipped LAZY — they now ALWAYS return a `Stream<T>`
//! (never container-preserving). The four assertions below that used to check "still a
//! PersistentVector" directly now materialize via `(:wat::core::into (:wat::core::PersistentVector) …)`
//! first — the parity this probe proves (PersistentVector is accepted as INPUT to every op) still
//! holds; only the "and comes back out the same container kind" half is retired by design.
//! `foldl`/`foldr`/`reverse`/`concat` are untouched by 118.2a and keep their original assertions.
//!
//! Run: cargo test --release -p wat --test probe_arc278_0c_persistent_parity -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

#[test]
fn persistent_vector_transform_parity() {
    let world = startup_bare().expect("startup");

    let ev = |expr: &str| -> Value {
        let ast = wat::parse_one!(expr).expect("parse");
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval `{expr}` raised: {e:?}"))
            .value_owned()
    };

    let pv = "(:wat::core::PersistentVector 1 2 3)";
    let sum = "(:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))";
    let dbl = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))";
    let gt1 = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 1))";

    // foldl / foldr (fn-first; return the accumulator)
    assert_eq!(ev(&format!("(:wat::core::foldl {sum} 0 {pv})")), Value::i64(6), "foldl over PersistentVector");
    assert_eq!(ev(&format!("(:wat::core::foldr {sum} 0 {pv})")), Value::i64(6), "foldr over PersistentVector");

    // map / filter (fn-first; arc 118.2a: LAZY, returns Stream<T> — materialize via `into`
    // (PersistentVector) to prove PersistentVector is accepted as input).
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::map {dbl} {pv})))")),
        Value::i64(3), "map accepts a PersistentVector, materializes back to one"
    );
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter {gt1} {pv})))")),
        Value::i64(2), "filter accepts a PersistentVector, materializes back to one"
    );

    // reverse (type-preserving; head after reverse == 3 — get returns Option<T>)
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/get (:wat::core::reverse {pv}) 0)")),
        Value::Option(Arc::new(Some(Value::i64(3)))), "reverse a PersistentVector"
    );

    // take / drop (coll-first; arc 118.2a: LAZY, returns Stream<T> — materialize via `into`).
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::take {pv} 2)))")),
        Value::i64(2), "take n accepts a PersistentVector, materializes back to one"
    );
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::drop {pv} 1)))")),
        Value::i64(2), "drop n accepts a PersistentVector, materializes back to one"
    );

    // concat (two PersistentVectors → a PersistentVector)
    assert_eq!(
        ev("(:wat::core::PersistentVector/length (:wat::core::concat (:wat::core::PersistentVector 1 2) (:wat::core::PersistentVector 3)))"),
        Value::i64(3), "concat two PersistentVectors"
    );
}

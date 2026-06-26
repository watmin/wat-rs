//! Arc 278 stone 0c — disconfirming probe: PersistentVector transform/sequence op parity. RED at HEAD.
//!
//! 0a/0b gave PersistentVector the accessor ops (length/get/conj/first/rest/…); PersistentMap is already at
//! full HashMap parity. The remaining miss: the TRANSFORM + SEQUENCE ops std `Vec` has —
//! `map`/`filter`/`foldl`/`foldr`/`concat`/`reverse`/`take`/`drop` — are std-`Vec`-only. This probe runs a
//! PersistentVector through all 8 and asserts each works (type-preserving: a transformed PersistentVector
//! returns a PersistentVector). RED at HEAD: these don't dispatch on PersistentVector.
//!
//! Run: cargo test --release -p wat --test probe_arc278_0c_persistent_parity -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

#[test]
fn persistent_vector_transform_parity() {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");

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

    // map / filter (fn-first; TYPE-PRESERVING → a PersistentVector, so PersistentVector/length applies)
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::map {dbl} {pv}))")),
        Value::i64(3), "map returns a PersistentVector"
    );
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::filter {gt1} {pv}))")),
        Value::i64(2), "filter returns a PersistentVector"
    );

    // reverse (type-preserving; head after reverse == 3 — get returns Option<T>)
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/get (:wat::core::reverse {pv}) 0)")),
        Value::Option(Arc::new(Some(Value::i64(3)))), "reverse a PersistentVector"
    );

    // take / drop (coll-first; type-preserving)
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::take {pv} 2))")),
        Value::i64(2), "take n from a PersistentVector"
    );
    assert_eq!(
        ev(&format!("(:wat::core::PersistentVector/length (:wat::core::drop {pv} 1))")),
        Value::i64(2), "drop n from a PersistentVector"
    );

    // concat (two PersistentVectors → a PersistentVector)
    assert_eq!(
        ev("(:wat::core::PersistentVector/length (:wat::core::concat (:wat::core::PersistentVector 1 2) (:wat::core::PersistentVector 3)))"),
        Value::i64(3), "concat two PersistentVectors"
    );
}

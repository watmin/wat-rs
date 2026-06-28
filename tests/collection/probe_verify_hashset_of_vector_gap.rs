//! Historical evidence: the HashSet<Vector<T>> runtime gap that catalyzed arc 216.5a-d.
//!
//! Stone 216.4 SCORE Delta 2 + audit findings surfaced the gap:
//! "`hashmap_key` does not handle `Value::Vec` — means `HashSet<Vector<i64>>`
//!  passes the predicate at check time but fails at runtime."
//!
//! **The gap is closed.** Stone 216.5d deleted `fn hashmap_key` entirely.
//! The canonical-key crutch that caused the gap no longer exists in the substrate.
//! `Value::wat__std__HashSet` now stores `Arc<HashSet<Value>>` (Stone 216.5b);
//! `Value: Hash + Eq` (Stone 216.5a) is the equality contract.
//! This probe is historical evidence — it documents the gap that was there
//! and confirms it cannot reopen because the mechanism no longer exists.
//! The test still passes: `HashSet<Vector<i64>>` constructs and evaluates correctly.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

#[test]
fn verify_hashset_of_vector_constructs_or_errors() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::verify)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(v) => println!("RUNTIME OK: HashSet<Vector<i64>> produced value {:?}", v.value_owned()),
        Err(e) => panic!("RUNTIME FAILED:\n{}\n---\n{:?}", e, e),
    }
}

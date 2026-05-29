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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Environment;

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    )
}

#[test]
fn verify_hashset_of_vector_constructs_or_errors() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [v1     (:wat::core::Vector :wat::core::i64 1 2)
                       v2     (:wat::core::Vector :wat::core::i64 3 4)
                       outer  (:wat::core::HashSet :wat::type::Infer v1 v2)]
                      (:wat::core::HashSet/length outer)))
    "#;
    let src = with_nil_main(src);

    let world = match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("STARTUP FAILED (check-time rejection):\n{}\n---\n{:?}", e, e),
    };

    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env) {
        Ok(v) => println!("RUNTIME OK: HashSet<Vector<i64>> produced value {:?}", v),
        Err(e) => panic!("RUNTIME FAILED:\n{}\n---\n{:?}", e, e),
    }
}

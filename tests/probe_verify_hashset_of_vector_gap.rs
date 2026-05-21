//! Verify the alleged HashSet<Vector<T>> runtime gap.
//!
//! Sonnet's Stone 216.4 SCORE Delta 2 + audit findings claim:
//! "`hashmap_key` does not handle `Value::Vec` — means `HashSet<Vector<i64>>`
//!  passes the predicate at check time but fails at runtime."
//!
//! This probe constructs `HashSet<Vector<i64>>` at the WAT surface and
//! asserts the runtime behavior, so we know definitively whether the gap exists.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Environment;

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

#[test]
fn verify_hashset_of_vector_constructs_or_errors() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
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

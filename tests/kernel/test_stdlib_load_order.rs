//! Arc 275 Stone 275.2 — permanent enforcement gate for stdlib load order.
//!
//! `(:wat::deporder::verify-stdlib)` must return an empty vector. Any
//! future change that introduces an eval-time dependency on a later-loaded
//! file turns this test red immediately.
//!
//! Doctrine: a file in STDLIB_FILES may only reference symbols defined
//! in files that appear BEFORE it in the array (eval-deps). defmacro
//! refs are order-free (registered in the pre-expansion pass) and are
//! exempt. See `src/stdlib.rs` for the doctrine comment on STDLIB_FILES.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

#[test]
fn verify_stdlib_has_no_load_order_violations() {
    let src = concat!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64",
        "  (:wat::core::length (:wat::deporder::verify-stdlib)))\n",
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup must succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let val = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval must succeed");
    match val {
        Value::i64(n) => {
            assert_eq!(
                n, 0,
                "stdlib has {n} load-order violation(s) — \
                 run `cargo test --release --test probe_arc275_verify_stdlib \
                 -- --nocapture` for the full violation list"
            );
        }
        other => panic!("expected i64 count, got {other:?}"),
    }
}

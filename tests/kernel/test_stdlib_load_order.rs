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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn verify_stdlib_has_no_load_order_violations() {
    let world = startup_beside(file!()).expect("startup must succeed");
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

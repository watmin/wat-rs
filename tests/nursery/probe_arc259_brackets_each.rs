//! Arc 259 S3.3 — `:wat::bracket::each`: the side-effect pool (Ruby's `Parallel.each`).
//!
//! `each` is `map` that DISCARDS: it runs `work-fn` over every item through the same
//! bounded, dynamically-balanced pool, then returns `nil` (the results are dropped).
//! Built as a thin wrapper: `(do (brackets/map host items work-fn) nil)` — `map`
//! already blocks until all M results arrive (its collect-loop returns only when
//! `collected == M`), so by the time `map` returns, every work-fn has run. `each`
//! discards the Vector and returns nil.
//!
//! The delta this probe pins (over the already-proven `map`): `each` returns nil AND
//! drains every item. Completion IS the drainage proof — a single-shot or
//! under-draining pool would block forever on the M-th `select'` and HANG this test;
//! a non-hanging nil return proves all M items were processed.
//!
//! RED at HEAD: `:wat::bracket::each` does not exist (UnknownFunction).
//!
//! Run SERIALLY (spawns threads):
//!   `cargo test --release -p wat --test nursery probe_arc259_brackets_each -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Eval a body whose `compute` returns nil; assert the result is `Value::Unit`.
fn run_compute_nil(body: &str) {
    let src = format!("{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned();
    assert_eq!(got, Value::Unit, "brackets/each returns nil; got {got:?}");
}

/// `brackets/each` over 50 items returns nil — and, by completing at all (no hang),
/// proves the pool drained all 50 (the collect-loop only returns at collected==M).
#[test]
fn brackets_each_drains_50_and_returns_nil() {
    run_compute_nil(
        "(:wat::core::defn :user::compute [] -> :wat::core::nil \
           (:wat::bracket::each (:wat::spawn::thread) \
             (:wat::core::range 0 50) \
             (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2))))",
    );
}

/// Small case: 3 items, returns nil.
#[test]
fn brackets_each_small_returns_nil() {
    run_compute_nil(
        "(:wat::core::defn :user::compute [] -> :wat::core::nil \
           (:wat::bracket::each (:wat::spawn::thread) \
             (:wat::core::Vector :wat::core::i64 10 20 30) \
             (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ x 1))))",
    );
}

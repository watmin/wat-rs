//! Reach-stumble: `:wat::core::nth` — the positional, TOTAL accessor.
//!
//! `Vector/get` is the associative, nil-safe form: `Vec<T> × i64 -> Option<T>`
//! (None on out-of-range, never raises). `nth` is the Clojure positional idiom:
//! `Vec<T> × i64 -> T` — "there IS an i-th element; give it or fail" — raising on
//! out-of-range. NOT an alias; the opposite contract at the edge. `nth` is sugar
//! over `Option/expect (Vector/get v i)` with that total promise.
//!
//! RED at HEAD: `:wat::core::nth` does not exist.
//!
//! Run: `cargo test --release -p wat --test nursery probe_nth`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute_i64(body: &str) -> i64 {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body}) \
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// `(nth [10 20 30] 1)` → 20 — the positional element, returned as T (not Option).
#[test]
fn nth_returns_the_positional_element() {
    assert_eq!(
        run_compute_i64("(:wat::core::nth (:wat::core::Vector :wat::core::i64 10 20 30) 1)"),
        20,
        "nth returns the i-th element directly as T"
    );
}

/// `nth` out-of-range RAISES (unlike `get`, which returns None) — the total contract.
#[test]
#[should_panic] // the raise is a structured AssertionFailure payload (not a String),
                // so match any panic — same as the assert-true probe.
fn nth_raises_on_out_of_range() {
    let _ = run_compute_i64("(:wat::core::nth (:wat::core::Vector :wat::core::i64 10 20 30) 9)");
}

//! FM 2-bis probe — arc 238: `:wat::core::=` structural completeness.
//!
//! See `docs/arc/2026/05/238-core-equality-completeness/DESIGN.md`.
//!
//! `values_equal` (the function behind the `=` verb) is missing arms for records, HashMap,
//! HashSet (proven: today these ERROR with TypeMismatch). This probe asserts the TARGET:
//! `=` deep-structurally compares records + maps + sets — type-strict (same type + same values),
//! order-independent for maps/sets.
//!
//! RED today (every contract errors → the `expected bool` panic fires). GREEN once arc 238.1
//! adds the missing `values_equal` arms. Committed atomically WITH the fix (a failing test does
//! not land on the green baseline alone — `feedback_no_broken_commits`).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PRELUDE: &str = "\
(:wat::Record::def :my::Pt [x <- :wat::core::i64  y <- :wat::core::i64])\n";

/// Evaluate a `(= ...)` expr; return the bool, panic with the actual value on anything else
/// (a `TypeMismatch` Err is the RED state this probe documents).
fn eq(expr: &str) -> bool {
    let full = format!(
        "{PRELUDE}\
         (:wat::core::defn :user::compute [] -> :wat::core::bool {expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup/check error for `{}`: {:?}", expr, e));
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).map(|tv| tv.value_owned()) {
        Ok(Value::bool(b)) => b,
        other => panic!("expected bool for `{}`; got {:?}", expr, other),
    }
}

// ─── Records (type-strict: same type + same values) ───────────────────────────
#[test]
fn record_equal() {
    assert!(eq("(:wat::core::= (:my::Pt 1 2) (:my::Pt 1 2))"));
}
#[test]
fn record_unequal_value() {
    assert!(!eq("(:wat::core::= (:my::Pt 1 2) (:my::Pt 1 9))"));
}

// ─── HashMap (order-independent structural) ───────────────────────────────────
#[test]
fn map_equal() {
    assert!(eq("(:wat::core::= {:a 1 :b 2} {:a 1 :b 2})"));
}
#[test]
fn map_order_independent() {
    assert!(eq("(:wat::core::= {:a 1 :b 2} {:b 2 :a 1})"));
}
#[test]
fn map_unequal() {
    assert!(!eq("(:wat::core::= {:a 1} {:a 2})"));
}

// ─── HashSet (order-independent structural) ───────────────────────────────────
#[test]
fn set_equal() {
    assert!(eq("(:wat::core::= #{1 2 3} #{1 2 3})"));
}
#[test]
fn set_order_independent() {
    assert!(eq("(:wat::core::= #{1 2 3} #{3 2 1})"));
}
#[test]
fn set_unequal() {
    assert!(!eq("(:wat::core::= #{1 2} #{1 2 3})"));
}

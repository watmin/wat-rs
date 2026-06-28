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
//!
//! Wat source lives in the co-located fixture: probe_arc238_eq_completeness.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_bool(world: &wat::freeze::FrozenWorld, expr: &str) -> bool {
    let ast = wat::parse_one!(expr).expect("parse expr");
    match eval_in_frozen(&ast, world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
    {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Records (type-strict: same type + same values) ───────────────────────────

#[test]
fn record_equal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, "(:t::record-equal)"));
}

#[test]
fn record_unequal_value() {
    let world = startup_beside(file!()).expect("startup");
    assert!(!run_bool(&world, "(:t::record-unequal-value)"));
}

// ─── HashMap (order-independent structural) ───────────────────────────────────

#[test]
fn map_equal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, "(:t::map-equal)"));
}

#[test]
fn map_order_independent() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, "(:t::map-order-independent)"));
}

#[test]
fn map_unequal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(!run_bool(&world, "(:t::map-unequal)"));
}

// ─── HashSet (order-independent structural) ───────────────────────────────────

#[test]
fn set_equal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, "(:t::set-equal)"));
}

#[test]
fn set_order_independent() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, "(:t::set-order-independent)"));
}

#[test]
fn set_unequal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(!run_bool(&world, "(:t::set-unequal)"));
}

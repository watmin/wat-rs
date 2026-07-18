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

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `:t::…` fixture fn is a zero-arg `-> :wat::core::bool` probe;
// fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run_bool(world: &wat::freeze::FrozenWorld, fn_name: &str) -> bool {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
    {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Records (type-strict: same type + same values) ───────────────────────────

#[test]
fn record_equal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, ":t::record-equal"));
}

#[test]
fn record_unequal_value() {
    let world = startup_beside(file!()).expect("startup");
    assert!(!run_bool(&world, ":t::record-unequal-value"));
}

// ─── HashMap (order-independent structural) ───────────────────────────────────

#[test]
fn map_equal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, ":t::map-equal"));
}

#[test]
fn map_order_independent() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, ":t::map-order-independent"));
}

#[test]
fn map_unequal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(!run_bool(&world, ":t::map-unequal"));
}

// ─── HashSet (order-independent structural) ───────────────────────────────────

#[test]
fn set_equal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, ":t::set-equal"));
}

#[test]
fn set_order_independent() {
    let world = startup_beside(file!()).expect("startup");
    assert!(run_bool(&world, ":t::set-order-independent"));
}

#[test]
fn set_unequal() {
    let world = startup_beside(file!()).expect("startup");
    assert!(!run_bool(&world, ":t::set-unequal"));
}

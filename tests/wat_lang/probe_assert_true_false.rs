//! `:wat::test::assert-true` / `assert-false` — the basic boolean assertions.
//!
//! Reach-stumble tool: a test reaches for `assert-true` before anything else, and
//! it was absent (only `assert-eq` / `assert-contains` / `assert-coincident`
//! existed). Made first-class in `wat/test.wat`. Each fires `assertion-failed!`
//! (failing the run) on the wrong bool, and is a no-op on the right one.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

/// Outside a sandbox an assertion fires by PANIC (the sandbox would catch it into
/// a RunResult; raw eval lets it propagate — "an assertion firing outside a harness
/// IS a program error", per assertion.rs).
fn call_fn(world: &wat::freeze::FrozenWorld, name: &str) {
    let ast = wat::parse_one!(name).expect("parse");
    let _ = eval_in_frozen(&ast, world, &Environment::new());
}

#[test]
fn assert_true_passes_on_true() {
    let world = startup_beside(file!()).expect("startup");
    call_fn(&world, "(:t::assert-true-on-true)");
    // no panic → assertion did not fire → correct
}

/// Assertion fires by PANIC; `#[should_panic]` catches it.
#[test]
#[should_panic]
fn assert_true_panics_on_false() {
    let world = startup_beside(file!()).expect("startup");
    call_fn(&world, "(:t::assert-true-on-false)");
}

#[test]
fn assert_false_passes_on_false() {
    let world = startup_beside(file!()).expect("startup");
    call_fn(&world, "(:t::assert-false-on-false)");
    // no panic → assertion did not fire → correct
}

#[test]
#[should_panic]
fn assert_false_panics_on_true() {
    let world = startup_beside(file!()).expect("startup");
    call_fn(&world, "(:t::assert-false-on-true)");
}

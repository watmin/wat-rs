//! End-to-end tests for `:wat::holon::Bundle`'s capacity guard.
//!
//! Bundle's return type is always
//! `:Result<wat::holon::HolonAST, :wat::holon::CapacityExceeded>`. The
//! `:wat::config::capacity-mode` setter picks what the runtime does
//! when a Bundle's constituent count exceeds `floor(sqrt(dims))`:
//!
//! - `:error`  → `Ok(h)` under; `Err(CapacityExceeded{cost, budget})`
//!   over — caller holds the error, program continues.
//! - `:panic`  → `Ok(h)` under; `panic!()` over — fail-closed.
//!
//! Arc 037 (2026-04-24) retired `:silent` and `:warn`. Overflow
//! either crashes or is handled; no middle ground.
//! Arc 045 (2026-04-24) renamed `:abort` → `:panic` for honesty
//! with Rust's `panic!()` macro behavior (which unwinds, unlike
//! `std::process::abort`).
//!
//! At `d=1024`, `budget = floor(sqrt(1024)) = 32`. The tests below
//! pick list sizes deliberately on either side.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.
//!
//! Fixture files generated with distinct atom counts live in tests/collection/.
//! Each is loaded via startup_from_file (static, committed). Negative fixture
//! for type-check rejection: tests/collection/bundle_capacity_bad_return_type.wat.

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run(fixture: &str) -> Value {
    let world = startup_from_file(fixture).expect("startup should succeed");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

// ─── Under budget: Ok across all modes ───────────────────────────────

#[test]
fn bundle_under_budget_returns_ok_under_error_mode() {
    // d=1024 → budget=32. Bundle 5 atoms — well under. Ok(h) expected.
    match run("tests/collection/bundle_capacity_under_error.wat") {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

#[test]
fn bundle_under_budget_returns_ok_under_panic_mode() {
    match run("tests/collection/bundle_capacity_under_panic.wat") {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── Over budget under :error — populates CapacityExceeded ───────────

// Arc 077 — one d per program, default 10000. Budget at d=10000 is
// floor(sqrt(10000)) = 100. Any Bundle with 101+ items overflows.
// CapacityExceeded reports cost AND budget (no router-None signal).

#[test]
fn bundle_over_budget_under_error_mode_returns_err_struct() {
    // 317 atoms — far past sqrt(10000)=100. Overflows the program-d.
    match run("tests/collection/bundle_capacity_over_error.wat") {
        Value::Result(r) => match &*r {
            Err(Value::Struct(sv)) => {
                assert_eq!(sv.type_name, ":wat::holon::CapacityExceeded");
                assert_eq!(sv.fields.len(), 2, "CapacityExceeded has cost + budget");
                match (&sv.fields[0], &sv.fields[1]) {
                    (Value::i64(cost), Value::i64(budget)) => {
                        assert_eq!(*cost, 317, "cost is the constituent count");
                        assert_eq!(*budget, 100, "budget = floor(sqrt(10000)) at default d");
                    }
                    other => panic!("expected (i64, i64) fields; got {:?}", other),
                }
            }
            other => panic!("expected Err(Struct); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

#[test]
fn bundle_err_cost_and_budget_readable_via_accessors() {
    // Round-trip through user wat: with 400 atoms against the
    // default d=10000 (budget=100), cost - budget = 300.
    match run("tests/collection/bundle_capacity_accessors.wat") {
        Value::i64(n) => assert_eq!(n, 300, "400 - floor(sqrt(10000)) = 300"),
        other => panic!("expected i64 300; got {:?}", other),
    }
}

// ─── Over budget under :panic — panic ────────────────────────────────

#[test]
fn bundle_over_budget_under_panic_mode_panics() {
    // :panic fails closed. 500 atoms overflow all tiers → panic.
    let world = startup_from_file("tests/collection/bundle_capacity_over_panic.wat")
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        eval_in_frozen(&ast, &world, &env)
    }));
    assert!(caught.is_err(), ":panic + over budget must panic");
}

// ─── Try form propagates Bundle's Err ────────────────────────────────

#[test]
fn try_propagates_bundle_err_across_function_boundary() {
    // Helper returns Result. Its body calls Bundle and `try`s the
    // result. Main calls the helper and matches. This is the cleanest
    // handler shape once `try` is available for Bundle's Result.
    // 400 atoms overflow the default tier (post-arc-067); helper's
    // Bundle returns Err(CapacityExceeded{cost=400, budget=0});
    // try propagates it across the function boundary; main's Err
    // arm reads cost=400.
    match run("tests/collection/bundle_capacity_try_propagate.wat") {
        Value::i64(400) => {}
        other => panic!("expected i64 400 (the cost); got {:?}", other),
    }
}

// ─── Check-time refusals ─────────────────────────────────────────────

#[test]
fn bundle_return_type_mismatch_rejected_at_check() {
    // probe fn's return type is :wat::holon::HolonAST but Bundle returns
    // :Result<wat::holon::HolonAST, CapacityExceeded>. Must fail at check.
    // Negative fixture: tests/collection/bundle_capacity_bad_return_type.wat
    match startup_from_file("tests/collection/bundle_capacity_bad_return_type.wat") {
        Err(_) => {}
        Ok(_) => panic!("expected check failure — Bundle is Result-typed, caller declared :wat::holon::HolonAST"),
    }
}

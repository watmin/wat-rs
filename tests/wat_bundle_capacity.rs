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

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    invoke_user_main(&world, Vec::new()).expect("main should run")
}

/// Emit `n` distinct `(:wat::holon::Atom "i")` calls inside a
/// `(:wat::core::list :wat::holon::HolonAST ...)` literal — used to pack
/// Bundle with exactly `n` constituents.
fn atoms_list(n: usize) -> String {
    let mut s = String::from("(:wat::core::list :wat::holon::HolonAST");
    for i in 0..n {
        s.push_str(&format!(" (:wat::holon::Atom \"atom-{}\")", i));
    }
    s.push(')');
    s
}

// ─── Under budget: Ok across all modes ───────────────────────────────

#[test]
fn bundle_under_budget_returns_ok_under_error_mode() {
    // d=1024 → budget=32. Bundle 5 atoms — well under. Ok(h) expected.
    let src = format!(
        r#"

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(5)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

#[test]
fn bundle_under_budget_returns_ok_under_panic_mode() {
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :panic)

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(5)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── Over budget under :error — populates CapacityExceeded ───────────

// Arc 037 slice 1 layer 3: the ambient router picks dim per
// construction from DEFAULT_TIERS = [256, 4096, 10000, 100000].
// Largest tier d=100000 has budget floor(sqrt(100000)) = 316.
// Any Bundle with 317+ items overflows every tier: router returns
// None → CapacityExceeded with budget=0 (the None signal).
// `set-dims!` is retired (arc 037 — config-collect rejects it).

#[test]
fn bundle_over_budget_under_error_mode_returns_err_struct() {
    // 317 atoms — one past sqrt(100000). Every tier overflows;
    // router returns None.
    let src = format!(
        r#"

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(317)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Err(Value::Struct(sv)) => {
                assert_eq!(sv.type_name, ":wat::holon::CapacityExceeded");
                assert_eq!(sv.fields.len(), 2, "CapacityExceeded has cost + budget");
                match (&sv.fields[0], &sv.fields[1]) {
                    (Value::i64(cost), Value::i64(budget)) => {
                        assert_eq!(*cost, 317, "cost is the constituent count");
                        assert_eq!(*budget, 0, "budget=0 signals router returned None (no tier fits)");
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
    // Round-trip through user wat: the program reads cost and budget
    // from the CapacityExceeded instance via the auto-generated
    // accessors. With 400 atoms against DEFAULT_TIERS, router
    // returns None → budget=0 → cost-budget = 400.
    let src = format!(
        r#"

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (:wat::holon::Bundle {}) -> :i64
            ((Ok _) 0)
            ((Err e)
              (:wat::core::i64::-
                (:wat::holon::CapacityExceeded/cost e)
                (:wat::holon::CapacityExceeded/budget e)))))
        "#,
        atoms_list(400)
    );
    match run(&src) {
        Value::i64(n) => assert_eq!(n, 400, "400-atom bundle overflows all tiers → diff 400"),
        other => panic!("expected i64 400; got {:?}", other),
    }
}

// ─── Over budget under :panic — panic ────────────────────────────────

#[test]
fn bundle_over_budget_under_panic_mode_panics() {
    // :panic fails closed. 500 atoms overflow all tiers → panic.
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :panic)

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(500)
    );
    let caught = std::panic::catch_unwind(|| run(&src));
    assert!(caught.is_err(), ":panic + over budget must panic");
}

// ─── Try form propagates Bundle's Err ────────────────────────────────

#[test]
fn try_propagates_bundle_err_across_function_boundary() {
    // Helper returns Result. Its body calls Bundle and `try`s the
    // result. Main calls the helper and matches. This is the cleanest
    // handler shape once `try` is available for Bundle's Result.
    // 400 atoms overflow all DEFAULT_TIERS; helper's Bundle returns
    // Err(CapacityExceeded{cost=400, budget=0}); try propagates it
    // across the function boundary; main's Err arm reads cost=400.
    let src = format!(
        r#"

        (:wat::core::define (:app::build-composite
                            (items :wat::holon::Holons)
                            -> :wat::holon::BundleResult)
          (Ok (:wat::core::try (:wat::holon::Bundle items))))

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (:app::build-composite {}) -> :i64
            ((Ok _) 0)
            ((Err e) (:wat::holon::CapacityExceeded/cost e))))
        "#,
        atoms_list(400)
    );
    match run(&src) {
        Value::i64(400) => {}
        other => panic!("expected i64 400 (the cost); got {:?}", other),
    }
}

// ─── Check-time refusals ─────────────────────────────────────────────

#[test]
fn bundle_return_type_mismatch_rejected_at_check() {
    // main's return type is :wat::holon::HolonAST but Bundle returns
    // :Result<wat::holon::HolonAST, CapacityExceeded>. Must fail at check.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::holon::HolonAST)
          (:wat::holon::Bundle (:wat::core::list :wat::holon::HolonAST
            (:wat::holon::Atom "a")
            (:wat::holon::Atom "b"))))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Err(_) => {}
        Ok(_) => panic!("expected check failure — Bundle is Result-typed, caller declared :wat::holon::HolonAST"),
    }
}

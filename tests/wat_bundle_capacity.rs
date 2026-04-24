//! End-to-end tests for `:wat::holon::Bundle`'s capacity guard.
//!
//! Bundle's return type is always
//! `:Result<wat::holon::HolonAST, :wat::holon::CapacityExceeded>`. The
//! `:wat::config::capacity-mode` setter picks what the runtime does
//! when a Bundle's constituent count exceeds `floor(sqrt(dims))`:
//!
//! - `:silent` → always `Ok(h)`. No check. Degraded vector produced.
//! - `:warn`   → always `Ok(h)`. `eprintln!` diagnostic when over.
//! - `:error`  → `Ok(h)` under; `Err(CapacityExceeded{cost, budget})`
//!   over — caller holds the error, program continues.
//! - `:abort`  → `Ok(h)` under; `panic!()` over — fail-closed.
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
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

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
fn bundle_under_budget_returns_ok_under_silent_mode() {
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :silent)
        (:wat::config::set-dims! 1024)

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

#[test]
fn bundle_over_budget_under_error_mode_returns_err_struct() {
    // d=1024 → budget=32. Bundle 33 atoms — one over. Err fires.
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(33)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Err(Value::Struct(sv)) => {
                assert_eq!(sv.type_name, ":wat::holon::CapacityExceeded");
                assert_eq!(sv.fields.len(), 2, "CapacityExceeded has cost + budget");
                // cost first field, budget second — per struct declaration order.
                match (&sv.fields[0], &sv.fields[1]) {
                    (Value::i64(cost), Value::i64(budget)) => {
                        assert_eq!(*cost, 33, "cost is the constituent count");
                        assert_eq!(*budget, 32, "budget is floor(sqrt(1024))");
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
    // accessors and computes their difference.
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (:wat::holon::Bundle {}) -> :i64
            ((Ok _) 0)
            ((Err e)
              (:wat::core::i64::-
                (:wat::holon::CapacityExceeded/cost e)
                (:wat::holon::CapacityExceeded/budget e)))))
        "#,
        atoms_list(40)
    );
    match run(&src) {
        Value::i64(n) => assert_eq!(n, 40 - 32, "40-atom bundle over budget 32 → diff 8"),
        other => panic!("expected i64 8; got {:?}", other),
    }
}

// ─── Over budget under :silent — still Ok, degraded vector ───────────

#[test]
fn bundle_over_budget_under_silent_mode_still_returns_ok() {
    // :silent deliberately skips the check. Bundle returns Ok with the
    // (degraded) vector even though cost > budget. Author opted in.
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :silent)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(200)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST) even over budget under :silent; got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── Over budget under :warn — Ok with diagnostic ────────────────────

#[test]
fn bundle_over_budget_under_warn_mode_still_returns_ok() {
    // :warn prints to stderr but still produces Ok. We can't easily
    // capture stderr from invoke_user_main inside this test — the
    // stderr check lives in a full CLI-spawning test if we add one
    // later. Here we verify the return shape only.
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :warn)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(100)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST) under :warn; got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── Over budget under :abort — panic ────────────────────────────────

#[test]
fn bundle_over_budget_under_abort_mode_panics() {
    // :abort fails closed — the process terminates before any bad
    // vector escapes. invoke_user_main propagates the panic; we catch
    // via std::panic::catch_unwind to assert the panic path fires.
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :abort)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(50)
    );
    let caught = std::panic::catch_unwind(|| run(&src));
    assert!(caught.is_err(), ":abort + over budget must panic");
}

// ─── Try form propagates Bundle's Err ────────────────────────────────

#[test]
fn try_propagates_bundle_err_across_function_boundary() {
    // Helper returns Result. Its body calls Bundle and `try`s the
    // result. Main calls the helper and matches. This is the cleanest
    // handler shape once `try` is available for Bundle's Result.
    let src = format!(
        r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:app::build-composite
                            (items :wat::holon::Holons)
                            -> :wat::holon::BundleResult)
          (Ok (:wat::core::try (:wat::holon::Bundle items))))

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (:app::build-composite {}) -> :i64
            ((Ok _) 0)
            ((Err e) (:wat::holon::CapacityExceeded/cost e))))
        "#,
        atoms_list(50)
    );
    match run(&src) {
        Value::i64(50) => {}
        other => panic!("expected i64 50 (the cost); got {:?}", other),
    }
}

// ─── Check-time refusals ─────────────────────────────────────────────

#[test]
fn bundle_return_type_mismatch_rejected_at_check() {
    // main's return type is :wat::holon::HolonAST but Bundle returns
    // :Result<wat::holon::HolonAST, CapacityExceeded>. Must fail at check.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

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

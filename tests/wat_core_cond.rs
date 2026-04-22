//! Integration tests for `:wat::core::cond` — multi-way conditional
//! factoring the nested-if ceremony caught in
//! `wat/std/hermetic.wat`'s exit-code-prefix.
//!
//! Shape: `(:wat::core::cond -> :T ((test) body) ... (:else body))`.
//! Typed once at the head; each test unifies with :bool; each body
//! unifies with :T; last arm must be (:else body).

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

fn run_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

fn unwrap_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn unwrap_i64(v: Value) -> i64 {
    match v {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Happy paths ────────────────────────────────────────────────────────

#[test]
fn cond_first_arm_matches() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::cond -> :String
            ((:wat::core::= 1 1) "first")
            ((:wat::core::= 2 2) "second")
            (:else "none")))
    "#;
    assert_eq!(unwrap_string(run(src)), "first");
}

#[test]
fn cond_middle_arm_matches() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::cond -> :String
            ((:wat::core::= 1 2) "first")
            ((:wat::core::= 3 3) "middle")
            ((:wat::core::= 4 5) "third")
            (:else "none")))
    "#;
    assert_eq!(unwrap_string(run(src)), "middle");
}

#[test]
fn cond_falls_through_to_else() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::cond -> :String
            ((:wat::core::= 1 2) "first")
            ((:wat::core::= 3 4) "second")
            (:else "defaulted")))
    "#;
    assert_eq!(unwrap_string(run(src)), "defaulted");
}

#[test]
fn cond_with_single_else_only() {
    // Minimal cond — just the else arm.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::cond -> :i64
            (:else 42)))
    "#;
    assert_eq!(unwrap_i64(run(src)), 42);
}

#[test]
fn cond_dispatches_on_bound_value() {
    // The exit-code-prefix shape — cond on an :i64 binding.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::label (code :i64) -> :String)
          (:wat::core::cond -> :String
            ((:wat::core::= code 1) "[runtime error]")
            ((:wat::core::= code 2) "[panic]")
            ((:wat::core::= code 3) "[startup error]")
            (:else "[nonzero exit]")))

        (:wat::core::define (:user::main -> :String)
          (:my::label 3))
    "#;
    assert_eq!(unwrap_string(run(src)), "[startup error]");
}

// ─── Type-checker refusals ──────────────────────────────────────────────

#[test]
fn cond_refuses_missing_else() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::cond -> :String
            ((:wat::core::= 1 1) "first")
            ((:wat::core::= 2 2) "second")))
    "#;
    let err = run_err(src);
    assert!(
        err.contains(":else") || err.contains("explicit default"),
        "expected missing-:else diagnostic; got: {}",
        err
    );
}

#[test]
fn cond_refuses_non_bool_test() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::cond -> :String
            (42 "first")
            (:else "none")))
    "#;
    let err = run_err(src);
    assert!(
        err.contains(":bool"),
        "expected bool-type diagnostic; got: {}",
        err
    );
}

#[test]
fn cond_refuses_mismatched_body_type() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::cond -> :String
            ((:wat::core::= 1 1) 42)
            (:else "default")))
    "#;
    let err = run_err(src);
    assert!(
        err.contains("TypeMismatch") && err.contains("arm #1"),
        "expected arm-body type mismatch; got: {}",
        err
    );
}

// ─── Tail position ──────────────────────────────────────────────────────

#[test]
fn cond_preserves_tail_call() {
    // Tail-recursive countdown with a cond at the tail — verifies
    // eval_cond_tail threads tail position into the selected body.
    // Without TCO through cond, this would overflow the stack.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::countdown (n :i64) -> :i64)
          (:wat::core::cond -> :i64
            ((:wat::core::= n 0) 0)
            ((:wat::core::< n 0) -1)
            (:else (:my::countdown (:wat::core::i64::- n 1)))))

        (:wat::core::define (:user::main -> :i64)
          (:my::countdown 100000))
    "#;
    assert_eq!(unwrap_i64(run(src)), 0);
}

// ─── Nested cond ────────────────────────────────────────────────────────

#[test]
fn cond_composes_with_other_cond() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::cond -> :String
            ((:wat::core::= 1 2) "outer-first")
            ((:wat::core::= 1 1)
              (:wat::core::cond -> :String
                ((:wat::core::= 7 8) "inner-first")
                (:else "inner-else")))
            (:else "outer-else")))
    "#;
    assert_eq!(unwrap_string(run(src)), "inner-else");
}

//! Integration tests for `:wat::core::cond` — multi-way conditional
//! factoring the nested-if ceremony caught in
//! `wat/kernel/hermetic.wat`'s exit-code-prefix.
//!
//! Shape: `(:wat::core::cond -> :T ((test) body) ... (:else body))`.
//! Typed once at the head; each test unifies with :wat::core::bool; each body
//! unifies with :T; last arm must be (:else body).

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn run_err_file(rel_path: &str) -> String {
    match startup_from_file(rel_path) {
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
    assert_eq!(unwrap_string(run_expr("(:t::cond-first)")), "first");
}

#[test]
fn cond_middle_arm_matches() {
    assert_eq!(unwrap_string(run_expr("(:t::cond-middle)")), "middle");
}

#[test]
fn cond_falls_through_to_else() {
    assert_eq!(unwrap_string(run_expr("(:t::cond-else)")), "defaulted");
}

#[test]
fn cond_with_single_else_only() {
    assert_eq!(unwrap_i64(run_expr("(:t::cond-only-else)")), 42);
}

#[test]
fn cond_dispatches_on_bound_value() {
    assert_eq!(unwrap_string(run_expr("(:t::cond-dispatch)")), "[startup error]");
}

// ─── Type-checker refusals ──────────────────────────────────────────────

#[test]
fn cond_refuses_missing_else() {
    let err = run_err_file("tests/wat_lang/wat_core_cond_no_else_bad.wat");
    wat::assert_edn_matches_file!(
        err,
        "wat_core_cond__cond_refuses_missing_else.edn",
        "expected missing-:else diagnostic"
    );
}

#[test]
fn cond_refuses_non_bool_test() {
    let err = run_err_file("tests/wat_lang/wat_core_cond_non_bool_bad.wat");
    wat::assert_edn_matches_file!(
        err,
        "wat_core_cond__cond_refuses_non_bool_test.edn",
        "expected TypeMismatch on non-bool cond test"
    );
}

#[test]
fn cond_refuses_mismatched_body_type() {
    let err = run_err_file("tests/wat_lang/wat_core_cond_body_mismatch_bad.wat");
    wat::assert_edn_matches_file!(
        err,
        "wat_core_cond__cond_refuses_mismatched_body_type.edn",
        "expected TypeMismatch on arm-body mismatch after cond macro expansion"
    );
}

// ─── Tail position ──────────────────────────────────────────────────────

#[test]
fn cond_preserves_tail_call() {
    assert_eq!(unwrap_i64(run_expr("(:t::cond-tail)")), 0);
}

// ─── Nested cond ────────────────────────────────────────────────────────

#[test]
fn cond_composes_with_other_cond() {
    assert_eq!(unwrap_string(run_expr("(:t::cond-nested)")), "inner-else");
}

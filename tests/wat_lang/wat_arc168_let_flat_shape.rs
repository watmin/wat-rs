//! Integration tests for arc 168 — let flat-shape binding vector +
//! body implicit-do.
//!
//! `:wat::core::let` consumes outer bindings as a `WatAST::Vector`
//! whose elements alternate `binder expr binder expr ...`. Body is
//! 1+ trailing forms (implicit-do); empty body legal — value is
//! `:wat::core::nil`.
//!
//! Body implicit-do extends to `:wat::core::fn` and `:wat::core::defn`
//! symmetry — fn/defn body slots become `body1 body2 ... bodyN` after
//! the `-> :T` arrow.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

fn startup_err_file(rel_path: &str) -> String {
    match startup_from_file(rel_path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Test 1 — single_binding ─────────────────────────────────────────────────

#[test]
fn single_binding() {
    match run_expr(":t::test1-single") {
        Value::i64(n) => assert_eq!(n, 2, "expected 1+1=2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 2 — multiple_bindings ──────────────────────────────────────────────

#[test]
fn multiple_bindings() {
    match run_expr(":t::test2-multi") {
        Value::i64(n) => assert_eq!(n, 3, "expected 1+2=3; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 3 — sequential_references ──────────────────────────────────────────

#[test]
fn sequential_references() {
    match run_expr(":t::test3-seq") {
        Value::i64(n) => assert_eq!(n, 2, "expected y=x+1=2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 4 — empty_bindings ─────────────────────────────────────────────────

#[test]
fn empty_bindings() {
    match run_expr(":t::test4-empty") {
        Value::i64(n) => assert_eq!(n, 2, "expected (let [] (+ 1 1)) = 2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 5 — empty_body ─────────────────────────────────────────────────────

#[test]
fn empty_body() {
    match run_expr(":t::test5-empty-body") {
        Value::Unit => {}
        other => panic!("expected Value::Unit (:wat::core::nil); got {:?}", other),
    }
}

// ─── Test 6 — destructure_binding ────────────────────────────────────────────

#[test]
fn destructure_binding() {
    match run_expr(":t::test6-destructure") {
        Value::i64(n) => assert_eq!(n, 7, "expected 3+4=7; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// Tests 7 + 8 retired per arc 168 slice 3.

// ─── Test 9 — odd_count_vector_errors ────────────────────────────────────────

#[test]
fn odd_count_vector_errors() {
    let err = startup_err_file("tests/wat_lang/wat_arc168_let_flat_shape_odd.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc168_let_flat_shape__odd_count_vector_errors_1.edn",
        "expected MalformedForm for odd-count [x]"
    );

    let err3 = startup_err_file("tests/wat_lang/wat_arc168_let_flat_shape_odd3.wat.bad");
    wat::assert_edn_matches_file!(
        err3,
        "wat_arc168_let_flat_shape__odd_count_vector_errors_3.edn",
        "expected MalformedForm for odd-count [x 1 y]"
    );
}

// ─── Test 10 — multi_form_let_body ───────────────────────────────────────────

#[test]
fn multi_form_let_body() {
    match run_expr(":t::test10-multi-body") {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 11 — multi_form_let_body_typecheck ─────────────────────────────────

#[test]
fn multi_form_let_body_typecheck() {
    let err = startup_err_file(
        "tests/wat_lang/wat_arc168_let_flat_shape_typecheck.wat.bad",
    );
    wat::assert_edn_matches_file!(
        err,
        "wat_arc168_let_flat_shape__multi_form_let_body_typecheck.edn",
        "expected TypeMismatch on non-final body form"
    );
}

// ─── Test 12 — multi_form_fn_body ────────────────────────────────────────────

#[test]
fn multi_form_fn_body() {
    match run_expr(":t::test12-fn-body") {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form return value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 13 — multi_form_defn_body ──────────────────────────────────────────

#[test]
fn multi_form_defn_body() {
    match run_expr(":t::test13-defn-body") {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form return value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 14 — single_body_let_regression ────────────────────────────────────

#[test]
fn single_body_let_regression() {
    match run_expr(":t::test14-single-let") {
        Value::i64(n) => assert_eq!(n, 30, "expected 10+20=30; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 15 — single_body_fn_regression ─────────────────────────────────────

#[test]
fn single_body_fn_regression() {
    match run_expr(":t::test15-single-fn") {
        Value::i64(n) => assert_eq!(n, 15, "expected 7+8=15; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

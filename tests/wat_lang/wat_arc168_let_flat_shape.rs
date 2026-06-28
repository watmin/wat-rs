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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn startup_err_file(rel_path: &str) -> String {
    let src = format!("{}\n---\n{:?}", "", "");
    let _ = src; // hint: we format both Display and Debug
    match startup_from_file(rel_path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Test 1 — single_binding ─────────────────────────────────────────────────

#[test]
fn single_binding() {
    match run_expr("(:t::test1-single)") {
        Value::i64(n) => assert_eq!(n, 2, "expected 1+1=2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 2 — multiple_bindings ──────────────────────────────────────────────

#[test]
fn multiple_bindings() {
    match run_expr("(:t::test2-multi)") {
        Value::i64(n) => assert_eq!(n, 3, "expected 1+2=3; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 3 — sequential_references ──────────────────────────────────────────

#[test]
fn sequential_references() {
    match run_expr("(:t::test3-seq)") {
        Value::i64(n) => assert_eq!(n, 2, "expected y=x+1=2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 4 — empty_bindings ─────────────────────────────────────────────────

#[test]
fn empty_bindings() {
    match run_expr("(:t::test4-empty)") {
        Value::i64(n) => assert_eq!(n, 2, "expected (let [] (+ 1 1)) = 2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 5 — empty_body ─────────────────────────────────────────────────────

#[test]
fn empty_body() {
    match run_expr("(:t::test5-empty-body)") {
        Value::Unit => {}
        other => panic!("expected Value::Unit (:wat::core::nil); got {:?}", other),
    }
}

// ─── Test 6 — destructure_binding ────────────────────────────────────────────

#[test]
fn destructure_binding() {
    match run_expr("(:t::test6-destructure)") {
        Value::i64(n) => assert_eq!(n, 7, "expected 3+4=7; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// Tests 7 + 8 retired per arc 168 slice 3.

// ─── Test 9 — odd_count_vector_errors ────────────────────────────────────────

#[test]
fn odd_count_vector_errors() {
    let err = startup_err_file("tests/wat_lang/wat_arc168_let_flat_shape_odd_bad.wat");
    assert!(
        err.contains("even number of elements") || err.contains("MalformedForm"),
        "expected clear error on odd-count `[x]`; got: {}",
        err
    );

    let err3 = startup_err_file("tests/wat_lang/wat_arc168_let_flat_shape_odd3_bad.wat");
    assert!(
        err3.contains("even number of elements") || err3.contains("MalformedForm"),
        "expected clear error on odd-count `[x 1 y]`; got: {}",
        err3
    );
}

// ─── Test 10 — multi_form_let_body ───────────────────────────────────────────

#[test]
fn multi_form_let_body() {
    match run_expr("(:t::test10-multi-body)") {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 11 — multi_form_let_body_typecheck ─────────────────────────────────

#[test]
fn multi_form_let_body_typecheck() {
    let err = startup_err_file(
        "tests/wat_lang/wat_arc168_let_flat_shape_typecheck_bad.wat",
    );
    assert!(
        err.contains("TypeMismatch")
            || err.contains("type mismatch")
            || err.contains("expected"),
        "expected type mismatch on non-final body form; got: {}",
        err
    );
}

// ─── Test 12 — multi_form_fn_body ────────────────────────────────────────────

#[test]
fn multi_form_fn_body() {
    match run_expr("(:t::test12-fn-body)") {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form return value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 13 — multi_form_defn_body ──────────────────────────────────────────

#[test]
fn multi_form_defn_body() {
    match run_expr("(:t::test13-defn-body)") {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form return value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 14 — single_body_let_regression ────────────────────────────────────

#[test]
fn single_body_let_regression() {
    match run_expr("(:t::test14-single-let)") {
        Value::i64(n) => assert_eq!(n, 30, "expected 10+20=30; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 15 — single_body_fn_regression ─────────────────────────────────────

#[test]
fn single_body_fn_regression() {
    match run_expr("(:t::test15-single-fn)") {
        Value::i64(n) => assert_eq!(n, 15, "expected 7+8=15; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

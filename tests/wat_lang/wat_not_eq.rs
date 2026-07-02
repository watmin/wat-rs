//! Arc 056 carry-along — `:wat::core::not=` + Enum equality.
//!
//! Clojure-tradition inequality. Shares the polymorphic-compare
//! inference rules with `=`; the runtime is `not(=)`. Also fills the
//! prior gap where `=` couldn't compare two `Value::Enum` values
//! (added an Enum arm to `values_equal`).

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn unwrap_bool(v: Value) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

#[test]
fn not_eq_i64_true_when_different() {
    assert!(unwrap_bool(run_expr("(:t::test1-not-eq-true)")));
}

#[test]
fn not_eq_i64_false_when_same() {
    assert!(!unwrap_bool(run_expr("(:t::test2-not-eq-false)")));
}

#[test]
fn not_eq_f64_cross_numeric_coerce() {
    // Arc-237 Stone 237.8a: cross-numeric coercion for equality DELETED.
    // `(:wat::core::not= 3 3.0)` is now a TypeMismatch (same-type-only
    // relational intrinsic). Startup must fail with a type check error.
    let result = startup_from_file("tests/wat_lang/wat_not_eq_cross_numeric_bad.wat");
    assert!(
        result.is_err(),
        "expected cross-numeric not= to produce a type error; got Ok"
    );
    let msg = format!("{:?}", result.unwrap_err());
    assert_eq!(
        msg,
        r##"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_not_eq_cross_numeric_bad.wat", line: 4, col: 23, end_line: 4, end_col: 26 }, kind: TypeMismatch { callee: ":wat::core::not=", param: "#2", expected: ":wat::core::i64", got: ":wat::core::f64" } }]))"##,
        "expected TypeMismatch on cross-numeric not= call"
    );
}

#[test]
fn eq_on_enum_unit_variants() {
    assert!(
        unwrap_bool(run_expr("(:t::test4-enum-eq)")),
        "expected enum eq/not= to return true"
    );
}

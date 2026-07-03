//! Arc 049 — newtype value support. End-to-end coverage of:
//! - Constructor `:Type/new(value)` round-trip
//! - Accessor `:Type/0(self)` returns the inner value
//! - Nominal distinction enforced by the type checker
//!   (cannot mix newtype with its inner type)
//! - Newtype as a struct field round-trip
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main/stdout-capture to
//! eval_in_frozen. Check-error tests use :my::probe + canonical nil main.

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

fn run_expecting_check_error(path: &str) -> String {
    let err = startup_from_file(path).expect_err("startup should fail with check error");
    format!("{:?}", err)
}

// ─── Construct + access round-trip ────────────────────────────────────

#[test]
fn newtype_construct_and_accessor_roundtrip() {
    match run("tests/types/newtype_construct_and_accessor_roundtrip.wat") {
        Value::String(s) => assert_eq!(&*s, "100", "expected '100'; got {}", s),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Nominal distinction in argument position ─────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn newtype_rejects_inner_type_at_arg_position() {
    let err = run_expecting_check_error("tests/types/newtype_rejects_inner_type_bad.wat");
    assert_eq!(err, r##"Check(CheckErrors([CheckError { span: Span { file: "tests/types/newtype_rejects_inner_type_bad.wat", line: 4, col: 77, end_line: 4, end_col: 82 }, kind: TypeMismatch { callee: ":my::trading::pretty", param: "#1", expected: ":my::trading::Price", got: ":wat::core::f64" } }]))"##);
}

// ─── Inverse: newtype rejected where inner expected ───────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn newtype_rejected_where_inner_expected() {
    let err = run_expecting_check_error("tests/types/newtype_rejected_where_inner_expected_bad.wat");
    assert_eq!(err, r##"Check(CheckErrors([CheckError { span: Span { file: "tests/types/newtype_rejected_where_inner_expected_bad.wat", line: 6, col: 32, end_line: 6, end_col: 33 }, kind: TypeMismatch { callee: ":wat::core::f64::+", param: "#1", expected: ":wat::core::f64", got: ":my::trading::Price" } }]))"##);
}

// ─── Newtype as struct field round-trip ────────────────────────────────

#[test]
fn newtype_as_struct_field_roundtrip() {
    match run("tests/types/newtype_as_struct_field_roundtrip.wat") {
        Value::String(s) => assert_eq!(&*s, "99.5", "expected '99.5'; got {}", s),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Two distinct newtypes over the same inner stay distinct ──────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn distinct_newtypes_over_same_inner_are_distinct_types() {
    let err = run_expecting_check_error("tests/types/newtype_distinct_newtypes_bad.wat");
    assert_eq!(err, r##"Check(CheckErrors([CheckError { span: Span { file: "tests/types/newtype_distinct_newtypes_bad.wat", line: 8, col: 33, end_line: 8, end_col: 34 }, kind: TypeMismatch { callee: ":my::trading::price-pretty", param: "#1", expected: ":my::trading::Price", got: ":my::trading::Amount" } }]))"##);
}

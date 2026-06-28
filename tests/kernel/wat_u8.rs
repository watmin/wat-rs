//! End-to-end tests for `:wat::core::u8` — arc 008 slice 1.
//!
//! Covers:
//! - `(:wat::core::u8 <i64>)` cast: in-range produces `:wat::core::u8`; out-of-range errors.
//! - Comparison (`:wat::core::=`) works on `:wat::core::u8` values.
//! - `:wat::core::Vector<u8>` construction via `(:wat::core::Vector :wat::core::u8 ...)` round-trips.
//! - Passing `:wat::core::u8` values through function parameters and return types.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_fn(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let call = format!("({fn_name})");
    let ast = wat::parse_one!(&call).expect("parse compute call");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

#[test]
fn u8_cast_from_i64_in_range_succeeds() {
    match run_fn(":my::compute-u8-42") {
        Value::u8(42) => {}
        other => panic!("expected u8(42); got {:?}", other),
    }
}

#[test]
fn u8_cast_boundary_values() {
    // 0 and 255 are the edges of :wat::core::u8's range.
    assert!(matches!(run_fn(":my::compute-u8-zero"), Value::u8(0)));
    assert!(matches!(run_fn(":my::compute-u8-max"), Value::u8(255)));
}

#[test]
fn u8_cast_out_of_range_errors_at_runtime() {
    // 256 is one past :wat::core::u8 max — runtime should reject.
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:my::compute-u8-256)").expect("parse compute call");
    let err = eval_in_frozen(&ast, &world, &Environment::new())
        .expect_err("expected runtime error");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("u8") && msg.contains("256"),
        "expected out-of-range u8 error mentioning 256; got {:?}",
        err
    );
}

#[test]
fn u8_cast_negative_errors_at_runtime() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:my::compute-u8-neg1)").expect("parse compute call");
    let err = eval_in_frozen(&ast, &world, &Environment::new())
        .expect_err("expected runtime error");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("u8") && msg.contains("-1"),
        "expected out-of-range u8 error mentioning -1; got {:?}",
        err
    );
}

#[test]
fn u8_equality_works() {
    assert!(matches!(run_fn(":my::compute-u8-eq"), Value::bool(true)));
}

#[test]
fn u8_inequality_works() {
    assert!(matches!(run_fn(":my::compute-u8-neq"), Value::bool(false)));
}

#[test]
fn vec_u8_construction_round_trips() {
    // (:wat::core::Vector :wat::core::u8 0 65 127 255) — cast each from i64 literal.
    match run_fn(":my::compute-vec-u8") {
        Value::Vec(items) => {
            assert_eq!(items.len(), 4);
            for (i, v) in items.iter().enumerate() {
                let expected = [0u8, 65, 127, 255][i];
                match v {
                    Value::u8(b) => assert_eq!(*b, expected),
                    other => panic!("expected u8 at index {}; got {:?}", i, other),
                }
            }
        }
        other => panic!("expected Vec; got {:?}", other),
    }
}

#[test]
fn u8_type_mismatch_rejected_at_check_time() {
    // Passing :wat::core::i64 directly where :wat::core::u8 is expected should fail type
    // check — not silently coerce.
    let result = startup_from_file("tests/kernel/wat_u8_bad.wat");
    assert!(
        result.is_err(),
        "expected type check to reject :wat::core::i64 literal where :wat::core::u8 was expected"
    );
}

#[test]
fn u8_parameter_and_return_roundtrip() {
    // A function that takes :wat::core::u8 and returns :wat::core::u8 (identity). Caller
    // provides a properly-cast :wat::core::u8 value. Both sides type-check.
    assert!(matches!(run_fn(":my::compute-identity"), Value::u8(100)));
}

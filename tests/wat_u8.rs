//! End-to-end tests for `:u8` — arc 008 slice 1.
//!
//! Covers:
//! - `(:wat::core::u8 <i64>)` cast: in-range produces `:u8`; out-of-range errors.
//! - Comparison (`:wat::core::=`) works on `:u8` values.
//! - `:Vec<u8>` construction via `(:wat::core::vec :u8 ...)` round-trips.
//! - Passing `:u8` values through function parameters and return types.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

#[test]
fn u8_cast_from_i64_in_range_succeeds() {
    let src = r#"

        (:wat::core::define (:user::main -> :u8)
          (:wat::core::u8 42))
    "#;
    match run(src) {
        Value::u8(42) => {}
        other => panic!("expected u8(42); got {:?}", other),
    }
}

#[test]
fn u8_cast_boundary_values() {
    // 0 and 255 are the edges of :u8's range.
    let src_zero = r#"

        (:wat::core::define (:user::main -> :u8)
          (:wat::core::u8 0))
    "#;
    assert!(matches!(run(src_zero), Value::u8(0)));

    let src_max = r#"

        (:wat::core::define (:user::main -> :u8)
          (:wat::core::u8 255))
    "#;
    assert!(matches!(run(src_max), Value::u8(255)));
}

#[test]
fn u8_cast_out_of_range_errors_at_runtime() {
    // 256 is one past :u8 max — runtime should reject.
    let src = r#"

        (:wat::core::define (:user::main -> :u8)
          (:wat::core::u8 256))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let err = invoke_user_main(&world, Vec::new()).expect_err("expected runtime error");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("u8") && msg.contains("256"),
        "expected out-of-range u8 error mentioning 256; got {:?}",
        err
    );
}

#[test]
fn u8_cast_negative_errors_at_runtime() {
    let src = r#"

        (:wat::core::define (:user::main -> :u8)
          (:wat::core::u8 -1))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let err = invoke_user_main(&world, Vec::new()).expect_err("expected runtime error");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("u8") && msg.contains("-1"),
        "expected out-of-range u8 error mentioning -1; got {:?}",
        err
    );
}

#[test]
fn u8_equality_works() {
    let src = r#"

        (:wat::core::define (:user::main -> :bool)
          (:wat::core::= (:wat::core::u8 10) (:wat::core::u8 10)))
    "#;
    assert!(matches!(run(src), Value::bool(true)));
}

#[test]
fn u8_inequality_works() {
    let src = r#"

        (:wat::core::define (:user::main -> :bool)
          (:wat::core::= (:wat::core::u8 10) (:wat::core::u8 11)))
    "#;
    assert!(matches!(run(src), Value::bool(false)));
}

#[test]
fn vec_u8_construction_round_trips() {
    // (:wat::core::vec :u8 0 65 127 255) — cast each from i64 literal.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<wat::core::u8>)
          (:wat::core::vec :wat::core::u8
            (:wat::core::u8 0)
            (:wat::core::u8 65)
            (:wat::core::u8 127)
            (:wat::core::u8 255)))
    "#;
    match run(src) {
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
    // Passing :i64 directly where :u8 is expected should fail type
    // check — not silently coerce.
    let src = r#"

        (:wat::core::define (:my::app::byte-taker (b :u8) -> :u8) b)

        (:wat::core::define (:user::main -> :u8)
          (:my::app::byte-taker 42))
    "#;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "expected type check to reject :i64 literal where :u8 was expected"
    );
}

#[test]
fn u8_parameter_and_return_roundtrip() {
    // A function that takes :u8 and returns :u8 (identity). Caller
    // provides a properly-cast :u8 value. Both sides type-check.
    let src = r#"

        (:wat::core::define (:my::app::identity (b :u8) -> :u8) b)

        (:wat::core::define (:user::main -> :u8)
          (:my::app::identity (:wat::core::u8 100)))
    "#;
    assert!(matches!(run(src), Value::u8(100)));
}

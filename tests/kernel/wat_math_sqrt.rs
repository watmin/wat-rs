//! Carry-along — `:wat::std::math::sqrt`.
//!
//! Surfaced by holon-lab-trading arc 026 slice 4 (Bollinger's
//! RollingStddev needs `var.sqrt()`). Same shape as ln/exp/sin/cos —
//! single-method f64 unary; mirrors the existing dispatch.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_fn(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let call = format!("({fn_name})");
    let ast = wat::parse_one!(&call).expect("parse compute call");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn assert_str(val: Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(
            &*s, expected,
            "expected String({expected:?}); got String({s:?})"
        ),
        other => panic!("expected String({expected:?}); got {:?}", other),
    }
}

#[test]
fn sqrt_perfect_square() {
    assert_str(run_fn(":my::compute-perfect-square"), "4");
}

#[test]
fn sqrt_of_zero() {
    assert_str(run_fn(":my::compute-sqrt-zero"), "0");
}

#[test]
fn sqrt_round_trip_with_square() {
    assert_str(run_fn(":my::compute-round-trip"), "7.5");
}

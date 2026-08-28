//! Carry-along — `:wat::math::sqrt` (arc 255 Stone HOME-9 — moved off the dead `:wat::std::`
//! namespace).
//!
//! Surfaced by holon-lab-trading arc 026 slice 4 (Bollinger's
//! RollingStddev needs `var.sqrt()`). Same shape as ln/exp/sin/cos —
//! single-method f64 unary; mirrors the existing dispatch.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
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

//! `:wat::stat::*` — mean, variance, stddev (arc 255 Stone HOME-9 — moved off the dead
//! `:wat::std::` namespace).
//!
//! Surfaced by holon-lab-trading arc 026 slice 9 + slice 10 (Hurst
//! R/S, DFA, variance ratio all want windowed stats). Universal
//! enough to live in core stdlib. Population convention (numpy
//! default `ddof=0`); :wat::core::Option<wat::core::f64> for all three with None on empty
//! input (matches f64::min-of / max-of's reduction-empty pattern).

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
fn mean_known_input() {
    assert_str(run_fn(":my::compute-mean-known"), "3");
}

#[test]
fn mean_empty_is_none() {
    assert_str(run_fn(":my::compute-mean-empty"), "none");
}

#[test]
fn variance_population_known_input() {
    // {1, 2, 3, 4, 5}: mean=3, var = ((1-3)² + (2-3)² + 0 + (4-3)² + (5-3)²) / 5
    //                       = (4+1+0+1+4)/5 = 2.0.
    assert_str(run_fn(":my::compute-variance-known"), "2");
}

#[test]
fn variance_single_point_zero() {
    assert_str(run_fn(":my::compute-variance-single"), "0");
}

#[test]
fn stddev_known_input() {
    // {1, 2, 3, 4, 5}: variance=2, stddev = sqrt(2) ≈ 1.4142...
    assert_str(run_fn(":my::compute-stddev-known"), "ok");
}

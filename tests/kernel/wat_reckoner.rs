//! Arc 053 slice 3 — Reckoner as native wat value.

use wat::freeze::call_beside;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("eval should succeed")
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
fn reckoner_discrete_construct_dims_labels() {
    assert_str(run_fn(":my::compute-discrete-dims-labels"), "ok");
}

#[test]
fn reckoner_observe_then_predict() {
    assert_str(run_fn(":my::compute-observe-predict"), "ok");
}

#[test]
fn reckoner_continuous_construct() {
    assert_str(run_fn(":my::compute-continuous-construct"), "ok");
}

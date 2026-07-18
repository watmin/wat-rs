//! Arc 053 slices 4 + 5 — Engram + EngramLibrary as native wat values.

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
fn library_construct_empty() {
    assert_str(run_fn(":my::compute-empty"), "empty");
}

#[test]
fn library_add_subspace_then_count() {
    assert_str(run_fn(":my::compute-add-count"), "ok");
}

#[test]
fn library_match_returns_named_pairs() {
    assert_str(run_fn(":my::compute-match"), "one-match");
}

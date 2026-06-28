//! Arc 053 slices 4 + 5 — Engram + EngramLibrary as native wat values.

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

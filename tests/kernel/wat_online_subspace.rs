//! Arc 053 slice 2 — OnlineSubspace as native wat value.

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
fn subspace_construct_dim_k_n_zero() {
    assert_str(run_fn(":my::compute-construct"), "ok");
}

#[test]
fn subspace_update_increments_n_and_returns_residual() {
    assert_str(run_fn(":my::compute-update"), "incremented");
}

#[test]
fn subspace_eigenvalues_returns_k_floats() {
    assert_str(run_fn(":my::compute-eigenvalues"), "k-eigs");
}

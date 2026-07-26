//! FM 2-bis probe — arc 251 Stone 251.5a-iv: `with-children`, the kind-preserving REBUILD.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_with_children`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed bool.
fn eval_bool(fn_name: &str) -> Result<bool, String> {
    match call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::bool(b) => Ok(b),
        other => Err(format!("non-bool: {other:?}")),
    }
}

#[test]
fn contract_01_kind_preserved_vector_stays_non_list() {
    assert_eq!(
        eval_bool(":user::c01"),
        Ok(true),
        "a Vector node, decomposed and rebuilt via with-children, stays a Vector (not a List)"
    );
}

#[test]
fn contract_02_list_stays_list() {
    assert_eq!(
        eval_bool(":user::c02"),
        Ok(true),
        "a List node, decomposed and rebuilt via with-children, stays a List"
    );
}

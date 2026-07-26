//! FM 2-bis probe — arc 251 Stone 251.5a-i: the homoiconic `read`.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_read_string`

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
fn contract_01_read_string_returns_walkable_forms() {
    assert_eq!(
        eval_bool(":user::c01"),
        Ok(true),
        "read-string must return a forms-List the macro engine can walk (List? recognizes it)"
    );
}

#[test]
fn contract_02_read_string_reads_the_dirty_surface() {
    assert_eq!(
        eval_bool(":user::c02"),
        Ok(true),
        "read-string must read the dirty pre-251.5 surface (Vector<…>) the EDN reader can't"
    );
}

//! FM 2-bis probe — arc 251 Stone 251.5a-iii: the AST↔walkable bridge.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_ast_bridge`

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside` and inspect the returned typed bool.
fn eval_bool(fn_name: &str) -> Result<bool, String> {
    match call_beside(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::bool(b) => Ok(b),
        other => Err(format!("non-bool: {other:?}")),
    }
}

#[test]
fn contract_01_ast_children_is_walkable() {
    assert_eq!(
        eval_bool(":user::c01"),
        Ok(true),
        "ast->children yields a Vector the first/map vocab walks"
    );
}

#[test]
fn contract_02_recursion_works() {
    assert_eq!(
        eval_bool(":user::c02"),
        Ok(true),
        "ast->children of an ast->children result still walks — recursion is expressible in wat"
    );
}

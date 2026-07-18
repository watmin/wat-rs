//! FM 2-bis probe — arc 251 Stone 251.5a-v: node recognition + construction.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_recognition`

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
fn contract_01_ast_name_reads_symbol_verbatim() {
    assert_eq!(eval_bool(":user::c01"), Ok(true), "ast-name reads a bare Symbol node's text verbatim");
}

#[test]
fn contract_02_ast_kind_discriminates_keyword() {
    assert_eq!(eval_bool(":user::c02"), Ok(true), "ast-kind discriminates a Keyword node");
}

#[test]
fn contract_03_symbol_node_roundtrips() {
    assert_eq!(eval_bool(":user::c03"), Ok(true), "symbol-node constructs a Symbol whose ast-name is the input string");
}

#[test]
fn contract_04_keyword_node_roundtrips() {
    assert_eq!(eval_bool(":user::c04"), Ok(true), "keyword-node constructs a Keyword whose ast-name is the (':'-prefixed) input");
}

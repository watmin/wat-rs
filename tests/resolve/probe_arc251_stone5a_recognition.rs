//! FM 2-bis probe — arc 251 Stone 251.5a-v: node recognition + construction.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_recognition`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_bool(world: &wat::freeze::FrozenWorld, call: &str) -> Result<bool, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::bool(b) => Ok(b),
        other => Err(format!("non-bool: {other:?}")),
    }
}

#[test]
fn contract_01_ast_name_reads_symbol_verbatim() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_bool(&world, "(:user::c01)"), Ok(true), "ast-name reads a bare Symbol node's text verbatim");
}

#[test]
fn contract_02_ast_kind_discriminates_keyword() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_bool(&world, "(:user::c02)"), Ok(true), "ast-kind discriminates a Keyword node");
}

#[test]
fn contract_03_symbol_node_roundtrips() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_bool(&world, "(:user::c03)"), Ok(true), "symbol-node constructs a Symbol whose ast-name is the input string");
}

#[test]
fn contract_04_keyword_node_roundtrips() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_bool(&world, "(:user::c04)"), Ok(true), "keyword-node constructs a Keyword whose ast-name is the (':'-prefixed) input");
}

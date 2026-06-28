//! FM 2-bis probe — arc 251 Stone 251.5a-iii: the AST↔walkable bridge.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_ast_bridge`

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
fn contract_01_ast_children_is_walkable() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_bool(&world, "(:user::c01)"),
        Ok(true),
        "ast->children yields a Vector the first/map vocab walks"
    );
}

#[test]
fn contract_02_recursion_works() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_bool(&world, "(:user::c02)"),
        Ok(true),
        "ast->children of an ast->children result still walks — recursion is expressible in wat"
    );
}

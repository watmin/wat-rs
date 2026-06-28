//! FM 2-bis probe — arc 251 Stone 251.5a-i: the homoiconic `read`.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_read_string`

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
fn contract_01_read_string_returns_walkable_forms() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_bool(&world, "(:user::c01)"),
        Ok(true),
        "read-string must return a forms-List the macro engine can walk (List? recognizes it)"
    );
}

#[test]
fn contract_02_read_string_reads_the_dirty_surface() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_bool(&world, "(:user::c02)"),
        Ok(true),
        "read-string must read the dirty pre-251.5 surface (Vector<…>) the EDN reader can't"
    );
}

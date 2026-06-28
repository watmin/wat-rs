//! FM 2-bis probe — arc 251 Stone 251.5a-iv: `with-children`, the kind-preserving REBUILD.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_with_children`

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
fn contract_01_kind_preserved_vector_stays_non_list() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_bool(&world, "(:user::c01)"),
        Ok(true),
        "a Vector node, decomposed and rebuilt via with-children, stays a Vector (not a List)"
    );
}

#[test]
fn contract_02_list_stays_list() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_bool(&world, "(:user::c02)"),
        Ok(true),
        "a List node, decomposed and rebuilt via with-children, stays a List"
    );
}

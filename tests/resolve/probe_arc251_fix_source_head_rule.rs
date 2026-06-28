//! FM 2-bis probe — arc 251 fix-source head-rule: the head role-inversion in the WALK.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_head_rule`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_string(world: &wat::freeze::FrozenWorld, call: &str) -> Result<String, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn contract_01_bare_call_head_inverted() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c01)"), Ok("(wat.core/map f xs)".into()));
}

#[test]
fn contract_02_strip_and_head_compose() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c02)"),
        Ok("(wat.core/if true 1 2)".into()),
        "the annotation is stripped AND the if-head is inverted, in one pass"
    );
}

#[test]
fn contract_03_recurses_into_nested_heads() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c03)"),
        Ok("(wat.core/do (wat.core/first xs))".into()),
        "the nested call's head is inverted too"
    );
}

#[test]
fn contract_04_data_keyword_head_not_converted() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c04)"), Ok("(:else 1)".into()));
}

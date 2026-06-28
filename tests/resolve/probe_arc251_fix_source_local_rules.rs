//! Strike 3 (examinare disconfirming probe) — fix-source's position-aware LOCAL rules.
//!
//! RED at HEAD: fix.wat only does {strip-if, head-rule} — no arrows, no post-arrow/structural types.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_local_rules`

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
fn contract_01_arrow_in_binder() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c01)"), Ok("[x :- y]".into()));
}

#[test]
fn contract_02_post_arrow_scalar_type() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c02)"), Ok("[x :- wat.type/i64]".into()));
}

#[test]
fn contract_03_structural_parametric_type() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c03)"),
        Ok("[x :- (wat.type/Vector wat.type/i64)]".into())
    );
}

#[test]
fn contract_04_head_still_inverts() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c04)"), Ok("(wat.core/map f xs)".into()));
}

#[test]
fn contract_05_full_fn_literal() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c05)"),
        Ok("(wat.core/fn [a :- wat.type/i64] :- wat.type/bool a)".into()),
        "head inverts, binder + return arrows -> :-, both types -> wat.type/, in one pass"
    );
}

#[test]
fn contract_06_less_than_operator_is_not_a_type() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c06a)"), Ok("(wat.core/< a b)".into()));
    assert_eq!(eval_string(&world, "(:user::c06b)"), Ok("(wat.core/<= a b)".into()));
}

#[test]
fn contract_07_greater_than_operator() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c07)"), Ok("(wat.core/> a b)".into()));
}

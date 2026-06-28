//! Strike 2 (examinare disconfirming probe) — `keyword/to-type-form`: the type-converter.
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-type-form` does not exist (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_type_form`

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
fn contract_01_scalar() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c01a)"), Ok("wat.type/i64".into()));
    assert_eq!(eval_string(&world, "(:user::c01b)"), Ok("wat.holon/HolonAST".into()));
}

#[test]
fn contract_02_parametric() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c02)"),
        Ok("(wat.type/Vector wat.type/i64)".into())
    );
}

#[test]
fn contract_03_nested_parametric() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c03)"),
        Ok("(wat.type/Vector (wat.type/Vector wat.type/i64))".into())
    );
}

#[test]
fn contract_04_type_var_stays_bare() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c04)"),
        Ok("(wat.type/Vector T)".into()),
        "a type-var (Path with no `::`) renders as a bare symbol, not wat.type/T"
    );
}

#[test]
fn contract_05_multi_arg() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c05)"),
        Ok("(wat.type/HashMap wat.type/String wat.type/i64)".into())
    );
}

#[test]
fn contract_06_tuple() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c06)"),
        Ok("(wat.type/Tuple wat.type/i64 wat.type/String)".into())
    );
}

#[test]
fn contract_07_empty_tuple_is_not_nil() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c07)"), Ok("(wat.type/Tuple)".into()));
}

#[test]
fn contract_08_nested_tuple() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c08)"),
        Ok("(wat.type/Tuple (wat.type/Vector T) wat.type/i64)".into())
    );
}

#[test]
fn contract_09_tuple_form_round_trips_as_a_type() {
    // The c09-f defn in the fixture uses `(wat.type/Tuple …)` as a param type.
    // Startup succeeding proves the parser handles it as a tuple type.
    let r = startup_beside(file!());
    assert!(r.is_ok(), "(wat.type/Tuple wat.type/i64 wat.type/String) must parse as a type; got {r:?}");
}

//! FM 2-bis probe — arc 251 head role-inversion: `keyword/to-symbol`.
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-symbol` does not exist yet (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_symbol`

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
fn contract_01_simple_head() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::convert-c01a)"), Ok("wat.core/if".into()));
    assert_eq!(eval_string(&world, "(:user::convert-c01b)"), Ok("wat.holon/HolonAST".into()));
    assert_eq!(eval_string(&world, "(:user::convert-c01c)"), Ok("user/main".into()));
}

#[test]
fn contract_02_division_is_clojure_core_slashslash() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::convert-c02a)"), Ok("wat.core//".into()));
    assert_eq!(eval_string(&world, "(:user::convert-c02b)"), Ok("wat.core/+".into()));
}

#[test]
fn contract_03_type_method_folds_type_into_namespace() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::convert-c03a)"), Ok("wat.core.Option/expect".into()));
    assert_eq!(eval_string(&world, "(:user::convert-c03b)"), Ok("wat.core.HashMap/dissoc".into()));
}

#[test]
fn contract_04_deep_and_nested() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::convert-c04a)"),
        Ok("wat.kernel.services.StdErrService/handle".into())
    );
    assert_eq!(
        eval_string(&world, "(:user::convert-c04b)"),
        Ok("wat.kernel.services.StdErrService.Rep/new".into())
    );
}

#[test]
fn contract_05_result_is_a_symbol_not_a_keyword() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c05)"),
        Ok("symbol".into()),
        "the converted head is a Symbol node (a call head), not a Keyword"
    );
}

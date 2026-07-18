//! Strike 2 (examinare disconfirming probe) — `keyword/to-type-form`: the type-converter.
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-type-form` does not exist (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_type_form`

use wat::freeze::{call_beside, startup_beside};
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside` and inspect the returned typed String.
fn eval_string(fn_name: &str) -> Result<String, String> {
    match call_beside(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn contract_01_scalar() {
    assert_eq!(
        eval_string(":user::c01a"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-01a-scalar-i64.wat").into())
    );
    assert_eq!(
        eval_string(":user::c01b"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-01b-scalar-user.wat").into())
    );
}

#[test]
fn contract_02_parametric() {
    assert_eq!(
        eval_string(":user::c02"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-02-parametric.wat").into())
    );
}

#[test]
fn contract_03_nested_parametric() {
    assert_eq!(
        eval_string(":user::c03"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-03-nested-parametric.wat").into())
    );
}

#[test]
fn contract_04_type_var_stays_bare() {
    assert_eq!(
        eval_string(":user::c04"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-04-type-var-bare.wat").into()),
        "a type-var (Path with no `::`) renders as a bare symbol, not wat.type/T"
    );
}

#[test]
fn contract_05_multi_arg() {
    assert_eq!(
        eval_string(":user::c05"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-05-multi-arg.wat").into())
    );
}

#[test]
fn contract_06_tuple() {
    assert_eq!(
        eval_string(":user::c06"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-06-tuple.wat").into())
    );
}

#[test]
fn contract_07_empty_tuple_is_not_nil() {
    assert_eq!(
        eval_string(":user::c07"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-07-empty-tuple.wat").into())
    );
}

#[test]
fn contract_08_nested_tuple() {
    assert_eq!(
        eval_string(":user::c08"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-08-nested-tuple.wat").into())
    );
}

#[test]
fn contract_09_tuple_form_round_trips_as_a_type() {
    // The c09-f defn in the fixture uses `(wat.type/Tuple …)` as a param type.
    // Startup succeeding proves the parser handles it as a tuple type.
    let r = startup_beside(file!());
    assert!(r.is_ok(), "(wat.type/Tuple wat.type/i64 wat.type/String) must parse as a type; got {r:?}");
}

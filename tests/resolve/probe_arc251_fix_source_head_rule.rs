//! FM 2-bis probe — arc 251 fix-source head-rule: the head role-inversion in the WALK.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_head_rule`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed String.
fn eval_string(fn_name: &str) -> Result<String, String> {
    match call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn contract_01_bare_call_head_inverted() {
    assert_eq!(
        eval_string(":user::c01"),
        Ok(include_str!("probe_arc251_fix_source_head_rule__contract-01-bare-call-head-inverted.wat").into())
    );
}

#[test]
fn contract_02_strip_and_head_compose() {
    assert_eq!(
        eval_string(":user::c02"),
        Ok(include_str!("probe_arc251_fix_source_head_rule__contract-02-strip-and-head-compose.wat").into()),
        "the annotation is stripped AND the if-head is inverted, in one pass"
    );
}

#[test]
fn contract_03_recurses_into_nested_heads() {
    assert_eq!(
        eval_string(":user::c03"),
        Ok(include_str!("probe_arc251_fix_source_head_rule__contract-03-nested-heads.wat").into()),
        "the nested call's head is inverted too"
    );
}

#[test]
fn contract_04_data_keyword_head_not_converted() {
    assert_eq!(
        eval_string(":user::c04"),
        Ok(include_str!("probe_arc251_fix_source_head_rule__contract-04-data-keyword-head.wat").into())
    );
}

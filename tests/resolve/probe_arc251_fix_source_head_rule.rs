//! FM 2-bis probe — arc 251 fix-source head-rule: the head role-inversion in the WALK.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_head_rule`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed String.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; the "wrong Value shape" arm is minted as the same `RuntimeErrorKind::TypeMismatch`
// the runtime itself raises for this shape (see `src/assertion.rs::eval_opt_string`).
fn eval_string(fn_name: &str) -> Result<String, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn contract_01_bare_call_head_inverted() {
    assert_eq!(
        eval_string(":user::c01").expect("eval_string"),
        include_str!("probe_arc251_fix_source_head_rule__contract-01-bare-call-head-inverted.wat")
    );
}

#[test]
fn contract_02_strip_and_head_compose() {
    assert_eq!(
        eval_string(":user::c02").expect("eval_string"),
        include_str!("probe_arc251_fix_source_head_rule__contract-02-strip-and-head-compose.wat"),
        "the annotation is stripped AND the if-head is inverted, in one pass"
    );
}

#[test]
fn contract_03_recurses_into_nested_heads() {
    assert_eq!(
        eval_string(":user::c03").expect("eval_string"),
        include_str!("probe_arc251_fix_source_head_rule__contract-03-nested-heads.wat"),
        "the nested call's head is inverted too"
    );
}

#[test]
fn contract_04_data_keyword_head_not_converted() {
    assert_eq!(
        eval_string(":user::c04").expect("eval_string"),
        include_str!("probe_arc251_fix_source_head_rule__contract-04-data-keyword-head.wat")
    );
}

//! FM 2-bis probe — arc 251 head role-inversion: `keyword/to-symbol`.
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-symbol` does not exist yet (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_symbol`

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:user::…` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside` and inspect the returned typed String.
fn eval_string(fn_name: &str) -> Result<String, String> {
    match call_beside(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn contract_01_simple_head() {
    assert_eq!(eval_string(":user::convert-c01a"), Ok("wat.core/if".into()));
    assert_eq!(eval_string(":user::convert-c01b"), Ok("wat.holon/HolonAST".into()));
    assert_eq!(eval_string(":user::convert-c01c"), Ok("user/main".into()));
}

#[test]
fn contract_02_division_is_clojure_core_slashslash() {
    assert_eq!(eval_string(":user::convert-c02a"), Ok("wat.core//".into()));
    assert_eq!(eval_string(":user::convert-c02b"), Ok("wat.core/+".into()));
}

#[test]
fn contract_03_type_method_folds_type_into_namespace() {
    assert_eq!(eval_string(":user::convert-c03a"), Ok("wat.core.Option/expect".into()));
    assert_eq!(eval_string(":user::convert-c03b"), Ok("wat.core.HashMap/dissoc".into()));
}

#[test]
fn contract_04_deep_and_nested() {
    assert_eq!(
        eval_string(":user::convert-c04a"),
        Ok("wat.kernel.services.StdErrService/handle".into())
    );
    assert_eq!(
        eval_string(":user::convert-c04b"),
        Ok("wat.kernel.services.StdErrService.Rep/new".into())
    );
}

#[test]
fn contract_05_result_is_a_symbol_not_a_keyword() {
    assert_eq!(
        eval_string(":user::c05"),
        Ok("symbol".into()),
        "the converted head is a Symbol node (a call head), not a Keyword"
    );
}

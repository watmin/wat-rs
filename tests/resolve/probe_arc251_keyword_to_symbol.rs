//! FM 2-bis probe — arc 251 head role-inversion: `keyword/to-symbol`.
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-symbol` does not exist yet (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_symbol`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::…` zero-arg fn lives in the co-located fixture;
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
fn contract_01_simple_head() {
    assert_eq!(eval_string(":user::convert-c01a").expect("eval_string"), "wat.core/if");
    assert_eq!(eval_string(":user::convert-c01b").expect("eval_string"), "wat.holon/HolonAST");
    assert_eq!(eval_string(":user::convert-c01c").expect("eval_string"), "user/main");
}

#[test]
fn contract_02_division_is_clojure_core_slashslash() {
    assert_eq!(eval_string(":user::convert-c02a").expect("eval_string"), "wat.core//");
    assert_eq!(eval_string(":user::convert-c02b").expect("eval_string"), "wat.core/+");
}

#[test]
fn contract_03_type_method_folds_type_into_namespace() {
    assert_eq!(eval_string(":user::convert-c03a").expect("eval_string"), "wat.core.Option/expect");
    assert_eq!(eval_string(":user::convert-c03b").expect("eval_string"), "wat.core.HashMap/dissoc");
}

#[test]
fn contract_04_deep_and_nested() {
    assert_eq!(
        eval_string(":user::convert-c04a").expect("eval_string"),
        "wat.kernel.services.StdErrService/handle"
    );
    assert_eq!(
        eval_string(":user::convert-c04b").expect("eval_string"),
        "wat.kernel.services.StdErrService.Rep/new"
    );
}

#[test]
fn contract_05_result_is_a_symbol_not_a_keyword() {
    assert_eq!(
        eval_string(":user::c05").expect("eval_string"),
        "symbol",
        "the converted head is a Symbol node (a call head), not a Keyword"
    );
}

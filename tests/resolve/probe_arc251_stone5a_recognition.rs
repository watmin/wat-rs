//! FM 2-bis probe — arc 251 Stone 251.5a-v: node recognition + construction.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_recognition`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed bool.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; the "wrong Value shape" arm is minted as the same `RuntimeErrorKind::TypeMismatch`
// the runtime itself raises for this shape (see `src/assertion.rs::eval_opt_string`).
fn eval_bool(fn_name: &str) -> Result<bool, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::bool(b) => Ok(b),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "bool",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn contract_01_ast_name_reads_symbol_verbatim() {
    assert!(eval_bool(":user::c01").expect("eval_bool"), "ast-name reads a bare Symbol node's text verbatim");
}

#[test]
fn contract_02_ast_kind_discriminates_keyword() {
    assert!(eval_bool(":user::c02").expect("eval_bool"), "ast-kind discriminates a Keyword node");
}

#[test]
fn contract_03_symbol_node_roundtrips() {
    assert!(eval_bool(":user::c03").expect("eval_bool"), "symbol-node constructs a Symbol whose ast-name is the input string");
}

#[test]
fn contract_04_keyword_node_roundtrips() {
    assert!(eval_bool(":user::c04").expect("eval_bool"), "keyword-node constructs a Keyword whose ast-name is the (':'-prefixed) input");
}

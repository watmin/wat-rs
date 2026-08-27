//! FM 2-bis probe — arc 251 Stone 251.5a-iii: the AST↔walkable bridge.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_ast_bridge`

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
fn contract_01_ast_children_is_walkable() {
    assert!(
        eval_bool(":user::c01").expect("eval_bool"),
        "ast->children yields a Vector the first/map vocab walks"
    );
}

#[test]
fn contract_02_recursion_works() {
    assert!(
        eval_bool(":user::c02").expect("eval_bool"),
        "ast->children of an ast->children result still walks — recursion is expressible in wat"
    );
}

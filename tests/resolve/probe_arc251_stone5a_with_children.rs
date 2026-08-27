//! FM 2-bis probe — arc 251 Stone 251.5a-iv: `with-children`, the kind-preserving REBUILD.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_with_children`

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
fn contract_01_kind_preserved_vector_stays_non_list() {
    assert!(
        eval_bool(":user::c01").expect("eval_bool"),
        "a Vector node, decomposed and rebuilt via with-children, stays a Vector (not a List)"
    );
}

#[test]
fn contract_02_list_stays_list() {
    assert!(
        eval_bool(":user::c02").expect("eval_bool"),
        "a List node, decomposed and rebuilt via with-children, stays a List"
    );
}

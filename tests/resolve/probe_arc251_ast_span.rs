//! Forward-proof probe — Stone 251.5 / Slice 4.2a: `ast-span` (intueri-named).
//!
//! `(:wat::core::ast-span node) -> {:line N :col N}` — a plain map (HashMap<keyword,i64>) of the
//! node's source START location.
//!
//! RED at HEAD: `:wat::core::ast-span` is UnknownFunction.
//!
//! Run: `cargo test --release --test probe_arc251_ast_span`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed i64.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; the "wrong Value shape" arm is minted as the same `RuntimeErrorKind::TypeMismatch`
// the runtime itself raises for this shape (see `src/assertion.rs::eval_opt_string`).
fn eval_i64(fn_name: &str) -> Result<i64, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn c01_ast_span_head_line() {
    assert_eq!(eval_i64(":user::c01").expect("eval_i64"), 1, "head keyword line should be 1");
}

#[test]
fn c02_ast_span_head_col() {
    assert_eq!(eval_i64(":user::c02").expect("eval_i64"), 2, "head keyword col should be 2 (just after `(`)");
}

#[test]
fn c03_ast_span_symbol_col() {
    assert_eq!(eval_i64(":user::c03").expect("eval_i64"), 18, "symbol x col should be 18");
}

//! FM 2-bis probe — arc 251 Stone 251.0/251.1: a SYMBOL head resolves to the
//! entity its keyword FQDN resolves to.
//!
//! Run: `cargo test --release --test probe_arc251_stone0_symbol_head`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::compute-cNN` zero-arg fn lives in the co-located fixture;
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

// ─── C01: THE GAP — dotted symbol head resolves like the keyword FQDN ───────────

#[test]
fn contract_01_symbol_head_resolves_like_keyword() {
    assert_eq!(
        eval_i64(":user::compute-c01").expect("eval_i64"),
        3,
        "dotted symbol head wat.core/+ must resolve to the :wat::core::+ entity"
    );
}

// ─── C02: PRESERVATION — keyword head still resolves during the transition ──────

#[test]
fn contract_02_keyword_head_still_resolves() {
    assert_eq!(
        eval_i64(":user::compute-c02").expect("eval_i64"),
        3,
        ":wat::core::+ keyword head must keep working during the transition"
    );
}

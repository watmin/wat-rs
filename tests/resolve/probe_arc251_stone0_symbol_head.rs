//! FM 2-bis probe — arc 251 Stone 251.0/251.1: a SYMBOL head resolves to the
//! entity its keyword FQDN resolves to.
//!
//! Run: `cargo test --release --test probe_arc251_stone0_symbol_head`

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:user::compute-cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside` and inspect the returned typed i64.
fn eval_i64(fn_name: &str) -> Result<i64, String> {
    match call_beside(file!(), fn_name).map_err(|e| format!("eval: {:?}", e))? {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {:?}", other)),
    }
}

// ─── C01: THE GAP — dotted symbol head resolves like the keyword FQDN ───────────

#[test]
fn contract_01_symbol_head_resolves_like_keyword() {
    assert_eq!(
        eval_i64(":user::compute-c01"),
        Ok(3),
        "dotted symbol head wat.core/+ must resolve to the :wat::core::+ entity"
    );
}

// ─── C02: PRESERVATION — keyword head still resolves during the transition ──────

#[test]
fn contract_02_keyword_head_still_resolves() {
    assert_eq!(
        eval_i64(":user::compute-c02"),
        Ok(3),
        ":wat::core::+ keyword head must keep working during the transition"
    );
}

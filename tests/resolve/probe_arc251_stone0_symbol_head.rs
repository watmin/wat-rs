//! FM 2-bis probe — arc 251 Stone 251.0/251.1: a SYMBOL head resolves to the
//! entity its keyword FQDN resolves to.
//!
//! Run: `cargo test --release --test probe_arc251_stone0_symbol_head`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_i64(world: &wat::freeze::FrozenWorld, call: &str) -> Result<i64, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {:?}", other)),
    }
}

// ─── C01: THE GAP — dotted symbol head resolves like the keyword FQDN ───────────

#[test]
fn contract_01_symbol_head_resolves_like_keyword() {
    let world = startup_beside(file!()).map_err(|e| format!("startup/check: {:?}", e)).expect("startup");
    assert_eq!(
        eval_i64(&world, "(:user::compute-c01)"),
        Ok(3),
        "dotted symbol head wat.core/+ must resolve to the :wat::core::+ entity"
    );
}

// ─── C02: PRESERVATION — keyword head still resolves during the transition ──────

#[test]
fn contract_02_keyword_head_still_resolves() {
    let world = startup_beside(file!()).map_err(|e| format!("startup/check: {:?}", e)).expect("startup");
    assert_eq!(
        eval_i64(&world, "(:user::compute-c02)"),
        Ok(3),
        ":wat::core::+ keyword head must keep working during the transition"
    );
}

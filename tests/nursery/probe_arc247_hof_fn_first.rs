//! FM-2-bis probe for Arc 247 — Clojure-honest seq-HOF order (fn-first).
//!
//! Dialect compliance: the seq-HOFs flip coll-first → fn-first (Clojure order):
//!   (map f xs)  (filter pred xs)  (foldl f init xs)  (foldr f init xs)  (sort-by keyfn xs)
//!
//! ROW STATUS (initial):
//!   - REGRESSION (GREEN at HEAD + after): variadic arithmetic uses `foldl` internally;
//!     flipping foldl's order must not change the result.
//!   - MINT-CONFIRMERS (RED at HEAD; fn-first order doesn't exist yet; `#[ignore]`'d):
//!     un-ignored by sonnet after the flip lands.
//!   - HARD-CUT confirmer (RED at HEAD; coll-first still works now; `#[ignore]`'d):
//!     after the strike, the OLD coll-first order must be a check error.
//!
//! Run: cargo test --release --test probe_arc247_hof_fn_first

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

/// Define a 0-ary `:user::compute` returning bool with `body`, eval it, return the Value.
fn eval_bool_expr(body: &str) -> Result<Value, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::bool {})",
        body
    );
    let full = with_nil_main(&src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

/// True if startup (parse + check) accepts a bool-returning compute with `body`.
fn checks_ok(body: &str) -> bool {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::bool {})",
        body
    );
    let full = with_nil_main(&src);
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new())).is_ok()
}

const INC: &str = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))";
const ADD: &str = "(:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))";
const GT1: &str = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 1))";

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — variadic arithmetic uses foldl internally; flip must preserve it.
// GREEN at HEAD and after.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_variadic_plus_via_foldl() {
    // `+` 3+-ary folds via `:wat::core::foldl` (core.wat). Result must be unchanged.
    assert_eq!(eval_bool_expr("(:wat::core::= (:wat::core::+ 1 2 3 4) 10)").unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT-CONFIRMERS — fn-first order. RED at HEAD (doesn't exist) → `#[ignore]`.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mint_map_fn_first() {
    let body = format!("(:wat::core::= (:wat::core::map {INC} [1 2 3]) [2 3 4])");
    assert_eq!(eval_bool_expr(&body).unwrap(), Value::bool(true));
}

#[test]
fn mint_filter_fn_first() {
    let body = format!("(:wat::core::= (:wat::core::filter {GT1} [1 2 3]) [2 3])");
    assert_eq!(eval_bool_expr(&body).unwrap(), Value::bool(true));
}

#[test]
fn mint_foldl_fn_first() {
    let body = format!("(:wat::core::= (:wat::core::foldl {ADD} 0 [1 2 3]) 6)");
    assert_eq!(eval_bool_expr(&body).unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// HARD-CUT confirmer — the OLD coll-first order must be GONE (a check error).
// At HEAD coll-first still works, so this `is-error` assertion fails → `#[ignore]`.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mint_map_coll_first_is_gone() {
    let body = format!("(:wat::core::= (:wat::core::map [1 2 3] {INC}) [2 3 4])");
    assert!(!checks_ok(&body), "coll-first `(map xs f)` must be a check error after the flip");
}

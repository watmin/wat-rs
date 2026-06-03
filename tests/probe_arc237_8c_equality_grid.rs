//! FM-2-bis probe for Stone 237.8c — equality grid (Shape B: per-Type leaves + structural engine).
//!
//! Recipe (DESIGN-STONE-237.8c, four-questions verdict Shape B):
//!   - Mint `:wat::core::f64::=` / `:f64::not=` as type-locked f64 aliases into the
//!     structural engine (matching the existing `:i64::=` pattern).
//!   - Keep polymorphic `=`/`not=` STRUCTURAL (values_equal-backed) — equality is the
//!     justified asymmetry (universal + recursive + subtype-compatible).
//!   - Rename `infer_comparison` -> `infer_equality`; delete the dead cross-numeric
//!     arm in `values_equal`.
//!
//! ROW STATUS (initial):
//!   - REGRESSION (preserve behavior; GREEN at HEAD): polymorphic `=`/`not=` over
//!     scalars + composites; cross-numeric `(= 1 2.0)` and cross-type `(= 1 "x")`
//!     are check errors (THE DECISION + existing infer_comparison).
//!   - MINT-CONFIRMERS (RED at HEAD; `:f64::=`/`:f64::not=` do not exist yet;
//!     `#[ignore]`'d): un-ignored by sonnet after the f64 equality leaves are minted.
//!
//! Run: cargo test --release --test probe_arc237_8c_equality_grid

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

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — preserve the polymorphic structural `=`/`not=`. GREEN at HEAD.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_eq_scalars() {
    assert_eq!(eval_bool_expr("(:wat::core::= 1 1)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::= 1 2)").unwrap(), Value::bool(false));
    assert_eq!(eval_bool_expr("(:wat::core::= 1.0 1.0)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr(r#"(:wat::core::= "a" "a")"#).unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::= true false)").unwrap(), Value::bool(false));
}

#[test]
fn regression_eq_composites_recursive() {
    // Structural recursive equality over vectors (values_equal engine).
    assert_eq!(eval_bool_expr("(:wat::core::= [1 2 3] [1 2 3])").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::= [1 2] [1 2 3])").unwrap(), Value::bool(false));
}

#[test]
fn regression_not_eq() {
    assert_eq!(eval_bool_expr("(:wat::core::not= 1 2)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::not= 1 1)").unwrap(), Value::bool(false));
}

#[test]
fn regression_cross_numeric_is_check_error() {
    // THE DECISION: `(= 1 2.0)` is a check error, not a coerced comparison.
    assert!(!checks_ok("(:wat::core::= 1 2.0)"), "cross-numeric `=` must be a check error");
}

#[test]
fn regression_cross_type_is_check_error() {
    assert!(!checks_ok(r#"(:wat::core::= 1 "x")"#), "cross-type `=` must be a check error");
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT-CONFIRMERS — `:f64::=` / `:f64::not=` do not exist at HEAD. RED → `#[ignore]`.
// Un-ignore after the f64 equality leaves are minted (Stone 237.8c).
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "Stone 237.8c: un-ignore after :wat::core::f64::= is minted"]
fn mint_f64_eq_works() {
    assert_eq!(eval_bool_expr("(:wat::core::f64::= 1.0 1.0)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::f64::= 2.0 3.0)").unwrap(), Value::bool(false));
}

#[test]
#[ignore = "Stone 237.8c: un-ignore after :wat::core::f64::= type-locking is minted"]
fn mint_f64_eq_type_locks() {
    // The per-Type leaf accepts only f64 pairs; i64 args are a check error.
    assert!(!checks_ok("(:wat::core::f64::= 1 2)"), ":f64::= must type-lock to f64");
}

#[test]
#[ignore = "Stone 237.8c: un-ignore after :wat::core::f64::not= is minted"]
fn mint_f64_not_eq_works() {
    assert_eq!(eval_bool_expr("(:wat::core::f64::not= 1.0 2.0)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::f64::not= 1.0 1.0)").unwrap(), Value::bool(false));
}

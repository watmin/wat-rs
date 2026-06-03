//! FM-2-bis probe for Stone 237.8b — recipe-lock + numeric grid via wat-defclause.
//!
//! Four LOAD-BEARING GATES (must settle empirically at HEAD before BRIEFing):
//!   Gate 1 — defclause supports `&` rest-binders in args-vec
//!   Gate 2 — defclause first-match dispatches by arg-`<-`-Type (not just :guard)
//!   Gate 3 — 0-ary clause body literal `0` infers as `:wat::core::i64`
//!   Gate 4 — per-Type ordering primitives correctness (including f64 NaN)
//!
//! Plus regression contract (preserve existing behavior) + mint-confirmers
//! (new behavior; `#[ignore]`'d at HEAD; un-ignored by sonnet post-stone).
//!
//! ROW STATUS (initial):
//!   - GATES: run at HEAD; if any fails RED, reshape strategy / defer 8b
//!     until defclause extension lands.
//!   - REGRESSION: pass at HEAD via existing `infer_arithmetic` /
//!     `eval_arithmetic_variadic` / per-Type variadic wat fns / 237.3
//!     i64 ordering aliases. Preserved post-stone via defclause.
//!   - MINT-CONFIRMERS: fail at HEAD (primitives don't exist); `#[ignore]`'d.
//!     Sonnet's stone work mints them; un-ignore after substrate edits.
//!   - CROSS-TYPE REJECTION: pass at HEAD via 8a's tightening (handler-level).
//!     Post-stone: rejection now via defclause `:NoMatchingClause`.
//!
//! Run: cargo test --release --test probe_arc237_8b_defclause_arithmetic

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

fn eval_value(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute").value_owned()
}

fn try_startup(src: &str) -> Result<(), String> {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn try_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ═══════════════════════════════════════════════════════════════════════════
// GATES — load-bearing for 8b strategy; run at HEAD to settle empirically
// ═══════════════════════════════════════════════════════════════════════════

/// Gate 1 — defclause supports `&` rest-binders in args-vec.
///
/// Minimal defclause with a single 2+-ary clause using `& rest <- :Vector<i64>`.
/// Variadic call dispatches to the rest-binder clause and folds.
///
/// **EMPIRICALLY RED AT HEAD `3e3acbbb`+** — defclause's argspec parser rejects
/// `&` as a rest-binder marker (error: *"defclause arg-vector triple at
/// position 1 must be `name <- :T`; got symbol where `<-` was expected"*).
/// This surfaces the substrate gap: **defclause needs `&` rest-binder
/// extension before 8b's recipe can use 3+-ary fold clauses.**
///
/// Strategy: **Stone 237.8b-prep** — mint defclause `&` rest-binder support
/// (parser extension + clause-matching + binding-to-Vector<T> at eval). Then
/// 8b proceeds. Un-ignore this gate after 8b-prep ships.
#[test]
fn gate_1_defclause_supports_rest_binder() {
    let src = r#"
        (:wat::core::defclause :my::sum-all
          ([first <- :wat::core::i64
            & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::foldl rest first
              (:wat::core::fn [acc <- :wat::core::i64
                               n <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+ acc n)))))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::sum-all 1 2 3 4))
    "#;
    let result = try_compute(src);
    assert!(
        result.is_ok(),
        "GATE 1: defclause must support `&` rest-binders. If RED, defclause needs extension before 8b can ship. Got: {:?}",
        result
    );
    assert_eq!(
        result.unwrap(),
        Value::i64(10),
        "GATE 1 sanity: 1+2+3+4 = 10 via & rest-binder fold"
    );
}

/// Gate 2 — defclause first-match dispatches by arg-`<-`-Type (no :guard).
///
/// Two clauses differing ONLY in arg-Type annotation. Verifies dispatch by
/// arg-type, not just by :guard expression.
#[test]
fn gate_2_defclause_dispatches_by_arg_type() {
    let src = r#"
        (:wat::core::defclause :my::label
          ([x <- :wat::core::i64] -> :wat::core::String "i64")
          ([x <- :wat::core::f64] -> :wat::core::String "f64"))
        (:wat::core::defn :user::compute [] -> :wat::core::String (:my::label 42))
    "#;
    let result = try_compute(src);
    assert!(
        result.is_ok(),
        "GATE 2: defclause must dispatch on arg-`<-`-Type. If RED, 8b strategy must reshape. Got: {:?}",
        result
    );
    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), "i64", "GATE 2: i64 clause should fire for i64 arg"),
        other => panic!("GATE 2: expected Value::String('i64'); got {:?}", other),
    }
}

/// Gate 2-cross — cross-type call to type-dispatched defclause yields :NoMatchingClause.
#[test]
fn gate_2_cross_no_matching_clause() {
    let src = r#"
        (:wat::core::defclause :my::add
          ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+ x y))
          ([x <- :wat::core::f64 y <- :wat::core::f64] -> :wat::core::f64
            (:wat::core::f64::+ x y)))
        (:wat::core::defn :user::compute [] -> :wat::core::f64 (:my::add 1 2.0))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "GATE 2-cross: (i64, f64) mixed args MUST reject (no matching clause); got: {:?}",
        result
    );
}

/// Gate 3 — 0-ary clause body literal `0` infers as `:wat::core::i64`.
///
/// The Lisp identity default `(+ )` returns 0 typed i64. If the literal `0`
/// doesn't infer as i64 in clause-body position, the 0-ary clauses need
/// explicit type ascription.
#[test]
fn gate_3_zero_ary_literal_infers_i64() {
    let src = r#"
        (:wat::core::defclause :my::default
          ([] -> :wat::core::i64 0))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::default))
    "#;
    let result = try_compute(src);
    assert!(
        result.is_ok(),
        "GATE 3: 0-ary clause `[] -> :i64 0` must type-check and dispatch. If RED, 0-ary identity needs different syntax. Got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), Value::i64(0), "GATE 3 sanity: 0-ary returns literal 0");
}

/// Gate 4a — i64 ordering primitives correctness (existing 237.3 aliases).
#[test]
fn gate_4a_i64_ordering_works() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::i64::< 1 2))"#),
        Value::bool(true),
        "i64::< 1 2 → true"
    );
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::i64::> 5 3))"#),
        Value::bool(true),
        "i64::> 5 3 → true"
    );
}

/// Gate 4b — f64 NaN handling via per-Type primitive (after mint).
///
/// **Mint-confirmer**: `:wat::core::f64::<` doesn't exist at HEAD; sonnet
/// mints it. Post-mint: `1.0 < NaN` returns false (IEEE 754).
#[test]
fn gate_4b_f64_nan_ordering() {
    // 0.0 / 0.0 produces NaN
    let src = r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::f64::< 1.0 (:wat::core::f64::/ 0.0 0.0)))"#;
    let result = try_compute(src);
    assert!(result.is_ok(), "f64::< should accept NaN at runtime; got: {:?}", result);
    assert_eq!(
        result.unwrap(),
        Value::bool(false),
        "1.0 < NaN MUST return false per IEEE 754"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — preserve existing behavior; pass at HEAD; preserved post-stone
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_arith_i64_2ary_works() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::+ 1 2))"#),
        Value::i64(3),
    );
}

#[test]
fn regression_arith_f64_2ary_works() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::f64 (:wat::core::+ 1.0 2.0))"#),
        Value::f64(3.0),
    );
}

#[test]
fn regression_arith_variadic_3args_works() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::+ 1 2 3))"#),
        Value::i64(6),
    );
}

#[test]
fn regression_arith_minus_1ary_negate_i64() {
    // 1-ary `-` is identity-on-left (negation: 0 - x)
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::- 5))"#),
        Value::i64(-5),
    );
}

#[test]
fn regression_ordering_i64_lt_works() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::< 1 2))"#),
        Value::bool(true),
    );
}

#[test]
fn regression_cross_type_plus_rejected() {
    // Per 8a tightening: cross-type rejected at check.
    // Post-8b: rejection via defclause :NoMatchingClause (same outcome).
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::f64 (:wat::core::+ 1 2.0))"#,
    );
    assert!(result.is_err(), "cross-type i64+f64 MUST reject; got: {:?}", result);
}

#[test]
fn regression_cross_type_lt_rejected() {
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::< 1 2.0))"#,
    );
    assert!(result.is_err(), "cross-type i64<f64 MUST reject; got: {:?}", result);
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT-CONFIRMERS — new behavior; fail at HEAD; un-ignore post-stone
// ═══════════════════════════════════════════════════════════════════════════

/// `:wat::core::i64::<=` MUST exist + work (currently missing from i64 set).
#[test]
fn mint_i64_lte_works() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::i64::<= 5 5))"#),
        Value::bool(true),
        "i64::<= 5 5 → true (boundary)"
    );
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::i64::<= 5 3))"#),
        Value::bool(false),
    );
}

/// `:wat::core::f64::<` etc. MUST be minted (entire f64 ordering family).
#[test]
fn mint_f64_ordering_basic() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::f64::< 1.0 2.0))"#),
        Value::bool(true),
    );
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::f64::>= 5.0 5.0))"#),
        Value::bool(true),
    );
}

/// `:wat::core::i64::not=` MUST exist (rename from `:i64::!=` per Q-naming).
#[test]
fn mint_i64_not_eq_renamed() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::i64::not= 1 2))"#),
        Value::bool(true),
    );
}

/// The 0-ary `+` returning 0 (Lisp identity) via wat-defclause clause.
#[test]
fn mint_arith_zero_ary_plus_identity() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::+))"#),
        Value::i64(0),
        "0-ary + returns i64 0 (Lisp identity)"
    );
}

/// The 0-ary `*` returning 1 (Lisp multiplicative identity).
#[test]
fn mint_arith_zero_ary_star_identity() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::*))"#),
        Value::i64(1),
        "0-ary * returns i64 1 (Lisp identity)"
    );
}

/// 0-ary `-` ERRORS (no clause matches; :NoMatchingClause).
#[test]
fn mint_arith_zero_ary_minus_errors() {
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::-))"#,
    );
    assert!(result.is_err(), "0-ary `-` MUST error (no clause for it); got: {:?}", result);
}

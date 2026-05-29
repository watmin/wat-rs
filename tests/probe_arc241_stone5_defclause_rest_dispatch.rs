//! FM 2-bis probe for Stone 241.5 — defclause `&` rest-binder runtime dispatch.
//!
//! ## Why this probe
//!
//! Stone 241.4 settled the storage foundation:
//!   - Canonical parser parses `& name <- :T` when `allow_rest_binder: true`
//!   - A4 (parse_defclause_args inlined) sets allow_rest_binder: true
//!   - Clause struct gained `rest_param: Option<(String, TypeExpr)>`
//!   - Parser threads it through to clause storage
//!
//! Stone 241.5 wires the dispatch: `eval_call_to_defclause_with_vals` at
//! `src/runtime.rs:7198` consumes `Clause.rest_param` for:
//!   1. Variadic-min arity match (called_arity >= fixed_arity when rest exists)
//!   2. Element-type check per rest value (against Vector<T>'s T)
//!   3. Rest values collected into Value::Vector
//!   4. Bound at rest_param.name in the clause scope
//!
//! ## What this probe proves
//!
//! Pre-stone (HEAD `cfe93a22`+): contracts that exercise rest-binder DISPATCH
//! fail because the substrate's dispatcher uses strict arity equality. The
//! 237.8b Gate 1 (`gate_1_defclause_supports_rest_binder`) is currently
//! `#[ignore]`'d with this stone as its named follow-up.
//!
//! Post-stone: contracts pass; Gate 1 un-ignored and PASSING.
//!
//! ## FM 2-bis nature: BEHAVIORAL EXTENSION probe
//!
//! Stone 241.4 added storage (Clause.rest_param); Stone 241.5 adds the
//! behavior that consumes it. This probe disconfirms the missing behavior at
//! HEAD and confirms it post-stone.
//!
//! Run: `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn try_compute(src: &str) -> Result<Value, String> {
    let full = with_nil_main(src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

fn try_startup(src: &str) -> Result<(), String> {
    let full = with_nil_main(src);
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Contracts 1–4: rest-binder dispatch success paths ───────────────────────

#[test]
fn contract_01_variadic_min_with_rest_succeeds() {
    // defclause with [fixed & rest <- :Vector<:i64>]; called with fixed + N rest values.
    // Rest collected into Vector; foldl folds them; result computed.
    let src = r#"
        (:wat::core::defclause :my::sum-all
          ([first <- :wat::core::i64
            & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::foldl rest first
              (:wat::core::fn [acc <- :wat::core::i64
                               n <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+'2 acc n)))))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-all 1 2 3 4))
    "#;
    let result = try_compute(src);
    assert_eq!(
        result.expect("variadic dispatch should succeed"),
        Value::i64(10),
        "1+2+3+4 = 10 via & rest-binder fold"
    );
}

#[test]
fn contract_02_empty_rest_succeeds() {
    // Called with exactly fixed-arity values; rest is empty Vector.
    let src = r#"
        (:wat::core::defclause :my::sum-all
          ([first <- :wat::core::i64
            & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::foldl rest first
              (:wat::core::fn [acc <- :wat::core::i64
                               n <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+'2 acc n)))))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-all 42))
    "#;
    let result = try_compute(src);
    assert_eq!(
        result.expect("empty-rest dispatch should succeed"),
        Value::i64(42),
        "fold of empty rest with seed 42 returns 42"
    );
}

#[test]
fn contract_03_rest_only_succeeds() {
    // Rest-only clause (no fixed args before `&`).
    let src = r#"
        (:wat::core::defclause :my::count-args
          ([& rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::length rest)))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::count-args 10 20 30))
    "#;
    let result = try_compute(src);
    assert_eq!(
        result.expect("rest-only dispatch should succeed"),
        Value::i64(3),
        "3 args collected into rest Vector; length is 3"
    );
}

#[test]
fn contract_04_rest_only_empty_call_succeeds() {
    // Rest-only clause called with ZERO args; rest is empty Vector.
    let src = r#"
        (:wat::core::defclause :my::count-args
          ([& rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::length rest)))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::count-args))
    "#;
    let result = try_compute(src);
    assert_eq!(
        result.expect("rest-only zero-arg dispatch should succeed"),
        Value::i64(0),
        "0 args → empty rest Vector; length is 0"
    );
}

// ─── Contracts 5–6: error paths ──────────────────────────────────────────────

#[test]
fn contract_05_rest_element_type_mismatch_errors() {
    // Rest contains a wrong-type value (string in Vector<i64>).
    // Should error with NoMatchingClause (type mismatch at rest position).
    let src = r#"
        (:wat::core::defclause :my::sum-all
          ([first <- :wat::core::i64
            & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::foldl rest first
              (:wat::core::fn [acc <- :wat::core::i64
                               n <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+'2 acc n)))))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-all 1 2 "three"))
    "#;
    // Either startup fails (type-check catches it) OR compute fails (dispatch
    // rejects the wrong-type rest value). Both are acceptable error paths.
    let startup_result = try_startup(src);
    let compute_result = try_compute(src);
    assert!(
        startup_result.is_err() || compute_result.is_err(),
        "rest element type mismatch must error somewhere; got startup={:?} compute={:?}",
        startup_result, compute_result
    );
}

#[test]
fn contract_06_under_supply_below_fixed_errors() {
    // Called with FEWER than fixed-arity (rest doesn't allow under-supply).
    let src = r#"
        (:wat::core::defclause :my::pair
          ([a <- :wat::core::i64 b <- :wat::core::i64
            & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::i64::+'2 a b)))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::pair 1))
    "#;
    // Either startup fails (type-check catches arity) OR compute fails (dispatch rejects).
    let startup_result = try_startup(src);
    let compute_result = try_compute(src);
    assert!(
        startup_result.is_err() || compute_result.is_err(),
        "under-supply must error; got startup={:?} compute={:?}",
        startup_result, compute_result
    );
}

// ─── Contracts 7–8: regression on existing dispatch ──────────────────────────

#[test]
fn contract_07_fixed_only_strict_arity_preserved() {
    // Clause WITHOUT rest_param. Called with extra args → ArityMismatch (regression
    // — Stone 241.5's variadic-min behavior MUST NOT apply when rest_param is None).
    let src = r#"
        (:wat::core::defclause :my::strict
          ([x <- :wat::core::i64] -> :wat::core::i64 x))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::strict 1 2))
    "#;
    let startup_result = try_startup(src);
    let compute_result = try_compute(src);
    assert!(
        startup_result.is_err() || compute_result.is_err(),
        "strict-arity clause should reject over-supply; got startup={:?} compute={:?}",
        startup_result, compute_result
    );
}

#[test]
fn contract_08_mixed_clause_set_first_match_wins() {
    // First clause = fixed [x <- :i64]; second clause = [x <- :i64 & rest <- :Vector<:i64>].
    // Calling with (1) → first clause matches (returns x).
    // Calling with (1,2,3) → first clause arity-mismatches; second clause matches (returns sum).
    let src = r#"
        (:wat::core::defclause :my::flex
          ([x <- :wat::core::i64] -> :wat::core::i64 x)
          ([first <- :wat::core::i64
            & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
            (:wat::core::foldl rest first
              (:wat::core::fn [acc <- :wat::core::i64
                               n <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+'2 acc n)))))
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::flex 10 20 30))
    "#;
    let result = try_compute(src);
    assert_eq!(
        result.expect("mixed clause set should dispatch correctly"),
        Value::i64(60),
        "10+20+30 = 60 via second (rest-binder) clause; first clause arity-mismatched"
    );
}

//! FM 2-bis probe — arc 237 Stone S-C.3: the base/holonic macro split.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-C3.md`.
//!
//! `:wat::Record::def` → BASE (struct only; recordtype parent :wat::Record).
//! `:wat::holon::Record::def` → HOLONIC (struct + holon; parent :wat::holon::Record <: :wat::Record).
//! The recordtype parent IS the Liskov mechanism: a func wanting :wat::holon::Record rejects a
//! base-defined record at CHECK time; wanting :wat::Record accepts both.
//!
//! RED today: `:wat::holon::Record::def` does not exist, and `:wat::Record::def` still builds
//! holonic (so base ops + to-holon-error + Liskov rejection are unmet). GREEN after the stone.
//!
//! Coverage (feedback_logic_coverage_mandate): base ops · holonic preserved · Liskov accept/reject
//! · cross-flavor.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// base :my::Pt [x y] + holonic :my::HPt [x y] (same field names → cross-flavor same-data? true).
const PRELUDE: &str = "\
(:wat::Record::def :my::Pt [x <- :wat::core::i64  y <- :wat::core::i64])\n\
(:wat::holon::Record::def :my::HPt [x <- :wat::core::i64  y <- :wat::core::i64])\n";

/// Eval `expr` (typed `:bool`) → Value or error string.
fn eval(expr: &str) -> Result<Value, String> {
    eval_typed(expr, ":wat::core::bool")
}
/// Eval `expr` declared to return `ret_ty`.
fn eval_typed(expr: &str, ret_ty: &str) -> Result<Value, String> {
    let full = format!(
        "{PRELUDE}\
         (:wat::core::defn :user::compute [] -> {ret_ty} {expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}
fn is_true(expr: &str) -> bool { matches!(eval(expr), Ok(Value::bool(true))) }
fn is_false(expr: &str) -> bool { matches!(eval(expr), Ok(Value::bool(false))) }
fn i64_field(expr: &str) -> i64 {
    match eval_typed(expr, ":wat::core::i64") {
        Ok(Value::i64(n)) => n,
        other => panic!("expected i64 for `{}`; got {:?}", expr, other),
    }
}
/// Type-check ONLY a program (no compute harness); Ok iff it startups clean.
fn check(decls: &str) -> Result<(), String> {
    let src = format!(
        "{PRELUDE}{decls}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)"
    );
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── BASE flavor (:my::Pt via :wat::Record::def) ──────────────────────────────
#[test] fn base_construct_and_field() { assert_eq!(i64_field("(:x (:my::Pt 1 2))"), 1); }
#[test] fn base_accessor() { assert_eq!(i64_field("(:my::Pt/y (:my::Pt 1 2))"), 2); }
#[test] fn base_predicate_true() { assert!(is_true("(:my::is-Pt? (:my::Pt 1 2))")); }
#[test] fn base_predicate_false() { assert!(is_false("(:my::is-Pt? (:my::HPt 1 2))")); }
#[test] fn base_eq_equal() { assert!(is_true("(:wat::core::= (:my::Pt 1 2) (:my::Pt 1 2))")); }
#[test] fn base_eq_diff() { assert!(is_false("(:wat::core::= (:my::Pt 1 2) (:my::Pt 1 9))")); }
#[test] fn base_same_data() { assert!(is_true("(:wat::Record/same-data? (:my::Pt 1 2) (:my::Pt 1 2))")); }
#[test] fn base_assoc_then_read() { assert_eq!(i64_field("(:y (:wat::Record/assoc (:my::Pt 1 2) :y 9))"), 9); }
#[test] fn base_to_holon_errors() {
    // base has NO holon flavor — to-holon must error (teaching error), not return Ok.
    let h = eval_typed("(:wat::holon::to-holon (:my::Pt 1 2))", ":wat::holon::HolonAST");
    assert!(h.is_err(), "to-holon on a BASE record must error; got {:?}", h);
}

// ─── HOLONIC flavor (:my::HPt via :wat::holon::Record::def) ────────────────────
#[test] fn holonic_construct_field() { assert_eq!(i64_field("(:x (:my::HPt 7 8))"), 7); }
#[test] fn holonic_predicate_true() { assert!(is_true("(:my::is-HPt? (:my::HPt 7 8))")); }
#[test] fn holonic_to_holon_ok() {
    // holonic HAS a holon flavor — to-holon works.
    let t = eval_typed("(:wat::holon::to-holon (:my::HPt 1 2))", ":wat::holon::HolonAST");
    assert!(t.is_ok(), "to-holon on a HOLONIC record must work; got {:?}", t);
}

// ─── Liskov type-distinction (the static proof) ───────────────────────────────
const WANTS_BASE: &str =
    "(:wat::core::defn :wb [v <- :wat::Record] -> :wat::core::bool true)\n";
const WANTS_HOLON: &str =
    "(:wat::core::defn :wh [v <- :wat::holon::Record] -> :wat::core::bool true)\n";

#[test] fn liskov_base_into_base_ok() {
    assert!(check(&format!("{WANTS_BASE}\
        (:wat::core::defn :fb [p <- :my::Pt] -> :wat::core::bool (:wb p))")).is_ok());
}
#[test] fn liskov_holonic_into_base_ok() {
    // holonic <: base — a func wanting base accepts a holonic-defined record.
    assert!(check(&format!("{WANTS_BASE}\
        (:wat::core::defn :fh [p <- :my::HPt] -> :wat::core::bool (:wb p))")).is_ok());
}
#[test] fn liskov_holonic_into_holon_ok() {
    assert!(check(&format!("{WANTS_HOLON}\
        (:wat::core::defn :gh [p <- :my::HPt] -> :wat::core::bool (:wh p))")).is_ok());
}
#[test] fn liskov_base_into_holon_rejected() {
    // THE static proof: a base-defined record is NOT a :wat::holon::Record → check error.
    assert!(check(&format!("{WANTS_HOLON}\
        (:wat::core::defn :gb [p <- :my::Pt] -> :wat::core::bool (:wh p))")).is_err(),
        "a base-defined record must be REJECTED at a :wat::holon::Record param");
}

// ─── Cross-flavor (needs both macros) ─────────────────────────────────────────
#[test] fn cross_flavor_same_data_true() {
    // base Pt[0,0] vs holonic HPt[0,0], same field names → type-blind same-data? true
    assert!(is_true("(:wat::Record/same-data? (:my::Pt 0 0) (:my::HPt 0 0))"));
}
#[test] fn cross_flavor_eq_false() {
    // = is type-strict: different type/flavor → false
    assert!(is_false("(:wat::core::= (:my::Pt 0 0) (:my::HPt 0 0))"));
}

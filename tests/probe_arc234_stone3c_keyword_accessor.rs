//! Diagnostic probe — keyword-as-accessor fall-through (arc 234 Stone 234.3c).
//!
//! Verifies the Clojure-style sugar where a bare-name keyword head used
//! as a function dispatches to a field-accessor based on receiver type.
//!
//! Three receiver types: wat__Record (field by name), Struct (field by
//! name via TypeDef), wat__std__HashMap (key lookup returning Option).
//!
//! Closes #058/146 follow-up per umbrella DESIGN line 416-440.
//!
//! Probe contracts (5 + 1 optional):
//!   1. Single-field record: (:field r) returns field value
//!   2. Multi-field record: (:field r) returns correctly-typed value
//!   3. UnknownField on bad key for record
//!   4. HashMap key present: returns Some(v)
//!   5. HashMap key missing: returns None
//!   6. Struct field access (OPTIONAL — may defer if struct scaffolding heavy)
//!
//! Initial state: 5/6 FAIL (or 4/5) — UnknownFunction(":magnitude") etc.
//! Post-stone: full PASS.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
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

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_1_keyword_accessor_on_single_field_record() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage 5.0)]
      (:magnitude v)))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!(
            (f - 5.0).abs() < 1e-9,
            "Probe 1: (:magnitude v) should return 5.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 1: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_2_keyword_accessor_on_multi_field_record() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [t (:myapp::Triple 7 "hello" true)]
      (:b t)))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(
            s.as_str(),
            "hello",
            "Probe 2: (:b t) should return 'hello'; got {}",
            s.as_str()
        ),
        Ok(other) => panic!("Probe 2: expected Value::String; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_3_unknown_field_on_record_errors() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage 5.0)]
      (:nonexistent v)))
"#;
    match run_compute(src) {
        Ok(v) => panic!(
            "Probe 3 FAILED: expected error on unknown field; got Ok({:?})",
            v
        ),
        Err(msg) => assert!(
            msg.to_lowercase().contains("unknown") || msg.contains("nonexistent"),
            "Probe 3: expected error mentioning unknown/nonexistent; got {}",
            msg
        ),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_4_keyword_accessor_on_hashmap_some() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [m {:port 8080}
       v (:port m)]
      (:wat::core::Option/expect -> :wat::core::i64
        v
        "probe 4: expected :port key present")))
"#;
    match run_compute(src) {
        Ok(Value::i64(n)) => assert_eq!(
            n, 8080,
            "Probe 4: (:port m) should return Some(8080); got {}",
            n
        ),
        Ok(other) => panic!("Probe 4: expected Value::i64; got {:?}", other),
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_5_keyword_accessor_on_hashmap_none() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [m {:host "localhost"}
       v (:missing m)]
      (:wat::core::match v -> :wat::core::bool
        ((:wat::core::Some _) false)
        (:wat::core::None     true))))
"#;
    match run_compute(src) {
        Ok(Value::bool(b)) => assert!(
            b,
            "Probe 5: (:missing m) on map without :missing should yield None"
        ),
        Ok(other) => panic!("Probe 5: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// OPTIONAL: struct keyword-access. If struct scaffolding is heavy in
// the probe (needs :wat::core::struct declaration + struct-new
// invocation), document it as deferred and remove this test before
// shipping. Otherwise include.
#[test]
fn probe_6_keyword_accessor_on_struct() {
    let src = r#"
(:wat::core::defstruct :myapp::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [p (:wat::core::struct-new :myapp::Point 3 4)]
      (:x p)))
"#;
    match run_compute(src) {
        Ok(Value::i64(n)) => assert_eq!(
            n, 3,
            "Probe 6: (:x p) on struct should return 3; got {}",
            n
        ),
        Ok(other) => panic!("Probe 6: expected Value::i64; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

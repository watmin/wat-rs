//! Diagnostic probe — let-binding hash-destructure (arc 234 Stone 234.4).
//!
//! Verifies the Clojure-style `{var :field var2 :field2 ...}` brace-form
//! in let binding position. Receiver-polymorphic over record/struct/HashMap.
//!
//! Match-arm hash-destructure is named follow-up Stone 234.4.match
//! (NOT 234.4 scope; will ship separately).
//!
//! Probe contracts (6):
//!   1. Single-field record destructure
//!   2. Multi-field record destructure (3 fields)
//!   3. HashMap destructure with present key (Some)
//!   4. HashMap destructure with missing key (None)
//!   5. UnknownField error on record bad field
//!   6. Multiple bindings in same let (two destructures)
//!
//! Initial state: 6/6 FAIL with parser/check errors.
//! Post-stone: 6/6 PASS.

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
fn probe_1_single_field_record_destructure() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [{mag :magnitude} (:myapp::Voltage 5.0)]
      mag))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!((f - 5.0).abs() < 1e-9, "got {}", f),
        Ok(other) => panic!("Probe 1: expected f64; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_2_multi_field_record_destructure() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [{x :a  y :b  z :c} (:myapp::Triple 7 "hello" true)]
      y))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(s.as_str(), "hello"),
        Ok(other) => panic!("Probe 2: expected String; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_3_hashmap_destructure_some() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [{p :port} {:port 8080}]
      (:wat::core::Option/expect -> :wat::core::i64
        p
        "probe 3: :port key present")))
"#;
    match run_compute(src) {
        Ok(Value::i64(n)) => assert_eq!(n, 8080),
        Ok(other) => panic!("Probe 3: expected i64; got {:?}", other),
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_4_hashmap_destructure_none() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [{x :missing} {:host "localhost"}]
      (:wat::core::match x -> :wat::core::bool
        ((:wat::core::Some _) false)
        (:wat::core::None     true))))
"#;
    match run_compute(src) {
        Ok(Value::bool(b)) => assert!(b, "Probe 4: expected true (None branch)"),
        Ok(other) => panic!("Probe 4: expected bool; got {:?}", other),
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_5_unknown_field_errors() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [{x :nonexistent} (:myapp::Voltage 5.0)]
      x))
"#;
    match run_compute(src) {
        Ok(v) => panic!("Probe 5 FAILED: expected error; got Ok({:?})", v),
        Err(msg) => assert!(
            msg.to_lowercase().contains("unknown") || msg.contains("nonexistent"),
            "Probe 5: expected unknown-field-style error; got {}",
            msg
        ),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_6_multiple_destructures_in_same_let() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::Record::def :myapp::Counter  [count    <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [{m :magnitude} (:myapp::Voltage 3.5)
       {c :count}     (:myapp::Counter 7)]
      (:wat::core::+ m (:wat::core::i64/to-f64 c))))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!((f - 10.5).abs() < 1e-9, "got {}", f),
        Ok(other) => panic!("Probe 6: expected f64; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

//! Diagnostic probe — `:wat::core::record?` + `:wat::core::record->map`
//! (arc 234 Stone 234.3a).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.3a BRIEF. Verifies
//! two new substrate primitives:
//!
//!   - :wat::core::record?     (polymorphic predicate; true iff wat__Record)
//!   - :wat::core::record->map (extract HashMap<keyword, value> from record)
//!
//! Foundation for 234.3b (assoc polymorphic record arm) and 234.3c
//! (keyword-as-accessor fall-through). The field-name extraction
//! machinery established here gets reused by both.
//!
//! Probe contracts (6):
//!   1. record? true on a constructed record
//!   2. record? false on non-record values (i64, String, HashMap)
//!   3. record->map single-field — returns one-entry HashMap
//!   4. record->map multi-field heterogeneous — returns three-entry HashMap
//!      in declaration order
//!   5. record->map zero-field — returns empty HashMap
//!   6. Composition: predicate-then-map (defensive pattern)
//!
//! Initial state: 6/6 FAIL with UnknownFunction(":wat::core::record?")
//! and similar for record->map. The primitives don't exist yet.
//!
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
//
// record? returns true on a constructed wat-record.
#[test]
fn probe_1_record_q_true_on_record() {
    let src = r#"
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [v (:myapp::Voltage 5.0)]
      (:wat::core::record? v)))
"#;
    match run_compute(src) {
        Ok(Value::bool(b)) => assert!(
            b,
            "Probe 1: record? on a record should be true"
        ),
        Ok(other) => panic!("Probe 1: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// record? returns false on a non-record (i64). Probes the polymorphic-input
// pattern — any type is accepted; only wat__Record returns true.
#[test]
fn probe_2_record_q_false_on_i64() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::record? 42))
"#;
    match run_compute(src) {
        Ok(Value::bool(b)) => assert!(
            !b,
            "Probe 2: record? on an i64 should be false"
        ),
        Ok(other) => panic!("Probe 2: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// record->map on a single-field record returns a one-entry HashMap with
// keyword key + the typed value.
#[test]
fn probe_3_record_to_map_single_field() {
    let src = r#"
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage 5.0)
       m (:wat::core::record->map v)]
      (:wat::core::Option/expect
        (:wat::core::get m :magnitude)
        "record->map probe 3: :magnitude key missing")))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!(
            (f - 5.0).abs() < 1e-9,
            "Probe 3: record->map of {{:magnitude 5.0}} via get :magnitude should be 5.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 3: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// record->map on a 3-field heterogeneous record returns a HashMap with
// all three entries. Tests via get :b returning the String field.
#[test]
fn probe_4_record_to_map_multi_field_heterogeneous() {
    let src = r#"
(:wat::core::defrecord :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [t (:myapp::Triple 7 "hello" true)
       m (:wat::core::record->map t)]
      (:wat::core::Option/expect
        (:wat::core::get m :b)
        "record->map probe 4: :b key missing")))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(
            s.as_str(),
            "hello",
            "Probe 4: record->map of 3-field record + get :b should return 'hello'"
        ),
        Ok(other) => panic!("Probe 4: expected Value::String; got {:?}", other),
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// record->map on a zero-field record returns an empty HashMap.
// Verifies via :wat::core::empty? predicate.
#[test]
fn probe_5_record_to_map_zero_field() {
    let src = r#"
(:wat::core::defrecord :myapp::Tag [])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [t (:myapp::Tag)
       m (:wat::core::record->map t)]
      (:wat::core::empty? m)))
"#;
    match run_compute(src) {
        Ok(Value::bool(b)) => assert!(
            b,
            "Probe 5: zero-field record->map should produce empty HashMap"
        ),
        Ok(other) => panic!("Probe 5: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// Composition: predicate-then-map pattern. Defensive usage where
// record? guards the record->map call. Tests type-check + runtime
// behavior of the common idiom.
#[test]
fn probe_6_predicate_then_map_composition() {
    let src = r#"
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage 99.0)]
      (:wat::core::if
        (:wat::core::record? v)
        -> :wat::core::f64
        (:wat::core::Option/expect
          (:wat::core::get (:wat::core::record->map v) :magnitude)
          "probe 6: missing :magnitude")
        -1.0)))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!(
            (f - 99.0).abs() < 1e-9,
            "Probe 6: predicate-true → map-get path should return 99.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 6: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

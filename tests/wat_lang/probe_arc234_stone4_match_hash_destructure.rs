//! Diagnostic probe — match-arm hash-destructure (arc 234 Stone 234.4.match).
//!
//! Verifies the Clojure-style `{var :field var2 :field2 ...}` brace-form
//! in match-arm pattern position. Receiver-polymorphic over
//! record / struct / HashMap. Mirror of Stone 234.4 let-binding probe shape.
//!
//! Probe contracts (6):
//!   1. Match record with single {var :field} — extracts field; body uses var
//!   2. Match record with multi {var1 :f1 var2 :f2} — multi-field bind
//!   3. Match HashMap with {var :field} — Option<V> bind per key (Some)
//!   4. Match HashMap multi-key — multiple Option<V> bindings
//!   5. Match-arm fall-through: scrutinee is i64 → hash-destructure arm
//!      does not match → falls to next arm (wildcard)
//!   6. Mixed match: one arm hash-destructure; another a wildcard — selection
//!      is correct per scrutinee type
//!
//! Initial state: 6/6 FAIL (StructPattern in match-arm position returned Err).
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
// Match record with single {var :field} — extracts field; body uses var.
#[test]
fn probe_1_match_record_single_field() {
    let src = r#"
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [rec (:myapp::Voltage 7.5)]
      (:wat::core::match rec -> :wat::core::f64
        ({mag :magnitude} mag)
        (_ 0.0))))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!((f - 7.5).abs() < 1e-9, "got {}", f),
        Ok(other) => panic!("Probe 1: expected f64; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
// Match record with multi {var1 :f1 var2 :f2} — multi-field bind.
#[test]
fn probe_2_match_record_multi_field() {
    let src = r#"
(:wat::core::defrecord :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [pt (:myapp::Point 3 4)]
      (:wat::core::match pt -> :wat::core::i64
        ({px :x  py :y} (:wat::core::+ px py))
        (_ 0))))
"#;
    match run_compute(src) {
        Ok(Value::i64(n)) => assert_eq!(n, 7, "got {}", n),
        Ok(other) => panic!("Probe 2: expected i64; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
// Match HashMap with {var :field} — Option<V> bind per key (present key → Some).
#[test]
fn probe_3_match_hashmap_single_key_some() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [m {:port 9000}]
      (:wat::core::match m -> :wat::core::i64
        ({p :port} (:wat::core::Option/expect
                     p
                     "probe 3: :port key present"))
        (_ 0))))
"#;
    match run_compute(src) {
        Ok(Value::i64(n)) => assert_eq!(n, 9000, "got {}", n),
        Ok(other) => panic!("Probe 3: expected i64; got {:?}", other),
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
// Match HashMap multi-key — multiple Option<V> bindings; body uses both.
// Uses a homogeneous String-valued HashMap to satisfy the type checker.
#[test]
fn probe_4_match_hashmap_multi_key() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [m {:host "localhost"  :user "admin"}]
      (:wat::core::match m -> :wat::core::bool
        ({h :host  mv :missing}
         (:wat::core::match h -> :wat::core::bool
           ((:wat::core::Some _)
            (:wat::core::match mv -> :wat::core::bool
              ((:wat::core::Some _) false)
              (:wat::core::None     true)))
           (:wat::core::None false)))
        (_ false))))
"#;
    // h = :host → Some("localhost"), mv = :missing → None
    // → h arm matches Some → check mv → None → true
    match run_compute(src) {
        Ok(Value::bool(b)) => assert!(b, "Probe 4: expected true (h=Some, mv=None)"),
        Ok(other) => panic!("Probe 4: expected bool; got {:?}", other),
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
// Match-arm fall-through: scrutinee is i64 → hash-destructure arm does not
// match → falls to next wildcard arm which returns the integer.
#[test]
fn probe_5_fall_through_on_non_receiver() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [v 42]
      (:wat::core::match v -> :wat::core::i64
        (_ 99))))
"#;
    // The match above doesn't use hash-destructure in the first arm because
    // i64 scrutinee with a hash-destructure arm would need a wildcard fallback.
    // Instead, verify that a match where a hash-destructure arm COMES FIRST
    // and the scrutinee is an i64 falls through to the next arm.
    // We test this by passing an i64 scrutinee and confirming we get the
    // wildcard arm's value.
    let src2 = r#"
(:wat::core::defrecord :myapp::Tag [label <- :wat::core::String])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [v 42]
      (:wat::core::match v -> :wat::core::i64
        ({lbl :label} 0)
        (_ 99))))
"#;
    match run_compute(src2) {
        Ok(Value::i64(n)) => assert_eq!(n, 99, "expected fall-through to wildcard arm (99); got {}", n),
        Ok(other) => panic!("Probe 5: expected i64; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
// Mixed match: one arm hash-destructure on record; another wildcard.
// Selection is correct per scrutinee type.
#[test]
fn probe_6_mixed_match_arm_selection() {
    let src = r#"
(:wat::core::defrecord :myapp::Sensor [reading <- :wat::core::f64])

(:wat::core::defn :user::compute-from-record [] -> :wat::core::String
  (:wat::core::let
      [s (:myapp::Sensor 3.14)]
      (:wat::core::match s -> :wat::core::String
        ({r :reading} "record-matched")
        (_ "wildcard"))))

(:wat::core::defn :user::compute [] -> :wat::core::String (:user::compute-from-record))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(s.as_str(), "record-matched",
            "Probe 6: hash-destructure arm should have matched the record"),
        Ok(other) => panic!("Probe 6: expected String; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

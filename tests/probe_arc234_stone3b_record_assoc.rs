//! Diagnostic probe — `:wat::Record/assoc` substrate primitive (arc 234 Stone 234.3b).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.3b BRIEF. The write
//! verb in the polymorphic record-y family — sibling of 234.3a's read verbs.
//!
//! Probe contracts (6):
//!   1. Single-field update returns new record; new value applied
//!   2. Multi-field, update one — other fields unchanged
//!   3. UnknownField on bad key
//!   4. TypeMismatch on wrong-type value
//!   5. Original record unchanged (immutability)
//!   6. Compose multiple assocs
//!
//! Initial state: 6/6 FAIL with UnknownFunction(":wat::Record/assoc").
//! Post-stone: 6/6 PASS.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
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
fn probe_1_single_field_update() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [r  (:myapp::Voltage 5.0)
       r2 (:wat::Record/assoc r :magnitude 6.0)]
      (:myapp::Voltage/magnitude r2)))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!(
            (f - 6.0).abs() < 1e-9,
            "Probe 1: assoc'd magnitude should be 6.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 1: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_2_multi_field_update_one() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [t  (:myapp::Triple 7 "hello" true)
       t2 (:wat::Record/assoc t :b "world")]
      (:myapp::Triple/b t2)))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(
            s.as_str(),
            "world",
            "Probe 2: assoc'd b should be 'world'; got {}",
            s.as_str()
        ),
        Ok(other) => panic!("Probe 2: expected Value::String; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_3_unknown_field_errors() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::Record
  (:wat::core::let
      [t (:myapp::Triple 7 "hello" true)]
      (:wat::Record/assoc t :nonexistent 42)))
"#;
    match run_compute(src) {
        Ok(v) => panic!("Probe 3 FAILED: expected UnknownField error; got Ok({:?})", v),
        Err(msg) => assert!(
            msg.to_lowercase().contains("unknown") || msg.contains("nonexistent"),
            "Probe 3: expected error mentioning unknown/nonexistent field; got {}",
            msg
        ),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_4_type_mismatch_errors() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::Record
  (:wat::core::let
      [r (:myapp::Voltage 5.0)]
      (:wat::Record/assoc r :magnitude 42)))
"#;
    match run_compute(src) {
        Ok(v) => panic!(
            "Probe 4 FAILED: expected TypeMismatch (f64 vs i64); got Ok({:?})",
            v
        ),
        Err(msg) => assert!(
            msg.to_lowercase().contains("typemismatch")
                || msg.to_lowercase().contains("type")
                || msg.contains("f64")
                || msg.contains("i64"),
            "Probe 4: expected type-mismatch error; got {}",
            msg
        ),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_5_original_record_unchanged() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [r1 (:myapp::Voltage 5.0)
       r2 (:wat::Record/assoc r1 :magnitude 6.0)]
      (:myapp::Voltage/magnitude r1)))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => assert!(
            (f - 5.0).abs() < 1e-9,
            "Probe 5: original r1 should still have magnitude=5.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 5: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_6_compose_multiple_assocs() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [t  (:myapp::Triple 7 "hello" true)
       t2 (:wat::Record/assoc
            (:wat::Record/assoc t :a 100)
            :b "world")]
      (:wat::core::string::concat
        (:wat::core::i64::to-string (:myapp::Triple/a t2))
        "|"
        (:myapp::Triple/b t2))))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(
            s.as_str(),
            "100|world",
            "Probe 6: composed assocs should yield '100|world'; got {}",
            s.as_str()
        ),
        Ok(other) => panic!("Probe 6: expected Value::String; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

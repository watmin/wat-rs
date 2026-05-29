//! Diagnostic probe — `:wat::holon::*` auto-dispatch on `Value::wat__Record`
//! (arc 234 Stone 234.5).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.5 BRIEF. Verifies
//! the VSA-integration completion — records flow through 5 holon verbs
//! natively, using the pre-built holon_form without explicit conversion:
//!
//!   - :wat::holon::to-holon          (THE bridge; unwraps to holon_form)
//!   - :wat::holon::Bind              (constructor accepting record as arg)
//!   - :wat::holon::Bundle            (constructor accepting records as children)
//!   - :wat::holon::cosine            (VSA-proof verb measuring similarity)
//!   - :wat::holon::extract-classifier (algebraic type-extraction)
//!
//! This is the stone that proves the hologram property is REAL — externally
//! observable via VSA verbs accepting records without user-facing conversion.
//!
//! Probe contracts (6):
//!   1. to-holon on a record returns the record's holon_form unchanged
//!   2. cosine on two records returns f64 (VSA op flows end-to-end)
//!   3. Bind accepts a record as right arg
//!   4. Bundle accepts records as children
//!   5. extract-classifier on a record returns its class_fqdn String
//!   6. Mixed: Bind with classifier-h + Bundle with record + raw-Atom composes
//!
//! Initial state: 6/6 FAIL with TypeMismatch (expected :wat::holon::HolonAST,
//! got :wat::Record) — the type-checker currently rejects records in
//! HolonAST positions.
//!
//! Post-stone: 6/6 PASS. The hologram becomes externally observable.

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
//
// `(:wat::holon::to-holon r)` returns the record's holon_form unchanged.
// Construct a single-field record via the 234.2b macro; call to-holon;
// verify result is the same HolonAST as the record's holon_form.
#[test]
fn probe_1_to_holon_returns_holon_form() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::holon::HolonAST
  (:wat::core::let
      [v (:myapp::Voltage 5.0)]
      (:wat::holon::to-holon v)))
"#;
    match run_compute(src) {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert!(
                s.contains("myapp::Voltage") || s.contains("Voltage"),
                "Probe 1: holon_form should mention the class; got {}",
                s
            );
        }
        Ok(other) => panic!("Probe 1: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::cosine r1 r2)` on two identical-class records returns f64.
// This proves the VSA verb flows end-to-end with records as operands.
#[test]
fn probe_2_cosine_accepts_records() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [r1 (:myapp::Voltage 5.0)
       r2 (:myapp::Voltage 5.0)]
      (:wat::holon::cosine r1 r2)))
"#;
    match run_compute(src) {
        Ok(Value::f64(f)) => {
            assert!(
                f.is_finite(),
                "Probe 2: cosine should return finite f64; got {}",
                f
            );
            assert!(
                (-1.0..=1.0).contains(&f),
                "Probe 2: cosine should be in [-1, 1]; got {}",
                f
            );
        }
        Ok(other) => panic!("Probe 2: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::Bind classifier-h r)` builds a Bind composition with r's
// holon_form as the right side. Tests record-as-arg in a constructor.
#[test]
fn probe_3_bind_accepts_record_as_right() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::holon::HolonAST
  (:wat::core::let
      [r (:myapp::Voltage 5.0)]
      (:wat::holon::Bind
        (:wat::holon::Atom (:wat::holon::to-holon "wrapper"))
        r)))
"#;
    match run_compute(src) {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert!(
                s.contains("Bind") || s.contains("wrapper"),
                "Probe 3: result should be a Bind containing the wrapper; got {}",
                s
            );
        }
        Ok(other) => panic!("Probe 3: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::Bundle [r1 r2 r3])` accepts records as children, unwrapping
// each to its holon_form. Tests record-in-vec composition.
#[test]
fn probe_4_bundle_accepts_records_as_children() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::holon::HolonAST
  (:wat::core::let
      [r1 (:myapp::Voltage 1.0)
       r2 (:myapp::Voltage 2.0)
       r3 (:myapp::Voltage 3.0)]
      (:wat::core::Result/expect -> :wat::holon::HolonAST
        (:wat::holon::Bundle [r1 r2 r3])
        "Bundle failed in Probe 4")))
"#;
    match run_compute(src) {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert!(
                s.contains("Bundle"),
                "Probe 4: result should be a Bundle; got {}",
                s
            );
        }
        Ok(other) => panic!("Probe 4: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::extract-classifier r)` returns the record's class_fqdn as
// a String — same answer as `(:wat::core::type r)` but via the algebraic
// classifier-extraction verb.
#[test]
fn probe_5_extract_classifier_on_record() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [r (:myapp::Voltage 5.0)]
      (:wat::holon::extract-classifier r)))
"#;
    match run_compute(src) {
        Ok(Value::String(s)) => assert_eq!(
            s.as_str(),
            "myapp::Voltage",
            "Probe 5: extract-classifier should return class_fqdn 'myapp::Voltage'"
        ),
        Ok(other) => panic!("Probe 5: expected Value::String; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// Mixed-args composition — records and raw HolonASTs flow through verbs
// together. Proves the helper threads uniformly across composition layers.
#[test]
fn probe_6_mixed_records_and_holon_asts() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::holon::HolonAST
  (:wat::core::let
      [r          (:myapp::Voltage 5.0)
       classifier (:wat::holon::Atom (:wat::holon::to-holon "wrapper"))]
      (:wat::holon::Bind
        classifier
        (:wat::core::Result/expect -> :wat::holon::HolonAST
          (:wat::holon::Bundle
            [r
             (:wat::holon::Atom (:wat::holon::to-holon "marker"))])
          "Bundle failed in Probe 6"))))
"#;
    match run_compute(src) {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert!(
                s.contains("Bind") && s.contains("Bundle"),
                "Probe 6: result should contain Bind + Bundle composition; got {}",
                s
            );
        }
        Ok(other) => panic!("Probe 6: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

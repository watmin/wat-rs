//! Diagnostic probe — `:wat::Record::def` macro (arc 234 Stone 234.2b).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.2b BRIEF. Verifies
//! the wat-side macro that the user invokes to mint a new record-type:
//!
//!   (:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
//!
//! expands to (in `(:wat::core::do …)`):
//!   - Constructor `:myapp::Voltage` returning `:wat::Record`
//!   - Per-field accessor `:myapp::Voltage/magnitude` returning the field
//!   - Predicate `:myapp::is-Voltage?` returning `:wat::core::bool`
//!
//! The macro consumes Stone 234.2a's substrate primitives (`:wat::Record::of`
//! + `:wat::Record/field-at`) plus the holon-form construction pattern proven
//! by `:wat::Record::def` (arc 227 Stone 227.2 v3).
//!
//! Probe contracts (6):
//!   1. Single-field expansion + invocation — constructor returns
//!      Value::wat__Record with correct class_fqdn + struct_form
//!   2. Per-field accessor returns the correct value
//!   3. Predicate true on matching class
//!   4. Predicate false on non-matching class (two types defined; cross-call)
//!   5. Multi-field (3 fields) expansion + all three accessors work
//!   6. Zero-field expansion — constructor + predicate work
//!
//! Initial state: 6/6 FAIL with `UnknownFunction(":wat::Record::def")` (the
//! macro does not exist yet).
//!
//! Post-stone: 6/6 PASS. The macro expands cleanly + generated fns work.

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
// Single-field defrecord expansion + invocation. Constructor returns
// `Value::wat__Record` with class_fqdn = "myapp::Voltage" + struct_form
// holding the single declared field value.
#[test]
fn probe_1_single_field_construction() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::Record (:myapp::Voltage 5.0))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::wat__Record { class_fqdn, struct_form } => {
                assert_eq!(
                    class_fqdn.as_str(),
                    "myapp::Voltage",
                    "Probe 1: class_fqdn should be 'myapp::Voltage'"
                );
                assert_eq!(
                    struct_form.len(),
                    1,
                    "Probe 1: struct_form should have 1 element"
                );
                match &struct_form[0] {
                    Value::f64(f) => assert!(
                        (f - 5.0).abs() < 1e-9,
                        "Probe 1: struct_form[0] should be 5.0; got {}",
                        f
                    ),
                    other => panic!("Probe 1: expected f64 at index 0; got {:?}", other),
                }
            }
            other => panic!("Probe 1: expected Value::wat__Record; got {:?}", other),
        },
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// Per-field accessor returns the correct field value. Defines
// `:myapp::Voltage` with one field; constructs an instance; calls the
// generated accessor `:myapp::Voltage/magnitude`; asserts the returned f64
// matches the constructor argument.
#[test]
fn probe_2_per_field_accessor_returns_value() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage 42.5)]
      (:myapp::Voltage/magnitude v)))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::f64(f) => assert!(
                (f - 42.5).abs() < 1e-9,
                "Probe 2: accessor should return 42.5; got {}",
                f
            ),
            other => panic!("Probe 2: expected Value::f64; got {:?}", other),
        },
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Predicate returns true on a matching-class instance. Defines
// `:myapp::Voltage`; constructs an instance; calls `:myapp::is-Voltage?`;
// asserts true.
#[test]
fn probe_3_predicate_true_on_matching_class() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [v (:myapp::Voltage 5.0)]
      (:myapp::is-Voltage? v)))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::bool(b) => assert!(
                b,
                "Probe 3: predicate on matching-class instance should be true"
            ),
            other => panic!("Probe 3: expected Value::bool; got {:?}", other),
        },
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Predicate returns false on a non-matching-class instance. Defines two
// record-types (`:myapp::Voltage` + `:myapp::Counter`); constructs a Counter
// instance; calls `:myapp::is-Voltage?`; asserts false.
//
// This validates that predicates discriminate via class_fqdn equality, NOT
// by struct shape (Voltage + Counter both have one i64-or-f64 field but
// different class_fqdn strings).
#[test]
fn probe_4_predicate_false_on_non_matching_class() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::Record::def :myapp::Counter [count <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [c (:myapp::Counter 42)]
      (:myapp::is-Voltage? c)))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::bool(b) => assert!(
                !b,
                "Probe 4: predicate on non-matching-class instance should be false"
            ),
            other => panic!("Probe 4: expected Value::bool; got {:?}", other),
        },
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// Multi-field expansion — three fields of mixed types. Defines
// `:myapp::Triple [a <- i64  b <- String  c <- bool]`; constructs an
// instance; calls all three generated accessors; asserts each returns
// the correctly-typed value from its declared position.
//
// This validates the per-field accessor loop emits N defns in declaration
// order with correct positional indices (0, 1, 2) and per-field return
// types (i64, String, bool).
#[test]
fn probe_5_multi_field_accessors_in_order() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

(:wat::core::defn :user::compute-a [] -> :wat::core::i64
  (:wat::core::let
      [t (:myapp::Triple 7 "hello" true)]
      (:myapp::Triple/a t)))

(:wat::core::defn :user::compute-b [] -> :wat::core::String
  (:wat::core::let
      [t (:myapp::Triple 7 "hello" true)]
      (:myapp::Triple/b t)))

(:wat::core::defn :user::compute-c [] -> :wat::core::bool
  (:wat::core::let
      [t (:myapp::Triple 7 "hello" true)]
      (:myapp::Triple/c t)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [a (:user::compute-a)
       b (:user::compute-b)
       c (:user::compute-c)]
      (:wat::core::string::concat
        (:wat::core::i64::to-string a)
        "|"
        b
        "|"
        (:wat::core::bool::to-string c))))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::String(s) => assert_eq!(
                s.as_str(),
                "7|hello|true",
                "Probe 5: all three accessors should return their fields in order"
            ),
            other => panic!("Probe 5: expected Value::String; got {:?}", other),
        },
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// Zero-field expansion. Defines `:myapp::Tag []` (empty field vector);
// constructs an instance via zero-arg constructor; calls predicate;
// asserts true.
//
// This validates the zero-field corner case: empty `~@fields` splice
// into the constructor signature, empty Bundle inside holon_form, no
// accessors emitted, predicate still works.
#[test]
fn probe_6_zero_field_defrecord() {
    let src = r#"
(:wat::Record::def :myapp::Tag [])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
      [t (:myapp::Tag)]
      (:myapp::is-Tag? t)))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::bool(b) => assert!(
                b,
                "Probe 6: zero-field record predicate should be true"
            ),
            other => panic!("Probe 6: expected Value::bool; got {:?}", other),
        },
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

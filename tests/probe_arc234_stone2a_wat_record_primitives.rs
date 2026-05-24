//! Diagnostic probe — `:wat::core::wat-record/of` + `:wat::core::wat-record/field-at`
//! substrate primitives (arc 234 Stone 234.2a).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.2a BRIEF. Verifies
//! the substrate primitives that the Stone 234.2b defrecord macro will consume:
//!
//!   - `:wat::core::wat-record/of <class-fqdn> <struct-form> <holon-form>`
//!     constructs a `Value::wat_record` instance
//!   - `:wat::core::wat-record/field-at <wat-record> <index>` returns the
//!     field value at struct_form[index]
//!   - `:wat::core::wat-record` type registered in check.rs so signatures
//!     can declare `[v <- :wat::core::wat-record]`
//!
//! Stone 234.2a is the SUBSTRATE LAYER for the defrecord macro. The macro
//! (234.2b) generates user-facing constructors that use wat-record/of internally
//! and per-field accessors that use wat-record/field-at internally.
//!
//! Power users CAN call these primitives directly to construct wat-records by
//! hand; the macro just makes the common path ergonomic.
//!
//! Probe contracts (7):
//!   1. Construction returns wat-record (type check passes; resulting Value is
//!      Value::wat_record with class_fqdn populated correctly)
//!   2. Type extraction via :wat::core::type returns the class_fqdn (validates
//!      construction populated the variant correctly)
//!   3. Single-field construction + Rust inspection (struct_form[0] is the field)
//!   4. Multi-field construction + Rust inspection (struct_form[0]+[1] both
//!      present)
//!   5. wat-record/field-at returns field value at index (positional accessor
//!      works); test via Rust-side inspection of constructed instance + the
//!      field-at primitive called from wat (returns wat-record-typed result OR
//!      typed-narrowed via recipient inference)
//!   6. Leading-colon stripping on class_fqdn input
//!   7. Equality via holon_form — two wat-records with same construction args
//!      compare equal
//!
//! Initial state: 7/7 FAIL with `UnknownFunction(":wat::core::wat-record/of")`
//! (and similar for field-at) — the primitives don't exist.
//!
//! Post-stone: 7/7 PASS. The primitives exist + propagate correctly through
//! the type-checker.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
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
// `wat-record/of` construction returns a value of type `:wat::core::wat-record`
// — verified by destructuring the resulting Value in Rust + checking it's
// Value::wat_record with class_fqdn populated correctly.
#[test]
fn probe_1_construction_returns_wat_record() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::wat-record)
  (:wat::core::wat-record/of
    "myapp::Voltage"
    [5.0]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
      (:wat::core::Result/expect -> :wat::holon::HolonAST
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
             (:wat::holon::Atom (:wat::holon::to-holon 5.0)))])
        "Bundle failed in Probe 1"))))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::wat_record { class_fqdn, struct_form, holon_form: _ } => {
                assert_eq!(
                    class_fqdn.as_str(),
                    "myapp::Voltage",
                    "Probe 1: class_fqdn should be 'myapp::Voltage'"
                );
                assert_eq!(struct_form.len(), 1, "Probe 1: struct_form should have 1 element");
            }
            other => panic!("Probe 1: expected Value::wat_record; got {:?}", other),
        },
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// `:wat::core::type` on a constructed wat-record returns the class_fqdn
// — validates Stone 234.1's dispatch arm + Stone 234.2a's construction
// integration end-to-end.
#[test]
fn probe_2_type_returns_class_fqdn() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::let
    [v (:wat::core::wat-record/of
         "myapp::Voltage"
         [5.0]
         (:wat::holon::Bind
           (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
           (:wat::core::Result/expect -> :wat::holon::HolonAST
             (:wat::holon::Bundle
               [(:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
                  (:wat::holon::Atom (:wat::holon::to-holon 5.0)))])
             "Bundle failed in Probe 2")))]
    (:wat::core::type v)))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::String(s) => assert_eq!(
                s.as_str(),
                "myapp::Voltage",
                "Probe 2: :wat::core::type should return class_fqdn 'myapp::Voltage'"
            ),
            other => panic!("Probe 2: expected Value::String; got {:?}", other),
        },
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Single-field struct_form construction; Rust-side inspection confirms the
// field value lives at struct_form[0].
#[test]
fn probe_3_struct_form_field_at_zero() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::wat-record)
  (:wat::core::wat-record/of
    "myapp::Voltage"
    [42.0]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
      (:wat::core::Result/expect -> :wat::holon::HolonAST
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
             (:wat::holon::Atom (:wat::holon::to-holon 42.0)))])
        "Bundle failed in Probe 3"))))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::wat_record { struct_form, .. } => {
                assert_eq!(struct_form.len(), 1);
                match &struct_form[0] {
                    Value::f64(f) => assert!(
                        (f - 42.0).abs() < 1e-9,
                        "Probe 3: expected 42.0; got {}",
                        f
                    ),
                    other => panic!("Probe 3: expected f64 at index 0; got {:?}", other),
                }
            }
            other => panic!("Probe 3: expected Value::wat_record; got {:?}", other),
        },
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Multi-field struct_form construction; Rust-side inspection confirms both
// fields present in declaration order.
#[test]
fn probe_4_multi_field_construction() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::wat-record)
  (:wat::core::wat-record/of
    "myapp::Point"
    [3 4]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Point"))
      (:wat::core::Result/expect -> :wat::holon::HolonAST
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "x"))
             (:wat::holon::Atom (:wat::holon::to-holon 3)))
           (:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "y"))
             (:wat::holon::Atom (:wat::holon::to-holon 4)))])
        "Bundle failed in Probe 4"))))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::wat_record { class_fqdn, struct_form, .. } => {
                assert_eq!(class_fqdn.as_str(), "myapp::Point");
                assert_eq!(struct_form.len(), 2);
                match (&struct_form[0], &struct_form[1]) {
                    (Value::i64(a), Value::i64(b)) => {
                        assert_eq!(*a, 3, "Probe 4: struct_form[0] should be 3");
                        assert_eq!(*b, 4, "Probe 4: struct_form[1] should be 4");
                    }
                    (a, b) => panic!("Probe 4: expected (i64, i64); got ({:?}, {:?})", a, b),
                }
            }
            other => panic!("Probe 4: expected Value::wat_record; got {:?}", other),
        },
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// `wat-record/field-at` returns the field value at the positional index.
// The wat-level call returns the wat-record (typed `:wat::core::wat-record`);
// then we use a separate let-binding to extract a known field via field-at
// and return that. Result is the field value (an i64 here).
#[test]
fn probe_5_field_at_positional_access() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::i64)
  (:wat::core::let
    [v (:wat::core::wat-record/of
         "myapp::Point"
         [3 4]
         (:wat::holon::Bind
           (:wat::holon::Atom (:wat::holon::to-holon "myapp::Point"))
           (:wat::core::Result/expect -> :wat::holon::HolonAST
             (:wat::holon::Bundle
               [(:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "x"))
                  (:wat::holon::Atom (:wat::holon::to-holon 3)))
                (:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "y"))
                  (:wat::holon::Atom (:wat::holon::to-holon 4)))])
             "Bundle failed in Probe 5")))]
    (:wat::core::wat-record/field-at v 1)))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::i64(n) => assert_eq!(
                n, 4,
                "Probe 5: wat-record/field-at v 1 should return 4 (the y field)"
            ),
            other => panic!("Probe 5: expected Value::i64; got {:?}", other),
        },
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// Leading-colon stripping on class_fqdn input — pass `:myapp::Voltage` with
// leading colon; expect the constructed wat-record's class_fqdn to be
// `myapp::Voltage` (without colon) per arc 234 doctrine.
#[test]
fn probe_6_leading_colon_stripped() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::let
    [v (:wat::core::wat-record/of
         ":myapp::Voltage"
         [5.0]
         (:wat::holon::Bind
           (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
           (:wat::core::Result/expect -> :wat::holon::HolonAST
             (:wat::holon::Bundle
               [(:wat::holon::Bind
                  (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
                  (:wat::holon::Atom (:wat::holon::to-holon 5.0)))])
             "Bundle failed in Probe 6")))]
    (:wat::core::type v)))
"#;
    match run_compute(src) {
        Ok(v) => match v {
            Value::String(s) => {
                assert_eq!(
                    s.as_str(),
                    "myapp::Voltage",
                    "Probe 6: leading ':' should be stripped from class_fqdn input"
                );
                assert!(
                    !s.starts_with(':'),
                    "Probe 6: returned class_fqdn must not have leading colon"
                );
            }
            other => panic!("Probe 6: expected Value::String; got {:?}", other),
        },
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
//
// Equality via holon_form (per Stone 234.1's Eq impl) — two wat-records
// constructed with same class + same holon_form structure must compare equal.
// Construct via wat-record/of twice with identical args; assert equality
// via Rust-side PartialEq.
#[test]
fn probe_7_equality_via_holon_form() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::wat-record)
  (:wat::core::wat-record/of
    "myapp::Voltage"
    [5.0]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
      (:wat::core::Result/expect -> :wat::holon::HolonAST
        (:wat::holon::Bundle
          [(:wat::holon::Bind
             (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
             (:wat::holon::Atom (:wat::holon::to-holon 5.0)))])
        "Bundle failed in Probe 7"))))
"#;
    let a = run_compute(src).expect("Probe 7: first construction failed");
    let b = run_compute(src).expect("Probe 7: second construction failed");
    assert_eq!(
        a, b,
        "Probe 7: two wat-records constructed with identical args must be equal"
    );
}

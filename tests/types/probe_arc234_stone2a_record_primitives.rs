//! Diagnostic probe — `:wat::holon::Record::of` + `:wat::Record/field-at`
//! substrate primitives (arc 234 Stone 234.2a).
//!
//! Wat source: tests/types/probe_arc234_stone2a_record_primitives.wat (loaded via startup_beside).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run(fn_name: &str) -> Result<Value, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!(&format!("({fn_name})")).map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_1_construction_returns_wat_record() {
    match run(":user::probe-1") {
        Ok(v) => match v {
            Value::wat__holon__Record { class_fqdn, struct_form, holon_form: _ } => {
                assert_eq!(
                    class_fqdn.as_str(),
                    "myapp::Voltage",
                    "Probe 1: class_fqdn should be 'myapp::Voltage'"
                );
                assert_eq!(struct_form.len(), 1, "Probe 1: struct_form should have 1 element");
            }
            other => panic!("Probe 1: expected Value::wat__holon__Record; got {:?}", other),
        },
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_2_type_returns_class_fqdn() {
    match run(":user::probe-2") {
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
#[test]
fn probe_3_struct_form_field_at_zero() {
    match run(":user::probe-3") {
        Ok(v) => match v {
            Value::wat__holon__Record { struct_form, .. } => {
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
            other => panic!("Probe 3: expected Value::wat__holon__Record; got {:?}", other),
        },
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_4_multi_field_construction() {
    match run(":user::probe-4") {
        Ok(v) => match v {
            Value::wat__holon__Record { class_fqdn, struct_form, .. } => {
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
            other => panic!("Probe 4: expected Value::wat__holon__Record; got {:?}", other),
        },
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_5_field_at_positional_access() {
    match run(":user::probe-5") {
        Ok(v) => match v {
            Value::i64(n) => assert_eq!(
                n, 4,
                "Probe 5: Record/field-at v 1 should return 4 (the y field)"
            ),
            other => panic!("Probe 5: expected Value::i64; got {:?}", other),
        },
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 RETIRED ────────────────────────────────────────────────────────
// Previously: "leading-colon stripping on class_fqdn input." Retired in post-doctrine.

// ─── Probe 7 ────────────────────────────────────────────────────────────────
#[test]
fn probe_7_equality_via_holon_form() {
    let a = run(":user::probe-7").expect("Probe 7: first construction failed");
    let b = run(":user::probe-7").expect("Probe 7: second construction failed");
    assert_eq!(
        a, b,
        "Probe 7: two records constructed with identical args must be equal"
    );
}

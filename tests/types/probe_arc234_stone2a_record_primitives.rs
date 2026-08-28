//! Diagnostic probe — construction shape / `:wat::core::type` / `Record/field-at` /
//! hologram equality over a `:wat::holon::defrecord`-declared HolonRecord (arc 234 Stone 234.2a).
//!
//! Arc 296 G-1b — re-expressed: the fixture used to hand-build holograms through the
//! `:wat::holon::Record::of` primitive, which was deleted ("finish the kill", arc 294.c.2a
//! superseded it with `aggregate-new`). It now declares `:myapp::Voltage` / `:myapp::Point`
//! with `:wat::holon::defrecord` and constructs through the generated ctor — the assertions
//! below are unchanged; only how the fixture builds its input changed.
//!
//! Wat source: tests/types/probe_arc234_stone2a_record_primitives.wat (loaded via startup_beside).

use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;
use wat::types::Nature;

fn run(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(StartupError::from)
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_1_construction_returns_wat_record() {
    match run(":user::probe-1") {
        Ok(v) => match v {
            Value::Aggregate(a) if a.nature == Nature::HolonRecord => {
                assert_eq!(
                    a.class.as_ref(),
                    "myapp::Voltage",
                    "Probe 1: class should be 'myapp::Voltage'"
                );
                assert_eq!(a.fields.len(), 1, "Probe 1: fields should have 1 element");
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
            Value::Aggregate(a) if a.nature == Nature::HolonRecord => {
                assert_eq!(a.fields.len(), 1);
                match &a.fields[0] {
                    Value::f64(f) => assert!(
                        (f - 42.0).abs() < 1e-9,
                        "Probe 3: expected 42.0; got {}",
                        f
                    ),
                    other => panic!("Probe 3: expected f64 at index 0; got {:?}", other),
                }
            }
            other => panic!("Probe 3: expected Value::Aggregate(HolonRecord); got {:?}", other),
        },
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_4_multi_field_construction() {
    match run(":user::probe-4") {
        Ok(v) => match v {
            Value::Aggregate(a) if a.nature == Nature::HolonRecord => {
                assert_eq!(a.class.as_ref(), "myapp::Point");
                assert_eq!(a.fields.len(), 2);
                match (&a.fields[0], &a.fields[1]) {
                    (Value::i64(x), Value::i64(y)) => {
                        assert_eq!(*x, 3, "Probe 4: fields[0] should be 3");
                        assert_eq!(*y, 4, "Probe 4: fields[1] should be 4");
                    }
                    (x, y) => panic!("Probe 4: expected (i64, i64); got ({:?}, {:?})", x, y),
                }
            }
            other => panic!("Probe 4: expected Value::Aggregate(HolonRecord); got {:?}", other),
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

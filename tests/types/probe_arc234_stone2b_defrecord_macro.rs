//! Diagnostic probe — `:wat::core::defrecord` macro (arc 234 Stone 234.2b).
//!
//! Wat source: tests/types/probe_arc234_stone2b_defrecord_macro.wat (loaded via startup_beside).

use wat::freeze::call_beside_value;
use wat::runtime::Value;
use wat::types::Nature;

fn run(fn_name: &str) -> Result<Value, String> {
    call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_1_single_field_construction() {
    match run(":user::probe-1") {
        Ok(v) => match v {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                assert_eq!(
                    a.class.as_ref(),
                    "myapp::Voltage",
                    "Probe 1: class should be 'myapp::Voltage'"
                );
                assert_eq!(
                    a.fields.len(),
                    1,
                    "Probe 1: fields should have 1 element"
                );
                match &a.fields[0] {
                    Value::f64(f) => assert!(
                        (f - 5.0).abs() < 1e-9,
                        "Probe 1: fields[0] should be 5.0; got {}",
                        f
                    ),
                    other => panic!("Probe 1: expected f64 at index 0; got {:?}", other),
                }
            }
            other => panic!("Probe 1: expected Value::Aggregate(Record); got {:?}", other),
        },
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_2_per_field_accessor_returns_value() {
    match run(":user::probe-2") {
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
#[test]
fn probe_3_predicate_true_on_matching_class() {
    match run(":user::probe-3") {
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
#[test]
fn probe_4_predicate_false_on_non_matching_class() {
    match run(":user::probe-4") {
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
#[test]
fn probe_5_multi_field_accessors_in_order() {
    match run(":user::probe-5") {
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
#[test]
fn probe_6_zero_field_defrecord() {
    match run(":user::probe-6") {
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

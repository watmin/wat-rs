//! Diagnostic probe — `:wat::core::record?` + `:wat::core::record->map`
//! (arc 234 Stone 234.3a).
//!
//! Wat source: tests/types/probe_arc234_stone3a_record_read_verbs.wat (loaded via startup_beside).

use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;

fn run(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(StartupError::from)
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// record? returns true on a constructed wat-record.
#[test]
fn probe_1_record_q_true_on_record() {
    match run(":user::probe-1") {
        Ok(Value::bool(b)) => assert!(b, "Probe 1: record? on a record should be true"),
        Ok(other) => panic!("Probe 1: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// record? returns false on a non-record (i64).
#[test]
fn probe_2_record_q_false_on_i64() {
    match run(":user::probe-2") {
        Ok(Value::bool(b)) => assert!(!b, "Probe 2: record? on an i64 should be false"),
        Ok(other) => panic!("Probe 2: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// record->map on a single-field record returns a one-entry HashMap.
#[test]
fn probe_3_record_to_map_single_field() {
    match run(":user::probe-3") {
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
// record->map on a 3-field heterogeneous record returns all three entries.
#[test]
fn probe_4_record_to_map_multi_field_heterogeneous() {
    match run(":user::probe-4") {
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
#[test]
fn probe_5_record_to_map_zero_field() {
    match run(":user::probe-5") {
        Ok(Value::bool(b)) => assert!(b, "Probe 5: zero-field record->map should produce empty HashMap"),
        Ok(other) => panic!("Probe 5: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// Composition: predicate-then-map pattern (defensive usage).
#[test]
fn probe_6_predicate_then_map_composition() {
    match run(":user::probe-6") {
        Ok(Value::f64(f)) => assert!(
            (f - 99.0).abs() < 1e-9,
            "Probe 6: predicate-true → map-get path should return 99.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 6: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

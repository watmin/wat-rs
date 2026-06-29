//! Diagnostic probe — `:wat::core::Record/assoc` substrate primitive (arc 234 Stone 234.3b).
//!
//! Wat source: tests/types/probe_arc234_stone3b_record_assoc.wat (loaded via startup_beside).
//!
//! Probes 3 and 4 are expected to produce errors at eval time (unknown field / type mismatch).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run(fn_name: &str) -> Result<Value, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!(&format!("({fn_name})")).map_err(|e| format!("parse: {:?}", e))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_1_single_field_update() {
    match run(":user::probe-1") {
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
    match run(":user::probe-2") {
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
//
// Record/assoc with an unknown field key → expect error at eval time.
#[test]
fn probe_3_unknown_field_errors() {
    match run(":user::probe-3") {
        Ok(v) => panic!("Probe 3 FAILED: expected UnknownField error; got Ok({:?})", v),
        Err(msg) => assert!(
            msg.to_lowercase().contains("unknown") || msg.contains("nonexistent"),
            "Probe 3: expected error mentioning unknown/nonexistent field; got {}",
            msg
        ),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Record/assoc with a wrong-type value (f64 field, i64 given) → expect error.
#[test]
fn probe_4_type_mismatch_errors() {
    match run(":user::probe-4") {
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
//
// Original record unchanged after assoc (immutability).
#[test]
fn probe_5_original_record_unchanged() {
    match run(":user::probe-5") {
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
//
// Compose multiple assocs.
#[test]
fn probe_6_compose_multiple_assocs() {
    match run(":user::probe-6") {
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

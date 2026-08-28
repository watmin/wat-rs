//! Diagnostic probe — `:wat::core::Record/assoc` substrate primitive (arc 234 Stone 234.3b).
//!
//! Wat source: tests/types/probe_arc234_stone3b_record_assoc.wat (loaded via startup_beside).
//!
//! Probes 3 and 4 are expected to produce errors at eval time (unknown field / type mismatch).

use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;

/// The structured EDN body of a startup/eval failure.
///
/// The error IS data — `#wat.runtime/UnknownField {…}` — so it is asserted STRUCTURALLY
/// against a co-located `.edn` golden, never by `.contains` on a rendered string. These
/// sites previously carried a loose-assert exemption because the span embedded an
/// ABSOLUTE machine-specific path and no exact assertion was possible. Spans now carry a
/// repo-relative path (`load::span_display_path`), so the reason no longer holds and the
/// runes are retired rather than re-justified — the exemption existed for a constraint
/// that is gone.
///
/// Arc 296 Stone M: `run` used to flatten the error to a `String` prefixed with
/// `"eval: "`, which this helper stripped. `run` now returns the typed `StartupError`
/// directly, whose `Display`/`Debug` IS the raw EDN (no prefix to strip) — this is a
/// straight `to_string()`, kept as a named helper only so the call sites read the same.
fn edn_body(e: &StartupError) -> String {
    e.to_string()
}


fn run(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(StartupError::from)
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
        Err(msg) => wat::assert_edn_matches_file!(edn_body(&msg), "probe_arc234_stone3b_record_assoc__probe3_unknown_field.edn"),
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
        Err(msg) => wat::assert_edn_matches_file!(edn_body(&msg), "probe_arc234_stone3b_record_assoc__probe4_type_mismatch.edn"),
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

//! Diagnostic probe — keyword-as-accessor fall-through (arc 234 Stone 234.3c).
//!
//! Wat source: tests/types/probe_arc234_stone3c_keyword_accessor.wat (loaded via startup_beside).
//!
//! Probe 3 is expected to produce an error at eval time (unknown field :nonexistent).

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
//
// (:magnitude v) on a single-field record returns the field value.
#[test]
fn probe_1_keyword_accessor_on_single_field_record() {
    match run(":user::probe-1") {
        Ok(Value::f64(f)) => assert!(
            (f - 5.0).abs() < 1e-9,
            "Probe 1: (:magnitude v) should return 5.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 1: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// (:b t) on a multi-field record returns the correctly-typed value.
#[test]
fn probe_2_keyword_accessor_on_multi_field_record() {
    match run(":user::probe-2") {
        Ok(Value::String(s)) => assert_eq!(
            s.as_str(),
            "hello",
            "Probe 2: (:b t) should return 'hello'; got {}",
            s.as_str()
        ),
        Ok(other) => panic!("Probe 2: expected Value::String; got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// (:nonexistent v) on a record → error.
#[test]
fn probe_3_unknown_field_on_record_errors() {
    match run(":user::probe-3") {
        Ok(v) => panic!(
            "Probe 3 FAILED: expected error on unknown field; got Ok({:?})",
            v
        ),
        Err(msg) => wat::assert_edn_matches_file!(edn_body(&msg), "probe_arc234_stone3c_keyword_accessor__probe3_unknown_field.edn"),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// (:port m) on a HashMap with :port key → Some(8080), unwrapped via Option/expect.
#[test]
fn probe_4_keyword_accessor_on_hashmap_some() {
    match run(":user::probe-4") {
        Ok(Value::i64(n)) => assert_eq!(
            n, 8080,
            "Probe 4: (:port m) should return Some(8080) → 8080; got {}",
            n
        ),
        Ok(other) => panic!("Probe 4: expected Value::i64; got {:?}", other),
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// (:missing m) on a HashMap without :missing → None → bool true via match.
#[test]
fn probe_5_keyword_accessor_on_hashmap_none() {
    match run(":user::probe-5") {
        Ok(Value::bool(b)) => assert!(
            b,
            "Probe 5: (:missing m) on map without :missing should yield None → true"
        ),
        Ok(other) => panic!("Probe 5: expected Value::bool; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// (:x p) on a defstruct instance returns the field value (struct keyword access).
#[test]
fn probe_6_keyword_accessor_on_struct() {
    match run(":user::probe-6") {
        Ok(Value::i64(n)) => assert_eq!(
            n, 3,
            "Probe 6: (:x p) on struct should return 3; got {}",
            n
        ),
        Ok(other) => panic!("Probe 6: expected Value::i64; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

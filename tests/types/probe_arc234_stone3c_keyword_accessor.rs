//! Diagnostic probe — keyword-as-accessor fall-through (arc 234 Stone 234.3c).
//!
//! Wat source: tests/types/probe_arc234_stone3c_keyword_accessor.wat (loaded via startup_beside).
//!
//! Probe 3's unknown field is refused at CHECK time (arc 251), not eval — so it lives in its
//! own fixture (`probe_arc251_type_the_polymorphic_accessor__unknown_field.wat`); a check error
//! in the shared beside file would block probes 1/2/4/5/6 from loading.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;


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
    // Arc 251 — a missing field on a known receiver is a located CHECK error.
    // `:user::probe-3` was removed from the beside file so probes 1/2/4/5/6
    // still load; the same program is `…__unknown_field.wat`.
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join("probe_arc251_type_the_polymorphic_accessor__unknown_field.wat");
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(&p)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    assert_eq!(out.status.code().unwrap_or(-1), 1, "got: {s}");
    // Arc 251 — the error IS DATA. Asserted STRUCTURALLY against a co-located golden,
    // never `.contains` on a rendered face: that is the exact pattern
    // `tests/lint/no_loose_string_assert.rs` bans, and the loose-assert runes on these
    // sites were RETIRED (spans are repo-relative now), not re-justified.
    wat::assert_edn_matches_file!(
        s,
        "probe_arc234_stone3c_keyword_accessor__probe3_unknown_field.edn"
    );
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

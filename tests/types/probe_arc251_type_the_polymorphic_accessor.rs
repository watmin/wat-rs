//! PROBE — the bare keyword accessor `(:field receiver)` was accepted and never typed.
//!
//! Arc 236.2's "polymorphic accessor placeholder" returned a fresh var, which
//! unifies with anything (bug ①, a lie accepted). A parametric Aggregate was
//! absent from the `acceptable` match, so `(:x cell)` on `(:Cell :- [i64])` was
//! UnknownCallee (bug ②, the truth refused). Named `:T/field` and `{:keys}`
//! already instantiate; this stone types the remaining shorthand.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_check(case: &str) -> (i32, String) {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc251_type_the_polymorphic_accessor__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
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
    (out.status.code().unwrap_or(-1), s)
}

fn check(case: &str) -> i32 {
    run_check(case).0
}

fn run_program(case: &str) -> (i32, String) {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc251_type_the_polymorphic_accessor__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&p)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}

#[test]
fn record_monomorphic_truth_is_accepted() {
    let (code, out) = run_check("record_mono_truth");
    assert_eq!(code, 0, "got: {out}");
}

#[test]
fn record_monomorphic_lie_is_refused() {
    let (code, out) = run_check("record_mono_lie");
    assert_eq!(code, 1, "got: {out}");
    // The refusal IS DATA — structural golden, not `.contains` on a rendered face.
    // Pins the exact error kind, which is strictly stronger than the substring test
    // it replaces (`tests/lint/no_loose_string_assert.rs`).
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_type_the_polymorphic_accessor__record_mono_lie.edn"
    );
}

#[test]
fn record_parametric_truth_is_accepted() {
    let (code, out) = run_check("record_param_truth");
    assert_eq!(code, 0, "got: {out}");
}

#[test]
fn record_parametric_lie_is_refused() {
    let (code, out) = run_check("record_param_lie");
    assert_eq!(code, 1, "got: {out}");
    // The refusal IS DATA — structural golden, not `.contains` on a rendered face.
    // Pins the exact error kind, which is strictly stronger than the substring test
    // it replaces (`tests/lint/no_loose_string_assert.rs`).
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_type_the_polymorphic_accessor__record_param_lie.edn"
    );
}

#[test]
fn variant_parametric_truth_is_accepted() {
    let (code, out) = run_check("variant_param_truth");
    assert_eq!(code, 0, "got: {out}");
}

#[test]
fn variant_parametric_lie_is_refused() {
    let (code, out) = run_check("variant_param_lie");
    assert_eq!(code, 1, "got: {out}");
    // The refusal IS DATA — structural golden, not `.contains` on a rendered face.
    // Pins the exact error kind, which is strictly stronger than the substring test
    // it replaces (`tests/lint/no_loose_string_assert.rs`).
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_type_the_polymorphic_accessor__variant_param_lie.edn"
    );
}

#[test]
fn unknown_field_on_a_known_receiver_is_refused() {
    let (code, out) = run_check("unknown_field");
    assert_eq!(code, 1, "got: {out}");
    // The refusal IS DATA — structural golden, not `.contains` on a rendered face.
    // Pins the exact error kind, which is strictly stronger than the substring test
    // it replaces (`tests/lint/no_loose_string_assert.rs`).
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_type_the_polymorphic_accessor__unknown_field.edn"
    );
}

#[test]
fn hashmap_receiver_still_accepts_the_keyword_accessor() {
    let (code, out) = run_check("hashmap_receiver");
    assert_eq!(code, 0, "got: {out}");
}

#[test]
fn named_record_accessor_lie_still_refused() {
    assert_eq!(check("named_record_accessor_lie"), 1);
}

#[test]
fn named_variant_accessor_lie_still_refused() {
    assert_eq!(check("named_variant_accessor_lie"), 1);
}

#[test]
fn variant_keys_lie_still_refused() {
    assert_eq!(check("variant_keys_lie"), 1);
}

#[test]
fn named_variant_accessor_truth_still_accepted() {
    assert_eq!(check("named_variant_accessor_truth"), 0);
}

#[test]
fn correct_read_runs_and_prints() {
    let (code, out) = run_program("correct_read_runs");
    assert_eq!(code, 0, "got: {out}");
    assert_eq!(out.trim(), "42", "the field's value must be the WHOLE of stdout");
}

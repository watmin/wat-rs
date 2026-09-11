//! PROBE — a parametric aggregate's type argument is accepted and then ignored.
//!
//! Measured 2026-09-10: a signature may name `(:u::Cell :- [:wat::core::i64])`
//! and both `{:keys}` and a variant's `:T/field` accessor hand back the
//! *uninstantiated* parameter (`:X` / `:T`), not `i64`. Record accessors and
//! `match` on the enclosing enum already instantiate. No test covered the
//! concrete-argument case — A-2's `process-full-box` uses the function's own
//! `:T`, so the uninstantiated field type and the return type agreed by
//! accident.
//!
//! Site ① `process_let_binding` keys-destructure dropped Parametric args and
//! used the TypeDef's declared field types verbatim. Site ② `register_enum_methods`
//! never minted `:Enum.Variant/field` schemes (records get them from
//! `register_aggregate_methods` with `type_params`, so `instantiate` at the
//! call site works).

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_check(case: &str) -> (i32, String) {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc251_instantiate_the_type_argument__{case}.wat"));
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
        .join(format!("probe_arc251_instantiate_the_type_argument__{case}.wat"));
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

/// SUBJECT — parametric record, `{:keys}` must yield i64, not :X.
#[test]
fn parametric_record_keys_yields_the_argument() {
    let (code, out) = run_check("parametric_record_keys");
    assert_eq!(code, 0, "got: {out}");
}

/// SUBJECT — parametric struct, `{:keys}` must yield i64, not :X.
#[test]
fn parametric_struct_keys_yields_the_argument() {
    let (code, out) = run_check("parametric_struct_keys");
    assert_eq!(code, 0, "got: {out}");
}

/// SUBJECT — parametric variant, `{:keys}` must yield i64, not :T.
#[test]
fn parametric_variant_keys_yields_the_argument() {
    let (code, out) = run_check("parametric_variant_keys");
    assert_eq!(code, 0, "got: {out}");
}

/// SUBJECT — parametric variant's `:T/field` accessor must yield i64, not :T.
#[test]
fn parametric_variant_accessor_yields_the_argument() {
    let (code, out) = run_check("parametric_variant_accessor");
    assert_eq!(code, 0, "got: {out}");
}

/// CONTROL — parametric record's `:T/field` accessor already instantiated.
#[test]
fn parametric_record_accessor_still_yields_the_argument() {
    let (code, out) = run_check("parametric_record_accessor");
    assert_eq!(code, 0, "got: {out}");
}

/// CONTROL — reaching the field through the enum and a `match` already instantiated.
#[test]
fn parametric_enum_via_match_still_yields_the_argument() {
    let (code, out) = run_check("parametric_enum_via_match");
    assert_eq!(code, 0, "got: {out}");
}

/// CONTROL — monomorphic record `{:keys}` was never the defect.
#[test]
fn non_parametric_record_keys_still_works() {
    assert_eq!(check("non_parametric_record_keys"), 0);
}

/// CONTROL — monomorphic variant `{:keys}` was never the defect.
#[test]
fn non_parametric_variant_keys_still_works() {
    assert_eq!(check("non_parametric_variant_keys"), 0);
}

/// CONTROL — `Demo.Has <: Demo` is one-way widening; must stay accepted.
#[test]
fn variant_still_flows_where_the_enum_is_expected() {
    assert_eq!(check("variant_widens_to_enum"), 0);
}

/// CONTROL — widening is one-way; an enum value is not a variant.
#[test]
fn enum_still_does_not_narrow_to_a_variant() {
    let (code, out) = run_check("enum_does_not_narrow");
    assert_eq!(code, 1, "got: {out}");
    // The whole refusal, captured from the binary — never a `contains`. The DISCRIMINATION this
    // row exists to make (a TypeMismatch naming both types, NOT an UnknownNamedType) is visible
    // in the golden itself, and a change in WHICH error fires is a finding, not a rephrase.
    wat::assert_edn_eq!(
        out,
        include_str!("probe_arc251_instantiate_the_type_argument__enum_does_not_narrow_stderr.edn")
    );
}

/// RUNTIME twin of the variant accessor — the scheme is not enough if the
/// synthesized body cannot read an Enum value.
#[test]
fn parametric_variant_accessor_runs() {
    let (code, out) = run_program("parametric_variant_accessor_runs");
    assert_eq!(code, 0, "got: {out}");
    assert_eq!(out.trim(), "42", "the field's value must be the WHOLE of stdout");
}

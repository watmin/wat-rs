//! PROBE — Variant <: Enum must flow through a function argument, not only a value slot.
//!
//! `subtype?` already answers true (the edge has been registered since A-2). A
//! `defrecord` slot `Alarm<Op.Mark>` vs `Alarm<Op>` already assignable via the
//! SAME-head parametric arm. foldl's reducer is a FUNCTION type, and `assignable`
//! fell through to invariant `unify` — so the lattice never ran. Negative control:
//! Demo.Has → Demo accepted, Demo → Demo.Has refused, Demo.Has → Demo.Has accepted.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_check(case: &str) -> (i32, String) {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc251_enrol_the_variant_in_the_lattice__{case}.wat"));
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

/// Demo.Has → Demo slot ACCEPTED.
#[test]
fn variant_flows_where_the_enum_is_expected() {
    let (code, out) = run_check("variant_to_enum");
    assert_eq!(code, 0, "got: {out}");
}

/// Demo → Demo.Has slot REFUSED. Widening is one-way.
#[test]
fn enum_does_not_narrow_to_a_variant() {
    let (code, out) = run_check("enum_to_variant");
    assert_eq!(code, 1, "got: {out}");
    // The REFUSAL is the claim, so it is pinned STRUCTURALLY, not by a substring.
    // `.contains("TypeMismatch")` passes on any TypeMismatch anywhere in the file —
    // including one that refuses for a reason this probe is not about
    // (`tests/lint/no_loose_string_assert.rs`).
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_enrol_the_variant_in_the_lattice__enum_to_variant.edn"
    );
}

/// Demo.Has → Demo.Has slot ACCEPTED.
#[test]
fn variant_flows_where_the_same_variant_is_expected() {
    let (code, out) = run_check("variant_to_variant");
    assert_eq!(code, 0, "got: {out}");
}

/// THE DISCRIMINATOR — a fn that takes Alarm<Op> (wider) stands in for a
/// slot that will pass Alarm<Op.Mark> (narrower). This is foldl's reducer.
#[test]
fn fn_taking_the_enum_accepts_a_variant_argument_slot() {
    let (code, out) = run_check("fn_wide_param_for_narrow_slot");
    assert_eq!(code, 0, "got: {out}");
}

/// Reverse of the discriminator — a fn that takes only Alarm<Op.Mark> cannot
/// stand in for a slot that will pass Alarm<Op> (an Other would not fit).
#[test]
fn fn_taking_the_variant_does_not_accept_an_enum_argument_slot() {
    let (code, out) = run_check("fn_narrow_param_for_wide_slot");
    assert_eq!(code, 1, "got: {out}");
    // The REFUSAL is the claim, so it is pinned STRUCTURALLY, not by a substring.
    // `.contains("TypeMismatch")` passes on any TypeMismatch anywhere in the file —
    // including one that refuses for a reason this probe is not about
    // (`tests/lint/no_loose_string_assert.rs`).
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_enrol_the_variant_in_the_lattice__fn_narrow_param_for_wide_slot.edn"
    );
}

/// Value-level: Alarm<Op.Mark> flows where Alarm<Op> is expected (SAME-head lattice).
#[test]
fn alarm_of_variant_flows_where_alarm_of_enum_is_expected() {
    assert_eq!(check("alarm_variant_to_enum"), 0);
}

/// Value-level reverse: Alarm of a sibling variant does not flow into Alarm<Op.Mark>.
#[test]
fn alarm_of_sibling_does_not_flow_into_alarm_of_variant() {
    let (code, out) = run_check("alarm_sibling_to_variant");
    assert_eq!(code, 1, "got: {out}");
}

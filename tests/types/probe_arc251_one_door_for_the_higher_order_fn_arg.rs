//! PROBE — map / mapv / filter must share foldl's assignable door.
//!
//! A Vector of `Op.Mark` passed to a function over `Op` is REFUSED today
//! (unify on Fn args) and must be ACCEPTED after step 2. Narrowing stays
//! one-way: a function over `Op.Mark` is still REFUSED for a Vector of `Op`.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_check(case: &str) -> (i32, String) {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc251_one_door_for_the_higher_order_fn_arg__{case}.wat"));
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

/// Vector of Op.Mark, mapped by a fn over Op — the lattice through map.
#[test]
fn map_variant_elems_accepted_by_enum_fn() {
    let (code, out) = run_check("map_variant_elems");
    assert_eq!(code, 0, "got: {out}");
}

/// Same shape, eager mapv.
#[test]
fn mapv_variant_elems_accepted_by_enum_fn() {
    let (code, out) = run_check("mapv_variant_elems");
    assert_eq!(code, 0, "got: {out}");
}

/// Same shape, filter predicate over the parent enum.
#[test]
fn filter_variant_elems_accepted_by_enum_pred() {
    let (code, out) = run_check("filter_variant_elems");
    assert_eq!(code, 0, "got: {out}");
}

/// foldl already went through assignable; this row stays green across both steps.
#[test]
fn foldl_variant_elems_accepted_by_enum_fn() {
    let (code, out) = run_check("foldl_variant_elems");
    assert_eq!(code, 0, "got: {out}");
}

/// STOP-1 — mapv's U is still bound: Vector of i64, not a leftover fresh var.
#[test]
fn mapv_output_element_is_still_concrete() {
    let (code, out) = run_check("mapv_output_concrete");
    assert_eq!(code, 0, "got: {out}");
}

/// Narrowing: pred over Op.Mark refused for a Vector of Op.
/// filter's expected return is bool, so the TypeMismatch has no drifting :?N.
#[test]
fn filter_enum_elems_refused_by_variant_pred() {
    let (code, out) = run_check("filter_enum_elems_narrow");
    assert_eq!(code, 1, "got: {out}");
    wat::assert_edn_matches_file!(
        out,
        "probe_arc251_one_door_for_the_higher_order_fn_arg__filter_enum_elems_narrow.edn"
    );
}

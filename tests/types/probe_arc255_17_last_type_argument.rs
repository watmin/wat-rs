//! Stone 255.17 — the last type argument of a parametric annotation is not forgotten (arc 255).
//!
//! WHY: in a generic `defn`, a parameter annotated `(:probe::R :- [A B C])` typed its LAST
//! argument as a hole whenever that argument was a ONE-LETTER type variable (`T`, `C`, `S`, `E`,
//! `U`, …; a multi-letter `Elem` was always checked). `(:probe::R/z r)` inferred an unbound
//! fresh var, so `-> :wat::core::i64` accepted a `C`; `(match o [Some {:value v} v] …)` on an
//! `(Option :- [T])` did the same, and `--check` rc=0 programs then failed at run with
//! `:wat::i64::+: expected i64, got wat::core::String`.
//!
//! SITE (measured): `assignable`'s same-head parametric "transport slot instantiates" arm
//! (`src/check.rs`). It unified only the PREFIX args and returned `true` when the last pair
//! satisfied `transport_param_instantiates`, whose `is_type_param_letter` test counts every
//! `Var` AND every single-uppercase-letter name as a transport "letter" — so the callee's
//! fresh var for the last slot was never bound to the caller's rigid `:C`. The fix was parked
//! at 255.17 (it reddened the 57 process-child tests until the child main declared its
//! transport, 255.25) and landed at 255.27 (C-b5) by DELETING the arm: a transport is an
//! ordinary type parameter, so the last argument goes through `unify` like every other.
//!
//! Every `.wat.bad` row is a generic `defn :probe::k` returning one field as the wrong type;
//! a correct checker refuses each one with `ReturnTypeMismatch { got: <the field's type> }`.
//! Rows marked (was 0) were accepted before the fix; the others are CONTROLS that were
//! already refused (first/middle arg, concrete last arg, bare var, multi-letter var) and prove
//! the assertion can fail for the right reason. `twins.wat` is the accepted, running half.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const DIR: &str = "tests/types/probe_arc255_17_last_type_argument";

/// Assert `<suffix>.wat.bad` is refused with `:probe::k`'s body producing `got`, declared `expected`.
fn refused(suffix: &str, got_ty: &str, expected_ty: &str) {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must fail check"));
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("{path}: expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::k" && expected == expected_ty && got == got_ty);
}

const I64: &str = ":wat::core::i64";

#[test]
fn record_first_arg_control() {
    refused("rec3_first", ":A", I64);
}

#[test]
fn record_middle_arg_control() {
    refused("rec3_middle", ":B", I64);
}

#[test]
fn record_last_arg_is_checked() {
    // was 0
    refused("rec3_last", ":C", I64);
}

#[test]
fn record_only_arg_is_checked() {
    // was 0
    refused("rec1", ":T", I64);
}

#[test]
fn record_last_var_after_a_concrete_first_is_checked() {
    // was 0
    refused("rec_concrete_first", ":T", I64);
}

#[test]
fn record_concrete_last_arg_control() {
    refused("rec_concrete_last", ":wat::core::String", I64);
}

#[test]
fn enum_first_arm_control() {
    refused("enum2_first", ":T", I64);
}

#[test]
fn enum_last_arm_is_checked() {
    // was 0
    refused("enum2_last", ":S", I64);
}

#[test]
fn enum_only_arg_is_checked() {
    // was 0
    refused("enum1", ":T", I64);
}

#[test]
fn result_ok_arm_control() {
    refused("result_ok", ":T", I64);
}

#[test]
fn result_err_arm_is_checked() {
    // was 0
    refused("result_err", ":E", I64);
}

#[test]
fn result_ok_arm_with_concrete_last_control() {
    refused("result_concrete_last", ":T", I64);
}

#[test]
fn option_t_is_checked() {
    // was 0
    refused("option_t", ":T", I64);
}

#[test]
fn option_u_is_checked() {
    // was 0
    refused("option_u", ":U", I64);
}

#[test]
fn option_multi_letter_var_control() {
    refused("option_multi_letter", ":Elem", I64);
}

#[test]
fn bare_var_control() {
    refused("bare_var", ":T", I64);
}

// rune:lint(no-inlined-wat) — the two `"(:probe::R :- [...])"` literals are golden COMPARISON
// text for the refusal's rendered `expected`/`got` fields; nothing here builds or runs a program
// from a string — every program is a `.wat` fixture.
#[test]
fn a_rigid_t_is_not_a_rigid_u_in_the_last_slot() {
    // was 0 — the same site: letter↔letter was a "transport instantiation".
    refused("rigid_t_as_u", "(:probe::R :- [:T])", "(:probe::R :- [:U])");
}

#[test]
fn the_runtime_reproducer_is_refused_at_check() {
    // was 0 at `--check`, then `:wat::i64::+: expected i64, got wat::core::String` at run.
    refused("runtime", ":T", I64);
}

#[test]
fn the_twins_check_and_run() {
    let rel = format!("{DIR}_twins.wat");
    startup_from_file(&rel)
        .unwrap_or_else(|e| panic!("{rel}: each field at its own type variable must be accepted: {e:?}"));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg(&rel)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "run must succeed; stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["\"z\"", "\"b\"", "\"e\"", "\"o\"", "\"3\""], "stdout:\n{stdout}");
}

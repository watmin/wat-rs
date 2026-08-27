//! Strike (examinare probe) — 251.7 implicit generics (HM / bare type-vars).
//!
//! F01 suffix-generic is REALLY checked: ill-typed call is REJECTED.
//! F02 suffix-generic valid call PASSES.
//! bare_var_no_suffix_now_checks: bare-var form auto-generalizes and PASSES.
//! bare_var_no_suffix_rejects_illtyped: bare-var rejects T:=i64 + b=String.
//! bare_var_two_instantiations: bare-var generic called at i64 and bool independently.
//!
//! Run: `cargo test --release --test probe_arc251_implicit_generics`

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};

#[test]
fn fact_01_suffix_generic_is_really_checked() {
    let r = startup_from_file("tests/resolve/probe_arc251_implicit_generics_fact01.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":user::pair-first"
            && param == "#2"
            && expected == ":wat::core::i64"
            && got == ":wat::core::String"
    );
}

#[test]
fn fact_02_suffix_generic_valid_call_passes() {
    assert!(
        startup_beside(file!()).is_ok(),
        "valid same-type call (pair-first 1 2) must check: fixture failed startup"
    );
}

#[test]
fn bare_var_no_suffix_now_checks() {
    // Stone 251.7: the faithful bare-var-no-suffix form must now auto-generalize
    // from the signature (T appears in params + return → collected → type_params = ["T"]).
    // (pair-first2 1 2) — both args i64, return i64 — must CHECK.
    assert!(
        startup_beside(file!()).is_ok(),
        "bare-var-no-suffix generic MUST check after 251.7 (auto-generalize free sig vars). \
         Fixture startup failed."
    );
}

#[test]
fn bare_var_no_suffix_rejects_illtyped() {
    // Proves the auto-generalized vars are REALLY unified (not opaquely accepted).
    let r = startup_from_file("tests/resolve/probe_arc251_implicit_generics_bare.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":user::pair-first3"
            && param == "#2"
            && expected == ":wat::core::i64"
            && got == ":wat::core::String"
    );
}

#[test]
fn bare_var_two_instantiations() {
    // Bare-var generic called at two distinct types — proves distinct fresh vars
    // per call site (no aliasing between the two calls).
    assert!(
        startup_beside(file!()).is_ok(),
        "bare-var generic must check at both i64 and bool call sites independently. \
         Fixture startup failed."
    );
}

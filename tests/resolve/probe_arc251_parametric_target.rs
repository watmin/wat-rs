//! Strike 1 (examinare verify-probe) — does the faithful-Clojure parametric type TARGET
//! type-check END-TO-END, not merely parse?
//!
//! Run: `cargo test --release --test probe_arc251_parametric_target`

use wat::freeze::{startup_beside, startup_from_file};

#[test]
fn contract_01_parametric_form_accepts_matching_value() {
    assert!(
        startup_beside(file!()).is_ok(),
        "a Vector<i64> value must satisfy the form `(wat.type/Vector wat.type/i64)`"
    );
}

#[test]
fn contract_02_parametric_form_rejects_mismatched_element() {
    let r = startup_from_file("tests/resolve/probe_arc251_parametric_target_bad_new.wat");
    assert!(r.is_err(), "Vector<String> must be REJECTED against `(wat.type/Vector wat.type/i64)` — the form must ENFORCE its arg, not just parse");
}

#[test]
fn contract_03_full_faithful_surface_colon_arrow_plus_form() {
    assert!(
        startup_beside(file!()).is_ok(),
        "`:-` + `(wat.type/Vector wat.type/i64)` must type-check together"
    );
}

#[test]
fn contract_04_new_form_equiv_to_legacy_angle_bracket() {
    let new_form = startup_from_file("tests/resolve/probe_arc251_parametric_target_bad_new.wat");
    let old_form = startup_from_file("tests/resolve/probe_arc251_parametric_target_bad_old.wat");
    assert!(new_form.is_err() && old_form.is_err(),
        "both spellings must reject the mismatch identically; new={new_form:?} old={old_form:?}");
}

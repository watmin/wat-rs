//! FM 2-bis probe — arc 251 Stone 251.3: a parametric type written as a FORM
//! `(wat.type/Vector wat.type/i64)` type-checks like the `<>` keyword spelling.
//!
//! Run: `cargo test --release --test probe_arc251_stone3_parametric_form`

use wat::freeze::startup_beside;

#[test]
fn contract_01_parametric_form_type_checks() {
    assert!(
        startup_beside(file!()).is_ok(),
        "(wat.type/Vector wat.type/i64) must type-check as Vector<i64> (unify with the keyword spelling)"
    );
}

#[test]
fn contract_03_parametric_form_in_type_declaration() {
    assert!(
        startup_beside(file!()).is_ok(),
        "a typealias declared with (wat.type/Vector wat.type/i64) must resolve to Vector<i64>"
    );
}

#[test]
fn contract_02_angle_bracket_spelling_still_checks() {
    assert!(
        startup_beside(file!()).is_ok(),
        ":wat::core::Vector<wat::core::i64> keyword spelling must keep type-checking"
    );
}

//! FM 2-bis probe — arc 251 Stone 251.4b: `(ann-form expr type)` ascription.
//!
//! C01: `(:wat::core::ann-form 41 :wat::core::i64)` type-checks AND evaluates to 41.
//! C02: `(:wat::core::ann-form 42 :wat::core::String)` is REJECTED (42 is i64, not String).
//! C03: `(:wat::core::ann-form 41 wat.type/i64)` checks clean — the type slot reuses
//!      `parse_type_node`, inheriting the `wat.type/` surface (251.2a).
//!
//! Run: `cargo test --release --test probe_arc251_stone4b_ann_form`

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_beside, startup_from_file};
use wat::runtime::Value;

#[test]
fn contract_01_ann_form_checks_and_evaluates() {
    // just-eval (rubric): `:user::compute-c01` lives in the co-located fixture.
    match call_beside_value(file!(), ":user::compute-c01").expect("eval: compute-c01 must not fail at runtime") {
        Value::i64(n) => assert_eq!(
            n, 41,
            "(ann-form 41 :i64) must type-check and evaluate to 41 (type-erased identity)"
        ),
        other => panic!("non-i64 from compute-c01: {other:?}"),
    }
}

#[test]
fn contract_02_mismatched_ascription_rejected() {
    let result = startup_from_file("tests/resolve/probe_arc251_stone4b_ann_form.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":wat::core::ann-form"
            && param == "expr"
            && expected == ":wat::core::String"
            && got == ":wat::core::i64"
    );
}

#[test]
fn contract_03_ann_form_accepts_wat_type_surface() {
    assert!(
        startup_beside(file!()).is_ok(),
        "(ann-form 41 wat.type/i64) must check — the type slot accepts the wat.type/ surface"
    );
}

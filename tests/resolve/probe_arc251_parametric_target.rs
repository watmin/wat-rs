//! Strike 1 (examinare verify-probe) — does the faithful-Clojure parametric type TARGET
//! type-check END-TO-END, not merely parse?
//!
//! Run: `cargo test --release --test probe_arc251_parametric_target`

// rune:lint(no-inlined-wat) — arc 296 Stone L. The expected/got strings in this file are
// golden COMPARISON text for a rendered diagnostic field (a TypeMismatch's `expected`/`got`),
// never a wat world or driver. They parse as forms only because the checker's error renderer
// emits real `(Head :- [args])` syntax; nothing here builds or runs a wat program from them,
// and there is no file a single field of a compound match-guard could move to. Same class and
// same reason as tests/services/probe_arc170_w2a_kwargs_check_mint.rs:35.
use wat::check::error::CheckErrorKind;
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
    wat::assert_startup_error!(r, check
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":user::mk"
            && expected == "(:wat::core::Vector :- [:wat::core::i64])"
            && got == "(:wat::core::Vector :- [:wat::core::String])"
    );
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
    // Both spellings must reject with the IDENTICAL structured mismatch (arc 296 Stone L:
    // proving both raise SOME error is not proving they raise the SAME one).
    wat::assert_startup_error!(new_form, check
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":user::mk"
            && expected == "(:wat::core::Vector :- [:wat::core::i64])"
            && got == "(:wat::core::Vector :- [:wat::core::String])"
    );
    wat::assert_startup_error!(old_form, check
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":user::mk"
            && expected == "(:wat::core::Vector :- [:wat::core::i64])"
            && got == "(:wat::core::Vector :- [:wat::core::String])"
    );
}

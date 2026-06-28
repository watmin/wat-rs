//! FM 2-bis probe for Stone 241.8 — `:wat::core::defstruct` HARD CUT.
//!
//! Each contract uses a separate WAT fixture in tests/types/.
//! Positive contracts expect startup_from_file to succeed.
//! Negative contracts expect startup_from_file to fail.

use wat::freeze::startup_from_file;

fn try_startup(path: &str) -> Result<(), String> {
    startup_from_file(path)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Contracts 1–5: defstruct success paths ──────────────────────────────────

#[test]
fn contract_01_defstruct_plain_struct() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c01.wat");
    assert!(
        result.is_ok(),
        "plain defstruct should startup cleanly; got: {:?}",
        result
    );
}

#[test]
fn contract_02_defstruct_with_restricted_to_metadata() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c02.wat");
    assert!(
        result.is_ok(),
        "defstruct with :restricted-to metadata should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_03_defstruct_with_field_metadata() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c03.wat");
    assert!(
        result.is_ok(),
        "defstruct with :field-metadata should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_04_defstruct_with_both_form_and_field_metadata() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c04.wat");
    assert!(
        result.is_ok(),
        "defstruct with both form + field metadata should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_05_defstruct_multi_field_triples() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c05.wat");
    assert!(
        result.is_ok(),
        "multi-field defstruct should startup; got: {:?}",
        result
    );
}

// ─── Contract 6: error paths ────────────────────────────────────────────────

#[test]
fn contract_06_defstruct_empty_metadata_rejected() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c06_bad.wat");
    assert!(
        result.is_err(),
        "empty {{}} metadata-map must error; got Ok"
    );
}

// ─── Contracts 7–8: HARD CUT — legacy verbs REJECTED ─────────────────────────

#[test]
fn contract_07_legacy_struct_hard_cut() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c07_bad.wat");
    assert!(
        result.is_err(),
        "legacy :wat::core::struct must be HARD CUT REJECTED; got Ok"
    );
}

#[test]
fn contract_08_legacy_struct_restricted_hard_cut() {
    let result = try_startup("tests/types/probe_arc241_stone8_defstruct_c08_bad.wat");
    assert!(
        result.is_err(),
        "legacy :wat::core::struct-restricted must be HARD CUT REJECTED; got Ok"
    );
}

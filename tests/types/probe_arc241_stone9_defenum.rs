//! FM 2-bis probe for Stone 241.9 — `:wat::core::defenum` HARD CUT.
//!
//! Each contract uses a separate WAT fixture in tests/types/.
//! Positive contracts expect startup_from_file to succeed.
//! Negative contracts expect startup_from_file to fail.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

fn try_startup(path: &str) -> Result<(), String> {
    startup_from_file(path)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Contracts 1-4: defenum success paths ─────────────────────────────────────

#[test]
fn contract_01_defenum_unit_only() {
    let result = try_startup("tests/types/probe_arc241_stone9_defenum_c01.wat");
    assert!(
        result.is_ok(),
        "plain defenum with unit variants should startup cleanly; got: {:?}",
        result
    );
}

#[test]
fn contract_02_defenum_mixed_unit_and_tagged() {
    let result = try_startup("tests/types/probe_arc241_stone9_defenum_c02.wat");
    assert!(
        result.is_ok(),
        "defenum with mixed unit + tagged should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_03_defenum_interleaved_variants() {
    let result = try_startup("tests/types/probe_arc241_stone9_defenum_c03.wat");
    assert!(
        result.is_ok(),
        "defenum with interleaved unit + tagged variants should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_04_defenum_with_variant_metadata() {
    let result = try_startup("tests/types/probe_arc241_stone9_defenum_c04.wat");
    assert!(
        result.is_ok(),
        "defenum with :variant-metadata should startup; got: {:?}",
        result
    );
}

// ─── Contract 5: rejection (empty {} metadata) ────────────────────────────────

#[test]
fn contract_05_defenum_empty_metadata_rejected() {
    let result = startup_from_file("tests/types/probe_arc241_stone9_defenum_c05.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Type(e) if matches!(e.kind(), TypeErrorKind::MalformedDecl { head, reason }
            if head == ":wat::core::defenum"
            && reason == "empty `{}` metadata-map is illegal (use no metadata-map arg for plain defenum)")
    );
}

// ─── Contracts 6-7: HARD CUT rejection of legacy enum form ────────────────────

#[test]
fn contract_06_legacy_enum_unit_form_rejected() {
    let result = startup_from_file("tests/types/probe_arc241_stone9_defenum_c06.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::enum"
            && reason == "':wat::core::enum' is retired (Stone 241.9)"
    );
}

#[test]
fn contract_07_legacy_enum_tagged_pair_form_rejected() {
    let result = startup_from_file("tests/types/probe_arc241_stone9_defenum_c07.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::enum"
            && reason == "':wat::core::enum' is retired (Stone 241.9)"
    );
}

// ─── Contract 8: defenum REGISTERS the type (semantic gap check) ──────────────

#[test]
fn contract_08_defenum_registers_usable_variants() {
    let result = try_startup("tests/types/probe_arc241_stone9_defenum_c08.wat");
    assert!(
        result.is_ok(),
        "defenum should register :app::Status and its variants; got: {:?}",
        result
    );
}

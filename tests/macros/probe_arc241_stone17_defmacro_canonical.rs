//! FM 2-bis probe for Stone 241.17 — `:wat::core::defmacro` SIGNATURE MIGRATION TO CANONICAL.
//!
//! Stone 241.17 absorbs arc 177's scope: defmacro signature shape migrates from
//! arc 010/150 paren-pair-with-type form to canonical Vector-of-triples form
//! mirroring arc 166 defn shape.
//!
//! HEAD-disconfirmation map:
//! - C01: defmacro with new canonical Vector-triple shape WORKS
//! - C02: old paren-pair shape REJECTED post-stone
//! - C03: defmacro with `& rest` rest-binder in canonical shape WORKS
//!
//! Run: `cargo nextest run --release -E 'binary(macros)' -F probe_arc241_stone17_defmacro_canonical`

use wat::freeze::startup_from_file;

// ─── C01: defmacro with new canonical Vector-triple shape WORKS ────────────────

#[test]
fn contract_01_defmacro_canonical_shape_works() {
    let result = startup_from_file("tests/macros/probe_arc241_stone17_defmacro_canonical_c01.wat");
    assert!(
        result.is_ok(),
        "defmacro with new canonical Vector-triple shape must work post-stone; got: {:?}",
        result
    );
}

// ─── C02: old paren-pair shape REJECTED post-stone ─────────────────────────────

#[test]
fn contract_02_old_paren_pair_shape_rejected() {
    let result = startup_from_file("tests/macros/probe_arc241_stone17_defmacro_canonical_c02_bad.wat");
    assert!(
        result.is_err(),
        "old paren-pair defmacro shape must be HARD-CUT-rejected post-stone (canonical Vector-triple is the only way); got Ok"
    );
}

// ─── C03: defmacro with `& rest` rest-binder works in canonical shape ──────────

#[test]
fn contract_03_defmacro_canonical_rest_binder_works() {
    let result = startup_from_file("tests/macros/probe_arc241_stone17_defmacro_canonical_c03.wat");
    assert!(
        result.is_ok(),
        "defmacro with canonical rest-binder shape must work post-stone; got: {:?}",
        result
    );
}

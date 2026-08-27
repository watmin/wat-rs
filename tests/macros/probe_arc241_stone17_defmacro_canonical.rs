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

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};

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
    let result = startup_from_file("tests/macros/probe_arc241_stone17_defmacro_canonical_c02.wat.bad");
    // Not a `StartupError::Check` — a defmacro signature-shape retirement raises
    // `StartupError::Macro(MacroError { kind: MacroErrorKind::MalformedDefmacro, .. })`
    // directly (verified via `--check`); `assert_startup_error!`'s `check` arm doesn't apply.
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError { kind: MacroErrorKind::MalformedDefmacro { reason }, .. })
            if reason == "old defmacro signature shape (paren-pair-with-type) is retired \
                (Stone 241.17); use canonical Vector-of-triples form: \
                (:wat::core::defmacro :name [param <- :Type ...] -> :Ret body)"
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

//! FM 2-bis probe for Stone 241.14 — `:wat::core::def-restricted` + `:wat::core::defn-restricted` ABSORB INTO METADATA-MAP.
//!
//! Stone 241.14 honors broken Stone 241.6 D10 + line-182 commitment: arc 198's
//! `def-restricted` / `defn-restricted` legacy retires; `:restricted-to` migrates
//! into binding metadata on def/defn.
//!
//! HEAD-disconfirmation map (5/6 DISCONFIRM at HEAD; C01 PRESERVATION):
//! - C01: allowed caller under metadata-map :restricted-to passes (PRESERVATION)
//! - C02: non-allowed caller fails with DefRestrictedCallerNotAllowed ⇒ FAILS at HEAD
//! - C03: defn metadata-map :restricted-to enforces restriction ⇒ FAILS at HEAD
//! - C04: `:wat::core::def-restricted` HARD-CUT-rejected ⇒ FAILS at HEAD
//! - C05: `:wat::core::defn-restricted` HARD-CUT-rejected ⇒ FAILS at HEAD
//! - C06: rejection remedies name `:wat::core::def` / `:wat::core::defn` ⇒ FAILS at HEAD
//!
//! Post-stone: all 6 contracts PASS.

use wat::freeze::startup_from_file;

// ─── C01: def metadata-map :restricted-to — allowed caller passes ──────────────

#[test]
fn contract_01_def_metadata_restricted_allowed_caller_passes() {
    // Allowed caller (matching the :test:: prefix whitelist) must succeed.
    startup_from_file("tests/wat_lang/probe_arc241_stone14_restricted_absorbed.wat")
        .expect("allowed caller (matching :test:: prefix) must pass under metadata-map restriction");
}

// ─── C02: def metadata-map :restricted-to — non-allowed caller fails ───────────

#[test]
fn contract_02_def_metadata_restricted_non_allowed_caller_fails() {
    // Non-allowed caller (:other:: namespace) must fail post-stone.
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone14_restricted_absorbed_non_allowed.wat.bad",
    );
    assert!(
        result.is_err(),
        "non-allowed caller (not matching :test:: prefix) must fail metadata-map restriction post-stone; got Ok"
    );
}

// ─── C03: defn metadata-map :restricted-to — enforcement works ─────────────────

#[test]
fn contract_03_defn_metadata_restricted_enforces() {
    // defn with metadata-map restriction: non-allowed caller must fail.
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone14_restricted_absorbed_non_allowed.wat.bad",
    );
    assert!(
        result.is_err(),
        "defn metadata-map :restricted-to must enforce; non-allowed caller must fail post-stone; got Ok"
    );
}

// ─── C04: :wat::core::def-restricted HARD-CUT-rejected ─────────────────────────

#[test]
fn contract_04_def_restricted_hard_cut_rejected() {
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone14_restricted_absorbed_def_restricted.wat.bad",
    );
    assert!(
        result.is_err(),
        "`:wat::core::def-restricted` must be HARD-CUT-rejected post-stone (def + metadata-map is the only way); got Ok"
    );
}

// ─── C05: :wat::core::defn-restricted HARD-CUT-rejected ────────────────────────

#[test]
fn contract_05_defn_restricted_hard_cut_rejected() {
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone14_restricted_absorbed_defn_restricted.wat.bad",
    );
    assert!(
        result.is_err(),
        "`:wat::core::defn-restricted` must be HARD-CUT-rejected post-stone (defn + metadata-map is the only way); got Ok"
    );
}

// ─── C06: rejection remedies name def / defn respectively ──────────────────────

#[test]
fn contract_06_rejection_remedies_name_replacements() {
    // def-restricted → remedy names :wat::core::def
    let result_def = startup_from_file(
        "tests/wat_lang/probe_arc241_stone14_restricted_absorbed_def_restricted.wat.bad",
    );
    let msg_def = format!("{}", result_def.unwrap_err());
    wat::assert_edn_matches_file!(
        msg_def,
        "probe_arc241_stone14_restricted_absorbed__contract_06_rejection_remedies_name_replacements_def.edn",
        "def-restricted retirement remedy must name :wat::core::def with [replaces a retired form]"
    );

    // defn-restricted → remedy names :wat::core::defn
    let result_defn = startup_from_file(
        "tests/wat_lang/probe_arc241_stone14_restricted_absorbed_defn_restricted.wat.bad",
    );
    let msg_defn = format!("{}", result_defn.unwrap_err());
    wat::assert_edn_matches_file!(
        msg_defn,
        "probe_arc241_stone14_restricted_absorbed__contract_06_rejection_remedies_name_replacements_defn.edn",
        "defn-restricted retirement remedy must name :wat::core::defn with [replaces a retired form]"
    );
}

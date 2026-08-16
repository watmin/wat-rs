//! FM 2-bis probe for Stone 241.15 — ZOMBIE PURGE.
//!
//! Three retired-but-operational forms die together:
//! - A: `:wat::core::try` → canonical `:wat::core::Result/try`
//! - B: `:wat::core::option::expect` → canonical `:wat::core::Option/expect`
//! - C: `:wat::core::result::expect` → canonical `:wat::core::Result/expect`
//!
//! THE DOCTRINE: HARD CUT is total. No "stays for help table" framings.
//!
//! HEAD-disconfirmation map (all 6 contracts FAIL at HEAD):
//! - C01: `:wat::core::try` HARD-CUT-rejected ⇒ FAILS at HEAD
//! - C02: C01 rejection remedy names `:wat::core::Result/try` ⇒ FAILS at HEAD
//! - C03: `:wat::core::option::expect` HARD-CUT-rejected ⇒ FAILS at HEAD
//! - C04: C03 rejection remedy names `:wat::core::Option/expect` ⇒ FAILS at HEAD
//! - C05: `:wat::core::result::expect` HARD-CUT-rejected ⇒ FAILS at HEAD
//! - C06: C05 rejection remedy names `:wat::core::Result/expect` ⇒ FAILS at HEAD
//!
//! Post-stone: all 6 contracts PASS.

use wat::freeze::startup_from_file;

// ─── C01: :wat::core::try HARD-CUT-rejected with Stone 241.15 signature ────────

#[test]
fn contract_01_try_hard_cut_rejected() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone15_zombie_purge_try.wat.bad");
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone15_zombie_purge__contract_01_try_hard_cut_rejected.edn",
        ":wat::core::try must be HARD-CUT-rejected via Stone 241.15 arm"
    );
}

// ─── C02: C01 rejection remedy names :wat::core::Result/try ────────────────────

#[test]
fn contract_02_try_rejection_remedy_names_result_try() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone15_zombie_purge_try.wat.bad");
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone15_zombie_purge__contract_02_try_rejection_remedy_names_result_try.edn",
        "try retirement remedy must name :wat::core::Result/try with [replaces a retired form]"
    );
}

// ─── C03: :wat::core::option::expect HARD-CUT via Stone 241.15 signature ───────

#[test]
fn contract_03_option_expect_lowercase_hard_cut_rejected() {
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone15_zombie_purge_option_expect.wat.bad",
    );
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone15_zombie_purge__contract_03_option_expect_lowercase_hard_cut_rejected.edn",
        ":wat::core::option::expect must be HARD-CUT-rejected via Stone 241.15 arm"
    );
}

// ─── C04: C03 rejection remedy names :wat::core::Option/expect ─────────────────

#[test]
fn contract_04_option_expect_lowercase_rejection_remedy_names_pascal() {
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone15_zombie_purge_option_expect.wat.bad",
    );
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone15_zombie_purge__contract_04_option_expect_lowercase_rejection_remedy_names_pascal.edn",
        "option::expect retirement remedy must name :wat::core::Option/expect with [replaces a retired form]"
    );
}

// ─── C05: :wat::core::result::expect HARD-CUT via Stone 241.15 signature ───────

#[test]
fn contract_05_result_expect_lowercase_hard_cut_rejected() {
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone15_zombie_purge_result_expect.wat.bad",
    );
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone15_zombie_purge__contract_05_result_expect_lowercase_hard_cut_rejected.edn",
        ":wat::core::result::expect must be HARD-CUT-rejected via Stone 241.15 arm"
    );
}

// ─── C06: C05 rejection remedy names :wat::core::Result/expect ─────────────────

#[test]
fn contract_06_result_expect_lowercase_rejection_remedy_names_pascal() {
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone15_zombie_purge_result_expect.wat.bad",
    );
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone15_zombie_purge__contract_06_result_expect_lowercase_rejection_remedy_names_pascal.edn",
        "result::expect retirement remedy must name :wat::core::Result/expect with [replaces a retired form]"
    );
}

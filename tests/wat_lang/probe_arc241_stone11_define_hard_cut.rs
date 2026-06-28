//! FM 2-bis probe for Stone 241.11 — `:wat::core::define` ⇒ `:wat::core::defn` HARD CUT.
//!
//! Stone 241.10 minted `src/remedy/` + ranked-remedy schema. The retirement table
//! grows with each HARD CUT. Stone 241.11 appends ONE LINE to RETIREMENT_TABLE
//! (`":wat::core::define" → ":wat::core::defn"`) and the substrate teaches
//! automatically via the existing remedy infrastructure consumed at the
//! check-time HARD-CUT-rejection arm.
//!
//! HEAD-disconfirmation map:
//! - C01: defn success path (passes at HEAD — defn already works post-241.5/.6/.7)
//! - C02: legacy `:wat::core::define` HARD CUT rejected at HEAD ⇒ FAILS at HEAD
//! - C03: error names `:wat::core::defn` as the remedy ⇒ FAILS at HEAD
//! - C04: error carries `[replaces a retired form]` annotation ⇒ FAILS at HEAD
//! - C05: retirement table contains 4 entries ⇒ FAILS at HEAD
//!
//! Post-stone: all 5 contracts PASS.

use wat::freeze::startup_from_file;

// ─── C01: defn success path (baseline) ─────────────────────────────────────────

#[test]
fn contract_01_defn_success_baseline() {
    // defn is the SURVIVING form; baseline verification it still works post-stone.
    startup_from_file("tests/wat_lang/probe_arc241_stone11_define_hard_cut.wat")
        .expect("defn baseline must continue to work post-stone");
}

// ─── C02: legacy define HARD CUT rejected ──────────────────────────────────────

#[test]
fn contract_02_legacy_define_hard_cut_rejected() {
    // Legacy `:wat::core::define` is RETIRED post-stone. HARD CUT.
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone11_define_hard_cut_bad.wat");
    assert!(
        result.is_err(),
        "legacy :wat::core::define must be HARD-CUT-rejected post-stone; got Ok"
    );
}

// ─── C03: error contains "did you mean: :wat::core::defn" ──────────────────────

#[test]
fn contract_03_retirement_remedy_names_defn() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone11_define_hard_cut_bad.wat");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("did you mean") && msg.contains(":wat::core::defn"),
        "retirement remedy must name :wat::core::defn; got:\n{}",
        msg
    );
}

// ─── C04: error carries [replaces a retired form] annotation ────────────────────

#[test]
fn contract_04_retirement_kind_annotation_present() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone11_define_hard_cut_bad.wat");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("[replaces a retired form]"),
        "retirement remedy must carry exact '[replaces a retired form]' annotation; got:\n{}",
        msg
    );
}

// ─── C05: retirement table has 4 entries (structural proof) ────────────────────

#[test]
fn contract_05_retirement_table_includes_define_entry() {
    // Indirect: :wat::core::defn appears in remedy text + [replaces a retired form]
    // proves the retirement table contains the entry.
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone11_define_hard_cut_bad.wat");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains(":wat::core::defn") && msg.contains("[replaces a retired form]"),
        "retirement table must include :wat::core::define → :wat::core::defn entry; got:\n{}",
        msg
    );
}

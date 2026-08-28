//! FM 2-bis probe for Stone 241.16 — `:wat::core::define` EVAL-TIME RESIDUE COMPLETION.
//!
//! Stone 241.11 HARD-CUT :wat::core::define at startup-check; eval-time scaffolding
//! survived deliberately (defense-in-depth). Stone 241.16 completes the cut:
//! parse_define_form DELETED; is_define_form DELETED; special_forms.rs entry DELETED.
//!
//! FINAL CONTRACT SET (4 contracts):
//! - C01: define rejection error mentions "Stone 241.16" marker (HARD CUT completion)
//! - C02: retirement remedy STILL names :wat::core::defn (preservation from Stone 241.11)
//! - C03: define in let body still rejected (consistency check)
//! - C04: define in fn-body do-prefix still rejected (consistency check)
//!
//! Post-stone: all 4 contracts PASS.

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

// ─── C01: define rejection error mentions "Stone 241.16" marker ────────────────

#[test]
fn contract_01_define_rejection_carries_stone_241_16_marker() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone16_define_eval_residue.wat.bad");
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone16_define_eval_residue__contract_01_define_rejection_carries_stone_241_16_marker.edn",
        ":wat::core::define rejection must carry Stone 241.16 marker (eval-time residue completion)"
    );
}

// ─── C02: retirement remedy STILL names :wat::core::defn (preservation) ────────

#[test]
fn contract_02_retirement_remedy_preserves_defn_replacement() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone16_define_eval_residue.wat.bad");
    let msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc241_stone16_define_eval_residue__contract_02_retirement_remedy_preserves_defn_replacement.edn",
        "define retirement remedy must continue to name :wat::core::defn with [replaces a retired form]"
    );
}

// ─── C03: define-headed AST in let body still rejected (consistency check) ─────

#[test]
fn contract_03_define_in_let_body_still_rejected() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone16_define_in_let.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::define"
            && reason == "':wat::core::define' is retired (Stone 241.11; eval-time residue completed Stone 241.16)"
    );
}

// ─── C04: define-headed AST in fn-body do-prefix still rejected ────────────────

#[test]
fn contract_04_define_in_fn_body_still_rejected() {
    let result =
        startup_from_file("tests/wat_lang/probe_arc241_stone16_define_in_fn.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::define"
            && reason == "':wat::core::define' is retired (Stone 241.11; eval-time residue completed Stone 241.16)"
    );
}

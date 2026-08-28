//! FM 2-bis probe for Stone 242.2 — Doctrine 1 enforcement at type-check.
//!
//! Doctrine 1: bare lexeme = value; keyword lexeme (`:wat::core::*`) = type.
//!
//! Stone 242.1 inscribed the doctrine + bare nil verified-operational + Char
//! HARD CUT. Stone 242.2 makes the doctrine SELF-ENFORCING — type keywords in
//! VALUE position are REJECTED at type-check with structured remedy per
//! Stone 241.10's apparatus.
//!
//! HEAD-disconfirmation map:
//! - C01: `:wat::core::nil` in body (value position) → REJECTED ⇒ FAILS at HEAD (currently accepted)
//! - C02: bare `nil` in body → PASSES (already operational per Stone 242.1)
//! - C03: `:wat::core::i64` in body → REJECTED with remedy ⇒ FAILS at HEAD
//! - C04: `42` (bare i64 value) in body → PASSES (already operational)
//! - C05: `:wat::core::nil` in let-binding value position → REJECTED ⇒ FAILS at HEAD
//! - C06: bare `nil` in let-binding value position → PASSES
//!
//! Post-stone: all 6 contracts PASS.
//!
//! WAT fixtures: tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c{01,02,03,04,05,06}[_bad].wat
//!
//! Run: `cargo nextest run --release -E 'binary(diagnostics)' -F probe_arc242_stone2_value_position_doctrine`

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

// ─── C01: :wat::core::nil in body REJECTED ─────────────────────────────────────

#[test]
fn contract_01_keyword_nil_in_body_rejected() {
    // (:wat::core::defn :f [] -> :wat::core::nil :wat::core::nil)
    // Per user direction: ILLEGAL — keyword form in value position.
    // (Note: the body's :wat::core::nil is the keyword-in-value-position
    // doctrine violation under test; do NOT migrate to bare nil — that
    // would defeat the test.)
    // Fixture: probe_arc242_stone2_value_position_doctrine_c01.wat.bad
    let result = startup_from_file("tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c01.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::nil"
            && reason == "Doctrine 1 (arc 242): ':wat::core::nil' is a TYPE keyword, not a value; use bare `nil` in value position"
    );
}

// ─── C02: bare nil in body PASSES ──────────────────────────────────────────────

#[test]
fn contract_02_bare_nil_in_body_passes() {
    // (:wat::core::defn :f [] -> :wat::core::nil nil) — the legal form.
    // Fixture: probe_arc242_stone2_value_position_doctrine_c02.wat
    let result = startup_from_file("tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c02.wat");
    assert!(
        result.is_ok(),
        "bare nil in value position must PASS; got: {:?}",
        result.err()
    );
}

// ─── C03: :wat::core::i64 in body REJECTED with remedy ────────────────────────

#[test]
fn contract_03_keyword_type_in_body_rejected_with_remedy() {
    // (:wat::core::defn :f [] -> :wat::core::i64 :wat::core::i64) — ILLEGAL.
    // Fixture: probe_arc242_stone2_value_position_doctrine_c03.wat.bad
    let msg = match startup_from_file("tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c03.wat.bad") {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    };
    // 296 recapture: staleness — EDN face (Stone B), same message/span/head/reason as the
    // pre-stone-B prose face; :remedies [] matches (the old expectation carried no remedy
    // phrasing either — this test's own doc comment about "structured remedy" describes the
    // 241.10 apparatus in general, not a remedy this specific MalformedForm ever emitted).
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc242_stone2_value_position_doctrine__contract_03_keyword_type_in_body_rejected_with_remedy.edn",
        ":wat::core::i64 in value position must be REJECTED with Doctrine 1 structured guidance"
    );
}

// ─── C04: bare value (42) in body PASSES ──────────────────────────────────────

#[test]
fn contract_04_bare_value_in_body_passes() {
    // (:wat::core::defn :f [] -> :wat::core::i64 42) — the legal form.
    // Fixture: probe_arc242_stone2_value_position_doctrine_c04.wat
    let result = startup_from_file("tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c04.wat");
    assert!(
        result.is_ok(),
        "bare value (42) in value position must PASS; got: {:?}",
        result.err()
    );
}

// ─── C05: :wat::core::nil in let-binding value REJECTED ───────────────────────

#[test]
fn contract_05_keyword_nil_in_let_binding_rejected() {
    // (:wat::core::let [x :wat::core::nil] x) — ILLEGAL (let-binding value).
    // Fixture: probe_arc242_stone2_value_position_doctrine_c05.wat.bad
    let result = startup_from_file("tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c05.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::nil"
            && reason == "Doctrine 1 (arc 242): ':wat::core::nil' is a TYPE keyword, not a value; use bare `nil` in value position"
    );
}

// ─── C06: bare nil in let-binding value PASSES ────────────────────────────────

#[test]
fn contract_06_bare_nil_in_let_binding_passes() {
    // (:wat::core::let [x nil] x) — the legal form.
    // Fixture: probe_arc242_stone2_value_position_doctrine_c06.wat
    let result = startup_from_file("tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c06.wat");
    assert!(
        result.is_ok(),
        "bare nil in let-binding value position must PASS; got: {:?}",
        result.err()
    );
}

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
//! Run: `cargo test --release --test probe_arc242_stone2_value_position_doctrine`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn try_startup(src: &str) -> Result<(), String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn try_startup_display(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── C01: :wat::core::nil in body REJECTED ─────────────────────────────────────

#[test]
fn contract_01_keyword_nil_in_body_rejected() {
    // (:wat::core::defn :f [] -> :wat::core::nil :wat::core::nil)
    // Per user direction: ILLEGAL — keyword form in value position.
    let src = r#"
        (:wat::core::defn :test::f [] -> :wat::core::nil :wat::core::nil)
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "keyword form :wat::core::nil in value position must be REJECTED (Doctrine 1); got Ok"
    );
}

// ─── C02: bare nil in body PASSES ──────────────────────────────────────────────

#[test]
fn contract_02_bare_nil_in_body_passes() {
    // (:wat::core::defn :f [] -> :wat::core::nil nil) — the legal form.
    let src = r#"
        (:wat::core::defn :test::f [] -> :wat::core::nil nil)
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "bare nil in value position must PASS; got: {:?}",
        result
    );
}

// ─── C03: :wat::core::i64 in body REJECTED with remedy ────────────────────────

#[test]
fn contract_03_keyword_type_in_body_rejected_with_remedy() {
    // (:wat::core::defn :f [] -> :wat::core::i64 :wat::core::i64) — ILLEGAL.
    let src = r#"
        (:wat::core::defn :test::f [] -> :wat::core::i64 :wat::core::i64)
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let msg = try_startup_display(src);
    // The error must contain a structured remedy per Stone 241.10's apparatus.
    // Specific phrasing: doctrine guidance pointing at value-position correctness.
    assert!(
        msg.contains("Doctrine 1") || msg.contains("value position") || msg.contains("did you mean"),
        ":wat::core::i64 in value position must be REJECTED with structured doctrine guidance; got:\n{}",
        msg
    );
}

// ─── C04: bare value (42) in body PASSES ──────────────────────────────────────

#[test]
fn contract_04_bare_value_in_body_passes() {
    // (:wat::core::defn :f [] -> :wat::core::i64 42) — the legal form.
    let src = r#"
        (:wat::core::defn :test::f [] -> :wat::core::i64 42)
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "bare value (42) in value position must PASS; got: {:?}",
        result
    );
}

// ─── C05: :wat::core::nil in let-binding value REJECTED ───────────────────────

#[test]
fn contract_05_keyword_nil_in_let_binding_rejected() {
    // (:wat::core::let [x :wat::core::nil] x) — ILLEGAL (let-binding value).
    let src = r#"
        (:wat::core::defn :test::f [] -> :wat::core::nil
          (:wat::core::let [x :wat::core::nil] x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "keyword :wat::core::nil in let-binding value position must be REJECTED; got Ok"
    );
}

// ─── C06: bare nil in let-binding value PASSES ────────────────────────────────

#[test]
fn contract_06_bare_nil_in_let_binding_passes() {
    // (:wat::core::let [x nil] x) — the legal form.
    let src = r#"
        (:wat::core::defn :test::f [] -> :wat::core::nil
          (:wat::core::let [x nil] x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "bare nil in let-binding value position must PASS; got: {:?}",
        result
    );
}

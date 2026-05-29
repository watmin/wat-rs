//! FM 2-bis probe for Stone 241.12 — `:wat::core::defalias` mint + alias-cascade completion.
//!
//! Stone 241.12 mints the missing def*-prefix-family surface form for binding aliases.
//! The existing `:wat::runtime::define-alias` runtime mechanism (26 callers) stays;
//! `:wat::core::defalias` is the user-facing surface form that compiles to it.
//!
//! HEAD-disconfirmation map:
//! - C01: defalias success path (legacy form resolves both names to same binding)
//!   ⇒ FAILS at HEAD (`:wat::core::defalias` doesn't exist; startup errors)
//! - C02: defalias additive (original name still works post-alias)
//!   ⇒ FAILS at HEAD (no defalias to test)
//! - C03: defalias function-typed alias works (alias to a defn'd function)
//!   ⇒ FAILS at HEAD
//!
//! Post-stone: all 3 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone12_defalias`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn try_startup(src: &str) -> Result<(), String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    );
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── C01: defalias success path ────────────────────────────────────────────────

#[test]
fn contract_01_defalias_startup_clean() {
    // defalias binds new name to existing function. Startup must succeed cleanly.
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
        (:wat::core::defalias :app::salutation :app::greet)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defalias should startup cleanly post-stone; got: {:?}",
        result
    );
}

// ─── C02: defalias additive (original still works) ────────────────────────────

#[test]
fn contract_02_defalias_additive_original_still_resolves() {
    // After defalias, BOTH names resolve. The alias is additive; not destructive.
    // Test by calling the ORIGINAL name from main's body.
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
        (:wat::core::defalias :app::salutation :app::greet)
        (:wat::core::defn :test::pick [] -> :wat::core::String (:app::greet))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "original name must still resolve post-defalias (additive); got: {:?}",
        result
    );
}

// ─── C03: defalias resolves to same binding ────────────────────────────────────

#[test]
fn contract_03_defalias_new_name_resolves_to_original() {
    // After defalias, the NEW name resolves to the same binding as the original.
    // Test by calling the ALIAS name from a function body that expects the
    // original's return type.
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
        (:wat::core::defalias :app::salutation :app::greet)
        (:wat::core::defn :test::pick [] -> :wat::core::String (:app::salutation))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "alias name must resolve to same binding as original; got: {:?}",
        result
    );
}

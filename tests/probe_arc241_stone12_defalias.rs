//! FM 2-bis probe for Stone 241.12 — `:wat::core::defalias` mint + `:wat::runtime::define-alias` HARD CUT.
//!
//! Stone 241.12 mints the missing def*-prefix-family surface form for binding aliases AND
//! retires `:wat::runtime::define-alias` per user direction 2026-05-29 late:
//!   *"at the end of this work :wat::runtime::define-alias is dead - :wat::core::defalias
//!    is the only way to do name aliasing."*
//!
//! THE DOCTRINE (per `feedback_hard_cut_admits_no_bypasses`): HARD CUT is total. No
//! privileged paths. No "user-facing surface compiles to runtime mechanism" two-layer model.
//! `:wat::runtime::define-alias` DIES. `:wat::core::defalias` is the SOLE alias mechanism.
//!
//! HEAD-disconfirmation map (all 5 contracts FAIL at HEAD):
//! - C01: `:wat::core::defalias` mint works (startup clean) ⇒ FAILS at HEAD (form doesn't exist)
//! - C02: defalias additive — original name still resolves ⇒ FAILS at HEAD (defalias unminted)
//! - C03: defalias new name resolves to same binding as original ⇒ FAILS at HEAD
//! - C04: `:wat::runtime::define-alias` HARD-CUT-rejected at startup ⇒ FAILS at HEAD
//!        (form is still live; 13 active callers across wat/ and tests/)
//! - C05: rejection error carries structured retirement remedy naming `:wat::core::defalias`
//!        ⇒ FAILS at HEAD (no rejection fires; no error to inspect)
//!
//! Post-stone: all 5 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone12_defalias`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn try_startup(src: &str) -> Result<(), String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn try_startup_display(src: &str) -> String {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    match startup_from_source(&full, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── C01: defalias produces a callable alias (resolvable from a fn body) ──────

#[test]
fn contract_01_defalias_alias_name_resolves() {
    // The alias name must be CALLABLE post-defalias. Pure startup-Ok is insufficient
    // (an ignored unknown form would also be Ok); the new name must actually resolve.
    // At HEAD: alias name unresolved because defalias is unknown / no-op.
    // Post-stone: alias is a registered binding; call site resolves.
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
        (:wat::core::defalias :app::salutation :app::greet)
        (:wat::core::defn :test::call-alias [] -> :wat::core::String (:app::salutation))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "alias name must resolve to a callable binding post-defalias; got: {:?}",
        result
    );
}

// ─── C02: defalias additive — BOTH names resolve in the same program ───────────

#[test]
fn contract_02_defalias_additive_both_names_callable() {
    // Additive means BOTH the alias and the original resolve in the same program.
    // At HEAD: original works trivially; alias unresolved (would fail if called) —
    // but a caller of only the original passes at HEAD, masking the gap.
    // Sharpened: this contract calls BOTH from separate fns so the alias resolution
    // is exercised; failure of either drops the test.
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
        (:wat::core::defalias :app::salutation :app::greet)
        (:wat::core::defn :test::call-original [] -> :wat::core::String (:app::greet))
        (:wat::core::defn :test::call-alias    [] -> :wat::core::String (:app::salutation))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "BOTH alias and original must resolve post-defalias (additive); got: {:?}",
        result
    );
}

// ─── C03: defalias works for built-in/intrinsic bindings (wat/core.wat pattern) ─

#[test]
fn contract_03_defalias_can_alias_a_builtin() {
    // Defalias must work for substrate built-ins, not just user-defined functions.
    // This is the load-bearing pattern for the wat/core.wat callers that Stone
    // 241.12 migrates:
    //   (:wat::core::defalias :wat::core::dissoc :wat::core::HashMap/dissoc)
    //   (:wat::core::defalias :wat::core::concat :wat::core::Vector/concat)
    // — all four wat/core.wat aliases route a short name to a built-in long name.
    //
    // At HEAD: defalias unknown; alias unresolved.
    // Post-stone: defalias resolves the alias to the built-in's binding; the new
    // short name is callable wherever the long built-in name is.
    let src = r#"
        (:wat::core::defalias :user::my-length :wat::core::length)
        (:wat::core::defn :test::use [] -> :wat::core::i64 (:user::my-length [1 2 3]))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defalias must work for built-in bindings (wat/core.wat pattern); got: {:?}",
        result
    );
}

// ─── C04: `:wat::runtime::define-alias` HARD-CUT-rejected at startup ───────────

#[test]
fn contract_04_runtime_define_alias_hard_cut_rejected() {
    // Per user direction 2026-05-29 late: `:wat::runtime::define-alias` DIES.
    // The form must be HARD-CUT-rejected at startup with structured remedy.
    // At HEAD: define-alias is the live runtime macro; startup succeeds.
    // Post-stone: startup REJECTS with retirement error.
    //
    // Per `feedback_hard_cut_admits_no_bypasses`: NO privileged paths. The
    // substrate has ONE alias form (:wat::core::defalias), not two layers.
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
        (:wat::runtime::define-alias :app::salutation :app::greet)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "`:wat::runtime::define-alias` must be HARD-CUT-rejected post-stone (Doctrine: no privileged paths); got Ok"
    );
}

// ─── C05: rejection carries structured retirement remedy naming defalias ───────

#[test]
fn contract_05_rejection_remedy_names_defalias() {
    // The HARD-CUT rejection must include structured remedy per Stone 241.10's
    // apparatus, naming `:wat::core::defalias` as the replacement
    // (via RETIREMENT_TABLE entry consumed by remedies_for).
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
        (:wat::runtime::define-alias :app::salutation :app::greet)
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains(":wat::core::defalias"),
        "retirement remedy must name :wat::core::defalias; got:\n{}",
        msg
    );
    assert!(
        msg.contains("[retirement replacement]"),
        "retirement remedy must carry '[retirement replacement]' annotation per Stone 241.10's apparatus; got:\n{}",
        msg
    );
}

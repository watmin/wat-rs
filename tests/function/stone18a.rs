//! FM 2-bis probe for Stone 241.18a — `src/function/` namespaced home mint.
//!
//! First stepping stone of the bar-raise chain (Stone 241.18a-g). Moves fn-form
//! parsers + eval + infer from runtime.rs + check.rs into a dedicated namespaced
//! home `src/function/` per `feedback_namespaced_home_vigilia_gate` REMARKABLE bar.
//!
//! Intueri-affirmed name: `src/function/` (NOT `src/fn/`). Rust-keyword constraint
//! resolved per intueri's verdict — `r#fn` fails Honest + UX; `function` carries
//! the domain concept without leaking implementation friction.
//!
//! DISCONFIRMATION SHAPE — this stone is a MIGRATION (move code without changing
//! behavior). Behavioral disconfirmation isn't the right test — the probe contracts
//! are PRESERVATION (pass at HEAD via existing parsers in runtime.rs / check.rs;
//! pass post-stone via crate::function::* path).
//!
//! The TRUE FM 2-bis disconfirmation is STRUCTURAL: src/function/ doesn't exist at
//! HEAD; exists post-stone. Verified by orchestrator's structural verification rows
//! in EXPECTATIONS-STONE-241.18a.md (grep for src/function/mod.rs; verify Cargo.toml
//! [[test]] entry for "function"; verify caller imports updated to crate::function::*).
//!
//! These probe contracts BEHAVIORALLY verify that the migration didn't break fn-form
//! parsing / eval / infer for any reasonable program. Wat source lives in
//! `tests/function/stone18a.wat` (positive) and `tests/function/stone18a_eNN.wat`
//! (negative, one per error contract).
//!
//! Post-stone: both contracts continue to PASS (preservation).
//!
//! Run: `cargo test --release --test function`

use wat::freeze::{startup_from_file, StartupError};

/// Load a fixture by path and return Ok(()) or the raw `StartupError` — the
/// typed discriminant, not a flattened string (arc 296 Stone M). Shared by the
/// positive contracts in this file (which only ever check `.is_ok()`) and by
/// `stone18a_errors.rs`'s negative contracts (which need to match the inner
/// `CheckErrorKind`, so this signature is what makes them reachable by
/// `assert_startup_error!` directly, no parallel typed helper needed).
pub(super) fn try_startup(path: &str) -> Result<(), StartupError> {
    startup_from_file(path).map(|_| ())
}

// ─── C01: fn program (single typed param) preserved post-migration ─────────────

#[test]
fn contract_01_fn_single_param_preserved() {
    // Behavioral preservation contract — fn-form parser must work the same
    // before and after migration to src/function/.
    let result = try_startup("tests/function/stone18a.wat");
    assert!(
        result.is_ok(),
        "fn-form with single typed param must work post-migration; got: {:?}",
        result
    );
}

// ─── C02: fn program with multi-param triple-arrow preserved ──────────────────

#[test]
fn contract_02_fn_with_multi_param_triple_arrow_preserved() {
    // Behavioral preservation — fn with 2+ typed params via triple-arrow form.
    let result = try_startup("tests/function/stone18a.wat");
    assert!(
        result.is_ok(),
        "fn-form with multi-param triple-arrow must work post-migration; got: {:?}",
        result
    );
}

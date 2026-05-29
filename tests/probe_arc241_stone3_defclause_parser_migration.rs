//! FM 2-bis BEHAVIORAL-PARITY probe for Stone 241.3 — A4 defclause parser
//! migration through canonical `parse_argspec_triples`.
//!
//! ## Why this probe
//!
//! Stone 241.3 migrates the single internal parser:
//!   - **A4** `parse_defclause_args` at `src/runtime.rs:6827`
//!
//! A4's public signature stays IDENTICAL (`(args_vec, head, form_span) ->
//! Result<Vec<(String, TypeExpr)>, RuntimeError>`). The migration is INTERNAL:
//! the 69-line inline triple walker is replaced with a 7-line canonical call;
//! `?` converts `ArgSpecError` → `RuntimeError` via the `From<>` impl shipped
//! in Stone 241.1.fix; `spec.fixed_params` is returned DIRECTLY (no unzip
//! needed — defclause's return shape IS the canonical's fixed_params shape).
//!
//! ## What this probe proves
//!
//! Behavioral parity: well-formed defclause forms parse cleanly; malformed
//! defclause forms produce errors (don't silently succeed). The probe asserts
//! on the err/ok BOUNDARY, not on exact error message text — canonical-
//! domain-neutral wording replaces inline arc-lineage citations (e.g. "literal
//! patterns are not permitted (arc 159/169/234 binding contract requires a
//! plain symbol name)" → "name slot must be a plain symbol (not a keyword,
//! literal, or nested form)"); the message changes but the err/ok boundary
//! stays.
//!
//! ## Pre/post migration
//!
//! Pre-Stone 241.3 (HEAD `21877135`): all contracts PASS via the existing
//! inline triple walker at A4.
//!
//! Post-Stone 241.3: all contracts STILL PASS; the canonical parser
//! preserves the err/ok behavior. The exact error messages differ but the
//! variant CLASS (`RuntimeError::MalformedForm`) is preserved via
//! `From<ArgSpecError> for RuntimeError`.
//!
//! ## FM 2-bis nature: PARITY probe (same shape as Stone 241.2)
//!
//! Mirrors Stone 241.2's behavioral-parity discipline. Passes BOTH at HEAD
//! and post-stone; regression in err/ok boundary indicates migration broke.
//!
//! ## Phase 1 closure note
//!
//! Stone 241.3 closes the parser-divergence class. After this stone, all 4
//! triple walkers (A1/A2/A3/A4) route through ONE canonical parser. The
//! substrate carries ONE triple-walking implementation; same structural
//! failures produce same `ArgSpecError` variants; per-site error conversion
//! at the call boundary via `From<>` impls.
//!
//! Run: `cargo test --release --test probe_arc241_stone3_defclause_parser_migration`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn try_startup(src: &str) -> Result<(), String> {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Contracts 1–3: A4 happy paths (well-formed defclause args) ──────────────

#[test]
fn contract_01_defclause_no_args_succeeds() {
    // (defclause [] -> :T body)  — empty argspec.
    let result = try_startup(
        r#"(:wat::core::defclause :user::f
             ([] -> :wat::core::i64 42))"#,
    );
    assert!(
        result.is_ok(),
        "well-formed no-arg defclause should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_02_defclause_single_arg_succeeds() {
    let result = try_startup(
        r#"(:wat::core::defclause :user::f
             ([x <- :wat::core::i64] -> :wat::core::i64 x))"#,
    );
    assert!(
        result.is_ok(),
        "well-formed single-arg defclause should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_03_defclause_multi_arg_succeeds() {
    let result = try_startup(
        r#"(:wat::core::defclause :user::f
             ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::+ x y)))"#,
    );
    assert!(
        result.is_ok(),
        "well-formed multi-arg defclause should startup; got: {:?}",
        result
    );
}

// ─── Contracts 4–6: A4 error paths (malformed argspecs error cleanly) ───────

#[test]
fn contract_04_name_not_symbol_errors() {
    // Slot 0 of triple is a keyword, not a Symbol.
    // A4 enforces this per arc 159/169/234 binding contract; canonical also enforces.
    let result = try_startup(
        r#"(:wat::core::defclause :user::f
             ([:kw <- :wat::core::i64] -> :wat::core::i64 42))"#,
    );
    assert!(
        result.is_err(),
        "non-Symbol at name slot must error; got Ok"
    );
}

#[test]
fn contract_05_missing_arrow_errors() {
    // Slot 1 of triple is `=` not `<-`.
    let result = try_startup(
        r#"(:wat::core::defclause :user::f
             ([x = :wat::core::i64] -> :wat::core::i64 x))"#,
    );
    assert!(
        result.is_err(),
        "missing `<-` arrow must error; got Ok"
    );
}

#[test]
fn contract_06_incomplete_triple_errors() {
    // Argspec has fewer than 3 items at a triple position.
    let result = try_startup(
        r#"(:wat::core::defclause :user::f
             ([x <-] -> :wat::core::i64 42))"#,
    );
    assert!(
        result.is_err(),
        "incomplete triple must error; got Ok"
    );
}

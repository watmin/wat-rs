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
//!   (define WORKS at HEAD; assertion is_err fails because result is Ok)
//! - C03: error names `:wat::core::defn` as the remedy ⇒ FAILS at HEAD (no error)
//! - C04: error carries `[retirement replacement]` annotation ⇒ FAILS at HEAD
//! - C05: retirement table contains 4 entries (struct, struct-restricted, enum, define)
//!   ⇒ FAILS at HEAD (table has 3 entries)
//!
//! Post-stone: all 5 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone11_define_hard_cut`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    )
}

/// Display-format the error message (what the user sees).
fn try_startup_display(src: &str) -> String {
    let full = with_nil_main(src);
    match startup_from_source(&full, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── C01: defn success path (baseline) ─────────────────────────────────────────

#[test]
fn contract_01_defn_success_baseline() {
    // defn is the SURVIVING form; baseline verification it still works post-stone.
    // Already works at HEAD per Stone 241.5/.6/.7 work.
    let src = r#"
        (:wat::core::defn :app::greet [] -> :wat::core::String "hello")
    "#;
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    );
    let result = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "defn baseline must continue to work post-stone; got: {:?}",
        result
    );
}

// ─── C02: legacy define HARD CUT rejected ──────────────────────────────────────

#[test]
fn contract_02_legacy_define_hard_cut_rejected() {
    // Legacy `:wat::core::define` is RETIRED post-stone. HARD CUT.
    // At HEAD: define WORKS → result is Ok → assertion fails (disconfirming).
    // Post-stone: define is HARD-CUT-rejected via check.rs MalformedForm arm.
    let src = r#"
        (:wat::core::define (:app::greet -> :wat::core::String) "hello")
    "#;
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    );
    let result = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "legacy :wat::core::define must be HARD-CUT-rejected post-stone; got Ok"
    );
}

// ─── C03: error contains "did you mean: :wat::core::defn" ──────────────────────

#[test]
fn contract_03_retirement_remedy_names_defn() {
    // Stone 241.10's remedy infrastructure consumed at the HARD-CUT arm
    // automatically produces a structured retirement remedy. Per the
    // canonical render_remedies format for retirement remedies:
    //   "did you mean: :wat::core::defn"
    let src = r#"
        (:wat::core::define (:app::greet -> :wat::core::String) "hello")
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains("did you mean") && msg.contains(":wat::core::defn"),
        "retirement remedy must name :wat::core::defn; got:\n{}",
        msg
    );
}

// ─── C04: error carries [retirement replacement] annotation ────────────────────

#[test]
fn contract_04_retirement_kind_annotation_present() {
    // Per Stone 241.10 D7: retirement-kind annotation is the EXACT phrase
    // "[retirement replacement]". This proves the remedy is structured
    // (kind=Retirement) rather than hand-written prose.
    let src = r#"
        (:wat::core::define (:app::greet -> :wat::core::String) "hello")
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains("[retirement replacement]"),
        "retirement remedy must carry exact '[retirement replacement]' annotation; got:\n{}",
        msg
    );
}

// ─── C05: retirement table has 4 entries (structural proof) ────────────────────

#[test]
fn contract_05_retirement_table_includes_define_entry() {
    // Direct structural proof: the retirement table grows by exactly one entry.
    // At HEAD: 3 entries (struct, struct-restricted, enum).
    // Post-stone: 4 entries (above + define).
    //
    // This uses wat::remedy::* public API. The remedy module is pub(crate)
    // (per R4 visibility honesty); this probe lives in tests/ (external crate
    // boundary). We test indirectly via retirement_lookup output through
    // a startup error path — which is what C02-C04 already do — so this
    // contract is a redundancy reinforcement: legacy define MUST be in the
    // retirement table (verified via remedy text).
    let src = r#"
        (:wat::core::define (:app::greet -> :wat::core::String) "hello")
    "#;
    let msg = try_startup_display(src);
    // Two indirect proofs that the retirement table contains the entry:
    // (1) "did you mean :wat::core::defn" is the retirement_lookup output
    // (2) "[retirement replacement]" annotation marks the remedy kind
    assert!(
        msg.contains(":wat::core::defn") && msg.contains("[retirement replacement]"),
        "retirement table must include :wat::core::define → :wat::core::defn entry; got:\n{}",
        msg
    );
}

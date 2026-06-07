//! FM 2-bis probe for Stone 241.15 — ZOMBIE PURGE.
//!
//! Three retired-but-operational forms die together:
//! - A: `:wat::core::try` → canonical `:wat::core::Result/try` (arc 109 slice 1j retired the lowercase form; eval still dispatched)
//! - B: `:wat::core::option::expect` → canonical `:wat::core::Option/expect` (lowercase-namespace duplicate of PascalCase Type/method)
//! - C: `:wat::core::result::expect` → canonical `:wat::core::Result/expect` (same shape as B)
//!
//! Per user direction 2026-05-29 very late: *"annihilate the zombies - before define is
//! entertained - wipe the board of distractions."*
//!
//! THE DOCTRINE (per `feedback_hard_cut_admits_no_bypasses`): HARD CUT is total. No
//! "stays for help table" / "stays as sugar" / "soft retirement preserved" framings.
//!
//! HEAD-disconfirmation map (all 6 contracts FAIL at HEAD):
//! - C01: `:wat::core::try` HARD-CUT-rejected at startup
//!        ⇒ FAILS at HEAD (form dispatches via eval_try)
//! - C02: C01 rejection remedy names `:wat::core::Result/try`
//!        ⇒ FAILS at HEAD (no rejection fires; soft-deprecation arm emits a warning only)
//! - C03: `:wat::core::option::expect` HARD-CUT-rejected at startup
//!        ⇒ FAILS at HEAD (form dispatches via eval_option_expect)
//! - C04: C03 rejection remedy names `:wat::core::Option/expect`
//!        ⇒ FAILS at HEAD (no rejection fires)
//! - C05: `:wat::core::result::expect` HARD-CUT-rejected at startup
//!        ⇒ FAILS at HEAD (form dispatches via eval_result_expect)
//! - C06: C05 rejection remedy names `:wat::core::Result/expect`
//!        ⇒ FAILS at HEAD (no rejection fires)
//!
//! Post-stone: all 6 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone15_zombie_purge`

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

// ─── C01: :wat::core::try HARD-CUT-rejected with Stone 241.15 signature ────────

#[test]
fn contract_01_try_hard_cut_rejected() {
    // Zombie A — :wat::core::try is retired per arc 109 slice 1j. Canonical form
    // is :wat::core::Result/try. At HEAD: soft-deprecation arm at check.rs:1832
    // already emits "arc 109 slice 1j — retired" error. Post-stone: HARD CUT arm
    // supersedes with "Stone 241.15" marker + structured retirement remedy.
    //
    // Differentiator: post-stone error mentions "Stone 241.15"; at HEAD the
    // soft-deprecation error mentions "arc 109 slice 1j" instead.
    let src = r#"
        (:wat::core::defn :test::do-it [r <- :wat::core::Result<wat::core::i64,wat::core::String>] -> :wat::core::Result<wat::core::i64,wat::core::String>
          (:wat::core::Ok (:wat::core::try r)))
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains("Stone 241.15"),
        ":wat::core::try must be HARD-CUT-rejected via Stone 241.15 arm (not arc 109 soft-deprecation); got:\n{}",
        msg
    );
}

// ─── C02: C01 rejection remedy names :wat::core::Result/try ────────────────────

#[test]
fn contract_02_try_rejection_remedy_names_result_try() {
    let src = r#"
        (:wat::core::defn :test::do-it [r <- :wat::core::Result<wat::core::i64,wat::core::String>] -> :wat::core::Result<wat::core::i64,wat::core::String>
          (:wat::core::Ok (:wat::core::try r)))
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains(":wat::core::Result/try"),
        "try retirement remedy must name :wat::core::Result/try; got:\n{}",
        msg
    );
    assert!(
        msg.contains("[replaces a retired form]"),
        "try remedy must carry '[replaces a retired form]' annotation; got:\n{}",
        msg
    );
}

// ─── C03: :wat::core::option::expect HARD-CUT via Stone 241.15 signature ───────

#[test]
fn contract_03_option_expect_lowercase_hard_cut_rejected() {
    // Zombie B — :wat::core::option::expect lowercase namespace duplicate of
    // :wat::core::Option/expect PascalCase Type/method canonical form.
    // At HEAD: soft-deprecation arm at check.rs:1851 emits "arc 109 slice 1j" error.
    // Post-stone: HARD CUT arm fires with "Stone 241.15" marker.
    let src = r#"
        (:wat::core::defn :test::do-it [opt <- :wat::core::Option<wat::core::i64>] -> :wat::core::i64
          (:wat::core::option::expect -> :wat::core::i64 opt "expected value"))
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains("Stone 241.15"),
        ":wat::core::option::expect must be HARD-CUT-rejected via Stone 241.15 arm; got:\n{}",
        msg
    );
}

// ─── C04: C03 rejection remedy names :wat::core::Option/expect ─────────────────

#[test]
fn contract_04_option_expect_lowercase_rejection_remedy_names_pascal() {
    let src = r#"
        (:wat::core::defn :test::do-it [opt <- :wat::core::Option<wat::core::i64>] -> :wat::core::i64
          (:wat::core::option::expect -> :wat::core::i64 opt "expected value"))
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains(":wat::core::Option/expect"),
        "option::expect retirement remedy must name :wat::core::Option/expect (PascalCase); got:\n{}",
        msg
    );
    assert!(
        msg.contains("[replaces a retired form]"),
        "option::expect remedy must carry '[replaces a retired form]' annotation; got:\n{}",
        msg
    );
}

// ─── C05: :wat::core::result::expect HARD-CUT via Stone 241.15 signature ───────

#[test]
fn contract_05_result_expect_lowercase_hard_cut_rejected() {
    // Zombie C — :wat::core::result::expect lowercase namespace duplicate of
    // :wat::core::Result/expect PascalCase Type/method canonical form.
    // At HEAD: soft-deprecation arm at check.rs:1874 emits "arc 109 slice 1j" error.
    // Post-stone: HARD CUT arm fires with "Stone 241.15" marker.
    let src = r#"
        (:wat::core::defn :test::do-it [r <- :wat::core::Result<wat::core::i64,wat::core::String>] -> :wat::core::i64
          (:wat::core::result::expect -> :wat::core::i64 r "expected Ok"))
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains("Stone 241.15"),
        ":wat::core::result::expect must be HARD-CUT-rejected via Stone 241.15 arm; got:\n{}",
        msg
    );
}

// ─── C06: C05 rejection remedy names :wat::core::Result/expect ─────────────────

#[test]
fn contract_06_result_expect_lowercase_rejection_remedy_names_pascal() {
    let src = r#"
        (:wat::core::defn :test::do-it [r <- :wat::core::Result<wat::core::i64,wat::core::String>] -> :wat::core::i64
          (:wat::core::result::expect -> :wat::core::i64 r "expected Ok"))
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains(":wat::core::Result/expect"),
        "result::expect retirement remedy must name :wat::core::Result/expect (PascalCase); got:\n{}",
        msg
    );
    assert!(
        msg.contains("[replaces a retired form]"),
        "result::expect remedy must carry '[replaces a retired form]' annotation; got:\n{}",
        msg
    );
}

//! FM 2-bis probe — arc 258 Stone 258.2b: the clean macro-error primitive (the ceiling).
//!
//! `(:wat::core::macro-error <string>)` is a first-class macro-abort: evaluated in a macro
//! body it returns `Err`, which the engine wraps into a clean, catchable `MacroError`
//! (no panic, no sentinel keyword). It replaces cond's clever-ugly keyword sentinel with a
//! legible, general mechanism (any macro can raise a diagnostic) — and wat genuinely lacked
//! one. NOTE: the keyword-sentinel had NO reachable hole (C01/C02 hold at HEAD); this stone
//! is a legibility + capability upgrade, not a correctness fix.
//!
//! C01 (invariant): a non-exhaustive cond with KEYWORD bodies is rejected.
//! C02 (invariant): a non-exhaustive cond (string bodies) is rejected, naming `:else`.
//! C03 (RED at HEAD): `(:wat::core::macro-error "msg")` in a macro body surfaces "msg" as a
//!     clean diagnostic — at HEAD the head is not allow-listed, so the error is a generic
//!     RefusedInMacro that does NOT carry "msg".
//!
//! Negative fixtures (must fail at startup):
//!   tests/macros/probe_arc258_stone2b_macro_error_c01.wat
//!   tests/macros/probe_arc258_stone2b_macro_error_c02.wat
//!   tests/macros/probe_arc258_stone2b_macro_error_c03.wat
//!
//! Run: `cargo test --release --test probe_arc258_stone2b_macro_error`

use wat::freeze::startup_from_file;

fn check_path(rel_path: &str) -> Result<(), String> {
    startup_from_file(rel_path)
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn contract_01_keyword_bodied_non_exhaustive_cond_rejected() {
    let r = check_path("tests/macros/probe_arc258_stone2b_macro_error_c01.wat");
    assert!(r.is_err(), "a non-exhaustive cond (keyword bodies, no :else) must be rejected");
}

#[test]
fn contract_02_non_exhaustive_cond_names_else() {
    let r = check_path("tests/macros/probe_arc258_stone2b_macro_error_c02.wat");
    assert!(r.is_err(), "a non-exhaustive cond must be rejected");
    let err258_02 = r.unwrap_err();
    wat::assert_edn_matches_file!(
        err258_02,
        "probe_arc258_stone2b_macro_error__contract_02_non_exhaustive_cond_names_else.edn",
        "non-exhaustive cond must match diagnostic golden"
    );
}

#[test]
fn contract_03_macro_error_surfaces_its_message() {
    // A trivial macro that aborts. After 258.2b the abort message reaches the diagnostic;
    // at HEAD `macro-error` is not on the pure-combinator allow-list, so expansion refuses
    // it generically and the message never surfaces.
    let r = check_path("tests/macros/probe_arc258_stone2b_macro_error_c03.wat");
    assert!(r.is_err(), "a macro calling macro-error must abort");
    let err258_03 = r.unwrap_err();
    wat::assert_edn_matches_file!(
        err258_03,
        "probe_arc258_stone2b_macro_error__contract_03_macro_error_surfaces_its_message.edn",
        "macro-error sentinel must match diagnostic golden"
    );
}

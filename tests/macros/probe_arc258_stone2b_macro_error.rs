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

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};

#[test]
fn contract_01_keyword_bodied_non_exhaustive_cond_rejected() {
    // Not `check_path` here — that helper collapses the typed error into a Debug-formatted
    // `String` (`.map_err(|e| format!("{e:?}"))`), which erases the discriminant this stone
    // exists to reach. Call `startup_from_file` directly to keep the typed `StartupError`.
    let r = startup_from_file("tests/macros/probe_arc258_stone2b_macro_error_c01.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::core::cond"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "cond: non-exhaustive — needs a terminal :else arm"
            )
    );
}

#[test]
fn contract_02_non_exhaustive_cond_names_else() {
    // Same reasoning as contract_01: bypass `check_path`'s String-collapse to keep the typed
    // error for the kind-guard; the golden-file compare below still runs against its Debug
    // (EDN) rendering, so it keeps proving the SAME thing it always did — just no longer
    // preceded by a vacuous `is_err()`.
    let r = startup_from_file("tests/macros/probe_arc258_stone2b_macro_error_c02.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::core::cond"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "cond: non-exhaustive — needs a terminal :else arm"
            )
    );
    let err258_02 = format!("{:?}", r.unwrap_err());
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
    let r = startup_from_file("tests/macros/probe_arc258_stone2b_macro_error_c03.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":user::boom"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "kaboom-sentinel-9173"
            )
    );
    let err258_03 = format!("{:?}", r.unwrap_err());
    wat::assert_edn_matches_file!(
        err258_03,
        "probe_arc258_stone2b_macro_error__contract_03_macro_error_surfaces_its_message.edn",
        "macro-error sentinel must match diagnostic golden"
    );
}

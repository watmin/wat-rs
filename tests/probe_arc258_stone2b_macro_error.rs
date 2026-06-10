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
//! Run: `cargo test --release --test probe_arc258_stone2b_macro_error`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn check_src(body: &str) -> Result<(), String> {
    let src = format!("{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn contract_01_keyword_bodied_non_exhaustive_cond_rejected() {
    let r = check_src(
        "(:wat::core::defn :user::f [] -> :wat::core::Keyword \
           (:wat::core::cond ((:wat::core::= 1 1) :a) ((:wat::core::= 2 2) :b)))",
    );
    assert!(r.is_err(), "a non-exhaustive cond (keyword bodies, no :else) must be rejected");
}

#[test]
fn contract_02_non_exhaustive_cond_names_else() {
    let r = check_src(
        "(:wat::core::defn :user::g [] -> :wat::core::String \
           (:wat::core::cond ((:wat::core::= 1 1) \"x\") ((:wat::core::= 2 2) \"y\")))",
    );
    assert!(r.is_err(), "a non-exhaustive cond must be rejected");
    assert!(
        r.unwrap_err().contains(":else"),
        "the non-exhaustive diagnostic must name :else"
    );
}

#[test]
fn contract_03_macro_error_surfaces_its_message() {
    // A trivial macro that aborts. After 258.2b the abort message reaches the diagnostic;
    // at HEAD `macro-error` is not on the pure-combinator allow-list, so expansion refuses
    // it generically and the message never surfaces.
    let r = check_src(
        "(:wat::core::defmacro :user::boom [] -> :AST<wat::holon::HolonAST> \
           (:wat::core::macro-error \"kaboom-sentinel-9173\"))\n\
         (:wat::core::defn :user::h [] -> :wat::core::i64 (:user::boom))",
    );
    assert!(r.is_err(), "a macro calling macro-error must abort");
    assert!(
        r.unwrap_err().contains("kaboom-sentinel-9173"),
        "macro-error's message must surface in the diagnostic"
    );
}

//! Strike (examinare probe) — 251.7 implicit generics (HM / bare type-vars).
//!
//! Settles the FACT first (my breadcrumb "correction" claimed generic defns are
//! only leniently-tolerated; the disk says they're really ∀-checked via the
//! `<T,U>` name suffix → split_name_and_type_params → type_params → instantiate).
//! Then establishes the RED for the build (the faithful BARE-VAR-NO-SUFFIX form).
//!
//! F01 suffix-generic is REALLY checked : `(defn :pair-first<T> [a <- :T b <- :T] -> :T a)`
//!                                        called `(pair-first 1 "two")` → REJECTED (T:=i64, then
//!                                        b="two" ≠ i64). Proves suffix-generics are genuine today.
//! F02 suffix-generic valid call PASSES : `(pair-first 1 2)` → checks (T:=i64).
//! R03 BARE-VAR-NO-SUFFIX is the RED     : same body, name has NO `<T>`; `(pair-first2 1 2)` —
//!                                        at HEAD `T` is an opaque rigid Path, instantiate
//!                                        short-circuits → spurious TypeMismatch (unify i64 vs
//!                                        Path :T). After 251.7 (auto-generalize free sig vars):
//!                                        GREEN. This contract is the build's load-bearing row.
//!
//! Run: `cargo test --release --test probe_arc251_implicit_generics`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Returns Ok(()) if the program type-checks, Err(diag) if the checker rejects it.
fn check(src: &str) -> Result<(), String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

const MAIN: &str = "(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

#[test]
fn fact_01_suffix_generic_is_really_checked() {
    // `<T>` on the name → real type_params → instantiate → unify. T binds to i64
    // from `a`; then `b = "two"` (String) must unify with i64 → REJECT.
    let src = format!(
        "(:wat::core::defn :user::pair-first<T> [a <- :T b <- :T] -> :T a)\n\
         (:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::pair-first 1 \"two\"))\n\
         {MAIN}"
    );
    let r = check(&src);
    assert!(
        r.is_err(),
        "suffix-generic SHOULD reject the ill-typed call (T:=i64 then b=String); \
         if this passes, suffix-generics are NOT really checked. Got Ok."
    );
}

#[test]
fn fact_02_suffix_generic_valid_call_passes() {
    let src = format!(
        "(:wat::core::defn :user::pair-first<T> [a <- :T b <- :T] -> :T a)\n\
         (:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::pair-first 1 2))\n\
         {MAIN}"
    );
    assert!(check(&src).is_ok(), "valid same-type call must check: {:?}", check(&src));
}

#[test]
fn red_03_bare_var_no_suffix_is_the_target() {
    // The faithful form: bare type-var `:T` in the signature, NO `<T>` name suffix.
    // At HEAD: `type_params` is empty (nothing parses the bare sig var) → instantiate
    // short-circuits → `:T` stays a rigid Path → `(pair-first2 1 2)` unifies i64 vs
    // Path(":T") → spurious TypeMismatch. So this is RED at HEAD (the build makes it GREEN).
    let src = format!(
        "(:wat::core::defn :user::pair-first2 [a <- :T b <- :T] -> :T a)\n\
         (:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::pair-first2 1 2))\n\
         {MAIN}"
    );
    let r = check(&src);
    // Documenting the HEAD behavior — this asserts the RED so the build flips it.
    assert!(
        r.is_err(),
        "EXPECTED RED at HEAD: bare-var-no-suffix generic should currently FAIL \
         (no auto-generalization). If this is Ok at HEAD, the bare form already works \
         and 251.7's premise changes. Got Ok."
    );
}

//! Strike (examinare probe) — 251.7 implicit generics (HM / bare type-vars).
//!
//! Settles the FACT first (my breadcrumb "correction" claimed generic defns are
//! only leniently-tolerated; the disk says they're really ∀-checked via the
//! `<T,U>` name suffix → split_name_and_type_params → type_params → instantiate).
//! Then establishes the GREEN for the build (the faithful BARE-VAR-NO-SUFFIX form).
//!
//! F01 suffix-generic is REALLY checked : `(defn :pair-first<T> [a <- :T b <- :T] -> :T a)`
//!                                        called `(pair-first 1 "two")` → REJECTED (T:=i64, then
//!                                        b="two" ≠ i64). Proves suffix-generics are genuine today.
//! F02 suffix-generic valid call PASSES : `(pair-first 1 2)` → checks (T:=i64).
//! bare_var_no_suffix_now_checks        : same body, name has NO `<T>`; auto-generalize from sig;
//!                                        `(pair-first2 1 2)` must now CHECK (stone 251.7 GREEN).
//! bare_var_no_suffix_rejects_illtyped  : bare-var form rejects `(pair-first3 1 "two")` — proves
//!                                        auto-generalized vars are REALLY unified, not accepted.
//! bare_var_two_instantiations          : bare-var generic called at i64 and bool independently,
//!                                        proving distinct fresh vars per call site (no aliasing).
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
fn bare_var_no_suffix_now_checks() {
    // Stone 251.7: the faithful bare-var-no-suffix form must now auto-generalize
    // from the signature (T appears in params + return → collected → type_params = ["T"]).
    // (pair-first2 1 2) — both args i64, return i64 — must CHECK.
    let src = format!(
        "(:wat::core::defn :user::pair-first2 [a <- :T b <- :T] -> :T a)\n\
         (:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::pair-first2 1 2))\n\
         {MAIN}"
    );
    let r = check(&src);
    assert!(
        r.is_ok(),
        "bare-var-no-suffix generic MUST check after 251.7 (auto-generalize free sig vars). \
         Got Err: {:?}",
        r
    );
}

#[test]
fn bare_var_no_suffix_rejects_illtyped() {
    // Proves the auto-generalized vars are REALLY unified (not opaquely accepted).
    // T:=i64 from `a=1`; then `b="two"` (String) must unify with i64 → REJECT.
    // Mirror of fact_01 but for the bare-var form.
    let src = format!(
        "(:wat::core::defn :user::pair-first3 [a <- :T b <- :T] -> :T a)\n\
         (:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::pair-first3 1 \"two\"))\n\
         {MAIN}"
    );
    let r = check(&src);
    assert!(
        r.is_err(),
        "bare-var generic MUST reject the ill-typed call (T:=i64 then b=String); \
         auto-generalization must be real unification, not tolerance. Got Ok."
    );
}

#[test]
fn bare_var_two_instantiations() {
    // Bare-var generic called at two distinct types — proves distinct fresh vars
    // per call site (no aliasing between the two calls).
    // probe_i64: (pair-two 1 2) returns i64 — checks.
    // probe_bool: (pair-two true false) returns bool — checks.
    // If call sites aliased their vars, one of these would spuriously fail.
    let src = format!(
        "(:wat::core::defn :user::pair-two [a <- :T b <- :T] -> :T a)\n\
         (:wat::core::defn :user::probe-i64 [] -> :wat::core::i64 (:user::pair-two 1 2))\n\
         (:wat::core::defn :user::probe-bool [] -> :wat::core::bool (:user::pair-two true false))\n\
         {MAIN}"
    );
    let r = check(&src);
    assert!(
        r.is_ok(),
        "bare-var generic must check at both i64 and bool call sites independently. \
         Got Err: {:?}",
        r
    );
}

// NOTE: occurs_check_bare_var — skipped. Constructing a well-formed wat program that
// forces T = (List T) via the existing type syntax is not naturally possible without
// recursive type aliases or self-referential generic structs, neither of which wat
// currently supports in user-writable type expressions. The occurs check IS present
// in the HM pipeline (unify rejects infinite types); it is exercised transitively by
// the suffix-generic path. Adding it here would require inventing unsupported syntax.

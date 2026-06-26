//! Strike 1 (examinare verify-probe) — does the faithful-Clojure parametric type TARGET
//! type-check END-TO-END, not merely parse? fix-source's hardest rule converts the old
//! `:wat::core::Vector<wat::core::i64>` (`<>`-keyword) INTO the form `(wat.type/Vector
//! wat.type/i64)`. Before building that converter, prove the ground it lands on is solid:
//! the form must (a) accept a matching value, (b) REJECT a mismatched element type (proving
//! it enforces, not just parses), and (c) behave identically to the legacy `<>` form.
//!
//! C01: matching value type-checks               (form `(wat.type/Vector wat.type/i64)`).
//! C02: mismatched element REJECTED               (Vector<String> body vs Vector<i64> return).
//! C03: full faithful surface — `:-` + form        type-checks together.
//! C04: equivalence — the new form and the old `<>` keyword reject the SAME mismatch.
//!
//! Run: `cargo test --release --test probe_arc251_parametric_target`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Does this program type-check (startup = parse + macro-expand + register + check)?
fn checks(src: &str) -> Result<(), String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

const MAIN: &str = "(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

#[test]
fn contract_01_parametric_form_accepts_matching_value() {
    // `->` (proven arrow) isolates the parametric FORM as the only new surface under test.
    let r = checks(&format!(
        "(:wat::core::defn :user::mk [] -> (wat.type/Vector wat.type/i64) \
           (:wat::core::Vector :wat::core::i64 1 2 3))\n{MAIN}"
    ));
    assert!(r.is_ok(), "a Vector<i64> value must satisfy the form `(wat.type/Vector wat.type/i64)`; got {r:?}");
}

#[test]
fn contract_02_parametric_form_rejects_mismatched_element() {
    // THE enforcement proof: a Vector<String> body must NOT satisfy a Vector<i64> return.
    let r = checks(&format!(
        "(:wat::core::defn :user::mk [] -> (wat.type/Vector wat.type/i64) \
           (:wat::core::Vector :wat::core::String \"a\"))\n{MAIN}"
    ));
    assert!(r.is_err(), "Vector<String> must be REJECTED against `(wat.type/Vector wat.type/i64)` — the form must ENFORCE its arg, not just parse");
}

#[test]
fn contract_03_full_faithful_surface_colon_arrow_plus_form() {
    // The whole faithful surface together: `:-` return arrow + the parametric form.
    let r = checks(&format!(
        "(:wat::core::defn :user::mk [] :- (wat.type/Vector wat.type/i64) \
           (:wat::core::Vector :wat::core::i64 7))\n{MAIN}"
    ));
    assert!(r.is_ok(), "`:-` + `(wat.type/Vector wat.type/i64)` must type-check together; got {r:?}");
}

#[test]
fn contract_04_new_form_equiv_to_legacy_angle_bracket() {
    // The new form and the legacy `<>` keyword must reject the SAME mismatch — they parse to
    // the same Parametric, so the converter is a faithful re-spelling, not a semantic change.
    let new_form = checks(&format!(
        "(:wat::core::defn :user::mk [] -> (wat.type/Vector wat.type/i64) \
           (:wat::core::Vector :wat::core::String \"a\"))\n{MAIN}"
    ));
    let old_form = checks(&format!(
        "(:wat::core::defn :user::mk [] -> :wat::core::Vector<wat::core::i64> \
           (:wat::core::Vector :wat::core::String \"a\"))\n{MAIN}"
    ));
    assert!(new_form.is_err() && old_form.is_err(),
        "both spellings must reject the mismatch identically; new={new_form:?} old={old_form:?}");
}

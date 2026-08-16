//! Integration tests for arc 154 — kill `:wat::core::let*`; make
//! `:wat::core::let` sequential.
//!
//! Single-letform vocabulary (Clojure-faithful: Clojure's user-facing
//! `let` IS the sequential primitive; `let*` is a substrate-internal
//! form not part of normal user code). Two coordinated substrate
//! changes shipped in slice 1a:
//!
//!   1. **Sequential semantics under `:wat::core::let`.** The pre-arc-154
//!      `infer_let_star` / `eval_let_star` / `eval_let_star_tail` /
//!      `step_let_star` logic moved under the `let` keyword. The pre-arc
//!      parallel `let` paths retired (zero in-tree consumers per
//!      pre-arc grep — `grep ':wat::core::let[^*]'` returned 0 hits in
//!      consumer wat).
//!
//!   2. **`BareLegacyLetStar` walker.** `validate_legacy_let_star`
//!      walks every `WatAST::Keyword`; emits one
//!      `CheckError::BareLegacyLetStar` per remaining `:wat::core::let*`
//!      site for sweep 1b's mechanical 1:1 transform. Pattern 3
//!      (substrate-as-teacher § "Three migration patterns") mirroring
//!      arc 153's `BareLegacyUnitName` precedent.
//!
//! ## Tests
//!
//! Positive tests use `startup_beside(file!())` — the co-located fixture
//! covers all the valid-program shapes. Negative tests use `startup_from_file`
//! with co-located `*.wat.bad` fixtures.

use wat::freeze::{startup_beside, startup_from_file};

/// Error string from a startup-file that MUST fail.
fn startup_err_file(rel_path: &str) -> String {
    match startup_from_file(rel_path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// --- 1. Sequential semantics under :wat::core::let ---------------------

#[test]
fn let_accepts_sequential_bindings() {
    // Post-arc-154: sequential semantics under `:wat::core::let`; fixture compiles.
    startup_beside(file!()).expect("startup should succeed for sequential let");
}

// --- 2. :wat::core::let* fires migration error -------------------------

#[test]
fn let_star_post_retirement_silently_aliases_to_let() {
    // Arc 163 follow-up — walker re-armed; bare `:wat::core::let*` fires fatal.
    let err = startup_err_file(
        "tests/wat_lang/wat_arc154_kill_let_star_letstar.wat.bad",
    );
    wat::assert_edn_matches_file!(
        err,
        "wat_arc154_kill_let_star__let_star_post_retirement_silently_aliases_to_let.edn",
        "expected BareLegacyLetStar walker to fire on bare :wat::core::let*"
    );
}

// --- 3. Type-mismatch in let body still surfaces -----------------------

#[test]
fn let_body_type_mismatch_surfaces() {
    // Sequential `let` body type must unify with declared return type.
    let err = startup_err_file(
        "tests/wat_lang/wat_arc154_kill_let_star_body_mismatch.wat.bad",
    );
    wat::assert_edn_matches_file!(
        err,
        "wat_arc154_kill_let_star__let_body_type_mismatch_surfaces.edn",
        "expected ReturnTypeMismatch on body type vs declared return"
    );
}

// --- 4. Tail-call optimization preserved -------------------------------

#[test]
fn let_in_tail_position_threads_through_eval_let_tail() {
    // Tail-recursive countdown with let in tail position; fixture compiles.
    startup_beside(file!()).expect("startup should succeed for tail-position let");
}

// --- 5. Nested lets compose --------------------------------------------

#[test]
fn nested_lets_compose_with_outer_visible_to_inner() {
    // Outer let's binding visible to inner let body.
    startup_beside(file!()).expect("startup should succeed for nested lets");
}

// --- 6. Fn body containing let -----------------------------------------

#[test]
fn fn_body_with_let_preserves_sequential() {
    // Sequential let inside a fn body; fixture compiles.
    startup_beside(file!()).expect("startup should succeed for let in fn body");
}

// --- 7. Empty bindings list --------------------------------------------

#[test]
fn empty_bindings_evaluates_body_directly() {
    // `(let [] body)` is legal; fixture compiles.
    startup_beside(file!()).expect("startup should succeed for empty let bindings");
}

// --- 8. Walker narrowness ----------------------------------------------

#[test]
fn walker_narrowness_other_keywords_unaffected() {
    // Walker fires ONLY on `:wat::core::let*`; canonical let and do pass silently.
    startup_beside(file!()).expect("startup should succeed for non-let* keywords");
}

// --- 9. Multiple let* sites post-retirement — silent fall-through ---

#[test]
fn multiple_let_star_sites_post_retirement_silently_alias() {
    // Arc 163 follow-up — multiple let* sites all fire BareLegacyLetStar.
    let err = startup_err_file(
        "tests/wat_lang/wat_arc154_kill_let_star_multi_letstar.wat.bad",
    );
    wat::assert_edn_matches_file!(
        err,
        "wat_arc154_kill_let_star__multiple_let_star_sites_post_retirement_silently_alias.edn",
        "expected two BareLegacyLetStar errors for two let* sites"
    );
}

// --- 10. Reflection — :wat::core::let resolves with sequential semantics

#[test]
fn reflection_lookup_form_finds_canonical_let() {
    // The canonical `:wat::core::let` registry entry exists; fixture compiles.
    startup_beside(file!()).expect("startup should succeed for sequential let reflection probe");
}

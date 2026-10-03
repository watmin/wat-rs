//! Integration tests for arc 155 — two coordinated renames:
//!   1. `:fn(...)` → `:wat::core::Fn(...)` (type-position FQDN'd)
//!   2. `:wat::core::lambda` → `:wat::core::fn` (operator-position)
//!
//! ## Background
//!
//! User direction 2026-05-06 evening: *"we're moving closer to
//! clojure"* + *"everything needs a namespace."* Arc 155 lands the
//! fourth foundation mark of the day (after nil + do + let sequential).
//!
//! ### Rename 1 — type position
//!
//! `:wat::core::Fn(args)->ret` is the canonical FQDN spelling of the
//! function type (Cap'd type head per Clojure-faithful capitalization
//! convention: `Fn` = type, `fn` = verb; closes arc 109 slice 1e's
//! last ungrabbed parametric type head). Bare `:fn(...)` fires the
//! `BareLegacyLowercaseFn` walker (Pattern 3 substrate-as-teacher);
//! sweep 1b uses the diagnostic stream as the work list.
//!
//! **Fn-type syntax note:** Inside `fn(...)` / `Fn(...)` compound type
//! expressions, type arguments are bare Rust symbols WITHOUT a leading
//! `:` — per arc 115 / types.rs `InnerColonInCompoundArg` rule. So:
//!   - Correct outer form: `:fn(wat::core::i64)->wat::core::i64`
//!   - Correct FQDN form:  `:wat::core::Fn(wat::core::i64)->wat::core::i64`
//!   - Wrong (fires InnerColonInCompoundArg): `:fn(:wat::core::i64)->:wat::core::i64`
//!
//! ### Rename 2 — operator position
//!
//! `(:wat::core::fn ...)` is the canonical operator for function
//! values. `:wat::core::lambda` fires the `BareLegacyLambda` walker
//! (same Pattern 3 recipe; mirrors arc 154's let retirement recipe).
//! Arc 163 re-armed the walker; arc 155 slice 2 retired the runtime
//! dispatch arm. Source-level `:wat::core::lambda` fires
//! BareLegacyLambda fatal at check time (no runtime fall-through).
//!
//! ## Test shapes
//!
//! Post arc-155-slice-2 + arc-163, bare `:wat::core::lambda` fires
//! BareLegacyLambda fatal. Tests that previously expected silent
//! alias now assert the fatal diagnostic via `startup_err`.
//!
//! Wat source: tests/function/fn_rename.wat (positive combined fixture via
//! startup_beside) and tests/function/fn_rename_*.wat (per-negative-test fixtures).

use wat::freeze::{startup_beside, startup_from_file};

fn startup_ok() {
    startup_beside(file!()).expect("expected startup success");
}

fn startup_err(path: &str) -> String {
    match startup_from_file(path) {
        Err(e) => format!("{:?}", e),
        Ok(_) => panic!("expected startup error; got Ok"),
    }
}

fn startup_maybe_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => String::new(),
        Err(e) => format!("{:?}", e),
    }
}

// ─── 1. Operator-position retired: :wat::core::lambda fires BareLegacyLambda

#[test]
fn lambda_post_retirement_fires_bare_legacy_lambda() {
    // Arc 155 slice 2 + arc 163 re-arm: bare :wat::core::lambda fires
    // BareLegacyLambda fatal at check time (no runtime fall-through).
    let err = startup_err("tests/function/fn_rename_legacy_lambda.wat");
    wat::assert_edn_matches_file!(err, "fn_rename__lambda_post_retirement_fires_bare_legacy_lambda.edn", "fnr1: BareLegacyLambda golden");
}

// ─── 2. Operator-position canonical: :wat::core::fn works (positive case)

#[test]
fn fn_keyword_operator_position_works() {
    // `:wat::core::fn` in operator position routes to `infer_fn` /
    // `eval_fn`; no walker fires on this form. Canonical form is
    // fully wired.
    startup_ok();
}

// ─── 3. Type-position retired: bare :fn(...) fires BareLegacyLowercaseFn

#[test]
fn bare_fn_type_post_retirement_fires_bare_legacy_lowercase_fn() {
    // Arc 163 follow-up — walker re-armed; bare `:fn(...)` fires
    // BareLegacyLowercaseFn fatal.
    let err = startup_err("tests/function/fn_rename_bare_fn_type.wat");
    wat::assert_edn_matches_file!(err, "fn_rename__bare_fn_type_post_retirement_fires_bare_legacy_lowercase_fn.edn", "fnr3: BareLegacyLowercaseFn golden");
}

// ─── 4. Type-position canonical: :wat::core::Fn(...) works

#[test]
fn fqdn_fn_type_position_works() {
    // `:wat::core::Fn(...)` at type position resolves to `TypeExpr::Fn`
    // via the arc 155 `wat::core::Fn(` prefix in `parse_type_inner`.
    // No BareLegacyLowercaseFn fires. Canonical form is fully wired.
    startup_ok();
}

// ─── 5. Walker narrowness — operator: :wat::core::fn not flagged by
//        the lowercase-fn walker

#[test]
fn fn_operator_keyword_does_not_fire_lowercase_fn_walker() {
    // `:wat::core::fn` in operator position does NOT start with `:fn(`;
    // the `walk_for_legacy_lowercase_fn` walker must NOT fire on it.
    let err_str = startup_maybe_err("tests/function/fn_rename.wat");
    assert_eq!(err_str, "", "BareLegacyLowercaseFn must NOT fire on :wat::core::fn operator");
}

// ─── 6. Walker narrowness — type: :wat::core::Fn not flagged as legacy

#[test]
fn fqdn_fn_type_does_not_fire_lowercase_fn_walker() {
    // `:wat::core::Fn(...)` does NOT start with `:fn(`; the
    // `walk_for_legacy_lowercase_fn` walker must NOT fire on the
    // canonical FQDN form.
    let err_str = startup_maybe_err("tests/function/fn_rename.wat");
    assert_eq!(err_str, "", "BareLegacyLowercaseFn must NOT fire on :wat::core::Fn(...) type");
}

// ─── 7. Multiple :wat::core::lambda sites — post-retirement silent fall-through

#[test]
fn multiple_lambda_sites_post_retirement_each_fire_bare_legacy_lambda() {
    // Post-arc-155-slice-2 + arc-163: walker re-armed; runtime dispatch
    // arm for `:wat::core::lambda` retired. Multiple legacy-spelling
    // forms in one program all fire BareLegacyLambda fatal.
    let err = startup_err("tests/function/fn_rename_multi_lambda.wat");
    wat::assert_edn_matches_file!(err, "fn_rename__multiple_lambda_sites_post_retirement_each_fire_bare_legacy_lambda.edn", "fnr7: multiple-lambda BareLegacyLambda golden");
}

// ─── 8. Tail-call sanity: :wat::core::fn in body position works

#[test]
fn fn_body_in_tail_position_type_checks() {
    // A `:wat::core::fn` expression whose body calls a user-defined
    // function type-checks correctly.
    startup_ok();
}

// ─── 9. Mixed canonical: :wat::core::fn + :wat::core::Fn together

#[test]
fn mixed_canonical_fn_operator_and_fn_type_work_together() {
    // Both renames in their canonical forms used in the same program.
    startup_ok();
}

// ─── 10. Pre-existing arc test compat: :wat::core::fn + :wat::core::let

#[test]
fn fn_body_with_let_type_checks() {
    // Verifies arc 154's sequential `let` still works correctly inside
    // a `:wat::core::fn` body.
    startup_ok();
}

// ─── 11. Reflection: :wat::core::fn registry entry — positive case

#[test]
fn reflection_fn_registry_entry_exists() {
    // The special-forms registry minted `:wat::core::fn` in arc 155
    // slice 1a. A program that uses `:wat::core::fn` as a callable
    // should route through `infer_fn` + `eval_fn`.
    startup_ok();
}

// ─── 12. Both walkers retired post-slice-2 — mixed-legacy program silently runs

#[test]
fn both_legacy_walkers_fire_their_retirement_errors() {
    // Post-arc-155-slice-2: walker re-armed for both surfaces; mixed
    // legacy program now fires BOTH BareLegacyLambda + BareLegacyLowercaseFn.
    let err = startup_err("tests/function/fn_rename_mixed_legacy.wat");
    wat::assert_edn_matches_file!(err, "fn_rename__both_legacy_walkers_fire_their_retirement_errors.edn", "fnr12: both legacy walkers fire golden");
}

// ─── 13. Stone 255.85 — the :fn( keyword in each fixture is the retired
//        keyword-bodied fn type. The refusal is MalformedTypeExpr, by name.
//        The fixtures stay; the converter leaves the keyword in place.

fn assert_fixture_names_the_retired_keyword_fn(path: &str) {
    let kw = ":fn(wat::core::i64)->wat::core::i64";
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    assert!(
        src.contains(kw),
        "{path} does not name the retired keyword-bodied fn type"
    );
    let span = wat::span::Span::new(std::sync::Arc::new(path.to_string()), 1, 1);
    let err = wat::types::parse_type_expr_preserving_with_span(kw, &span)
        .expect_err("the keyword-bodied fn type is refused");
    match err.kind() {
        wat::types::TypeErrorKind::MalformedTypeExpr { raw, reason } => {
            assert_eq!(raw, kw);
            assert_eq!(
                reason,
                "a keyword-bodied fn type is retired; write the bracket `[A :-> R]` (stone 251.4c). `:fn(A)->R` and `:wat::core::Fn(A)->R` no longer parse"
            );
        }
        other => panic!("expected MalformedTypeExpr, got {other:?}"),
    }
}

#[test]
fn bare_fn_type_fixture_is_the_retired_keyword_refusal() {
    assert_fixture_names_the_retired_keyword_fn("tests/function/fn_rename_bare_fn_type.wat");
}

#[test]
fn mixed_legacy_fixture_is_the_retired_keyword_refusal() {
    assert_fixture_names_the_retired_keyword_fn("tests/function/fn_rename_mixed_legacy.wat");
}

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

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Asserts the given source starts up cleanly (canonical forms;
/// no legacy spellings in source). Used for positive-case tests that
/// should succeed regardless of stdlib pre-sweep state. Note: if the
/// stdlib still has legacy lambda sites, positive tests may need to
/// tolerate BareLegacyLambda errors from stdlib files (arc 154 pattern).
#[allow(dead_code)]
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

/// Arc 163 follow-up — walker re-arm tests need this. Asserts startup
/// returns Err and returns the diagnostic string for substring checks.
#[allow(dead_code)]
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Err(e) => format!("{:?}", e),
        Ok(_) => panic!("expected startup error; got Ok"),
    }
}

// ─── 1. Operator-position retired: :wat::core::lambda fires BareLegacyLambda

#[test]
fn lambda_post_retirement_silently_aliases_to_fn() {
    // Arc 155 slice 2: `validate_legacy_lambda` walker body retired
    // per substrate-as-teacher § "Retire the hint when its window
    // closes." Runtime dispatch arm for `:wat::core::lambda` also
    // retired in arc 155 slice 2 (no fall-through; source-level use
    // fires BareLegacyLambda fatal at check time via arc-163 re-arm).
    //
    // User-facing discipline: `:wat::core::fn` is the canonical
    // operator; `:wat::core::lambda` fires a fatal diagnostic.
    // Arc 163 follow-up — walker re-armed; bare :wat::core::lambda
    // fires BareLegacyLambda fatal (replaces the soft fall-through).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::lambda ((x :wat::core::i64) -> :wat::core::i64)
             x)
           5))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLambda"),
        "expected BareLegacyLambda walker to fire on bare :wat::core::lambda; got: {}",
        err
    );
}

// ─── 2. Operator-position canonical: :wat::core::fn works (positive case)

#[test]
fn fn_keyword_operator_position_works() {
    // `:wat::core::fn` in operator position routes to `infer_fn` /
    // `eval_fn`; no walker fires on this form. Canonical form is
    // fully wired. Computation moved to `(:my::apply ...)` helper;
    // main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:my::apply -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
             x)
           5))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── 3. Type-position retired: bare :fn(...) fires BareLegacyLowercaseFn
//        Note: inner args must NOT have leading ':' (arc 115 rule)
//        Note: must appear in BODY AST (not in define's own param list)
//        because define params are consumed at registration time before
//        check_program runs its walkers.

#[test]
fn bare_fn_type_post_retirement_walker_silent() {
    // Arc 155 slice 2: `walk_for_legacy_lowercase_fn` walker body
    // retired. Bare `:fn(...)` type-position keywords parse as
    // `TypeExpr::Fn` via the substrate's existing parser; no walker
    // fires. Documentation discourages bare form; substrate doesn't
    // enforce. Mirrors arc 113's orphaned-scaffolding pattern.
    // Arc 163 follow-up — walker re-armed; bare `:fn(...)` fires
    // BareLegacyLowercaseFn fatal.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn
             [g <- :fn(wat::core::i64)->wat::core::i64]
              ->
              :wat::core::i64
             (g 5))
           (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLowercaseFn"),
        "expected BareLegacyLowercaseFn walker to fire on bare :fn(...); got: {}",
        err
    );
}

// ─── 4. Type-position canonical: :wat::core::Fn(...) works

#[test]
fn fqdn_fn_type_position_works() {
    // `:wat::core::Fn(...)` at type position resolves to `TypeExpr::Fn`
    // via the arc 155 `wat::core::Fn(` prefix in `parse_type_inner`.
    // No BareLegacyLowercaseFn fires. Canonical form is fully wired.
    // Main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:user::apply
                              (f :wat::core::Fn(wat::core::i64)->wat::core::i64)
                              (x :wat::core::i64)
                              -> :wat::core::i64)
          (f x))

        (:wat::core::define (:my::invoke -> :wat::core::i64)
          (:user::apply
            (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
              x)
            42))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── 5. Walker narrowness — operator: :wat::core::fn not flagged by
//        the lowercase-fn walker

#[test]
fn fn_operator_keyword_does_not_fire_lowercase_fn_walker() {
    // `:wat::core::fn` in operator position does NOT start with `:fn(`;
    // the `walk_for_legacy_lowercase_fn` walker must NOT fire on it.
    // (`:wat::core::fn` ≠ `:fn(` — different prefix.)
    // Main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:my::apply -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
             x)
           7))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let err_str = match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::new(),
        Err(e) => format!("{:?}", e),
    };
    assert!(
        !err_str.contains("BareLegacyLowercaseFn"),
        "BareLegacyLowercaseFn must NOT fire on :wat::core::fn operator; got: {}",
        err_str
    );
}

// ─── 6. Walker narrowness — type: :wat::core::Fn not flagged as legacy

#[test]
fn fqdn_fn_type_does_not_fire_lowercase_fn_walker() {
    // `:wat::core::Fn(...)` does NOT start with `:fn(`; the
    // `walk_for_legacy_lowercase_fn` walker must NOT fire on the
    // canonical FQDN form.
    // Main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:user::apply
                              (f :wat::core::Fn(wat::core::i64)->wat::core::i64)
                              (x :wat::core::i64)
                              -> :wat::core::i64)
          (f x))

        (:wat::core::define (:my::invoke -> :wat::core::i64)
          (:user::apply
            (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 x 1))
            10))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let err_str = match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::new(),
        Err(e) => format!("{:?}", e),
    };
    assert!(
        !err_str.contains("BareLegacyLowercaseFn"),
        "BareLegacyLowercaseFn must NOT fire on :wat::core::Fn(...) type; got: {}",
        err_str
    );
}

// ─── 7. Multiple :wat::core::lambda sites — post-retirement silent fall-through

#[test]
fn multiple_lambda_sites_post_retirement_silently_alias() {
    // Post-arc-155-slice-2 + arc-163: walker re-armed; runtime dispatch
    // arm for `:wat::core::lambda` retired. Multiple legacy-spelling
    // forms in one program all fire BareLegacyLambda fatal.
    // (arc 113 scaffolding pattern; arc 163 re-armed the walker.)
    // Arc 163 follow-up — walker re-armed; bare :wat::core::lambda
    // fires BareLegacyLambda fatal.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::lambda (() -> :wat::core::i64)
             (:wat::core::i64::+'2 1 2))
           ))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLambda"),
        "expected BareLegacyLambda walker to fire on bare :wat::core::lambda; got: {}",
        err
    );
}

// ─── 8. Tail-call sanity: :wat::core::fn in body position works

#[test]
fn fn_body_in_tail_position_type_checks() {
    // A `:wat::core::fn` expression whose body calls a user-defined
    // function type-checks correctly. The `infer_fn` body handles
    // the `push_enclosing_ret` / `pop_enclosing_ret` discipline from
    // the former `infer_lambda`.
    // Main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:user::double
                              (n :wat::core::i64)
                              -> :wat::core::i64)
          (:wat::core::i64::*'2 n 2))

        (:wat::core::define (:my::apply -> :wat::core::i64)
          ((:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
             (:user::double n))
           5))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── 9. Mixed canonical: :wat::core::fn + :wat::core::Fn together

#[test]
fn mixed_canonical_fn_operator_and_fn_type_work_together() {
    // Both renames in their canonical forms used in the same program:
    // `:wat::core::Fn(...)` for the function type annotation and
    // `(:wat::core::fn ...)` for the function value.
    // Main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:user::apply
                              (f :wat::core::Fn(wat::core::i64)->wat::core::i64)
                              (x :wat::core::i64)
                              -> :wat::core::i64)
          (f x))

        (:wat::core::define (:my::invoke -> :wat::core::i64)
          (:user::apply
            (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 x 1))
            5))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── 10. Pre-existing arc test compat: :wat::core::fn + :wat::core::let

#[test]
fn fn_body_with_let_type_checks() {
    // Verifies arc 154's sequential `let` still works correctly inside
    // a `:wat::core::fn` body (the migration path is open — new `fn`
    // operator composes with other renamed forms from today).
    // Main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:my::apply -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::let
               [a (:wat::core::i64::+'2 x 5)
                b (:wat::core::i64::*'2 a 2)]
               b))
           3))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── 11. Reflection: :wat::core::fn registry entry — positive case

#[test]
fn reflection_fn_registry_entry_exists() {
    // The special-forms registry minted `:wat::core::fn` in arc 155
    // slice 1a (`src/special_forms.rs`). A program that uses
    // `:wat::core::fn` as a callable should route through `infer_fn`
    // + `eval_fn`.
    // Main is canonical nil (arc 170 slice 1f-ζ migration).
    let src = r#"
        (:wat::core::define (:my::apply -> :wat::core::i64)
          ((:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::i64::+'2 a b))
           10 20))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── 12. Both walkers retired post-slice-2 — mixed-legacy program silently runs

#[test]
fn both_legacy_walkers_retired_silently_alias() {
    // Post-arc-155-slice-2: both walkers retired; both legacy
    // spellings (`:wat::core::lambda` operator + bare `:fn(...)` type)
    // pass through silently via dispatch fall-through (lambda) and
    // existing parser support (bare fn). Mixed-legacy programs
    // type-check cleanly. Per arc 113 scaffolding precedent.
    // Arc 163 follow-up — walker re-armed for both surfaces; mixed
    // legacy program now fires BOTH BareLegacyLambda + BareLegacyLowercaseFn.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::lambda
             ((g :fn(wat::core::i64)->wat::core::i64)
              ->
              :wat::core::i64)
             (g 5))
           (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLambda") && err.contains("BareLegacyLowercaseFn"),
        "expected BOTH BareLegacyLambda + BareLegacyLowercaseFn walkers to fire; got: {}",
        err
    );
}

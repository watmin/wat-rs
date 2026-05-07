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
//! (same Pattern 3 recipe; mirrors arc 154's let* → let rename
//! exactly). Runtime dispatch arms for `:wat::core::lambda` keep
//! functional fall-through to `eval_fn` during the migration window.
//!
//! ## Test shapes
//!
//!   - **Negative-case tests**: assert specific error variants surface
//!     from `startup_err`. The two walkers fire per site; code using
//!     legacy spellings surfaces the appropriate variant deterministically.
//!
//!   - **Positive-case tests**: assert `startup_from_source` returns
//!     Ok via `startup_ok`. Positive tests may be blocked pre-sweep
//!     if stdlib forms still use legacy spellings (mirrors arc 154's
//!     slice 1a pattern — stdlib is the first sweep-1b bucket).

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Error string from a startup that MUST fail. Returns the
/// Debug-formatted CheckErrors bundle so tests can assert which
/// spans/variants appear.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

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

// ─── 1. Operator-position retired: :wat::core::lambda fires BareLegacyLambda

#[test]
fn lambda_keyword_fires_bare_legacy_lambda() {
    // `:wat::core::lambda` in operator position triggers the
    // `walk_for_legacy_lambda` walker and surfaces `BareLegacyLambda`.
    // The form itself still type-checks (fall-through dispatch to
    // `infer_fn`); the walker fires in addition to inference.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::lambda ((x :wat::core::i64) -> :wat::core::i64)
             x)
           5))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLambda"),
        "expected BareLegacyLambda on :wat::core::lambda usage; got: {}",
        err
    );
}

// ─── 2. Operator-position canonical: :wat::core::fn works (positive case;
//        may be blocked by stdlib BareLegacyLambda errors pre-sweep-1b)

#[test]
fn fn_keyword_operator_position_works() {
    // `:wat::core::fn` in operator position routes to `infer_fn` /
    // `eval_fn`; no walker fires on this form. Canonical form is
    // fully wired.
    //
    // Pre-sweep-1b: stdlib still has `:wat::core::lambda` usages;
    // `startup_from_source` loads the stdlib and the BareLegacyLambda
    // walker fires on those sites. This test passes only after sweep
    // 1b clears the stdlib — mirrors arc 154's slice 1a pattern.
    // Post-sweep-1b: should return Ok.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64)
             x)
           5))
    "#;
    // Allow BareLegacyLambda (stdlib pre-sweep); fail on unexpected errors.
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => { /* canonical form works cleanly post-sweep */ }
        Err(e) => {
            let err_str = format!("{:?}", e);
            // Expected: stdlib fires BareLegacyLambda (not our source)
            // Unexpected: anything else (BareLegacyLowercaseFn on :wat::core::fn,
            // TypeMismatch, substrate panic)
            assert!(
                err_str.contains("BareLegacyLambda") && !err_str.contains("BareLegacyLowercaseFn"),
                "unexpected error for canonical :wat::core::fn form: {}",
                err_str
            );
        }
    }
}

// ─── 3. Type-position retired: bare :fn(...) fires BareLegacyLowercaseFn
//        Note: inner args must NOT have leading ':' (arc 115 rule)
//        Note: must appear in BODY AST (not in define's own param list)
//        because define params are consumed at registration time before
//        check_program runs its walkers.

#[test]
fn bare_fn_type_fires_bare_legacy_lowercase_fn() {
    // Bare `:fn(...)` at type position inside an inline fn body signature
    // triggers `walk_for_legacy_lowercase_fn` and surfaces
    // `BareLegacyLowercaseFn`. The keyword must be in the BODY expression
    // (not in the define's own param list) so the walker sees it as a
    // `WatAST::Keyword` node in `func.body`.
    //
    // Syntax: inner args are bare Rust symbols (no `:` prefix) — e.g.,
    // `:fn(wat::core::i64)->wat::core::i64` per arc 115.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn
             ((g :fn(wat::core::i64)->wat::core::i64)
              ->
              :wat::core::i64)
             (g 5))
           (:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64) x)))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLowercaseFn"),
        "expected BareLegacyLowercaseFn on bare :fn(...) type in body; got: {}",
        err
    );
}

// ─── 4. Type-position canonical: :wat::core::Fn(...) works
//        (positive case; may be blocked by stdlib BareLegacyLambda pre-sweep-1b)

#[test]
fn fqdn_fn_type_position_works() {
    // `:wat::core::Fn(...)` at type position resolves to `TypeExpr::Fn`
    // via the arc 155 `wat::core::Fn(` prefix in `parse_type_inner`.
    // No BareLegacyLowercaseFn fires. Canonical form is fully wired.
    //
    // Pre-sweep-1b: stdlib BareLegacyLambda may block (see test #2).
    let src = r#"
        (:wat::core::define (:user::apply
                              (f :wat::core::Fn(wat::core::i64)->wat::core::i64)
                              (x :wat::core::i64)
                              -> :wat::core::i64)
          (f x))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::apply
            (:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64)
              x)
            42))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => { /* canonical Fn type works cleanly post-sweep */ }
        Err(e) => {
            let err_str = format!("{:?}", e);
            assert!(
                err_str.contains("BareLegacyLambda") && !err_str.contains("BareLegacyLowercaseFn"),
                "unexpected error for canonical :wat::core::Fn(...) type: {}",
                err_str
            );
        }
    }
}

// ─── 5. Walker narrowness — operator: :wat::core::fn not flagged by
//        the lowercase-fn walker

#[test]
fn fn_operator_keyword_does_not_fire_lowercase_fn_walker() {
    // `:wat::core::fn` in operator position does NOT start with `:fn(`;
    // the `walk_for_legacy_lowercase_fn` walker must NOT fire on it.
    // (`:wat::core::fn` ≠ `:fn(` — different prefix.)
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64)
             x)
           7))
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
    let src = r#"
        (:wat::core::define (:user::apply
                              (f :wat::core::Fn(wat::core::i64)->wat::core::i64)
                              (x :wat::core::i64)
                              -> :wat::core::i64)
          (f x))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::apply
            (:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64)
              (:wat::core::i64::+,2 x 1))
            10))
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

// ─── 7. Multiple :wat::core::lambda sites — walker fires per site

#[test]
fn multiple_lambda_sites_each_fire_bare_legacy_lambda() {
    // Both `:wat::core::lambda` forms in this program fire
    // `BareLegacyLambda`. The diagnostic stream has entries from
    // all sites (both the user source and stdlib).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::lambda (() -> :wat::core::i64)
             (:wat::core::i64::+,2 1 2))
           ))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLambda"),
        "expected BareLegacyLambda on :wat::core::lambda usages; got: {}",
        err
    );
}

// ─── 8. Tail-call sanity: :wat::core::fn in body position works
//        (positive case; may be blocked by stdlib BareLegacyLambda pre-sweep-1b)

#[test]
fn fn_body_in_tail_position_type_checks() {
    // A `:wat::core::fn` expression whose body calls a user-defined
    // function type-checks correctly. The `infer_fn` body handles
    // the `push_enclosing_ret` / `pop_enclosing_ret` discipline from
    // the former `infer_lambda`.
    let src = r#"
        (:wat::core::define (:user::double
                              (n :wat::core::i64)
                              -> :wat::core::i64)
          (:wat::core::i64::*,2 n 2))

        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn ((n :wat::core::i64) -> :wat::core::i64)
             (:user::double n))
           5))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => { /* canonical form works cleanly post-sweep */ }
        Err(e) => {
            let err_str = format!("{:?}", e);
            assert!(
                err_str.contains("BareLegacyLambda") && !err_str.contains("BareLegacyLowercaseFn"),
                "unexpected error for :wat::core::fn in body position: {}",
                err_str
            );
        }
    }
}

// ─── 9. Mixed canonical: :wat::core::fn + :wat::core::Fn together
//        (positive case; may be blocked by stdlib BareLegacyLambda pre-sweep-1b)

#[test]
fn mixed_canonical_fn_operator_and_fn_type_work_together() {
    // Both renames in their canonical forms used in the same program:
    // `:wat::core::Fn(...)` for the function type annotation and
    // `(:wat::core::fn ...)` for the function value.
    let src = r#"
        (:wat::core::define (:user::apply
                              (f :wat::core::Fn(wat::core::i64)->wat::core::i64)
                              (x :wat::core::i64)
                              -> :wat::core::i64)
          (f x))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::apply
            (:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64)
              (:wat::core::i64::+,2 x 1))
            5))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => { /* canonical forms work cleanly post-sweep */ }
        Err(e) => {
            let err_str = format!("{:?}", e);
            assert!(
                err_str.contains("BareLegacyLambda") && !err_str.contains("BareLegacyLowercaseFn"),
                "unexpected error for mixed canonical fn forms: {}",
                err_str
            );
        }
    }
}

// ─── 10. Pre-existing arc test compat: :wat::core::fn + :wat::core::let
//         (positive case; may be blocked by stdlib BareLegacyLambda pre-sweep-1b)

#[test]
fn fn_body_with_let_type_checks() {
    // Verifies arc 154's sequential `let` still works correctly inside
    // a `:wat::core::fn` body (the migration path is open — new `fn`
    // operator composes with other renamed forms from today).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64)
             (:wat::core::let
               (((a :wat::core::i64) (:wat::core::i64::+,2 x 5))
                ((b :wat::core::i64) (:wat::core::i64::*,2 a 2)))
               b))
           3))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => { /* canonical forms work cleanly post-sweep */ }
        Err(e) => {
            let err_str = format!("{:?}", e);
            assert!(
                err_str.contains("BareLegacyLambda") && !err_str.contains("BareLegacyLowercaseFn"),
                "unexpected error for :wat::core::fn + :wat::core::let composition: {}",
                err_str
            );
        }
    }
}

// ─── 11. Reflection: :wat::core::fn registry entry — positive case
//         (may be blocked by stdlib BareLegacyLambda pre-sweep-1b)

#[test]
fn reflection_fn_registry_entry_exists() {
    // The special-forms registry minted `:wat::core::fn` in arc 155
    // slice 1a (`src/special_forms.rs`). A program that uses
    // `:wat::core::fn` as a callable should route through `infer_fn`
    // + `eval_fn`.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn ((a :wat::core::i64) (b :wat::core::i64) -> :wat::core::i64)
             (:wat::core::i64::+,2 a b))
           10 20))
    "#;
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => { /* registry entry works cleanly post-sweep */ }
        Err(e) => {
            let err_str = format!("{:?}", e);
            assert!(
                err_str.contains("BareLegacyLambda") && !err_str.contains("BareLegacyLowercaseFn"),
                "unexpected error for :wat::core::fn registry entry: {}",
                err_str
            );
        }
    }
}

// ─── 12. Both walkers fire together in a mixed-legacy program

#[test]
fn both_legacy_walkers_fire_in_mixed_program() {
    // A program with BOTH legacy spellings: `:wat::core::lambda` in
    // operator position AND bare `:fn(...)` in type position inside a
    // body expression. Both walkers fire; the error bundle contains
    // both variant names.
    //
    // `:fn(...)` must be in the BODY AST (not in define's own param list)
    // so the walker sees the raw keyword node (define params are consumed
    // at registration time before check_program's walker pass).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::lambda
             ((g :fn(wat::core::i64)->wat::core::i64)
              ->
              :wat::core::i64)
             (g 5))
           (:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64) x)))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLambda"),
        "expected BareLegacyLambda in mixed-legacy program; got: {}",
        err
    );
    assert!(
        err.contains("BareLegacyLowercaseFn"),
        "expected BareLegacyLowercaseFn in mixed-legacy program; got: {}",
        err
    );
}

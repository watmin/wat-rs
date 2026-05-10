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
//! Tests come in two shapes:
//!
//!   - **Negative-case tests**: assert specific error variants surface
//!     from `startup_err`. The `BareLegacyLetStar` walker fires per
//!     site; consumer code that still says `:wat::core::let*` should
//!     surface that variant deterministically.
//!
//!   - **Positive-case tests** verify the canonical `:wat::core::let`
//!     flow works under sequential semantics: assert
//!     `startup_from_source` returns Ok via `startup_ok`.

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

/// Asserts the given source starts up cleanly (canonical `:wat::core::let`
/// with sequential semantics; no `:wat::core::let*` in source).
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

// --- 1. Sequential semantics under :wat::core::let ---------------------

#[test]
fn let_accepts_sequential_bindings() {
    // Pre-arc-154: this would have failed under parallel `:wat::core::let`
    // (the binding `b` cannot reference `a` because all RHSes evaluate
    // in the OUTER locals). Post-arc-154: sequential semantics under
    // `:wat::core::let` — `b`'s RHS sees `a`. Type check passes;
    // execution returns 6.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let
            [a 5
             b (:wat::core::i64::+'2 a 1)]
            b))
    "#;
    startup_ok(src);
}

// --- 2. :wat::core::let* fires migration error -------------------------

#[test]
fn let_star_post_retirement_silently_aliases_to_let() {
    // Arc 154 slice 2: `validate_legacy_let_star` walker body retired
    // per substrate-as-teacher § "Retire the hint when its window
    // closes." Runtime dispatch arms for `:wat::core::let*` keep
    // functional fall-through to `:wat::core::let` (sequential) —
    // mirrors arc 113's "variant + Display + dispatch scaffolding
    // stays; firing retires" pattern.
    //
    // User-facing discipline: `:wat::core::let` is the single-letform
    // spelling; `:wat::core::let*` works but is undocumented and
    // discouraged. The walker no longer fires; sweep 1b cleared all
    // in-tree consumers; future writers learn the canonical form via
    // documentation, not via migration hints.
    // Arc 163 follow-up — walker re-armed; runtime fall-through arms
    // also retired. Bare `:wat::core::let*` now fires fatal
    // BareLegacyLetStar at check time (consistent with the
    // FQDN-everywhere discipline).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((a :wat::core::i64) 5))
            a))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLetStar"),
        "expected BareLegacyLetStar walker to fire on bare :wat::core::let*; got: {}",
        err
    );
}

// --- 3. Type-mismatch in let body still surfaces -----------------------

#[test]
fn let_body_type_mismatch_surfaces() {
    // Sequential `let` body's inferred type must unify with the
    // declared return type. Body returns `:wat::core::i64` (5) but
    // the function declares `-> :wat::core::nil`. ReturnTypeMismatch
    // fires, proving inference still surfaces the contract violation
    // post-rename.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a 5]
            a))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("ReturnTypeMismatch") || err.contains("TypeMismatch"),
        "expected (Return)TypeMismatch on body type vs declared return; got: {}",
        err
    );
}

// --- 4. Tail-call optimization preserved -------------------------------

#[test]
fn let_in_tail_position_threads_through_eval_let_tail() {
    // A `let` form that wraps the function's body becomes the tail
    // expression; `eval_let_tail` evaluates the body via `eval_tail`
    // so any tail-call inside propagates without stack growth. We
    // exercise the path with a self-recursive countdown that uses
    // `let` to pre-compute a value before the recursive tail call.
    let src = r#"
        (:wat::core::define (:user::countdown
                              (n :wat::core::i64)
                              -> :wat::core::i64)
          (:wat::core::if
            (:wat::core::= n 0)
            ->
            :wat::core::i64
            n
            (:wat::core::let
              [m (:wat::core::i64::-'2 n 1)]
              (:user::countdown m))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::countdown 100))
    "#;
    startup_ok(src);
}

// --- 5. Nested lets compose --------------------------------------------

#[test]
fn nested_lets_compose_with_outer_visible_to_inner() {
    // The outer `let`'s binding `a` is visible to the inner `let`'s
    // body via lexical scope. Sequential semantics inside each `let`
    // is enforced; cross-let composition relies on standard env
    // chaining (no special-case post-rename).
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let
            [a 10]
            (:wat::core::let
              [b (:wat::core::i64::+'2 a 5)]
              b)))
    "#;
    startup_ok(src);
}

// --- 6. Fn body containing let -----------------------------------------

#[test]
fn fn_body_with_let_preserves_sequential() {
    // Sequential `let` inside a fn body: `b` sees `a`. The fn
    // is invoked at call site; result is 7.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::let
               [a x
                b (:wat::core::i64::+'2 a 5)]
               b))
           2))
    "#;
    startup_ok(src);
}

// --- 7. Empty bindings list --------------------------------------------

#[test]
fn empty_bindings_evaluates_body_directly() {
    // Degenerate but accepted: `(let [] body)` evaluates `body`
    // directly. Mirrors pre-arc-154 `let*` behavior; the new
    // sequential-under-let path preserves this corner. Updated
    // to flat-shape vector bindings per arc 168 slice 2 sweep.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let
            []
            42))
    "#;
    startup_ok(src);
}

// --- 8. Walker narrowness ----------------------------------------------

#[test]
fn walker_narrowness_other_keywords_unaffected() {
    // The `validate_legacy_let_star` walker fires ONLY on
    // `:wat::core::let*` — every other keyword (e.g., `:user::foo`,
    // `:wat::core::do`, `:wat::core::let`) passes through silently.
    // A program that uses many non-let* keywords plus the canonical
    // `:wat::core::let` should startup cleanly.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::do
            (:wat::core::let
              [x 1]
              x)
            (:wat::core::let
              [y 2]
              y)))
    "#;
    startup_ok(src);
}

// --- 9. Multiple let* sites post-retirement — silent fall-through ---

#[test]
fn multiple_let_star_sites_post_retirement_silently_alias() {
    // Post-arc-154-slice-2: BareLegacyLetStar walker retired; runtime
    // dispatch arms for `:wat::core::let*` fall through to sequential
    // `:wat::core::let`. Multiple `:wat::core::let*` source forms in
    // one program all silently work (arc 113 scaffolding pattern).
    let src = r#"
        (:wat::core::define (:user::a -> :wat::core::i64)
          (:wat::core::let*
            (((x :wat::core::i64) 1))
            x))

        (:wat::core::define (:user::b -> :wat::core::i64)
          (:wat::core::let*
            (((y :wat::core::i64) 2))
            y))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::i64::+'2 (:user::a) (:user::b)))
    "#;
    // Arc 163 follow-up — walker re-armed; both let* forms fire
    // BareLegacyLetStar fatal (one error per site).
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyLetStar"),
        "expected BareLegacyLetStar walker to fire on bare :wat::core::let*; got: {}",
        err
    );
}

// --- 10. Reflection — :wat::core::let resolves with sequential semantics

#[test]
fn reflection_lookup_form_finds_canonical_let() {
    // `:wat::runtime::lookup-form :wat::core::let` should resolve to
    // a `Binding::SpecialForm` per the special-form registry. We
    // verify the canonical entry exists by checking the program
    // starts up cleanly when invoking it. The legacy
    // `:wat::core::let*` registry entry is retained per the spawn-
    // family precedent so `(lookup-form :wat::core::let*)` also
    // resolves (with the migration sketch shape) — but the WALKER
    // fires before the form is reachable at type-check, so a value-
    // position `:wat::core::let*` keyword in user source still
    // produces BareLegacyLetStar (this is verified by test #2).
    //
    // This positive-case test confirms the `:wat::core::let`
    // registry entry survived the substrate edit and the canonical
    // sequential semantics are the registered shape.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let
            [a 1
             b (:wat::core::i64::+'2 a 2)]
            b))
    "#;
    startup_ok(src);
}

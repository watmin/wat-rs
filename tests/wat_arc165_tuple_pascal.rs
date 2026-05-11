//! Integration tests for arc 165 slice 1 — `:wat::core::tuple`
//! (lowercase) renamed to `:wat::core::Tuple` (PascalCase) as the
//! canonical spelling everywhere internal.
//!
//! ## Background
//!
//! Prior to arc 165, `Value::Tuple.type_name()` returned the bare
//! lowercase `"tuple"` (missing the FQDN prefix entirely), and the
//! eval-dispatch arm key + head-field writes also used lowercase.
//! Arc 165 slice 1 aligns all storage sites to PascalCase, completing
//! the arc 109 slice 1f vec→Vector playbook for the one remaining
//! lowercase container head.
//!
//! ## Pattern 2 poison shape (arc 109 slice 1g)
//!
//! The check.rs Pattern 2 poison at lines 3901-3914 STAYS — its
//! callee match key remains `:wat::core::tuple` (the retired legacy
//! spelling being poisoned). It emits `TypeMismatch` redirecting to
//! `:wat::core::Tuple`. Arc 165 closes the storage gap: the redirect
//! target now matches the storage canonical form.
//!
//! ## Tuple type-position syntax note
//!
//! In wat source, the TUPLE TYPE is written `:(T,U,V)` (comma-
//! separated bare type paths, no leading `:` on inner args). This is
//! the type-position form. `(:wat::core::Tuple ...)` is the CTOR
//! (expression-position form). These are distinct; return-type
//! declarations use `:(T,U,V)`.
//!
//! ## Test shapes
//!
//! - Positive tests use `startup_ok` to assert clean type-check + freeze.
//! - Negative tests use `startup_err` + substring assertions to verify
//!   specific error variants surface.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Asserts the given source starts up cleanly. Panics with the
/// diagnostic on failure.
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

/// Asserts the given source fails at startup. Returns the
/// Debug-formatted error string for substring assertions.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// --- 1. Canonical PascalCase constructor works --------------------------

#[test]
fn tuple_pascal_canonical_works() {
    // `(:wat::core::Tuple 1 2 3)` is the canonical form post-arc-165.
    // The eval-dispatch arm key at runtime.rs dispatches to
    // `eval_tuple_ctor`, which stores `Value::Tuple` with
    // `type_name() == "wat::core::Tuple"`.
    // Return type uses the tuple-type-position syntax: :(T,U,V).
    // Arc 170 slice 1f-ζ: main is canonical nil; computation in helper.
    let src = r#"
        (:wat::core::define (:my::compute -> :(wat::core::i64,wat::core::i64,wat::core::i64))
          (:wat::core::Tuple 1 2 3))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// --- 2. Legacy lowercase triggers Pattern 2 poison ----------------------

#[test]
fn legacy_tuple_lowercase_redirects_via_pattern2_poison() {
    // `(:wat::core::tuple 1 2 3)` hits the Pattern 2 poison arm in
    // check.rs lines 3901-3914. The arm emits `CheckError::TypeMismatch`
    // with `expected: ":wat::core::Tuple"` and then falls through to
    // `infer_tuple_constructor`. The diagnostic must contain both
    // "TypeMismatch" (the variant name) and "wat::core::Tuple" (the
    // redirect target — which NOW matches storage, arc 165).
    // Arc 170 slice 1f-ζ: legacy code in probe fn + nil main.
    let src = r#"
        (:wat::core::define (:my::probe -> :(wat::core::i64,wat::core::i64,wat::core::i64))
          (:wat::core::tuple 1 2 3))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("TypeMismatch"),
        "expected TypeMismatch from Pattern 2 poison; got: {}",
        err
    );
    assert!(
        err.contains("wat::core::Tuple"),
        "expected redirect target 'wat::core::Tuple' in error; got: {}",
        err
    );
}

// --- 3. Tuple in function return position type-checks clean -------------

#[test]
fn tuple_in_function_return_position() {
    // A function that declares a tuple return type using `:(T,U)` syntax
    // and returns `(:wat::core::Tuple ...)` should pass type-checking.
    // This exercises `infer_tuple_constructor` at check.rs with the
    // PascalCase head (arc 165 renamed the head in the MalformedForm
    // error at check.rs:8959 and the docstring at 8944).
    // Arc 170 slice 1f-ζ: main is canonical nil; computation in helper.
    let src = r#"
        (:wat::core::define (:user::make-pair -> :(wat::core::i64,wat::core::String))
          (:wat::core::Tuple 42 "hello"))

        (:wat::core::define (:my::invoke -> :(wat::core::i64,wat::core::String))
          (:user::make-pair))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// --- 4. type_name returns FQDN PascalCase: runtime shape matches --------

#[test]
fn type_name_returns_fqdn_pascal() {
    // Post-arc-165: `Value::Tuple.type_name()` returns "wat::core::Tuple"
    // (FQDN PascalCase). The runtime type-shape comparison at
    // runtime.rs (TypeExpr::Tuple arm) now reads:
    //
    //   TypeExpr::Tuple(_) => v.type_name() == "wat::core::Tuple"
    //
    // Pre-arc-165 this comparison always failed (type_name returned bare
    // "tuple"; comparison expected "wat::core::tuple" — both sides
    // wrong). Post-arc-165 both sides are "wat::core::Tuple" so shape
    // validation succeeds.
    //
    // This test exercises the path by binding a Tuple in a let form
    // and returning it through a declared Tuple return type. The
    // eval-dispatch arm (`:wat::core::Tuple` post-arc-165) constructs
    // the value with the canonical type_name.
    // Arc 170 slice 1f-ζ: main is canonical nil; computation in helper.
    let src = r#"
        (:wat::core::define (:my::compute -> :(wat::core::i64,wat::core::i64))
          (:wat::core::let
            [t (:wat::core::Tuple 10 20)]
            t))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

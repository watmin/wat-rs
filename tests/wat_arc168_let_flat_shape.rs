//! Integration tests for arc 168 — let flat-shape binding vector +
//! body implicit-do.
//!
//! `:wat::core::let` consumes outer bindings as a `WatAST::Vector`
//! whose elements alternate `binder expr binder expr ...`. Body is
//! 1+ trailing forms (implicit-do); empty body legal — value is
//! `:wat::core::nil`.
//!
//! Body implicit-do extends to `:wat::core::fn` and `:wat::core::defn`
//! symmetry — fn/defn body slots become `body1 body2 ... bodyN` after
//! the `-> :T` arrow.
//!
//! ## Test cases
//!
//!   1. single_binding — `[x 1]`
//!   2. multiple_bindings — `[x 1 y 2]`
//!   3. sequential_references — later RHS sees earlier names
//!   4. empty_bindings — `[]` legal
//!   5. empty_body — `(let [x 1])` returns nil
//!   6. destructure_binding — `[[a b] (Tuple ...)]`
//!   7. (retired slice 3) — was legacy_outer_list_fires_walker
//!   8. (retired slice 3) — was migration_message_text
//!   9. odd_count_vector_errors — `[x]` and `[x 1 y]`
//!  10. multi_form_let_body — non-final forms eval'd for side effect
//!  11. multi_form_let_body_typecheck — non-final form type errors surface
//!  12. multi_form_fn_body — fn with multiple body forms
//!  13. multi_form_defn_body — defn macro forwards multi-form body
//!  14. single_body_let_regression — old single-form body unchanged
//!  15. single_body_fn_regression — old single-form body unchanged

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

/// Asserts startup succeeds and `:user::main` returns the given value.
fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

/// Asserts startup fails and returns `format!("{}\n---\n{:?}", e, e)`.
/// Tests can match Display message text OR Debug variant name.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Test 1 — single_binding ─────────────────────────────────────────────────

#[test]
fn single_binding() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x 1]
            (:wat::core::i64::+'2 x 1)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 2, "expected 1+1=2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 2 — multiple_bindings ──────────────────────────────────────────────

#[test]
fn multiple_bindings() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x 1 y 2]
            (:wat::core::i64::+'2 x y)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 3, "expected 1+2=3; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 3 — sequential_references ──────────────────────────────────────────

#[test]
fn sequential_references() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x 1
                            y (:wat::core::i64::+'2 x 1)]
            y))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 2, "expected y=x+1=2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 4 — empty_bindings ─────────────────────────────────────────────────

#[test]
fn empty_bindings() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let []
            (:wat::core::i64::+'2 1 1)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 2, "expected (let [] (+ 1 1)) = 2; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 5 — empty_body ─────────────────────────────────────────────────────

/// `(:wat::core::let [x 1])` — empty body, returns `:wat::core::nil`.
/// Clojure-faithful: `(let [x 1])` evaluates to nil.
#[test]
fn empty_body() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let [x 1]))
    "#;
    let v = run(src);
    match v {
        Value::Unit => {}
        other => panic!("expected Value::Unit (:wat::core::nil); got {:?}", other),
    }
}

// ─── Test 6 — destructure_binding ────────────────────────────────────────────

/// `[[a b] (Tuple a-val b-val)]` — destructure binder is a Vector of
/// symbols. RHS is a tuple-returning expression.
#[test]
fn destructure_binding() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [[a b] (:wat::core::Tuple 3 4)]
            (:wat::core::i64::+'2 a b)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 7, "expected 3+4=7; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// Tests 7 + 8 (legacy_outer_list_fires_walker + migration_message_text)
// retired per arc 168 slice 3 — the `BareLegacyLetBindings` walker is
// gone; legacy outer-list shape now produces the standard
// `MalformedForm` parser error covered by `infer_let`'s non-Vector
// arm. No dedicated regression needed — the error shape is defined
// by the canonical Vector-only parser path.

// ─── Test 9 — odd_count_vector_errors ────────────────────────────────────────

/// `[x]` and `[x 1 y]` — odd-count bindings vector. Must surface a
/// clear MalformedForm error.
#[test]
fn odd_count_vector_errors() {
    let src_bare_one = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x]
            1))
    "#;
    let err = startup_err(src_bare_one);
    assert!(
        err.contains("even number of elements") || err.contains("MalformedForm"),
        "expected clear error on odd-count `[x]`; got: {}",
        err
    );

    let src_three = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x 1 y]
            x))
    "#;
    let err = startup_err(src_three);
    assert!(
        err.contains("even number of elements") || err.contains("MalformedForm"),
        "expected clear error on odd-count `[x 1 y]`; got: {}",
        err
    );
}

// ─── Test 10 — multi_form_let_body ───────────────────────────────────────────

/// `(let [x 1] f1 f2 f3)` — non-final forms evaluated for side
/// effect; final form is the let's value.
#[test]
fn multi_form_let_body() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x 1]
            (:wat::core::i64::+'2 x 99)
            (:wat::core::i64::+'2 x 50)
            (:wat::core::i64::+'2 x 41)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 11 — multi_form_let_body_typecheck ─────────────────────────────────

/// Non-final body form with a type mismatch surfaces a check-time
/// error (the form is type-inferred even though its value is
/// discarded).
#[test]
fn multi_form_let_body_typecheck() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x 1]
            (:wat::core::i64::+'2 x "not an int")
            (:wat::core::i64::+'2 x 41)))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("TypeMismatch")
            || err.contains("type mismatch")
            || err.contains("expected"),
        "expected type mismatch on non-final body form; got: {}",
        err
    );
}

// ─── Test 12 — multi_form_fn_body ────────────────────────────────────────────

/// `(:wat::core::fn [x <- :T] -> :T body1 body2 body3)` — fn with
/// multiple body forms after `-> :T`. Same implicit-do semantics as
/// let body.
#[test]
fn multi_form_fn_body() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64]
             -> :wat::core::i64
             (:wat::core::i64::+'2 x 99)
             (:wat::core::i64::+'2 x 50)
             (:wat::core::i64::+'2 x 41))
           1))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form return value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 13 — multi_form_defn_body ──────────────────────────────────────────

/// `(:wat::core::defn :name [args] -> :T body1 body2 body3)` — defn
/// macro forwards N body forms cleanly through fn expansion.
#[test]
fn multi_form_defn_body() {
    let src = r#"
        (:wat::core::defn :user::triple-body
          [x <- :wat::core::i64]
          -> :wat::core::i64
          (:wat::core::i64::+'2 x 99)
          (:wat::core::i64::+'2 x 50)
          (:wat::core::i64::+'2 x 41))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::triple-body 1))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected last-form return value (1+41=42); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 14 — single_body_let_regression ────────────────────────────────────

/// Single-body let — purely additive, single-form body works
/// unchanged from arc 167's behavior.
#[test]
fn single_body_let_regression() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let [x 10 y 20]
            (:wat::core::i64::+'2 x y)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 30, "expected 10+20=30; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 15 — single_body_fn_regression ─────────────────────────────────────

/// Single-body fn — purely additive, single-form body works
/// unchanged from arc 167's behavior.
#[test]
fn single_body_fn_regression() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64]
             -> :wat::core::i64
             (:wat::core::i64::+'2 x y))
           7 8))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 15, "expected 7+8=15; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

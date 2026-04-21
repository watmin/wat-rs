//! End-to-end tests for variadic `:wat::core::defmacro` — the `&`
//! rest-param syntax. Fixed params bind positionally; args after the
//! `&` rest-binder collect into a `WatAST::List` that a `,@name`
//! unquote-splicing drops into the template.
//!
//! This is the last macro-system prereq for the `pipeline` one-liner
//! composer (arc 004 item #5). With variadic, `(pipeline source
//! stage1 stage2 stage3 sink)` becomes expressible as a single macro
//! whose rest-param captures every stage and emits a let* chain.
//!
//! Coverage:
//!
//! - User-defined variadic macro that splices its rest-args into a
//!   core form (the canonical use).
//! - Zero rest-args — empty splice still produces a well-formed AST.
//! - Fixed-param-then-rest ordering.
//! - Arity error: fewer than fixed_arity args.
//! - Multiple `&` markers rejected at registration.
//! - Rest-param without a following binder rejected.

use wat::freeze::{invoke_user_main, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    let loader = InMemoryLoader::new();
    startup_from_source(src, None, &loader)
}

fn run(src: &str) -> Value {
    let world = startup(src).expect("startup should succeed");
    invoke_user_main(&world, Vec::new()).expect("main should run")
}

// ─── Canonical use: splice into a core form ───────────────────────────

#[test]
fn variadic_macro_splices_rest_into_vec_ctor() {
    // `(my::vec-of :i64 1 2 3)` expands to
    // `(:wat::core::vec :i64 1 2 3)`. The `& (items ...)` rest-binder
    // collects the trailing 1 2 3 into a list; `,@items` splices them.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::defmacro
          (:my::vec-of
            & (items :AST<Vec<holon::HolonAST>>)
            -> :AST<holon::HolonAST>)
          `(:wat::core::vec :i64 ,@items))

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::first (:my::vec-of 10 20 30)))
    "#;
    assert!(matches!(run(src), Value::i64(10)));
}

// ─── Zero rest-args ───────────────────────────────────────────────────

#[test]
fn variadic_macro_with_zero_rest_args_produces_empty_splice() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::defmacro
          (:my::empty-vec
            & (items :AST<Vec<holon::HolonAST>>)
            -> :AST<holon::HolonAST>)
          `(:wat::core::vec :i64 ,@items))

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:my::empty-vec))
    "#;
    match run(src) {
        Value::Vec(items) => assert_eq!(items.len(), 0),
        other => panic!("expected empty Vec; got {:?}", other),
    }
}

// ─── Fixed params + rest combined ─────────────────────────────────────

#[test]
fn variadic_macro_mixes_fixed_params_and_rest() {
    // `(my::prefix-sum 100 1 2 3)` expands to
    // `(+ 100 (+ 1 (+ 2 3)))` via a sum helper. Tests that fixed
    // params bind first and rest picks up only the trailing args.
    //
    // Simpler shape: macro expands to `(vec :i64 init ,@items)` and
    // we sum-fold the result. Keeps the splice the point of the test.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::defmacro
          (:my::sum-of
            (init :AST<i64>)
            & (items :AST<Vec<holon::HolonAST>>)
            -> :AST<holon::HolonAST>)
          `(:wat::core::foldl
              (:wat::core::vec :i64 ,@items)
              ,init
              (:wat::core::lambda ((acc :i64) (x :i64) -> :i64)
                (:wat::core::i64::+ acc x))))

        (:wat::core::define (:user::main -> :i64)
          (:my::sum-of 100 1 2 3))
    "#;
    assert!(matches!(run(src), Value::i64(106)));
}

// ─── Arity error: too few args ────────────────────────────────────────

#[test]
fn variadic_macro_requires_at_least_fixed_arity() {
    // `(my::sum-of)` with NO args — fixed-arity of :init is 1, so
    // zero args is a short call. Surfaces as a macro-expansion
    // ArityMismatch during startup.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::defmacro
          (:my::sum-of
            (init :AST<i64>)
            & (items :AST<Vec<holon::HolonAST>>)
            -> :AST<holon::HolonAST>)
          `(:wat::core::foldl
              (:wat::core::vec :i64 ,@items)
              ,init
              (:wat::core::lambda ((acc :i64) (x :i64) -> :i64)
                (:wat::core::i64::+ acc x))))

        (:wat::core::define (:user::main -> :i64)
          (:my::sum-of))
    "#;
    match startup(src) {
        Err(StartupError::Macro(_)) => {}
        Err(other) => panic!("expected Macro error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail"),
    }
}

// ─── Malformed rest-marker ────────────────────────────────────────────

#[test]
fn double_rest_marker_refused_at_registration() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::defmacro
          (:my::bogus
            &
            &
            (items :AST<Vec<holon::HolonAST>>)
            -> :AST<holon::HolonAST>)
          `(:wat::core::vec :i64 ,@items))

        (:wat::core::define (:user::main -> :i64) 0)
    "#;
    match startup(src) {
        Err(StartupError::Macro(_)) => {}
        Err(other) => panic!("expected Macro error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on duplicate `&`"),
    }
}

#[test]
fn rest_marker_without_binder_refused_at_registration() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::defmacro
          (:my::bogus
            (x :AST<i64>)
            &
            -> :AST<holon::HolonAST>)
          `(:wat::core::i64::+ ,x 0))

        (:wat::core::define (:user::main -> :i64) 0)
    "#;
    match startup(src) {
        Err(StartupError::Macro(_)) => {}
        Err(other) => panic!("expected Macro error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on `&` without binder"),
    }
}

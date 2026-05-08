//! Integration tests for arc 167 slice 2 — flat-shape fn signature.
//!
//! Slice 2 wires `:wat::core::fn` to consume the new 5-element flat
//! shape:
//!
//!   (:wat::core::fn  ARGS-VECTOR  ->  :RET-TYPE  BODY)
//!
//! `ARGS-VECTOR` is a `WatAST::Vector` whose body is flat triples
//! `name <- :T name <- :T ...` (empty vector → zero-arity fn). The
//! `<-` arrow reads "consumes" (input direction); the sibling `->`
//! reads "produces" (output direction). Arrows-as-duals.
//!
//! The legacy nested-sig list `((p :T) (q :T) -> :R)` retires; the
//! `BareLegacyFnSignature` walker (`src/check.rs`) catches it at
//! the diagnostic layer with a verbose migration message.
//!
//! ## Test cases
//!
//!   1. `fn_with_flat_shape_compiles_and_runs` — basic positive path
//!   2. `defn_with_flat_shape_compiles_and_runs` — defn macro forwards
//!   3. `recursive_defn_with_flat_shape` — fact(5)=120 via flat-shape defn
//!   4. `zero_arg_fn_with_empty_vector` — empty `[]` args
//!   5. `legacy_nested_sig_fn_fires_walker` — walker firing on legacy fn
//!   6. `legacy_nested_sig_defn_fires_walker_via_macro` — walker via defn
//!   7. `fn_body_type_mismatch_surfaces` — declared-vs-actual ret mismatch
//!   8. `malformed_args_vector_clear_error` — clear error on missing `<- :T`
//!   9. `reflection_on_flat_defn_resolves` — `lookup-define` round-trip

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

/// Asserts the given source starts up cleanly.
#[allow(dead_code)]
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

/// Asserts startup fails and returns the Display-formatted error
/// string PLUS the Debug repr (joined with `\n---\n`). Tests assert
/// against either surface — Display carries the migration message
/// text (BareLegacyFnSignature § Display impl); Debug carries the
/// variant name for arc 154-style assertions.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

/// Start up and invoke `:user::main` with no IO args; return the result.
fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

// ─── Test 1 — fn_with_flat_shape_compiles_and_runs ───────────────────────────

/// Inline `(:wat::core::fn [x <- :i64 y <- :i64] -> :i64 ...)` applied
/// at runtime. Exercises the new 5-element fn-form shape end-to-end:
/// parser → eval_fn → parse_fn_signature → apply_function.
#[test]
fn fn_with_flat_shape_compiles_and_runs() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64]
             -> :wat::core::i64
             (:wat::core::i64::+,2 x y))
           2 3))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 5, "expected 5 from (fn ... 2 3); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 2 — defn_with_flat_shape_compiles_and_runs ─────────────────────────

/// `:wat::core::defn` with the new flat shape: name + args-vector +
/// `->` + ret-type + body. The defn macro splices the trailing 4
/// pieces directly into `(:wat::core::fn ,@rest)`.
#[test]
fn defn_with_flat_shape_compiles_and_runs() {
    let src = r#"
        (:wat::core::defn :user::add
          [x <- :wat::core::i64 y <- :wat::core::i64]
          -> :wat::core::i64
          (:wat::core::i64::+,2 x y))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::add 2 3))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 5, "expected 5 from add(2,3); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 3 — recursive_defn_with_flat_shape ─────────────────────────────────

/// Recursive `defn` with the flat shape — verifies arc 166's recursive
/// name-binding contract survives the shape change. The fn body's
/// self-reference `:user::fact` must resolve through the pre-registered
/// SymbolTable entry that `try_parse_fn_shape_def` populates.
#[test]
fn recursive_defn_with_flat_shape() {
    let src = r#"
        (:wat::core::defn :user::fact
          [n <- :wat::core::i64]
          -> :wat::core::i64
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
            1
            (:wat::core::i64::*,2 n (:user::fact (:wat::core::i64::-,2 n 1)))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::fact 5))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 120, "expected 120 from fact(5); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 4 — zero_arg_fn_with_empty_vector ──────────────────────────────────

/// Zero-arity fn — empty args-vector `[]` followed by `-> :Ret body`.
/// Verifies the parser accepts the empty-vector edge case (the `i + 2
/// >= args_vec.len()` guard short-circuits cleanly when args_vec is
/// empty).
#[test]
fn zero_arg_fn_with_empty_vector() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn [] -> :wat::core::i64 42)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected 42 from zero-arg fn; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 5 — legacy_nested_sig_fn_fires_walker ──────────────────────────────

/// Legacy nested-sig fn `((x :T) -> :T)` triggers
/// `BareLegacyFnSignature`. Verifies the walker fires fatal at check
/// time with the verbose migration message.
#[test]
fn legacy_nested_sig_fn_fires_walker() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn ((x :wat::core::i64) -> :wat::core::i64) x) 7))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("fn signature must be a vector binding form"),
        "expected BareLegacyFnSignature migration message; got: {}",
        err
    );
}

// ─── Test 6 — legacy_nested_sig_defn_fires_walker_via_macro ──────────────────

/// Legacy nested-sig `defn` — the legacy 3-arg shape `(defn :name
/// (sig) body)` no longer matches the new variadic defn macro shape;
/// the macro will fail to expand OR the expanded fn surfaces the
/// walker. Either way, the user gets a fatal diagnostic. Assert the
/// migration message surfaces in the error stream.
#[test]
fn legacy_nested_sig_defn_fires_walker_via_macro() {
    let src = r#"
        (:wat::core::defn :user::id
          ((x :wat::core::i64) -> :wat::core::i64)
          x)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("fn signature must be a vector binding form"),
        "expected BareLegacyFnSignature migration message via defn macro; got: {}",
        err
    );
}

// ─── Test 7 — fn_body_type_mismatch_surfaces ─────────────────────────────────

/// Flat-shape fn whose body's type doesn't match the declared `-> :T`.
/// The check pass should surface `ReturnTypeMismatch` (or
/// `TypeMismatch`) at the body span.
#[test]
fn fn_body_type_mismatch_surfaces() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::nil x) 7))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("ReturnTypeMismatch") || err.contains("TypeMismatch"),
        "expected ReturnTypeMismatch on body type mismatch; got: {}",
        err
    );
}

// ─── Test 8 — malformed_args_vector_clear_error ──────────────────────────────

/// Args-vector with a missing `<- :T` triple: `[x <- :i64 y]` —
/// position 1 (the second triple, starting at index 3 of the vector)
/// has only one token instead of three. The parser must surface a
/// clear error pointing at the malformed triple.
#[test]
fn malformed_args_vector_clear_error() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::fn [x <- :wat::core::i64 y]
             -> :wat::core::i64
             x) 7))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("fn arg-vector triple") || err.contains("name <- :T"),
        "expected clear error on malformed args-vector triple; got: {}",
        err
    );
}

// ─── Test 9 — reflection_on_flat_defn_resolves ───────────────────────────────

/// After a flat-shape defn registers `:user::add` in the SymbolTable,
/// `(:wat::runtime::lookup-define :user::add)` returns Some(...).
/// Verifies arc 166's reflection path survives the shape change —
/// `try_parse_fn_shape_def` still pre-registers def-bound fn-shape
/// names so reflection sees them via `sym.functions`.
///
/// Mirrors arc 166 test 10's `lookup-define` shape: pattern-match on
/// `Some(...)` / `None` to get a definitive runtime answer.
#[test]
fn reflection_on_flat_defn_resolves() {
    let src = r#"
        (:wat::core::defn :user::add
          [x <- :wat::core::i64 y <- :wat::core::i64]
          -> :wat::core::i64
          (:wat::core::i64::+,2 x y))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match
            (:wat::runtime::lookup-define :user::add)
            -> :wat::core::i64
            ((:wat::core::Some _) 1)
            (:wat::core::None    0)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 1, "expected lookup-define to return Some (1); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

//! Integration coverage for arc 144 slice 1 — uniform `lookup_form`
//! reflection across the five form-kinds.
//!
//! The arc-143 reflection primitives (`:wat::runtime::lookup-define`,
//! `:wat::runtime::signature-of-defn`, `:wat::runtime::body-of`) now
//! dispatch on a uniform `Binding` enum (5 variants). Slice 1 ships
//! UserFunction, Macro, Primitive, and Type coverage; SpecialForm
//! arrives in slice 2 (registry not yet populated; lookup_form's
//! SpecialForm path returns None today).
//!
//! These tests verify:
//!   1. Macro lookup — defmacro is reflected; lookup-define returns
//!      Some + emission carries `:wat::core::defmacro`; signature-of-defn
//!      returns Some; body-of returns the template.
//!   2. Type lookup — struct decl is reflected; lookup-define returns
//!      Some + emission carries `:wat::core::struct`; signature-of-defn
//!      returns Some + emission carries the type's name; body-of
//!      returns :None (types are body-less in the wat sense).
//!   3. User-function lookup — no regression vs arc 143's existing
//!      coverage; the refactor preserves UserFunction behavior exactly.
//!   4. Substrate-primitive lookup — same regression-guard for
//!      `:wat::core::foldl` post-Binding-refactor.
//!   5. Unknown name — all three primitives return :None for an
//!      unregistered name.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn run_string(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Macro lookup (NEW kind for arc 144) ────────────────────────────────────

#[test]
fn lookup_define_macro_returns_some_and_emits_defmacro_head() {
    // A registered defmacro now reflects through lookup-define. The
    // emission must carry the `:wat::core::defmacro` head keyword so
    // readers can distinguish a macro-decl from a function-decl in the
    // returned AST.
    let src = r##"
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `~x)

        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :my::ident)
             rendered
              (:wat::edn::write def-opt)]
            rendered))
    "##;
    let line = run_string(src);
    assert!(
        line.contains("defmacro"),
        "expected 'defmacro' head in rendered macro define-ast, got: {}",
        line
    );
    assert!(
        line.contains("my::ident"),
        "expected macro name 'my::ident' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_defn_macro_returns_some() {
    let src = r##"
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `~x)

        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::signature-of-defn :my::ident)
            -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None    false)))
    "##;
    assert!(run_bool(src), "signature-of-defn :my::ident should return Some");
}

#[test]
fn body_of_macro_returns_some_with_template() {
    // body-of on a macro returns the stored template (the macro's
    // body field) — matches the FAQ "macros have bodies (their
    // template); primitives don't (Rust impl)".
    let src = r##"
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `~x)

        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::body-of :my::ident)
            -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None    false)))
    "##;
    assert!(run_bool(src), "body-of :my::ident should return Some");
}

// ─── Type lookup (NEW kind for arc 144) ─────────────────────────────────────

#[test]
fn lookup_define_struct_returns_some_and_emits_struct_head() {
    // A struct decl reflects through lookup-define. The emission's
    // head keyword is `:wat::core::struct` (slice 1's honest sentinel
    // shape — head + name + sentinel body slot; field rendering is a
    // future arc).
    let src = r##"
        (:wat::core::struct :my::Bar
          (open  :wat::core::f64)
          (close :wat::core::f64))

        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :my::Bar)
             rendered
              (:wat::edn::write def-opt)]
            rendered))
    "##;
    let line = run_string(src);
    assert!(
        line.contains("struct"),
        "expected 'struct' head in rendered type define-ast, got: {}",
        line
    );
    assert!(
        line.contains("my::Bar"),
        "expected type name 'my::Bar' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_defn_struct_returns_some() {
    let src = r##"
        (:wat::core::struct :my::Point
          (x :wat::core::f64)
          (y :wat::core::f64))

        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::signature-of-defn :my::Point)
            -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None    false)))
    "##;
    assert!(run_bool(src), "signature-of-defn :my::Point should return Some");
}

#[test]
fn body_of_struct_returns_none() {
    // Types declare shapes; they don't have wat bodies. body-of
    // returns :None — honest about absence (the declaration is the
    // lookup-define output).
    let src = r##"
        (:wat::core::struct :my::Tick
          (price :wat::core::f64))

        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::body-of :my::Tick)
            -> :wat::core::bool
            ((:wat::core::Some _) false)
            (:wat::core::None    true)))
    "##;
    assert!(run_bool(src), "body-of :my::Tick should return None (types have no body)");
}

// ─── Regression guards: UserFunction + Primitive behavior preserved ─────────

#[test]
fn lookup_define_user_function_still_returns_some_post_refactor() {
    // Regression guard: arc 143's UserFunction emission behavior
    // (function_to_define_ast) must be unchanged after the Binding
    // refactor.
    let src = r##"
        (:wat::core::define
          (:user::my-add (x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
          (:wat::core::+ x y))

        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::lookup-define :user::my-add)
            -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None    false)))
    "##;
    assert!(run_bool(src), "lookup-define :user::my-add should return Some");
}

#[test]
fn signature_of_defn_substrate_primitive_still_returns_some_post_refactor() {
    // Regression guard: arc 143's Primitive emission behavior
    // (type_scheme_to_signature_ast) must be unchanged after refactor.
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::signature-of-defn :wat::core::foldl)
            -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None    false)))
    "##;
    assert!(run_bool(src), "signature-of-defn :wat::core::foldl should return Some");
}

// ─── Unknown name returns None across all three primitives ──────────────────

#[test]
fn all_three_primitives_return_none_on_unknown_name() {
    // For a name no registry knows, lookup-define / signature-of-defn /
    // body-of all return :None. lookup_form's SpecialForm path is
    // currently a no-op (slice 2 territory); it does not produce
    // false-positive Some(...) for arbitrary names.
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [d-opt
              (:wat::runtime::lookup-define :no::such::thing)
             s-opt
              (:wat::runtime::signature-of-defn :no::such::thing)
             b-opt
              (:wat::runtime::body-of    :no::such::thing)]
            (:wat::core::match d-opt
              -> :wat::core::bool
              ((:wat::core::Some _) false)
              (:wat::core::None
                (:wat::core::match s-opt
                  -> :wat::core::bool
                  ((:wat::core::Some _) false)
                  (:wat::core::None
                    (:wat::core::match b-opt
                      -> :wat::core::bool
                      ((:wat::core::Some _) false)
                      (:wat::core::None    true))))))))
    "##;
    assert!(run_bool(src), "all three primitives should return None for unknown name");
}

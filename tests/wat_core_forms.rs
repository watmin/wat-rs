//! Integration coverage for `:wat::core::forms` (the variadic-quote
//! substrate primitive) and the stdlib-level `:wat::test::program`
//! defmacro that expands to it.
//!
//! `forms` is the variadic sibling of `quote`. `(:wat::core::forms
//! f1 f2 ... fn)` evaluates to a `:wat::core::Vector<wat::WatAST>` where each
//! element is the corresponding unevaluated form captured as data.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

fn unwrap_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn unwrap_bool(v: Value) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── :wat::core::forms — basic behavior ─────────────────────────────────

#[test]
fn forms_captures_each_arg_as_wat_ast() {
    // Pass three unevaluated forms; expect a Vec<wat::WatAST> of length 3.
    // Arc 170 slice 1f-ζ: main is canonical nil; compute returns bool.
    let src = r##"

        (:wat::core::define (:my::compute -> :wat::core::bool)
          (:wat::core::let
            [captured
              (:wat::core::forms (foo 1) (bar 2) (baz 3))
             n (:wat::core::length captured)]
            (:wat::core::= n 3)))
    "##;
    assert!(unwrap_bool(run(src)), "expected forms to capture 3 args");
}

#[test]
fn forms_empty_produces_empty_vec() {
    // Zero-arity must produce an empty Vec — same shape as (:wat::core::Vector :wat::WatAST).
    // Arc 170 slice 1f-ζ: main is canonical nil; compute returns bool.
    let src = r##"

        (:wat::core::define (:my::compute -> :wat::core::bool)
          (:wat::core::let
            [captured (:wat::core::forms)
             n (:wat::core::length captured)]
            (:wat::core::= n 0)))
    "##;
    assert!(unwrap_bool(run(src)), "expected forms() to produce empty vec");
}

#[test]
fn forms_args_are_not_evaluated() {
    // (undefined-symbol 99) would raise at runtime if evaluated.
    // Captured by forms, it lives as data — no evaluation, no error.
    // Arc 170 slice 1f-ζ: main is canonical nil; compute returns bool.
    let src = r##"

        (:wat::core::define (:my::compute -> :wat::core::bool)
          (:wat::core::let
            [captured
              (:wat::core::forms (:this::is::not::a::real::function 1 2 3))
             n (:wat::core::length captured)]
            (:wat::core::= n 1)))
    "##;
    assert!(unwrap_bool(run(src)), "expected forms to capture 1 unevaluated form");
}

// ─── End-to-end: forms → run-sandboxed-ast → evaluation ────────────────

#[test]
fn forms_composes_with_run_sandboxed_ast() {
    // The canonical use: build a program via forms, run it sandboxed,
    // verify the inner program's output.
    // Arc 170 slice 1f-ζ: inner program uses canonical nil main + :wat::kernel::println.
    let src = r##"

        (:wat::core::define (:my::compute -> :wat::core::String)
          (:wat::core::let
            [program
              (:wat::core::forms
                (:wat::core::define
                  (:user::main -> :wat::core::nil)
                  (:wat::kernel::println "hello-from-inside")))
             r
              (:wat::kernel::run-sandboxed-ast program
                (:wat::core::Vector :wat::core::String) :wat::core::None)
             captured (:wat::kernel::RunResult/stdout r)
             line
              (:wat::core::match (:wat::core::first captured) -> :wat::core::String
                ((:wat::core::Some s) s)
                (:wat::core::None ""))]
            line))
    "##;
    // (:wat::kernel::println "hello-from-inside") EDN-serializes strings with quotes.
    assert_eq!(unwrap_string(run(src)), "\"hello-from-inside\"");
}

// ─── :wat::test::program defmacro expands to :wat::core::forms ─────────

#[test]
fn test_program_macro_expands_correctly() {
    // The stdlib macro is a direct alias — behavior should be
    // identical to calling :wat::core::forms directly.
    // Arc 170 slice 1f-ζ: main is canonical nil; compute returns bool.
    let src = r##"

        (:wat::core::define (:my::compute -> :wat::core::bool)
          (:wat::core::let
            [captured
              (:wat::test::program (a 1) (b 2) (c 3))
             n (:wat::core::length captured)]
            (:wat::core::= n 3)))
    "##;
    assert!(unwrap_bool(run(src)), "expected :wat::test::program to capture 3 forms");
}

// ─── :wat::test::run-ast end-to-end via :wat::test::program ────────────

#[test]
fn test_run_ast_via_test_program_roundtrips_hello() {
    // The clean idiomatic shape. Compare to the string-based :wat::test::run
    // equivalent — no escapes, no nested quoting, the inner program
    // reads as actual s-expressions.
    // Arc 170 slice 1f-ζ: inner program uses canonical nil main + :wat::kernel::println.
    let src = r##"

        (:wat::core::define (:my::compute -> :wat::core::String)
          (:wat::core::let
            [r
              (:wat::test::run-ast
                (:wat::test::program
                  (:wat::core::define
                    (:user::main -> :wat::core::nil)
                    (:wat::kernel::println "hi")))
                (:wat::core::Vector :wat::core::String))
             captured (:wat::kernel::RunResult/stdout r)
             line
              (:wat::core::match (:wat::core::first captured) -> :wat::core::String
                ((:wat::core::Some s) s)
                (:wat::core::None ""))]
            line))
    "##;
    // (:wat::kernel::println "hi") EDN-serializes strings with quotes.
    assert_eq!(unwrap_string(run(src)), "\"hi\"");
}

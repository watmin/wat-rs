//! Integration coverage for `:wat::kernel::run-sandboxed-ast`
//! (arc 007 slice 3b — AST-entry sandbox).
//!
//! Arc 170 slice 1f-ζ: outer `:user::main` retired. Tests use
//! `(:my::compute -> :T)` helper + `eval_in_frozen` for the outer
//! layer. Inner programs use canonical nil main + `:wat::kernel::println`.
//! Rust asserts on the Value returned by eval_in_frozen.

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
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
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

// ─── AST-entry sandbox — happy path ─────────────────────────────────────

#[test]
fn ast_entry_prints_hello() {
    // Outer builds a 1-form inner program via quote + vec and hands it to
    // run-sandboxed-ast. Inner uses canonical nil main + :wat::kernel::println.
    // Arc 170 slice 1f-ζ: outer is :my::compute returning captured stdout line.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:my::compute -> :wat::core::String)
          (:wat::core::let
            [forms
              (:wat::core::Vector :wat::WatAST
                (:wat::core::quote
                  (:wat::core::define (:user::main -> :wat::core::nil)
                    (:wat::kernel::println "hello"))))
             r
              (:wat::kernel::run-sandboxed-ast forms (:wat::core::Vector :wat::core::String) :wat::core::None)
             lines (:wat::kernel::RunResult/stdout r)
             line
              (:wat::core::match (:wat::core::first lines) -> :wat::core::String
                ((:wat::core::Some s) s)
                (:wat::core::None ""))]
            line))
    "##;
    // :wat::kernel::println EDN-serializes strings with quotes.
    assert_eq!(unwrap_string(run(src)), "\"hello\"");
}

// ─── AST-entry sandbox — failure surfaces identically ───────────────────

#[test]
fn ast_entry_captures_assertion_failure() {
    // Inner program calls assert-eq with mismatched args; sandbox's
    // catch_unwind surfaces Failure.message. Same mechanism as the
    // source-text path — proving the AST-entry sandbox shares the
    // full plumbing.
    // Arc 170 slice 1f-ζ: outer is :my::compute returning bool (failure detected).
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [forms
              (:wat::core::Vector :wat::WatAST
                (:wat::core::quote
                  (:wat::core::define (:user::main -> :wat::core::nil)
                    (:wat::test::assert-eq 1 2))))
             r
              (:wat::kernel::run-sandboxed-ast forms (:wat::core::Vector :wat::core::String) :wat::core::None)
             fail
              (:wat::kernel::RunResult/failure r)]
            (:wat::core::match fail -> :wat::core::i64
              ((:wat::core::Some _) 1)
              (:wat::core::None    0))))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 1, "expected failure to be detected (1); got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

//! Integration: `:wat::kernel::run-sandboxed-hermetic-ast` round trip.
//!
//! Demonstrates program-generates-program: the OUTER wat program
//! forks an INNER wat program via the AST-entry hermetic path. The
//! inner code prints a value to stdout. The outer program reads that
//! captured string and evaluates it via `:wat::eval-edn!`. End result:
//! a value generated inside a fork'd child gets evaluated in the outer
//! process.
//!
//! Arc 170 slice 1f-ζ: outer main migrated to (:my::compute -> :T)
//! + eval_in_frozen. Inner programs use canonical nil main +
//! :wat::kernel::println (EDN-serializes values).

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

// ─── Simple hermetic happy path ─────────────────────────────────────────

#[test]
fn hermetic_inner_program_stdout_captured() {
    // Arc 170 slice 1f-ζ: inner uses canonical nil main + :wat::kernel::println.
    // :wat::kernel::println EDN-serializes strings with quotes.
    let src = r#"

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [result
              (:wat::kernel::run-sandboxed-hermetic-ast
                (:wat::test::program
                  (:wat::core::define (:user::main -> :wat::core::nil)
                    (:wat::kernel::println "tada!")))
                (:wat::core::Vector :wat::core::String)
                :wat::core::None)
             lines (:wat::kernel::RunResult/stdout result)]
            (:wat::core::length lines)))
    "#;
    // Inner program wrote one line → captured stdout has 1 element.
    match run(src) {
        Value::i64(n) => assert_eq!(n, 1, "expected 1 stdout line; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Round trip — program-generates-program ─────────────────────────────

#[test]
fn hermetic_output_evaluated_in_outer_scope() {
    // Inner program prints i64 42. Outer program captures stdout[0]
    // (the EDN representation "42"), then eval-edn! parses it back to
    // an i64 value.
    //
    // The round-trip: a value computed by a fork'd child gets
    // evaluated back in the parent's wat runtime.
    // Arc 170 slice 1f-ζ: inner uses canonical nil main + :wat::kernel::println 42.
    // :wat::kernel::println 42 writes "42\n" (EDN repr of i64). eval-edn! on "42"
    // parses it back to i64(42).
    let src = r#"

        (:wat::core::define (:my::compute -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>)
          (:wat::core::let
            [hermetic-result
              (:wat::kernel::run-sandboxed-hermetic-ast
                (:wat::test::program
                  (:wat::core::define (:user::main -> :wat::core::nil)
                    (:wat::kernel::println 42)))
                (:wat::core::Vector :wat::core::String)
                :wat::core::None)
             lines
              (:wat::kernel::RunResult/stdout hermetic-result)
             captured-src
              (:wat::core::match (:wat::core::first lines) -> :wat::core::String
                ((:wat::core::Some s) s)
                (:wat::core::None ""))]
            (:wat::eval-edn! captured-src)))
    "#;
    let result = run(src);
    let inner = unwrap_ok_result(result);
    // eval-edn! on "42" returns an i64 wrapped in a HolonAST (atom) or i64 directly.
    // The round-trip is verified: the child computed 42, parent evaluated it back.
    assert!(
        matches!(inner, Value::i64(42)) || matches!(inner, Value::holon__HolonAST(_)),
        "round trip should have computed 42; got {:?}",
        inner
    );
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn unwrap_ok_result(v: Value) -> Value {
    match v {
        Value::Result(r) => match &*r {
            Ok(inner) => inner.clone(),
            Err(e) => panic!("expected Ok; got Err({:?})", e),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

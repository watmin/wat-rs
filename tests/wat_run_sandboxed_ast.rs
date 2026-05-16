//! Integration coverage for the canonical body-AST entry path —
//! historically `:wat::kernel::run-sandboxed-ast`, now exercised through
//! `:wat::test::run-hermetic` / `:wat::test::run-thread` per arc 170
//! slice 4c-α-ii. The legacy substrate verb still exists (retires in
//! task #310 after the whole 4c-α chain lands); these tests now ride the
//! canonical macros so they share semantics with the rest of the test
//! corpus.
//!
//! Per-site destinations follow FM 7-ter (three-rule classification):
//! - `ast_entry_prints_hello` reads `RunResult/stdout` AND the body calls
//!   `:wat::kernel::println` → rules 1+2 → run-hermetic.
//! - `ast_entry_captures_assertion_failure` reads only `RunResult/failure`
//!   with no stdio activity in the body → run-thread is safe.
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

// ─── Body-AST entry — happy path (run-hermetic per rules 1+2) ──────────

#[test]
fn ast_entry_prints_hello() {
    // Arc 170 slice 4c-α-ii: migrated from `:wat::kernel::run-sandboxed-ast`
    // to `:wat::test::run-hermetic`. The body invokes
    // `:wat::kernel::println` and the outer reads `RunResult/stdout` —
    // rules 1+2 of FM 7-ter demand hermetic for accurate stdio capture.
    // Outer is :my::compute returning the captured stdout line.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:my::compute -> :wat::core::String)
          (:wat::core::let
            [r
              (:wat::test::run-hermetic
                (:wat::kernel::println "hello"))
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

// ─── Body-AST entry — failure surfaces identically (run-thread safe) ───

#[test]
fn ast_entry_captures_assertion_failure() {
    // The body calls assert-eq with mismatched args; the run-thread
    // driver's join-result Err arm surfaces the structured Failure.
    //
    // Arc 170 slice 4c-α-ii: migrated from `:wat::kernel::run-sandboxed-ast`
    // to `:wat::test::run-thread`. The body does not read stdio slots,
    // does not call stdio verbs, and does not mutate runtime config —
    // FM 7-ter's three rules do not fire, so thread is the correct
    // (cheaper) destination. The outer only inspects `RunResult/failure`
    // which the thread driver populates from the cascade chain.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [r
              (:wat::test::run-thread
                (:wat::test::assert-eq 1 2))
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

//! Integration: `:wat::kernel::run-sandboxed-hermetic` round trip.
//!
//! Demonstrates program-generates-program: the OUTER wat program
//! spawns a hermetic SUBPROCESS running INNER wat code. The inner
//! code prints a wat expression to stdout. The outer program reads
//! that captured string and evaluates it via `:wat::core::eval-edn!`.
//! End result: a wat expression generated inside a forked subprocess
//! gets evaluated in the outer process.
//!
//! This is the "wat generates wat, wat runs wat" loop, made
//! operational by:
//! - `:wat::kernel::run-sandboxed-hermetic` (arc 007 slice 2c) — the
//!   fork primitive using the same subprocess pattern sigusr tests
//!   use.
//! - `:wat::core::eval-edn!` (existing 2026-04-20 capability) —
//!   evaluates a runtime-constructed string as wat source.
//! - The `:wat::kernel::RunResult` + `Failure` types (arc 007
//!   pre-work) — the structured return of the hermetic call.
//!
//! The outer wat source takes the captured stdout line and passes
//! it to `eval-edn!`. The result is a `:Result<holon::HolonAST,
//! wat::core::EvalError>` carrying the evaluated value. We assert
//! the Ok arm carries the expected algebra AST.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

/// Set the hermetic-binary env var to point at the cargo-built
/// wat binary. Required before calling run-sandboxed-hermetic in
/// tests; production callers can set it themselves or install wat
/// at a canonical location.
fn ensure_hermetic_binary() {
    // env!() is a compile-time macro giving the path cargo built the
    // wat binary to. Safe to set per-test; the value is identical
    // across parallel tests so the set_var race is benign.
    std::env::set_var("WAT_HERMETIC_BINARY", env!("CARGO_BIN_EXE_wat"));
}

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

// ─── Simple hermetic happy path ─────────────────────────────────────────

#[test]
fn hermetic_inner_program_stdout_captured() {
    ensure_hermetic_binary();
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed-hermetic
            "(:wat::config::set-dims! 1024)
             (:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::io::IOWriter/println stdout \"tada!\"))"
            (:wat::core::vec :String)
            :None))
    "##;
    let result = run(src);
    let stdout = extract_stdout(&result);
    assert_eq!(
        stdout,
        vec!["tada!".to_string()],
        "hermetic inner should have written tada! to stdout"
    );
    assert!(!has_failure(&result), "expected no failure");
}

// ─── Round trip — program-generates-program ─────────────────────────────

#[test]
fn hermetic_output_evaluated_in_outer_scope() {
    ensure_hermetic_binary();
    // Inner program's stdout: a literal wat expression "(:wat::core::i64::+ 40 2)".
    // Outer program: takes the captured stdout[0] and passes it to eval-edn!.
    // The Ok arm carries i64(42). Test asserts that.
    //
    // This is the round-trip: wat source produced by a forked
    // subprocess gets evaluated back in the parent's wat runtime.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Result<holon::HolonAST,wat::core::EvalError>)
          (:wat::core::let*
            (((hermetic-result :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-hermetic
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::io::IOWriter/println stdout \"(:wat::core::i64::+ 40 2)\"))"
                (:wat::core::vec :String)
                :None))
             ((lines :Vec<String>)
              (:wat::kernel::RunResult/stdout hermetic-result))
             ((captured-src :String) (:wat::core::first lines)))
            (:wat::core::eval-edn! :wat::eval::string captured-src)))
    "##;
    let result = run(src);
    // result should be Value::Result(Ok(Value::holon__HolonAST(atom(i64=42))))
    // Actually eval-edn! wraps the final value — for i64 it'd likely be
    // atom(42). Let's just check it's Ok-variant and the inner is
    // a HolonAST carrying i64 42.
    let inner = unwrap_ok_result(result);
    // The round trip: inner subprocess wrote "(:wat::core::i64::+ 40 2)"
    // to stdout; outer captured the string and handed it to eval-edn!.
    // eval-edn! evaluated the expression and the Ok arm carries the
    // raw computed value — i64(42). wat generates wat, wat runs wat,
    // wat evaluates wat's output.
    assert!(
        matches!(inner, Value::i64(42)),
        "round trip should have computed i64(42); got {:?}",
        inner
    );
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn extract_stdout(result: &Value) -> Vec<String> {
    match result {
        Value::Struct(sv) => {
            assert_eq!(sv.type_name, ":wat::kernel::RunResult");
            match &sv.fields[0] {
                Value::Vec(items) => items
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => (**s).clone(),
                        other => panic!("stdout element not String: {:?}", other),
                    })
                    .collect(),
                other => panic!("stdout field not Vec: {:?}", other),
            }
        }
        other => panic!("expected RunResult Struct; got {:?}", other),
    }
}

fn has_failure(result: &Value) -> bool {
    match result {
        Value::Struct(sv) => match &sv.fields[2] {
            Value::Option(opt) => opt.is_some(),
            _ => false,
        },
        _ => false,
    }
}

fn unwrap_ok_result(v: Value) -> Value {
    match v {
        Value::Result(r) => match &*r {
            Ok(inner) => inner.clone(),
            Err(e) => panic!("expected Ok; got Err({:?})", e),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

//! Integration: `:wat::kernel::run-sandboxed-hermetic-ast` round trip.
//!
//! Demonstrates program-generates-program: the OUTER wat program
//! forks an INNER wat program via the AST-entry hermetic path. The
//! inner code prints a wat expression to stdout. The outer program
//! reads that captured string and evaluates it via
//! `:wat::core::eval-edn!`. End result: a wat expression generated
//! inside a fork'd child gets evaluated in the outer process.
//!
//! Arc 012 slice 3 note — this test used to exercise the string-
//! entry Rust primitive `:wat::kernel::run-sandboxed-hermetic`. That
//! primitive is retired; the AST-entry path lives in the wat stdlib
//! now. Callers that had source text go through `:wat::test::program`
//! plus `run-sandboxed-hermetic-ast` instead of escape-string-inside-
//! escape-string. The behavior tested here is identical; the shape
//! reads cleanly.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

// ─── Simple hermetic happy path ─────────────────────────────────────────

#[test]
fn hermetic_inner_program_stdout_captured() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed-hermetic-ast
            (:wat::test::program
              (:wat::config::set-capacity-mode! :error)
              (:wat::config::set-dims! 1024)
              (:wat::core::define (:user::main
                                   (stdin  :wat::io::IOReader)
                                   (stdout :wat::io::IOWriter)
                                   (stderr :wat::io::IOWriter)
                                   -> :())
                (:wat::io::IOWriter/println stdout "tada!")))
            (:wat::core::vec :String)
            :None))
    "#;
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
    // Inner program's stdout: a literal wat expression
    // "(:wat::core::i64::+ 40 2)". Outer program: takes the captured
    // stdout[0] and passes it to eval-edn!. The Ok arm carries
    // i64(42).
    //
    // The round-trip: wat source produced by a fork'd child gets
    // evaluated back in the parent's wat runtime. "wat generates
    // wat, wat runs wat, wat evaluates wat's output."
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Result<wat::holon::HolonAST,wat::core::EvalError>)
          (:wat::core::let*
            (((hermetic-result :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-hermetic-ast
                (:wat::test::program
                  (:wat::config::set-capacity-mode! :error)
                  (:wat::config::set-dims! 1024)
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout "(:wat::core::i64::+ 40 2)")))
                (:wat::core::vec :String)
                :None))
             ((lines :Vec<String>)
              (:wat::kernel::RunResult/stdout hermetic-result))
             ((captured-src :String) (:wat::core::first lines)))
            (:wat::core::eval-edn! captured-src)))
    "#;
    let result = run(src);
    let inner = unwrap_ok_result(result);
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

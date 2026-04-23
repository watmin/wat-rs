//! Integration coverage for `:wat::core::forms` (the variadic-quote
//! substrate primitive) and the stdlib-level `:wat::test::program`
//! defmacro that expands to it.
//!
//! `forms` is the variadic sibling of `quote`. `(:wat::core::forms
//! f1 f2 ... fn)` evaluates to a `:Vec<wat::WatAST>` where each
//! element is the corresponding unevaluated form captured as data.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args).expect("main");
    let bytes = stdout.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

// ─── :wat::core::forms — basic behavior ─────────────────────────────────

#[test]
fn forms_captures_each_arg_as_wat_ast() {
    // Pass three unevaluated forms; expect a Vec<wat::WatAST> of length 3.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((captured :Vec<wat::WatAST>)
              (:wat::core::forms (foo 1) (bar 2) (baz 3)))
             ((n :i64) (:wat::core::length captured)))
            (:wat::core::if (:wat::core::= n 3) -> :()
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn forms_empty_produces_empty_vec() {
    // Zero-arity must produce an empty Vec — same shape as (:wat::core::vec :wat::WatAST).
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((captured :Vec<wat::WatAST>) (:wat::core::forms))
             ((n :i64) (:wat::core::length captured)))
            (:wat::core::if (:wat::core::= n 0) -> :()
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

#[test]
fn forms_args_are_not_evaluated() {
    // (undefined-symbol 99) would raise at runtime if evaluated.
    // Captured by forms, it lives as data — no evaluation, no error.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((captured :Vec<wat::WatAST>)
              (:wat::core::forms (:this::is::not::a::real::function 1 2 3)))
             ((n :i64) (:wat::core::length captured)))
            (:wat::core::if (:wat::core::= n 1) -> :()
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── End-to-end: forms → run-sandboxed-ast → evaluation ────────────────

#[test]
fn forms_composes_with_run_sandboxed_ast() {
    // The canonical use: build a program via forms, run it sandboxed,
    // verify the inner program's output.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((program :Vec<wat::WatAST>)
              (:wat::core::forms
                (:wat::config::set-capacity-mode! :error)
                (:wat::config::set-dims! 1024)
                (:wat::core::define
                  (:user::main
                    (stdin  :wat::io::IOReader)
                    (stdout :wat::io::IOWriter)
                    (stderr :wat::io::IOWriter)
                    -> :())
                  (:wat::io::IOWriter/println stdout "hello-from-inside"))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-ast program
                (:wat::core::vec :String) :None))
             ((captured :Vec<String>) (:wat::kernel::RunResult/stdout r))
             ((line :String) (:wat::core::first captured)))
            (:wat::io::IOWriter/println stdout line)))
    "##;
    assert_eq!(run(src), vec!["hello-from-inside".to_string()]);
}

// ─── :wat::test::program defmacro expands to :wat::core::forms ─────────

#[test]
fn test_program_macro_expands_correctly() {
    // The stdlib macro is a direct alias — behavior should be
    // identical to calling :wat::core::forms directly.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((captured :Vec<wat::WatAST>)
              (:wat::test::program (a 1) (b 2) (c 3)))
             ((n :i64) (:wat::core::length captured)))
            (:wat::core::if (:wat::core::= n 3) -> :()
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── :wat::test::run-ast end-to-end via :wat::test::program ────────────

#[test]
fn test_run_ast_via_test_program_roundtrips_hello() {
    // The clean idiomatic shape. Compare to the string-based :wat::test::run
    // equivalent — no escapes, no nested quoting, the inner program
    // reads as actual s-expressions.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult)
              (:wat::test::run-ast
                (:wat::test::program
                  (:wat::config::set-capacity-mode! :error)
                  (:wat::config::set-dims! 1024)
                  (:wat::core::define
                    (:user::main
                      (stdin  :wat::io::IOReader)
                      (stdout :wat::io::IOWriter)
                      (stderr :wat::io::IOWriter)
                      -> :())
                    (:wat::io::IOWriter/println stdout "hi")))
                (:wat::core::vec :String)))
             ((captured :Vec<String>) (:wat::kernel::RunResult/stdout r))
             ((line :String) (:wat::core::first captured)))
            (:wat::io::IOWriter/println stdout line)))
    "##;
    assert_eq!(run(src), vec!["hi".to_string()]);
}

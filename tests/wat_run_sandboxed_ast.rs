//! Integration coverage for `:wat::kernel::run-sandboxed-ast`
//! (arc 007 slice 3b — AST-entry sandbox).
//!
//! Pattern: outer `:user::main` constructs a `Vec<wat::WatAST>` via
//! `(:wat::core::vec :wat::WatAST (:wat::core::quote <form>) ...)` and
//! hands it to run-sandboxed-ast. Outer reads the inner RunResult and
//! writes an observation to stdout; Rust asserts on stdout.

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
    let bytes = stdout.snapshot_bytes().expect("stdout snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

// ─── AST-entry sandbox — happy path ─────────────────────────────────────

#[test]
fn ast_entry_prints_hello() {
    // Outer builds a 3-form program via quote + vec and hands it to
    // run-sandboxed-ast. Inner writes "hello" to stdout; outer checks
    // captured stdout.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((forms :Vec<wat::WatAST>)
              (:wat::core::vec :wat::WatAST
                (:wat::core::quote (:wat::config::set-capacity-mode! :error))
                (:wat::core::quote
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :wat::core::unit)
                    (:wat::io::IOWriter/println stdout "hello")))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-ast forms (:wat::core::vec :wat::core::String) :None))
             ((lines :Vec<wat::core::String>) (:wat::kernel::RunResult/stdout r))
             ((line :wat::core::String)
              (:wat::core::match (:wat::core::first lines) -> :wat::core::String
                ((Some s) s)
                (:None ""))))
            (:wat::io::IOWriter/println stdout line)))
    "##;
    assert_eq!(run(src), vec!["hello"]);
}

// ─── AST-entry sandbox — failure surfaces identically ───────────────────

#[test]
fn ast_entry_captures_assertion_failure() {
    // Inner program calls assert-eq with mismatched args; sandbox's
    // catch_unwind surfaces Failure.message. Same mechanism as the
    // source-text path — proving the AST-entry sandbox shares the
    // full plumbing.
    let src = r##"
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((forms :Vec<wat::WatAST>)
              (:wat::core::vec :wat::WatAST
                (:wat::core::quote (:wat::config::set-capacity-mode! :error))
                (:wat::core::quote
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :wat::core::unit)
                    (:wat::test::assert-eq 1 2)))))
             ((r :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-ast forms (:wat::core::vec :wat::core::String) :None))
             ((fail :wat::core::Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :wat::core::unit
              ((Some f) (:wat::io::IOWriter/println stdout
                          (:wat::kernel::Failure/message f)))
              (:None    (:wat::io::IOWriter/println stdout "NO-FAILURE")))))
    "##;
    assert_eq!(run(src), vec!["assert-eq failed"]);
}


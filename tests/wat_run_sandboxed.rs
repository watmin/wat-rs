//! End-to-end tests for `:wat::kernel::run-sandboxed` — arc 007 slice 2a.
//!
//! The sandbox takes wat source + stdin lines + scope, freezes a fresh
//! inner world, invokes `:user::main` with StringIo-backed stdio, and
//! captures what the program wrote. Happy-path coverage only in slice
//! 2a; panic isolation / shutdown-wait / scope-enforcement tests land
//! in slice 2b.
//!
//! Covers:
//! - No-op main → empty stdout + stderr, failure: None.
//! - Main writes one line → stdout captured.
//! - Main writes multiple lines to both stdout and stderr.
//! - Main reads stdin and echoes to stdout.
//! - Main uses `print` (no newline) vs `println` (with newline).
//! - Scope `:None` isolates from disk.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

/// Unwrap a RunResult struct value into its three fields.
fn unwrap_run_result(v: Value) -> (Vec<String>, Vec<String>, bool) {
    match v {
        Value::Struct(sv) => {
            assert_eq!(sv.type_name, ":wat::kernel::RunResult");
            assert_eq!(sv.fields.len(), 3);
            let stdout = as_vec_string(&sv.fields[0]);
            let stderr = as_vec_string(&sv.fields[1]);
            let failure_is_some = match &sv.fields[2] {
                Value::Option(opt) => opt.is_some(),
                other => panic!("expected Option for failure; got {:?}", other),
            };
            (stdout, stderr, failure_is_some)
        }
        other => panic!("expected Struct; got {:?}", other),
    }
}

fn as_vec_string(v: &Value) -> Vec<String> {
    match v {
        Value::Vec(items) => items
            .iter()
            .map(|item| match item {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String; got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec; got {:?}", other),
    }
}

// ─── Happy path — no-op main ─────────────────────────────────────────────

#[test]
fn noop_main_yields_empty_stdout_and_stderr() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        ;; Outer program: runs a sandboxed no-op main.
        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-dims! 1024)
             (:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               ())"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert!(stdout.is_empty(), "expected empty stdout; got {:?}", stdout);
    assert!(stderr.is_empty(), "expected empty stderr; got {:?}", stderr);
    assert!(!failure, "expected failure: None; got Some");
}

// ─── Single stdout write ─────────────────────────────────────────────────

#[test]
fn main_writes_single_line_to_stdout() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-dims! 1024)
             (:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::io::IOWriter/println stdout \"hello\"))"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["hello".to_string()]);
    assert!(stderr.is_empty());
    assert!(!failure);
}

// ─── Multi-line + stderr ─────────────────────────────────────────────────

#[test]
fn main_writes_to_both_stdout_and_stderr() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-dims! 1024)
             (:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::core::let*
                 (((_ :()) (:wat::io::IOWriter/println stdout \"one\"))
                  ((_ :()) (:wat::io::IOWriter/println stdout \"two\"))
                  ((_ :()) (:wat::io::IOWriter/println stderr \"oops\")))
                 ()))"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["one".to_string(), "two".to_string()]);
    assert_eq!(stderr, vec!["oops".to_string()]);
    assert!(!failure);
}

// ─── Main echoes stdin to stdout ─────────────────────────────────────────

#[test]
fn main_echoes_stdin_to_stdout() {
    // r##"..."## delimiter so the outer vec :String "watmin" doesn't
    // need backslash-escaped quotes at the wat surface.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-dims! 1024)
             (:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
                 ((Some line) (:wat::io::IOWriter/println stdout line))
                 (:None ())))"
            (:wat::core::vec :String "watmin")
            :None))
    "##;
    let (stdout, stderr, failure) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["watmin".to_string()]);
    assert!(stderr.is_empty());
    assert!(!failure);
}

// ─── print (no newline) vs println ───────────────────────────────────────

#[test]
fn print_without_newline_does_not_split_into_lines() {
    // Three prints to stdout: "a" + "b" + "c". No newline.
    // Buffer: "abc". Split on \n: ["abc"]. No trailing \n to trim.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-dims! 1024)
             (:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:wat::core::let*
                 (((_ :()) (:wat::io::IOWriter/print stdout \"a\"))
                  ((_ :()) (:wat::io::IOWriter/print stdout \"b\"))
                  ((_ :()) (:wat::io::IOWriter/print stdout \"c\")))
                 ()))"
            (:wat::core::vec :String)
            :None))
    "#;
    let (stdout, _, _) = unwrap_run_result(run(src));
    assert_eq!(stdout, vec!["abc".to_string()]);
}

// ─── Multiple stdin lines ────────────────────────────────────────────────

#[test]
fn main_reads_multiple_stdin_lines() {
    // Read and println each line until EOF.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :wat::kernel::RunResult)
          (:wat::kernel::run-sandboxed
            "(:wat::config::set-dims! 1024)
             (:wat::config::set-capacity-mode! :error)
             (:wat::core::define (:my::echo-all
                                  (r :wat::io::IOReader)
                                  (w :wat::io::IOWriter)
                                  -> :())
               (:wat::core::match (:wat::io::IOReader/read-line r) -> :()
                 ((Some line)
                   (:wat::core::let*
                     (((_ :()) (:wat::io::IOWriter/println w line)))
                     (:my::echo-all r w)))
                 (:None ())))
             (:wat::core::define (:user::main
                                  (stdin  :wat::io::IOReader)
                                  (stdout :wat::io::IOWriter)
                                  (stderr :wat::io::IOWriter)
                                  -> :())
               (:my::echo-all stdin stdout))"
            (:wat::core::vec :String "alpha" "beta" "gamma")
            :None))
    "##;
    let (stdout, _, _) = unwrap_run_result(run(src));
    assert_eq!(
        stdout,
        vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
    );
}

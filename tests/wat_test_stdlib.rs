//! Integration coverage for `:wat::test::*` stdlib (arc 007 slice 3).
//!
//! The pattern: outer `:user::main` uses `:wat::test::run` to sandbox
//! an inner program that exercises the assertions; outer main inspects
//! the inner `RunResult` via `:wat::kernel::RunResult/failure` accessor
//! and writes an observation to stdout; Rust tests assert on stdout.
//!
//! This is also the proof point the DESIGN.md called out:
//!
//! > The first inscription of the arc will be a `.wat` test file whose
//! > `:user::main` runs a sandboxed wat program and asserts against its
//! > RunResult. That file is the proof point.

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

// ─── :wat::test::assert-eq — happy + failure paths ──────────────────────

#[test]
fn assert_eq_pass_returns_unit() {
    // Inner: calls assert-eq 42 42; completes cleanly. Outer checks the
    // inner RunResult has no failure.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult)
              (:wat::test::run
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::test::assert-eq 42 42))"
                (:wat::core::vec :String)))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some _) (:wat::io::IOWriter/println stdout "FAIL"))
              (:None    (:wat::io::IOWriter/println stdout "PASS")))))
    "##;
    assert_eq!(run(src), vec!["PASS"]);
}

#[test]
fn assert_eq_fail_populates_message() {
    // Inner: calls assert-eq 42 43; panics; run-sandboxed catches.
    // Outer reads failure.message and writes it.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult)
              (:wat::test::run
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::test::assert-eq 42 43))"
                (:wat::core::vec :String)))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some f) (:wat::io::IOWriter/println stdout
                          (:wat::kernel::Failure/message f)))
              (:None    (:wat::io::IOWriter/println stdout "NO-FAILURE")))))
    "##;
    assert_eq!(run(src), vec!["assert-eq failed"]);
}

// ─── :wat::test::assert-contains — actual/expected populated ────────────

#[test]
fn assert_contains_pass() {
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult)
              (:wat::test::run
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::test::assert-contains \"hello world\" \"world\"))"
                (:wat::core::vec :String)))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some _) (:wat::io::IOWriter/println stdout "FAIL"))
              (:None    (:wat::io::IOWriter/println stdout "PASS")))))
    "##;
    assert_eq!(run(src), vec!["PASS"]);
}

#[test]
fn assert_contains_fail_populates_actual_expected() {
    // Inner: haystack "hello" does not contain "xyz". Outer reads
    // failure.actual (haystack) and failure.expected (needle).
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult)
              (:wat::test::run
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::test::assert-contains \"hello\" \"xyz\"))"
                (:wat::core::vec :String)))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some f)
                (:wat::core::let*
                  (((actual :Option<String>) (:wat::kernel::Failure/actual f))
                   ((expected :Option<String>) (:wat::kernel::Failure/expected f))
                   ((_ :())
                    (:wat::core::match actual -> :()
                      ((Some a) (:wat::io::IOWriter/println stdout a))
                      (:None    (:wat::io::IOWriter/println stdout "NO-ACTUAL")))))
                  (:wat::core::match expected -> :()
                    ((Some e) (:wat::io::IOWriter/println stdout e))
                    (:None    (:wat::io::IOWriter/println stdout "NO-EXPECTED")))))
              (:None (:wat::io::IOWriter/println stdout "NO-FAILURE")))))
    "##;
    assert_eq!(run(src), vec!["hello", "xyz"]);
}

// ─── :wat::test::assert-stdout-is ───────────────────────────────────────

#[test]
fn assert_stdout_is_pass() {
    // Inner writes "hi" then "there" to stdout; assert_stdout_is matches.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((assertion-result :wat::kernel::RunResult)
              (:wat::test::run
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::core::let*
                     (((expected :Vec<String>)
                       (:wat::core::conj
                         (:wat::core::conj (:wat::core::vec :String) \"hi\")
                         \"there\"))
                      ((inner :wat::kernel::RunResult)
                       (:wat::test::run
                         \"(:wat::config::set-dims! 1024)
                          (:wat::config::set-capacity-mode! :error)
                          (:wat::core::define (:user::main
                                               (stdin  :wat::io::IOReader)
                                               (stdout :wat::io::IOWriter)
                                               (stderr :wat::io::IOWriter)
                                               -> :())
                            (:wat::core::let*
                              (((_ :()) (:wat::io::IOWriter/println stdout \\\"hi\\\"))
                               ((_ :()) (:wat::io::IOWriter/println stdout \\\"there\\\")))
                              ()))\"
                         (:wat::core::vec :String))))
                     (:wat::test::assert-stdout-is inner expected)))"
                (:wat::core::vec :String)))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure assertion-result)))
            (:wat::core::match fail -> :()
              ((Some _) (:wat::io::IOWriter/println stdout "FAIL"))
              (:None    (:wat::io::IOWriter/println stdout "PASS")))))
    "##;
    assert_eq!(run(src), vec!["PASS"]);
}

// ─── :wat::test::assert-stderr-matches ──────────────────────────────────

#[test]
fn assert_stderr_matches_pass() {
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult)
              (:wat::test::run
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::core::let*
                     (((assertion :wat::kernel::RunResult)
                       (:wat::test::run
                         \"(:wat::config::set-dims! 1024)
                          (:wat::config::set-capacity-mode! :error)
                          (:wat::core::define (:user::main
                                               (stdin  :wat::io::IOReader)
                                               (stdout :wat::io::IOWriter)
                                               (stderr :wat::io::IOWriter)
                                               -> :())
                            (:wat::io::IOWriter/println stderr \\\"error: code 42\\\"))\"
                         (:wat::core::vec :String))))
                     (:wat::test::assert-stderr-matches assertion \"code [0-9]+\")))"
                (:wat::core::vec :String)))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some _) (:wat::io::IOWriter/println stdout "FAIL"))
              (:None    (:wat::io::IOWriter/println stdout "PASS")))))
    "##;
    assert_eq!(run(src), vec!["PASS"]);
}

#[test]
fn assert_stderr_matches_fail_reports_pattern() {
    // Inner writes nothing to stderr; assert-stderr-matches fails.
    // Outer reads failure.expected (the pattern) to verify it's the
    // regex we passed.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult)
              (:wat::test::run
                "(:wat::config::set-dims! 1024)
                 (:wat::config::set-capacity-mode! :error)
                 (:wat::core::define (:user::main
                                      (stdin  :wat::io::IOReader)
                                      (stdout :wat::io::IOWriter)
                                      (stderr :wat::io::IOWriter)
                                      -> :())
                   (:wat::core::let*
                     (((silent :wat::kernel::RunResult)
                       (:wat::test::run
                         \"(:wat::config::set-dims! 1024)
                          (:wat::config::set-capacity-mode! :error)
                          (:wat::core::define (:user::main
                                               (stdin  :wat::io::IOReader)
                                               (stdout :wat::io::IOWriter)
                                               (stderr :wat::io::IOWriter)
                                               -> :())
                            ())\"
                         (:wat::core::vec :String))))
                     (:wat::test::assert-stderr-matches silent \"my-pattern\")))"
                (:wat::core::vec :String)))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some f)
                (:wat::core::let*
                  (((expected :Option<String>) (:wat::kernel::Failure/expected f)))
                  (:wat::core::match expected -> :()
                    ((Some e) (:wat::io::IOWriter/println stdout e))
                    (:None    (:wat::io::IOWriter/println stdout "NO-EXPECTED")))))
              (:None (:wat::io::IOWriter/println stdout "NO-FAILURE")))))
    "##;
    assert_eq!(run(src), vec!["my-pattern"]);
}

//! Integration coverage for `:wat::test::deftest` (arc 007 slice 3b).
//!
//! Pattern: outer program uses `deftest` to register a named test
//! function that returns `RunResult`. The outer `:user::main` invokes
//! the registered test and inspects the result — PASS if Failure is
//! None, FAIL (with message) otherwise. Rust asserts on stdout.

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

// ─── deftest — registers a named test; passing case ─────────────────────

#[test]
fn deftest_registers_named_passing_test() {
    // deftest :my::test::two-plus-two declares a zero-arg function.
    // :user::main calls it, unwraps RunResult, reports PASS.
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::test::deftest :my::test::two-plus-two 1024 :error
          (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult) (:my::test::two-plus-two))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some f) (:wat::io::IOWriter/println stdout
                          (:wat::kernel::Failure/message f)))
              (:None    (:wat::io::IOWriter/println stdout "PASS")))))
    "##;
    assert_eq!(run(src), vec!["PASS"]);
}

// ─── deftest — failing case surfaces through Failure ────────────────────

#[test]
fn deftest_failure_surfaces_message() {
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::test::deftest :my::test::bad-math 1024 :error
          (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 5))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult) (:my::test::bad-math))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some f) (:wat::io::IOWriter/println stdout
                          (:wat::kernel::Failure/message f)))
              (:None    (:wat::io::IOWriter/println stdout "UNEXPECTED-PASS")))))
    "##;
    assert_eq!(run(src), vec!["assert-eq failed"]);
}

// ─── deftest — body can call any :wat::test::* assertion ────────────────

#[test]
fn deftest_body_uses_assert_contains() {
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::test::deftest :my::test::str-check 1024 :error
          (:wat::test::assert-contains "hello world" "world"))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::kernel::RunResult) (:my::test::str-check))
             ((fail :Option<wat::kernel::Failure>)
              (:wat::kernel::RunResult/failure r)))
            (:wat::core::match fail -> :()
              ((Some _) (:wat::io::IOWriter/println stdout "FAIL"))
              (:None    (:wat::io::IOWriter/println stdout "PASS")))))
    "##;
    assert_eq!(run(src), vec!["PASS"]);
}

// ─── deftest — multiple tests coexist ────────────────────────────────────

#[test]
fn deftest_multiple_tests_coexist() {
    let src = r##"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::test::deftest :my::test::alpha 1024 :error
          (:wat::test::assert-eq 1 1))
        (:wat::test::deftest :my::test::beta 1024 :error
          (:wat::test::assert-eq 2 2))
        (:wat::test::deftest :my::test::gamma 1024 :error
          (:wat::test::assert-eq 3 3))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((a :wat::kernel::RunResult) (:my::test::alpha))
             ((b :wat::kernel::RunResult) (:my::test::beta))
             ((g :wat::kernel::RunResult) (:my::test::gamma))
             ((fa :Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure a))
             ((fb :Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure b))
             ((fg :Option<wat::kernel::Failure>) (:wat::kernel::RunResult/failure g))
             ((_ :())
              (:wat::core::match fa -> :()
                ((Some _) (:wat::io::IOWriter/println stdout "alpha:FAIL"))
                (:None    (:wat::io::IOWriter/println stdout "alpha:PASS"))))
             ((_ :())
              (:wat::core::match fb -> :()
                ((Some _) (:wat::io::IOWriter/println stdout "beta:FAIL"))
                (:None    (:wat::io::IOWriter/println stdout "beta:PASS")))))
            (:wat::core::match fg -> :()
              ((Some _) (:wat::io::IOWriter/println stdout "gamma:FAIL"))
              (:None    (:wat::io::IOWriter/println stdout "gamma:PASS")))))
    "##;
    assert_eq!(
        run(src),
        vec!["alpha:PASS", "beta:PASS", "gamma:PASS"]
    );
}

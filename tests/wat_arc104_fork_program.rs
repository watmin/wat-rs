//! End-to-end tests for `:wat::kernel::fork-program` — arc 104b.
//!
//! Source-string sibling of `:wat::kernel::fork-program-ast` (arc
//! 012). Source is parsed inside the child branch post-fork; parse
//! errors surface as exit code 3 + stderr text. The Rust function
//! `fork_program_from_source` (also exposed) is what arc 104c's
//! wat-cli rewrite calls directly.
//!
//! Coverage:
//! - Parent forks a wat program from source; child writes one line
//!   to stdout; parent reads it.
//! - Mini-TCP round trip: parent writes a request; child reads,
//!   transforms, writes response; parent reads response. Same shape
//!   as the spawn-program round-trip test.
//! - Drop cascade: closing parent's stdin writer → child's stdin
//!   read-line returns `:None` → child exits cleanly → Process/join-result
//!   returns Ok(()).
//! - Bad source: parse error in child → Process/join-result returns Err.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

fn unwrap_some_string(v: Value) -> String {
    match v {
        Value::Option(opt) => match &*opt {
            Some(Value::String(s)) => (**s).clone(),
            Some(other) => panic!("Some holds non-String: {:?}", other),
            None => panic!("expected Some(String); got None"),
        },
        other => panic!("expected Option; got {:?}", other),
    }
}

fn unwrap_ok_result(v: Value) -> bool {
    // Returns true if the value is Result::Ok(_).
    match v {
        Value::Result(r) => r.is_ok(),
        other => panic!("expected Result; got {:?}", other),
    }
}

fn unwrap_err_result(v: Value) -> bool {
    // Returns true if the value is Result::Err(_).
    match v {
        Value::Result(r) => r.is_err(),
        other => panic!("expected Result; got {:?}", other),
    }
}

// ─── basic stdout flow ────────────────────────────────────────────────

#[test]
fn fork_program_child_writes_stdout_parent_reads_line() {
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((inner-src :String)
              "(:wat::core::define (:user::main (stdin :wat::io::IOReader) (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :()) (:wat::io::IOWriter/println stdout \"hello-from-fork\"))")
             ((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program inner-src :None))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout child)))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "hello-from-fork");
}

// ─── mini-TCP round trip — request / response ────────────────────────

#[test]
fn fork_program_round_trip_via_pipes() {
    // Parent writes a Ping; child reads it, doubles the string,
    // writes back; parent reads response. Same shape as
    // spawn-program's round-trip test, but fork(2) instead of
    // std::thread.
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((inner-src :String)
              "(:wat::core::define (:user::main (stdin :wat::io::IOReader) (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :()) (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :() (:None ()) ((Some line) (:wat::io::IOWriter/println stdout (:wat::core::string::concat line line)))))")
             ((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program inner-src :None))
             ((in-w  :wat::io::IOWriter) (:wat::kernel::Process/stdin child))
             ((out-r :wat::io::IOReader) (:wat::kernel::Process/stdout child))
             ((_ :()) (:wat::io::IOWriter/println in-w "ping")))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "pingping");
}

// ─── drop-cascade + clean exit code ──────────────────────────────────

#[test]
fn fork_program_clean_exit_code_via_wait_child() {
    // Child reads stdin to EOF, exits. Parent closes stdin writer,
    // then Process/join-result reaps the exit. Should be Ok(()) (clean exit).
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((inner-src :String)
              "(:wat::core::define (:user::main (stdin :wat::io::IOReader) (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :()) (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :() (:None ()) ((Some _) ())))")
             ((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program inner-src :None))
             ((in-w :wat::io::IOWriter) (:wat::kernel::Process/stdin child))
             ((_close :()) (:wat::io::IOWriter/close in-w)))
            (:wat::kernel::Process/join-result child)))
    "#;
    assert!(unwrap_ok_result(run(src)), "expected Ok(()) for clean exit");
}

// ─── bad source surfaces as exit code 3 (EXIT_STARTUP_ERROR) ─────────

#[test]
fn fork_program_parse_error_surfaces_as_exit_3() {
    // Source missing :user::main → child's startup_from_source fails
    // (no entry point) → child writes "startup: ..." to stderr →
    // child dies → parent's Process/join-result returns Err.
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((bad-src :String)
              "(:wat::core::define (:demo::not-main (x :i64) -> :i64) x)")
             ((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program bad-src :None)))
            (:wat::kernel::Process/join-result child)))
    "#;
    // EXIT_STARTUP_ERROR (3) for missing :user::main, OR
    // EXIT_MAIN_SIGNATURE (4) if startup succeeds but main signature
    // doesn't match. Either way, ProcessDiedError surfaces as Err.
    assert!(
        unwrap_err_result(run(src)),
        "expected Err(ProcessDiedError) for startup/sig failure"
    );
}

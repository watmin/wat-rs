//! End-to-end tests for `:wat::kernel::spawn-program` family — arc 103a.
//!
//! The in-thread sibling of `fork-program-ast`. Allocates three
//! `pipe(2)` pairs, spawns a `std::thread` running `invoke_user_main`
//! with the child-side pipe ends, returns a `:wat::kernel::Process`
//! struct holding the parent-side ends + a `ProgramHandle<()>`.
//!
//! Covered:
//! - Child writes one line to stdout; parent reads it (`spawn-program-ast`
//!   variant — forms entry).
//! - Mini-TCP round trip: parent writes a request line; child reads it,
//!   writes a transformed response; parent reads the response.
//! - Drop cascade: parent reads stdout twice — first returns the child's
//!   one line, second returns `:None` because the child exited and its
//!   writer dropped. The pipe-EOF semantics that paragraph 1 of the
//!   DESIGN promised.
//! - `proc.join` returns `:()` after a clean child exit.
//! - Source-string entry (`spawn-program`) works the same way.

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

fn unwrap_none(v: Value) {
    match v {
        Value::Option(opt) => match &*opt {
            None => (),
            Some(other) => panic!("expected None; got Some({:?})", other),
        },
        other => panic!("expected Option; got {:?}", other),
    }
}

fn unwrap_unit(v: Value) {
    match v {
        Value::Unit => (),
        Value::Tuple(items) if items.is_empty() => (),
        other => panic!("expected unit; got {:?}", other),
    }
}

// ─── ast entry — child writes to stdout, parent reads ────────────────────

#[test]
fn spawn_program_ast_child_writes_stdout_parent_reads_line() {
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((proc :wat::kernel::Process)
              (:wat::kernel::spawn-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout "hello-from-thread")))
                :None))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout proc)))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "hello-from-thread");
}

// ─── mini-TCP round trip — request / response ────────────────────────────

#[test]
fn spawn_program_ast_round_trip_via_pipes() {
    // Parent writes one line to child's stdin; child reads it, writes
    // it back doubled to stdout; parent reads the response. The
    // mini-TCP shape: writeln request → blocks on readln response.
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((proc :wat::kernel::Process)
              (:wat::kernel::spawn-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
                      (:None ())
                      ((Some line)
                       (:wat::io::IOWriter/println stdout
                         (:wat::core::string::concat line line))))))
                :None))
             ((in-w  :wat::io::IOWriter) (:wat::kernel::Process/stdin proc))
             ((out-r :wat::io::IOReader) (:wat::kernel::Process/stdout proc))
             ((_ :()) (:wat::io::IOWriter/println in-w "ping")))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "pingping");
}

// ─── drop cascade — second read on stdout sees EOF after child exit ──────

#[test]
fn spawn_program_ast_stdout_eof_after_child_returns() {
    // Child writes one line and returns. After we read that line,
    // a second read returns :None — the child's stdout writer
    // dropped on thread exit; the OS pipe write-end closed; the
    // parent's read-line on the stdout reader sees EOF. The
    // drop-cascade in DESIGN.md, made testable.
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((proc :wat::kernel::Process)
              (:wat::kernel::spawn-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout "only-line")))
                :None))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout proc))
             ((first :Option<String>)
              (:wat::io::IOReader/read-line out-r))
             ((_check :())
              (:wat::core::match first -> :()
                ((Some s) (:wat::core::if (:wat::core::= s "only-line") -> :()
                            ()
                            (:wat::core::panic! "wrong line")))
                (:None (:wat::core::panic! "expected first line")))))
            ;; Second read — child has returned, its writer dropped,
            ;; pipe is empty + closed → :None.
            (:wat::io::IOReader/read-line out-r)))
    "#;
    unwrap_none(run(src));
}

// ─── stderr is its own pipe ──────────────────────────────────────────────

#[test]
fn spawn_program_ast_stderr_is_separate_pipe() {
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((proc :wat::kernel::Process)
              (:wat::kernel::spawn-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stderr "diag-line")))
                :None))
             ((err-r :wat::io::IOReader)
              (:wat::kernel::Process/stderr proc)))
            (:wat::io::IOReader/read-line err-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "diag-line");
}

// ─── join surfaces clean exit ────────────────────────────────────────────

#[test]
fn spawn_program_ast_join_returns_unit_on_clean_exit() {
    // Inner main returns :() cleanly; ProgramHandle<()> joins to :().
    let src = r#"

        (:wat::core::define (:user::main -> :())
          (:wat::core::let*
            (((proc :wat::kernel::Process)
              (:wat::kernel::spawn-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    ()))
                :None))
             ((handle :wat::kernel::ProgramHandle<()>)
              (:wat::kernel::Process/join proc)))
            (:wat::kernel::join handle)))
    "#;
    unwrap_unit(run(src));
}

// ─── source-string entry — same shape, src instead of forms ──────────────

#[test]
fn spawn_program_source_string_entry() {
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((inner-src :String)
              "(:wat::core::define (:user::main (stdin :wat::io::IOReader) (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :()) (:wat::io::IOWriter/println stdout \"from-source\"))")
             ((proc :wat::kernel::Process)
              (:wat::kernel::spawn-program inner-src :None))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout proc)))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "from-source");
}

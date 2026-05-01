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

/// Strip one `:wat::core::Result<T, _>` layer. Used to unwrap the test's
/// `:user::main` return after arc 105a — main returns
/// `:wat::core::Result<X, :wat::kernel::StartupError>` so that
/// `:wat::core::try` can propagate spawn failures cleanly. On Err,
/// the test fails with the captured StartupError message.
fn unwrap_ok(v: Value) -> Value {
    match v {
        Value::Result(r) => match &*r {
            Ok(inner) => inner.clone(),
            Err(e) => panic!("expected Ok; got Err({:?})", e),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

// ─── ast entry — child writes to stdout, parent reads ────────────────────

#[test]
fn spawn_program_ast_child_writes_stdout_parent_reads_line() {
    let src = r#"

        (:wat::core::define (:user::main
                             -> :wat::core::Result<wat::core::Option<wat::core::String>,wat::kernel::StartupError>)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::unit,wat::core::unit>)
              (:wat::core::try
                (:wat::kernel::spawn-program-ast
                  (:wat::test::program
                    (:wat::core::define (:user::main
                                         (stdin  :wat::io::IOReader)
                                         (stdout :wat::io::IOWriter)
                                         (stderr :wat::io::IOWriter)
                                         -> :wat::core::unit)
                      (:wat::io::IOWriter/println stdout "hello-from-thread")))
                  :wat::core::None)))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout proc)))
            (:wat::core::Ok (:wat::io::IOReader/read-line out-r))))
    "#;
    assert_eq!(unwrap_some_string(unwrap_ok(run(src))), "hello-from-thread");
}

// ─── mini-TCP round trip — request / response ────────────────────────────

#[test]
fn spawn_program_ast_round_trip_via_pipes() {
    // Parent writes one line to child's stdin; child reads it, writes
    // it back doubled to stdout; parent reads the response. The
    // mini-TCP shape: writeln request → blocks on readln response.
    let src = r#"

        (:wat::core::define (:user::main
                             -> :wat::core::Result<wat::core::Option<wat::core::String>,wat::kernel::StartupError>)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::unit,wat::core::unit>)
              (:wat::core::try
                (:wat::kernel::spawn-program-ast
                  (:wat::test::program
                    (:wat::core::define (:user::main
                                         (stdin  :wat::io::IOReader)
                                         (stdout :wat::io::IOWriter)
                                         (stderr :wat::io::IOWriter)
                                         -> :wat::core::unit)
                      (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :wat::core::unit
                        (:wat::core::None ())
                        ((:wat::core::Some line)
                         (:wat::io::IOWriter/println stdout
                           (:wat::core::string::concat line line))))))
                  :wat::core::None)))
             ((in-w  :wat::io::IOWriter) (:wat::kernel::Process/stdin proc))
             ((out-r :wat::io::IOReader) (:wat::kernel::Process/stdout proc))
             ((_ :wat::core::unit) (:wat::io::IOWriter/println in-w "ping")))
            (:wat::core::Ok (:wat::io::IOReader/read-line out-r))))
    "#;
    assert_eq!(unwrap_some_string(unwrap_ok(run(src))), "pingping");
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

        (:wat::core::define (:user::main
                             -> :wat::core::Result<wat::core::Option<wat::core::String>,wat::kernel::StartupError>)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::unit,wat::core::unit>)
              (:wat::core::try
                (:wat::kernel::spawn-program-ast
                  (:wat::test::program
                    (:wat::core::define (:user::main
                                         (stdin  :wat::io::IOReader)
                                         (stdout :wat::io::IOWriter)
                                         (stderr :wat::io::IOWriter)
                                         -> :wat::core::unit)
                      (:wat::io::IOWriter/println stdout "only-line")))
                  :wat::core::None)))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout proc))
             ((first :wat::core::Option<wat::core::String>)
              (:wat::io::IOReader/read-line out-r))
             ((_check :wat::core::unit)
              (:wat::core::match first -> :wat::core::unit
                ((:wat::core::Some s) (:wat::core::if (:wat::core::= s "only-line") -> :wat::core::unit
                            ()
                            (:wat::core::panic! "wrong line")))
                (:wat::core::None (:wat::core::panic! "expected first line")))))
            ;; Second read — child has returned, its writer dropped,
            ;; pipe is empty + closed → :None.
            (:wat::core::Ok (:wat::io::IOReader/read-line out-r))))
    "#;
    unwrap_none(unwrap_ok(run(src)));
}

// ─── stderr is its own pipe ──────────────────────────────────────────────

#[test]
fn spawn_program_ast_stderr_is_separate_pipe() {
    let src = r#"

        (:wat::core::define (:user::main
                             -> :wat::core::Result<wat::core::Option<wat::core::String>,wat::kernel::StartupError>)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::unit,wat::core::unit>)
              (:wat::core::try
                (:wat::kernel::spawn-program-ast
                  (:wat::test::program
                    (:wat::core::define (:user::main
                                         (stdin  :wat::io::IOReader)
                                         (stdout :wat::io::IOWriter)
                                         (stderr :wat::io::IOWriter)
                                         -> :wat::core::unit)
                      (:wat::io::IOWriter/println stderr "diag-line")))
                  :wat::core::None)))
             ((err-r :wat::io::IOReader)
              (:wat::kernel::Process/stderr proc)))
            (:wat::core::Ok (:wat::io::IOReader/read-line err-r))))
    "#;
    assert_eq!(unwrap_some_string(unwrap_ok(run(src))), "diag-line");
}

// ─── join surfaces clean exit ────────────────────────────────────────────

#[test]
fn spawn_program_ast_join_returns_unit_on_clean_exit() {
    // Inner main returns :() cleanly; Process/join-result yields
    // Ok(:()) on the success arm. Arc 114: ProgramHandle<R> retired;
    // every clean Process/Thread join returns :wat::core::Result<:wat::core::unit, :wat::core::Vector<...>>.
    let src = r#"

        (:wat::core::define (:user::main
                             -> :wat::core::Result<wat::core::unit,wat::kernel::StartupError>)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::unit,wat::core::unit>)
              (:wat::core::try
                (:wat::kernel::spawn-program-ast
                  (:wat::test::program
                    (:wat::core::define (:user::main
                                         (stdin  :wat::io::IOReader)
                                         (stdout :wat::io::IOWriter)
                                         (stderr :wat::io::IOWriter)
                                         -> :wat::core::unit)
                      ()))
                  :wat::core::None)))
             ((joined :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ProcessDiedError>>)
              (:wat::kernel::Process/join-result proc)))
            (:wat::core::match joined -> :wat::core::Result<wat::core::unit,wat::kernel::StartupError>
              ((:wat::core::Ok _)   (:wat::core::Ok ()))
              ((:wat::core::Err _e) (:wat::core::Ok ())))))
    "#;
    unwrap_unit(unwrap_ok(run(src)));
}

// ─── source-string entry — same shape, src instead of forms ──────────────

#[test]
fn spawn_program_source_string_entry() {
    let src = r#"

        (:wat::core::define (:user::main
                             -> :wat::core::Result<wat::core::Option<wat::core::String>,wat::kernel::StartupError>)
          (:wat::core::let*
            (((inner-src :wat::core::String)
              "(:wat::core::define (:user::main (stdin :wat::io::IOReader) (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :wat::core::unit) (:wat::io::IOWriter/println stdout \"from-source\"))")
             ((proc :wat::kernel::Program<wat::core::unit,wat::core::unit>)
              (:wat::core::try (:wat::kernel::spawn-program inner-src :wat::core::None)))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout proc)))
            (:wat::core::Ok (:wat::io::IOReader/read-line out-r))))
    "#;
    assert_eq!(unwrap_some_string(unwrap_ok(run(src))), "from-source");
}

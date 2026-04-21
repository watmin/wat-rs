//! End-to-end tests for `:wat::kernel::fork-with-forms` — arc 012
//! slice 2 core.
//!
//! Slice 2 core ships the fork primitive + ForkedChild struct +
//! ChildHandle opaque type. It does NOT ship `wait-child` (slice 2
//! adds that in the next task). So these tests exercise:
//!   - Fork runs.
//!   - Parent reads from the child's stdout pipe.
//!   - Parent reads from the child's stderr pipe.
//!   - ForkedChild struct accessors work.
//!   - ChildHandle::Drop reaps zombies (no explicit wait-child).
//!
//! The richer failure-mode matrix (runtime error, panic, startup
//! error, exit-code assertions) waits on the `wait-child` primitive
//! and lands under task #267.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
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

#[test]
fn fork_child_writes_stdout_parent_reads_line() {
    // Parent forks a child whose :user::main writes one line to
    // stdout. Parent reads that line via the ForkedChild/stdout
    // accessor. ChildHandle drops at :user::main exit; Drop reaps.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((child :wat::kernel::ForkedChild)
              (:wat::kernel::fork-with-forms
                (:wat::test::program
                  (:wat::config::set-dims! 1024)
                  (:wat::config::set-capacity-mode! :error)
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout "hello-from-fork")))))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::ForkedChild/stdout child)))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "hello-from-fork");
}

#[test]
fn fork_child_writes_stderr_parent_reads_line() {
    // Same shape, asserting stderr works end-to-end. Matches the
    // contract slice 3's hermetic reimplementation will rely on
    // (stderr carries diagnostic lines when the child's main
    // returns non-zero).
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((child :wat::kernel::ForkedChild)
              (:wat::kernel::fork-with-forms
                (:wat::test::program
                  (:wat::config::set-dims! 1024)
                  (:wat::config::set-capacity-mode! :error)
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stderr "diag-line")))))
             ((err-r :wat::io::IOReader)
              (:wat::kernel::ForkedChild/stderr child)))
            (:wat::io::IOReader/read-line err-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "diag-line");
}

#[test]
fn fork_child_reads_stdin_from_parent() {
    // Parent writes to the child's stdin, child echoes it back via
    // stdout. Exercises the ForkedChild/stdin accessor + parent-
    // to-child data flow.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((child :wat::kernel::ForkedChild)
              (:wat::kernel::fork-with-forms
                (:wat::test::program
                  (:wat::config::set-dims! 1024)
                  (:wat::config::set-capacity-mode! :error)
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
                      ((Some line) (:wat::io::IOWriter/println stdout line))
                      (:None ()))))))
             ((in-w  :wat::io::IOWriter) (:wat::kernel::ForkedChild/stdin child))
             ((out-r :wat::io::IOReader) (:wat::kernel::ForkedChild/stdout child))
             ((_ :i64) (:wat::io::IOWriter/writeln in-w "ping")))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "ping");
}

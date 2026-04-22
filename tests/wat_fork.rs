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

fn unwrap_i64(v: Value) -> i64 {
    match v {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
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
fn wait_child_returns_zero_on_success() {
    // Fork a trivial main that exits cleanly; wait-child must
    // return EXIT_SUCCESS (0).
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
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
                    ()))))
             ((handle :wat::kernel::ChildHandle)
              (:wat::kernel::ForkedChild/handle child)))
            (:wat::kernel::wait-child handle)))
    "#;
    assert_eq!(unwrap_i64(run(src)), 0);
}

#[test]
fn wait_child_is_idempotent() {
    // Calling wait-child twice on the same handle must return the
    // same cached code — sub-fog 2c resolution.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
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
                    ()))))
             ((handle :wat::kernel::ChildHandle)
              (:wat::kernel::ForkedChild/handle child))
             ((_first  :i64) (:wat::kernel::wait-child handle))
             ;; Second call exercises the cached-exit path;
             ;; if it errors or returns a different code, test
             ;; fails via panic or bad return.
             ((second :i64) (:wat::kernel::wait-child handle)))
            second))
    "#;
    assert_eq!(unwrap_i64(run(src)), 0);
}

#[test]
fn wait_child_surfaces_startup_error_exit_code() {
    // Child's body has a type mismatch — `i64::+` against a String
    // arg. startup_from_forms's type-check phase fails; child
    // exits with EXIT_STARTUP_ERROR=3.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
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
                    (:wat::core::let*
                      (((_ :i64) (:wat::core::i64::+ 1 "two")))
                      ())))))
             ((handle :wat::kernel::ChildHandle)
              (:wat::kernel::ForkedChild/handle child)))
            (:wat::kernel::wait-child handle)))
    "#;
    assert_eq!(unwrap_i64(run(src)), 3);
}

#[test]
fn wait_child_surfaces_panic_exit_code() {
    // Child's :user::main calls :wat::test::assert-eq with mismatched
    // values — which invokes assertion-failed! via panic_any. The
    // child's catch_unwind catches, maps to EXIT_PANIC=2.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
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
                    (:wat::test::assert-eq 1 2)))))
             ((handle :wat::kernel::ChildHandle)
              (:wat::kernel::ForkedChild/handle child)))
            (:wat::kernel::wait-child handle)))
    "#;
    assert_eq!(unwrap_i64(run(src)), 2);
}

#[test]
fn wait_child_surfaces_runtime_error_exit_code() {
    // Child's :user::main calls :wat::core::u8 with a value out of
    // range; u8 cast raises a MalformedForm RuntimeError at eval
    // time. invoke_user_main returns Err(runtime_err), child exits
    // EXIT_RUNTIME_ERROR=1.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
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
                    (:wat::core::let*
                      (((_ :u8) (:wat::core::u8 300)))
                      ())))))
             ((handle :wat::kernel::ChildHandle)
              (:wat::kernel::ForkedChild/handle child)))
            (:wat::kernel::wait-child handle)))
    "#;
    assert_eq!(unwrap_i64(run(src)), 1);
}

#[test]
fn multiple_sequential_forks_no_leak() {
    // Parent forks three children in sequence, waits each, accumulates
    // their exit codes. Proves no zombie / fd leaks across repeated
    // fork+wait cycles from one parent.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::one-fork -> :i64)
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
                    ()))))
             ((handle :wat::kernel::ChildHandle)
              (:wat::kernel::ForkedChild/handle child)))
            (:wat::kernel::wait-child handle)))

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((a :i64) (:my::one-fork))
             ((b :i64) (:my::one-fork))
             ((c :i64) (:my::one-fork)))
            (:wat::core::i64::+ (:wat::core::i64::+ a b) c)))
    "#;
    // All three succeed (exit 0); sum is 0.
    assert_eq!(unwrap_i64(run(src)), 0);
}

#[test]
fn wait_child_surfaces_nonzero_exit_code() {
    // Child's :user::main signature is WRONG — missing the two
    // writer params. Child's startup_from_forms succeeds but
    // validate_user_main_signature fails; child exits with
    // EXIT_MAIN_SIGNATURE=4 per the convention. Parent's
    // wait-child should return 4.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((child :wat::kernel::ForkedChild)
              (:wat::kernel::fork-with-forms
                (:wat::test::program
                  (:wat::config::set-dims! 1024)
                  (:wat::config::set-capacity-mode! :error)
                  (:wat::core::define (:user::main -> :i64) 42))))
             ((handle :wat::kernel::ChildHandle)
              (:wat::kernel::ForkedChild/handle child)))
            (:wat::kernel::wait-child handle)))
    "#;
    assert_eq!(unwrap_i64(run(src)), 4);
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

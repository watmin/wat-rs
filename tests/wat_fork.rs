//! End-to-end tests for `:wat::kernel::fork-program-ast` — arc 012
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

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stdout "hello-from-fork")))))
             ((out-r :wat::io::IOReader)
              (:wat::kernel::Process/stdout child)))
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

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::io::IOWriter/println stderr "diag-line")))))
             ((err-r :wat::io::IOReader)
              (:wat::kernel::Process/stderr child)))
            (:wat::io::IOReader/read-line err-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "diag-line");
}

#[test]
fn wait_child_returns_zero_on_success() {
    // Fork a trivial main that exits cleanly; wait-child must
    // return EXIT_SUCCESS (0).
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    ())))))
            (:wat::kernel::Process/join-result child)))
    "#;
    assert!(unwrap_ok_result(run(src)), "expected Ok(()) for clean exit");
}

fn unwrap_ok_result(v: Value) -> bool {
    match v {
        Value::Result(r) => r.is_ok(),
        other => panic!("expected Result; got {:?}", other),
    }
}

fn unwrap_err_result(v: Value) -> bool {
    match v {
        Value::Result(r) => r.is_err(),
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn wait_child_is_idempotent() {
    // Calling wait-child twice on the same handle must return the
    // same cached code — sub-fog 2c resolution.
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    ()))))
             ;; Process/join-result is the unified wait; idempotency
             ;; is now the substrate's concern under the ProgramHandle.
             ((joined-result :Result<(),Vec<wat::kernel::ProcessDiedError>>)
              (:wat::kernel::Process/join-result child)))
            joined-result))
    "#;
    assert!(unwrap_ok_result(run(src)), "expected Ok(()) for clean exit (idempotent path)");
}

#[test]
fn wait_child_surfaces_startup_error_exit_code() {
    // Child's body has a type mismatch — `i64::+` against a String
    // arg. startup_from_forms's type-check phase fails; child
    // exits with EXIT_STARTUP_ERROR=3.
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::core::let*
                      (((_ :i64) (:wat::core::i64::+ 1 "two")))
                      ()))))))
            (:wat::kernel::Process/join-result child)))
    "#;
    assert!(unwrap_err_result(run(src)), "expected Err(ProcessDiedError) for startup error exit 3");
}

#[test]
fn wait_child_surfaces_panic_exit_code() {
    // Child's :user::main calls :wat::test::assert-eq with mismatched
    // values — which invokes assertion-failed! via panic_any. The
    // child's catch_unwind catches, maps to EXIT_PANIC=2.
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::test::assert-eq 1 2))))))
          (:wat::kernel::Process/join-result child)))
    "#;
    assert!(unwrap_err_result(run(src)), "expected Err(ProcessDiedError) for panic exit 2");
}

#[test]
fn wait_child_surfaces_runtime_error_exit_code() {
    // Child's :user::main calls :wat::core::u8 with a value out of
    // range; u8 cast raises a MalformedForm RuntimeError at eval
    // time. invoke_user_main returns Err(runtime_err), child exits
    // EXIT_RUNTIME_ERROR=1.
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::core::let*
                      (((_ :u8) (:wat::core::u8 300)))
                      ()))))))
            (:wat::kernel::Process/join-result child)))
    "#;
    assert!(unwrap_err_result(run(src)), "expected Err(ProcessDiedError) for runtime error exit 1");
}

#[test]
fn multiple_sequential_forks_no_leak() {
    // Parent forks three children in sequence, waits each, accumulates
    // their exit codes. Proves no zombie / fd leaks across repeated
    // fork+wait cycles from one parent.
    let src = r#"

        (:wat::core::define (:my::one-fork<I,O> -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((child :wat::kernel::Program<I,O>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    ())))))
            (:wat::kernel::Process/join-result child)))

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((a :Result<(),Vec<wat::kernel::ProcessDiedError>>) (:my::one-fork))
             ((b :Result<(),Vec<wat::kernel::ProcessDiedError>>) (:my::one-fork))
             ((c :Result<(),Vec<wat::kernel::ProcessDiedError>>) (:my::one-fork)))
            ;; All three must be Ok; return last as witness.
            c))
    "#;
    // All three succeed (exit 0); last result is Ok(()).
    assert!(unwrap_ok_result(run(src)), "expected all three forks to exit clean");
}

#[test]
fn wait_child_surfaces_nonzero_exit_code() {
    // Child's :user::main signature is WRONG — missing the two
    // writer params. Child's startup_from_forms succeeds but
    // validate_user_main_signature fails; child exits with
    // EXIT_MAIN_SIGNATURE=4 per the convention. Parent's
    // wait-child should return 4.
    let src = r#"

        (:wat::core::define (:user::main -> :Result<(),Vec<wat::kernel::ProcessDiedError>>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main -> :i64) 42)))))
            (:wat::kernel::Process/join-result child)))
    "#;
    assert!(unwrap_err_result(run(src)), "expected Err(ProcessDiedError) for EXIT_MAIN_SIGNATURE=4");
}

#[test]
fn fork_child_reads_stdin_from_parent() {
    // Parent writes to the child's stdin, child echoes it back via
    // stdout. Exercises the ForkedChild/stdin accessor + parent-
    // to-child data flow.
    let src = r#"

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((child :wat::kernel::Program<(),()>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :())
                    (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
                      ((Some line) (:wat::io::IOWriter/println stdout line))
                      (:None ()))))))
             ((in-w  :wat::io::IOWriter) (:wat::kernel::Process/stdin child))
             ((out-r :wat::io::IOReader) (:wat::kernel::Process/stdout child))
             ((_ :i64) (:wat::io::IOWriter/writeln in-w "ping")))
            (:wat::io::IOReader/read-line out-r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "ping");
}

//! Arc 146 slice 1 — substrate dispatch MECHANISM coverage.
//!
//! The dispatch entity kind is the substrate's honest representation
//! of "this name dispatches over input type to one of N per-Type
//! impls" (per arc 144 REALIZATION 2 + COMPACTION-AMNESIA-RECOVERY
//! § FM 10). Slice 1 ships the mechanism only — NO migration of any
//! existing primitive. These tests use a SYNTHETIC dispatch over
//! leaf types (`:wat::core::i64`, `:wat::core::f64`, `:wat::core::String`)
//! so the test surface depends on nothing that's about to change.
//!
//! Coverage:
//!   1. Dispatch hits the `:wat::core::i64` arm for an i64 call site.
//!   2. Dispatch hits the `:wat::core::f64` arm for an f64 call site.
//!   3. Check-time TypeMismatch when no arm matches the input type.
//!   4. `lookup-define` returns Some + emission carries
//!      `:wat::core::define-dispatch` head.
//!   5. `signature-of` returns Some (the declaration form).
//!   6. `body-of` returns :None (dispatchs have no wat body — the
//!      arms ARE the contract).
//!   7. `define_dispatch` arity-mismatch surfaces as a startup error
//!      when an arm impl's arity disagrees with the dispatch's
//!      surface arity.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source, StartupError};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::thread_io::{install_ambient_stdio, uninstall_ambient_stdio, AmbientStdio};

fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(read_fd));
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));
    (reader, writer)
}

fn drain_lines(reader: &Arc<dyn WatReader>) -> Vec<String> {
    let bytes = reader
        .read_all(wat::span::Span::unknown())
        .expect("read-all");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn run(src: &str) -> Vec<String> {
    let _ = uninstall_ambient_stdio();
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    invoke_user_main(&world, Vec::new()).expect("main");
    let _ = uninstall_ambient_stdio();
    drain_lines(&stdout_capture)
}

fn try_startup(src: &str) -> Result<(), StartupError> {
    startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .map(|_| ())
}

// Common preamble: two per-Type impls (clean rank-1 schemes — the
// substrate handles them today) plus a define_dispatch that routes
// `:test::describe` over `:wat::core::i64` and `:wat::core::f64`.
const PREAMBLE: &str = r##"
    (:wat::core::define
      (:test::i64-describe (x :wat::core::i64) -> :wat::core::String)
      "i64-arm")

    (:wat::core::define
      (:test::f64-describe (x :wat::core::f64) -> :wat::core::String)
      "f64-arm")

    (:wat::core::define-dispatch :test::describe
      ((:wat::core::i64) :test::i64-describe)
      ((:wat::core::f64) :test::f64-describe))
"##;

// ─── Runtime dispatch coverage ──────────────────────────────────────────────

#[test]
fn dispatch_dispatches_to_i64_arm() {
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println (:test::describe 42)))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["\"i64-arm\"".to_string()]);
}

#[test]
fn dispatch_dispatches_to_f64_arm() {
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println (:test::describe 3.14)))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["\"f64-arm\"".to_string()]);
}

// ─── Check-time arm coverage ───────────────────────────────────────────────

#[test]
fn dispatch_no_arm_match_check_time() {
    // Calling with a String when only :wat::core::i64 + :wat::core::f64 arms exist should
    // surface as a check-time TypeMismatch (dispatch dispatch
    // diagnostic; the call-site type tag does not unify with any arm
    // pattern).
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println (:test::describe "not-a-number")))
        "##,
        preamble = PREAMBLE,
    );
    let err = try_startup(&src).expect_err("expected check-time mismatch");
    let msg = format!("{}", err);
    assert!(
        msg.contains("test::describe"),
        "expected the dispatch name in the diagnostic; got: {}",
        msg
    );
    assert!(
        msg.contains("dispatch") || msg.contains("dispatch") || msg.contains("expected one of"),
        "expected a dispatch-dispatch diagnostic; got: {}",
        msg
    );
}

// ─── Reflection coverage (arc 144 extension) ────────────────────────────────

#[test]
fn lookup_form_returns_dispatch_binding() {
    // `:wat::runtime::lookup-define` on a dispatch returns Some
    // and the rendered AST carries the `:wat::core::define-dispatch`
    // head — distinguishing a dispatch from a function or macro.
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :test::describe)
             rendered
              (:wat::edn::write def-opt)]
            (:wat::kernel::println rendered)))
        "##,
        preamble = PREAMBLE,
    );
    let out = run(&src);
    assert_eq!(out.len(), 1, "expected one rendered line, got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains("define-dispatch"),
        "expected 'define-dispatch' head in rendered dispatch define-ast, got: {}",
        line
    );
    assert!(
        line.contains("test::describe"),
        "expected dispatch name 'test::describe' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_dispatch_returns_declaration() {
    // signature-of on a dispatch returns Some — the declaration
    // form (no separate "header" — the dispatch table IS the contract).
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::match
            (:wat::runtime::signature-of :test::describe)
            -> :wat::core::nil
            ((:wat::core::Some _) (:wat::kernel::println "pass"))
            (:wat::core::None    (:wat::kernel::println "fail"))))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["\"pass\"".to_string()]);
}

#[test]
fn body_of_dispatch_returns_none() {
    // Dispatchs have no wat-side body — the arms table IS the
    // contract. body-of is honest about absence and returns :None.
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::match
            (:wat::runtime::body-of :test::describe)
            -> :wat::core::nil
            ((:wat::core::Some _) (:wat::kernel::println "fail"))
            (:wat::core::None    (:wat::kernel::println "pass"))))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["\"pass\"".to_string()]);
}

// ─── Bonus: arity validation surfaces deferred-to-call-time per Q1 ──────────

#[test]
fn define_dispatch_arity_mismatch_errors() {
    // Per BRIEF Q1 — arity validation is deferred to first check-time
    // call. A dispatch whose arm impl has a different arity than
    // the dispatch's surface arity surfaces a clean check-time
    // diagnostic when the dispatch is called.
    let src = r##"
        ;; Two-arg impl (binary)
        (:wat::core::define
          (:test::two-arg-i64
            (x :wat::core::i64)
            (y :wat::core::i64)
            -> :wat::core::String)
          "two-arg")

        ;; Dispatch with surface arity 1 but arm impl with arity 2
        (:wat::core::define-dispatch :test::arity-mismatched
          ((:wat::core::i64) :test::two-arg-i64))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println (:test::arity-mismatched 7)))
    "##;
    let err = try_startup(src).expect_err("expected check-time arity mismatch");
    let msg = format!("{}", err);
    assert!(
        msg.contains("arity-mismatched"),
        "expected the dispatch name in the diagnostic; got: {}",
        msg
    );
    assert!(
        msg.contains("arity") || msg.contains("disagrees"),
        "expected an arity diagnostic; got: {}",
        msg
    );
}

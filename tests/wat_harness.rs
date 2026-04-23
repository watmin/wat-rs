//! Integration coverage for `wat::Harness` (arc 007 slice 5).
//!
//! The Harness is a thin wrapper — its value is ergonomic. Tests
//! verify it threads through the full startup + invocation pipeline
//! correctly and that its error surface discriminates each failure
//! class cleanly.

use wat::harness::{Harness, HarnessError};

const DIMS_AND_MODE: &str = r##"
    (:wat::config::set-capacity-mode! :error)
    (:wat::config::set-dims! 1024)
"##;

fn main_body(body: &str) -> String {
    format!(
        r##"{}
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          {})
        "##,
        DIMS_AND_MODE, body
    )
}

// ─── happy path — stdout capture ────────────────────────────────────────

#[test]
fn harness_captures_stdout() {
    let src = main_body(r#"(:wat::io::IOWriter/println stdout "hello from wat")"#);
    let h = Harness::from_source(&src).expect("freeze");
    let out = h.run(&[]).expect("run");
    assert_eq!(out.stdout, vec!["hello from wat".to_string()]);
    assert!(out.stderr.is_empty());
}

// ─── happy path — stdin injection ───────────────────────────────────────

#[test]
fn harness_injects_stdin_lines() {
    // Program echoes every stdin line to stdout, line by line, until EOF.
    // Top-level `:echo-loop` + main that calls it — define is a
    // top-level form, not an expression.
    let src = format!(
        r##"{}
        (:wat::core::define (:echo-loop (r :wat::io::IOReader) (w :wat::io::IOWriter) -> :())
          (:wat::core::match (:wat::io::IOReader/read-line r) -> :()
            ((Some line)
              (:wat::core::let*
                (((_ :()) (:wat::io::IOWriter/println w line)))
                (:echo-loop r w)))
            (:None ())))
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:echo-loop stdin stdout))
        "##,
        DIMS_AND_MODE
    );
    let h = Harness::from_source(&src).expect("freeze");
    let out = h.run(&["alpha", "beta", "gamma"]).expect("run");
    assert_eq!(
        out.stdout,
        vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
    );
}

// ─── happy path — freeze once, run many ─────────────────────────────────

#[test]
fn harness_freeze_once_run_many() {
    let src = main_body(r##"(:wat::io::IOWriter/println stdout "tick")"##);
    let h = Harness::from_source(&src).expect("freeze");
    for _ in 0..3 {
        let out = h.run(&[]).expect("run");
        assert_eq!(out.stdout, vec!["tick".to_string()]);
    }
}

// ─── startup error surfaces through HarnessError::Startup ───────────────

#[test]
fn harness_startup_error_surfaces() {
    let bad = "(this-is-not-a-valid-form))";
    let err = Harness::from_source(bad).expect_err("bad source must fail");
    assert!(matches!(err, HarnessError::Startup(_)), "got {:?}", err);
}

// ─── :user::main signature mismatch surfaces separately ────────────────

#[test]
fn harness_main_signature_mismatch() {
    // :user::main returns i64 instead of :(); signature validator refuses.
    let src = format!(
        r##"{}
        (:wat::core::define (:user::main -> :i64) 42)
        "##,
        DIMS_AND_MODE
    );
    let err = Harness::from_source(&src).expect_err("sig mismatch must fail");
    assert!(matches!(err, HarnessError::MainSignature(_)), "got {:?}", err);
}

// ─── stderr capture ─────────────────────────────────────────────────────

#[test]
fn harness_captures_stderr() {
    let src = main_body(
        r##"(:wat::core::let*
              (((_ :()) (:wat::io::IOWriter/println stdout "out-line"))
               ((_ :()) (:wat::io::IOWriter/println stderr "err-line")))
              ())"##,
    );
    let h = Harness::from_source(&src).expect("freeze");
    let out = h.run(&[]).expect("run");
    assert_eq!(out.stdout, vec!["out-line".to_string()]);
    assert_eq!(out.stderr, vec!["err-line".to_string()]);
}

// ─── world() accessor for advanced callers ──────────────────────────────

#[test]
fn harness_world_accessor_exposes_frozen_world() {
    let src = main_body(r##"()"##);
    let h = Harness::from_source(&src).expect("freeze");
    // Should have :user::main registered; function lookup must succeed.
    let world = h.world();
    assert!(world.symbols().get(":user::main").is_some());
}

//! Arc 146 slice 1 — substrate multimethod MECHANISM coverage.
//!
//! The multimethod entity kind is the substrate's honest representation
//! of "this name dispatches over input type to one of N per-Type
//! impls" (per arc 144 REALIZATION 2 + COMPACTION-AMNESIA-RECOVERY
//! § FM 10). Slice 1 ships the mechanism only — NO migration of any
//! existing primitive. These tests use a SYNTHETIC multimethod over
//! leaf types (`:wat::core::i64`, `:wat::core::f64`, `:wat::core::String`)
//! so the test surface depends on nothing that's about to change.
//!
//! Coverage:
//!   1. Dispatch hits the `:i64` arm for an i64 call site.
//!   2. Dispatch hits the `:f64` arm for an f64 call site.
//!   3. Check-time TypeMismatch when no arm matches the input type.
//!   4. `lookup-define` returns Some + emission carries
//!      `:wat::core::defmultimethod` head.
//!   5. `signature-of` returns Some (the declaration form).
//!   6. `body-of` returns :None (multimethods have no wat body — the
//!      arms ARE the contract).
//!   7. `defmultimethod` arity-mismatch surfaces as a startup error
//!      when an arm impl's arity disagrees with the multimethod's
//!      surface arity.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source, StartupError};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
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
    let bytes = stdout.snapshot_bytes().expect("snapshot");
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

fn try_startup(src: &str) -> Result<(), StartupError> {
    startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .map(|_| ())
}

// Common preamble: two per-Type impls (clean rank-1 schemes — the
// substrate handles them today) plus a defmultimethod that routes
// `:test::describe` over `:wat::core::i64` and `:wat::core::f64`.
const PREAMBLE: &str = r##"
    (:wat::core::define
      (:test::i64-describe (x :wat::core::i64) -> :wat::core::String)
      "i64-arm")

    (:wat::core::define
      (:test::f64-describe (x :wat::core::f64) -> :wat::core::String)
      "f64-arm")

    (:wat::core::defmultimethod :test::describe
      ((:wat::core::i64) :test::i64-describe)
      ((:wat::core::f64) :test::f64-describe))
"##;

// ─── Runtime dispatch coverage ──────────────────────────────────────────────

#[test]
fn multimethod_dispatches_to_i64_arm() {
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::io::IOWriter/println stdout (:test::describe 42)))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["i64-arm".to_string()]);
}

#[test]
fn multimethod_dispatches_to_f64_arm() {
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::io::IOWriter/println stdout (:test::describe 3.14)))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["f64-arm".to_string()]);
}

// ─── Check-time arm coverage ───────────────────────────────────────────────

#[test]
fn multimethod_no_arm_match_check_time() {
    // Calling with a String when only :i64 + :f64 arms exist should
    // surface as a check-time TypeMismatch (multimethod dispatch
    // diagnostic; the call-site type tag does not unify with any arm
    // pattern).
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::io::IOWriter/println stdout (:test::describe "not-a-number")))
        "##,
        preamble = PREAMBLE,
    );
    let err = try_startup(&src).expect_err("expected check-time mismatch");
    let msg = format!("{}", err);
    assert!(
        msg.contains("test::describe"),
        "expected the multimethod name in the diagnostic; got: {}",
        msg
    );
    assert!(
        msg.contains("multimethod") || msg.contains("dispatch") || msg.contains("expected one of"),
        "expected a multimethod-dispatch diagnostic; got: {}",
        msg
    );
}

// ─── Reflection coverage (arc 144 extension) ────────────────────────────────

#[test]
fn lookup_form_returns_multimethod_binding() {
    // `:wat::runtime::lookup-define` on a multimethod returns Some
    // and the rendered AST carries the `:wat::core::defmultimethod`
    // head — distinguishing a multimethod from a function or macro.
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((def-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::lookup-define :test::describe))
             ((rendered :wat::core::String)
              (:wat::edn::write def-opt)))
            (:wat::io::IOWriter/println stdout rendered)))
        "##,
        preamble = PREAMBLE,
    );
    let out = run(&src);
    assert_eq!(out.len(), 1, "expected one rendered line, got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains("defmultimethod"),
        "expected 'defmultimethod' head in rendered multimethod define-ast, got: {}",
        line
    );
    assert!(
        line.contains("test::describe"),
        "expected multimethod name 'test::describe' in rendered AST, got: {}",
        line
    );
}

#[test]
fn signature_of_multimethod_returns_declaration() {
    // signature-of on a multimethod returns Some — the declaration
    // form (no separate "header" — the dispatch table IS the contract).
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::signature-of :test::describe)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["pass".to_string()]);
}

#[test]
fn body_of_multimethod_returns_none() {
    // Multimethods have no wat-side body — the arms table IS the
    // contract. body-of is honest about absence and returns :None.
    let src = format!(
        r##"
        {preamble}

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::body-of :test::describe)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "pass"))))
        "##,
        preamble = PREAMBLE,
    );
    assert_eq!(run(&src), vec!["pass".to_string()]);
}

// ─── Bonus: arity validation surfaces deferred-to-call-time per Q1 ──────────

#[test]
fn defmultimethod_arity_mismatch_errors() {
    // Per BRIEF Q1 — arity validation is deferred to first check-time
    // call. A multimethod whose arm impl has a different arity than
    // the multimethod's surface arity surfaces a clean check-time
    // diagnostic when the multimethod is called.
    let src = r##"
        ;; Two-arg impl (binary)
        (:wat::core::define
          (:test::two-arg-i64
            (x :wat::core::i64)
            (y :wat::core::i64)
            -> :wat::core::String)
          "two-arg")

        ;; Multimethod with surface arity 1 but arm impl with arity 2
        (:wat::core::defmultimethod :test::arity-mismatched
          ((:wat::core::i64) :test::two-arg-i64))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::io::IOWriter/println stdout (:test::arity-mismatched 7)))
    "##;
    let err = try_startup(src).expect_err("expected check-time arity mismatch");
    let msg = format!("{}", err);
    assert!(
        msg.contains("arity-mismatched"),
        "expected the multimethod name in the diagnostic; got: {}",
        msg
    );
    assert!(
        msg.contains("arity") || msg.contains("disagrees"),
        "expected an arity diagnostic; got: {}",
        msg
    );
}

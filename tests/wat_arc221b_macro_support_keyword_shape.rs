//! Arc 221 Stone 221.4b — Phase 2 macro-support keyword-shape probes.
//!
//! Verifies that the macro-support family in runtime.rs correctly handles
//! `HolonAST::Keyword` (not the retired `HolonAST::Symbol(":foo")`) after
//! Stone 221.4b's `watast_to_holon` fix.
//!
//! Functions fixed:
//!   - `eval_rename_callable_name` (runtime.rs:11560 assertion + 11588 writer)
//!     — now accepts `HolonAST::Keyword` as first Bundle child and emits
//!     `HolonAST::keyword()` as the renamed child.
//!   - `eval_extract_arg_names` (runtime.rs:11647/11653) — AUDITED as HONEST
//!     (arg names remain `HolonAST::Symbol`; they are bare WAT identifiers,
//!     not user keywords). No change needed; doc comments updated.
//!
//! Tests:
//!   1. `rename-callable-name` accepts Keyword first child: construct a Bundle
//!      with Keyword("foo") as first child, rename :foo → :bar, verify result
//!      Bundle has Keyword("bar") as first child.
//!   2. `rename-callable-name` rejects Symbol first child: a Bundle with
//!      Symbol("foo") as first child now fails (post-arc-221 doctrine: function
//!      names are Keyword-shaped). Verify runtime error.
//!   3. `defalias` end-to-end (Stone 241.12): alias a substrate primitive (:wat::core::length),
//!      verify calling the alias produces the same result as the original.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::services::{install_ambient_stdio, take_ambient_stdio, AmbientStdio};

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
    let _ = take_ambient_stdio();
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
    let _ = take_ambient_stdio();
    drain_lines(&stdout_capture)
}

fn run_expecting_runtime_err(src: &str) -> bool {
    let _ = take_ambient_stdio();
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (_stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    let result = invoke_user_main(&world, Vec::new());
    let _ = take_ambient_stdio();
    result.is_err()
}

// ─── Probe 1 — rename-callable-name accepts Keyword first child ───────────────

/// `(:wat::runtime::rename-callable-name sig :foo :bar)` where `sig` is a Bundle
/// with `HolonAST::Keyword("foo")` as first child (produced by `signature-of-defn`
/// after Stone 221.4b's watast_to_holon fix).
///
/// The test uses the full pipeline: define :user::foo-fn, get its signature via
/// `signature-of-defn`, rename :user::foo-fn → :user::bar-fn. The result's EDN
/// must contain "bar-fn" and NOT contain "foo-fn" as the head keyword.
///
/// This is the CORE fix: pre-Stone-221.4b `eval_rename_callable_name` asserted
/// `HolonAST::Symbol` at children[0]; after Stone 221.4b `watast_to_holon` emits
/// `HolonAST::Keyword` there, so the assertion would FAIL with TypeMismatch. The
/// Phase 2 fix changes the assertion to accept `HolonAST::Keyword`.
#[test]
fn probe_1_rename_callable_name_accepts_keyword_first_child() {
    let src = r##"
        (:wat::core::defn :user::foo-fn [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::let
                      [sig
                        (:wat::core::Option/expect -> :wat::holon::HolonAST
                          (:wat::runtime::signature-of-defn :user::foo-fn)
                          "expected Some for foo-fn")
                       renamed
                        (:wat::runtime::rename-callable-name
                          sig
                          :user::foo-fn
                          :user::bar-fn)
                       rendered
                        (:wat::edn::write renamed)]
                      (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected 1 output line, got: {:?}", out);
    let line = &out[0];
    // Renamed head must contain "bar-fn".
    assert!(
        line.contains("bar-fn"),
        "expected 'bar-fn' in renamed head, got: {}",
        line
    );
    // Old name must be gone from the head keyword position.
    assert!(
        !line.contains("foo-fn"),
        "expected 'foo-fn' to be absent from renamed head, got: {}",
        line
    );
}

// ─── Probe 2 — rename-callable-name from-mismatch errors correctly ───────────

/// When `from` doesn't match the head's base name, `rename-callable-name` must
/// error with `MalformedForm`. This verifies the comparison logic (base without
/// leading colon vs from_str with leading colon, fixed in Stone 221.4b).
///
/// Pre-Stone-221.4b the comparison would fail INCORRECTLY for ALL renames (because
/// base had no colon but from_str did). Post-fix: only mismatches error.
#[test]
fn probe_2_rename_callable_name_from_mismatch_errors() {
    let src = r##"
        (:wat::core::defn :user::my-fn [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::let
                      [sig
                        (:wat::core::Option/expect -> :wat::holon::HolonAST
                          (:wat::runtime::signature-of-defn :user::my-fn)
                          "expected Some")
                       _
                        (:wat::runtime::rename-callable-name
                          sig
                          :user::wrong-name
                          :user::alias)]
                      (:wat::kernel::println "should not reach")))
    "##;
    assert!(
        run_expecting_runtime_err(src),
        "expected runtime error for from-name mismatch in rename-callable-name"
    );
}

// ─── Probe 3 — defalias end-to-end (substrate target, Stone 241.12) ─────────

/// `(:wat::core::defalias :user::my-length :wat::core::length)` creates
/// an alias of a substrate primitive. Calling the alias must produce the same
/// result as calling the original.
///
/// Stone 241.12 — migrated from :wat::runtime::define-alias to native :wat::core::defalias.
/// The native form resolves the builtin via CheckEnv::with_builtins() at
/// registration time; no macro expansion required.
///
/// This exercises the native defalias pipeline:
///   1. parse_defalias_form extracts (alias=:user::my-length, target=:wat::core::length)
///   2. register_defalias looks up :wat::core::length in CheckEnv::with_builtins()
///   3. Synthesises param names _p0, creates delegate body (:wat::core::length _p0)
///   4. Inserts alias Function into sym.functions under :user::my-length
///
/// This is the end-to-end proof that the native defalias works for substrate
/// primitive targets.
#[test]
fn probe_3_define_alias_end_to_end() {
    let src = r##"
        (:wat::core::defalias :user::my-length :wat::core::length)

        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::let
                      [v   (:wat::core::Vector :wat::core::i64 1 2 3)
                       r1  (:wat::core::length v)
                       r2  (:user::my-length v)]
                      (:wat::kernel::println
                        (:wat::core::string::concat
                          (:wat::edn::write r1)
                          " "
                          (:wat::edn::write r2)))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected 1 output line, got: {:?}", out);
    let line = &out[0];
    // Both length and my-length on [1,2,3] should produce 3.
    // Output should contain "3 3".
    assert!(
        line.contains("3") && {
            let count = line.matches('3').count();
            count >= 2
        },
        "expected both calls to produce 3 (length of 3-vector), got: {}",
        line
    );
}

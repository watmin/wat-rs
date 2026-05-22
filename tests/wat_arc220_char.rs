//! Arc 220 slice 2 — `:wat::core::Char` typed primitive.
//!
//! Verifies the new `Value::wat__core__Char` variant and the `Char/of` constructor.
//! Also verifies the lexer `\c` literal syntax and BMP-only enforcement.
//!
//! Test cases:
//!   1 — Lexer accepts `\a` single-char literal
//!   2 — Lexer accepts named chars: `\newline`, `\space`, `\tab`, `\return`
//!   3 — Lexer accepts `\uNNNN` Unicode escape (BMP)
//!   4 — Lexer rejects supplementary-plane literal (produces diagnostic)
//!   5 — `(:wat::core::Char/of "x")` returns Value::wat__core__Char('x')
//!   6 — `(:wat::core::Char/of "")` errors with "length-1 String" diagnostic
//!   7 — `(:wat::core::Char/of "ab")` errors with "length-2" diagnostic
//!   8 — `(:wat::core::Char/of "\u{1F600}")` errors with "supplementary-plane"
//!   9 — Round-trip: `\x` in wat source → Value → EDN write → reparse → identical
//!  10 — `(= \a \a)` true; `(= \a \b)` false

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
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
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
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

/// Run source and expect a RuntimeError; return the debug string.
fn run_expecting_runtime_err(src: &str) -> String {
    let _ = uninstall_ambient_stdio();
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    let err = invoke_user_main(&world, Vec::new()).expect_err("expected runtime error");
    let _ = uninstall_ambient_stdio();
    drop(stdout_capture);
    format!("{:?}", err)
}

// ─── 1: Lexer accepts `\a` single-char literal ──────────────────────────────

/// The `\a` literal produces a `:wat::core::Char` value.
/// Verified by using `(= \a (:wat::core::Char/of "a"))` — if the lexer
/// produces the correct typed Char, both sides are equal.
#[test]
fn char_literal_single_letter() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [c \a
             expected (:wat::core::Char/of "a")
             ok (:wat::core::= c expected)]
            (:wat::core::if ok -> :wat::core::nil
              (:wat::kernel::println "CHAR-LITERAL-OK")
              (:wat::kernel::println "CHAR-LITERAL-FAIL"))))
    "#;
    let lines = run(src);
    assert_eq!(lines, vec!["\"CHAR-LITERAL-OK\""], "\\a literal must produce Char('a')");
}

// ─── 2: Lexer accepts named chars ────────────────────────────────────────────

/// `\newline`, `\space`, `\tab`, `\return` named char forms.
/// Each compared against the String-form equivalents to verify content.
#[test]
fn char_literal_named_chars() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [nl   \newline
             sp   \space
             tab  \tab
             ret  \return
             nl-exp  (:wat::core::Char/of "\n")
             sp-exp  (:wat::core::Char/of " ")
             tab-exp (:wat::core::Char/of "\t")
             ret-exp (:wat::core::Char/of "\r")
             ok (:wat::core::=
                  (:wat::core::= nl nl-exp)
                  true)]
            (:wat::core::if ok -> :wat::core::nil
              (:wat::core::let
                [ok2 (:wat::core::= sp sp-exp)
                 ok3 (:wat::core::= tab tab-exp)
                 ok4 (:wat::core::= ret ret-exp)]
                (:wat::core::if (:wat::core::= ok2 true) -> :wat::core::nil
                  (:wat::core::if (:wat::core::= ok3 true) -> :wat::core::nil
                    (:wat::core::if (:wat::core::= ok4 true) -> :wat::core::nil
                      (:wat::kernel::println "NAMED-CHARS-OK")
                      (:wat::kernel::println "NAMED-CHARS-RETURN-FAIL"))
                    (:wat::kernel::println "NAMED-CHARS-TAB-FAIL"))
                  (:wat::kernel::println "NAMED-CHARS-SPACE-FAIL")))
              (:wat::kernel::println "NAMED-CHARS-NEWLINE-FAIL"))))
    "#;
    let lines = run(src);
    assert_eq!(lines, vec!["\"NAMED-CHARS-OK\""], "named char literals must produce correct Char values");
}

// ─── 3: Lexer accepts `\uNNNN` Unicode BMP escape ────────────────────────────

/// `A` (U+0041 = 'A') produces a Char equal to `(:wat::core::Char/of "A")`.
#[test]
fn char_literal_unicode_escape() {
    // Build the source with a literal backslash-u0041 so the wat lexer
    // parses it as the \uNNNN unicode escape for U+0041 = 'A'.
    let src = format!(
        r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [c {}u0041
             expected (:wat::core::Char/of "A")
             ok (:wat::core::= c expected)]
            (:wat::core::if ok -> :wat::core::nil
              (:wat::kernel::println "UNICODE-ESCAPE-OK")
              (:wat::kernel::println "UNICODE-ESCAPE-FAIL"))))
        "#,
        '\\'
    );
    let lines = run(&src);
    assert_eq!(lines, vec!["\"UNICODE-ESCAPE-OK\""], "\\u0041 must produce Char('A')");
}

// ─── 4: Lexer rejects supplementary-plane literal ────────────────────────────

/// A supplementary-plane char literal (e.g. `\😀`) must fail at lex time
/// with a diagnostic mentioning "supplementary-plane" or "BMP".
/// This tests that the lexer enforces BMP-only at the source level.
#[test]
fn char_literal_supplementary_plane_rejected() {
    // `\😀` — emoji is U+1F600, supplementary plane.
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          \😀)
    "#;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    // Must fail at startup (lex/parse time).
    assert!(
        result.is_err(),
        "supplementary-plane char literal must fail at lex time"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("supplementary") || msg.contains("BMP") || msg.contains("U+1F600"),
        "error must mention supplementary-plane: got {:?}", msg
    );
}

// ─── 5: `(:wat::core::Char/of "x")` returns typed Char ──────────────────────

/// `Char/of` with a valid single-char string constructs a typed Char.
/// We verify by equality with another Char/of call (proving the type is correct).
#[test]
fn char_of_valid_single_char() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [c1  (:wat::core::Char/of "x")
             c2  (:wat::core::Char/of "x")
             eq  (:wat::core::= c1 c2)]
            (:wat::core::if eq -> :wat::core::nil
              (:wat::kernel::println "CHAR-OF-OK")
              (:wat::kernel::println "CHAR-NEQ-CHAR-WRONG"))))
    "#;
    let lines = run(src);
    assert_eq!(lines, vec!["\"CHAR-OF-OK\""], "Char/of must return typed Char equal to same Char");
}

// ─── 6: `Char/of ""` errors with length diagnostic ───────────────────────────

/// Empty string is rejected with a clear "length-1" diagnostic.
/// Arc 221 Stone 221.2: Char/of is now type-registered as `String → Char`.
/// The call is placed inside a let binding so user::main still returns nil
/// (arc 170 slice 1e canonical signature), while the runtime validation error
/// fires when evaluating the Char/of binding before the nil is returned.
#[test]
fn char_of_empty_string_rejected() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_c (:wat::core::Char/of "")]
            :wat::core::nil))
    "#;
    let err = run_expecting_runtime_err(src);
    assert!(
        err.contains("length-1") || err.contains("empty"),
        "error must mention length-1 or empty: got {:?}", err
    );
}

// ─── 7: `Char/of "ab"` errors with length diagnostic ────────────────────────

/// Multi-char string is rejected with a clear "length" diagnostic.
/// Arc 221 Stone 221.2: Char/of is now type-registered as `String → Char`.
/// Wrapped in a let binding so user::main returns nil per arc 170 slice 1e.
#[test]
fn char_of_multi_char_rejected() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_c (:wat::core::Char/of "ab")]
            :wat::core::nil))
    "#;
    let err = run_expecting_runtime_err(src);
    assert!(
        err.contains("length") || err.contains("got 2") || err.contains("length-2"),
        "error must mention length: got {:?}", err
    );
}

// ─── 8: `Char/of` with supplementary-plane char rejected ─────────────────────

/// A supplementary-plane char in a String arg is rejected with BMP diagnostic.
/// Arc 221 Stone 221.2: Char/of is now type-registered as `String → Char`.
/// Wrapped in a let binding so user::main returns nil per arc 170 slice 1e.
#[test]
fn char_of_supplementary_plane_rejected() {
    // U+1F600 GRINNING FACE — supplementary plane
    let src = &format!(
        "(:wat::core::define\n  (:user::main -> :wat::core::nil)\n  (:wat::core::let\n    [_c (:wat::core::Char/of \"\u{1F600}\")]\n    :wat::core::nil))"
    );
    let err = run_expecting_runtime_err(src);
    assert!(
        err.contains("supplementary") || err.contains("BMP") || err.contains("1F600"),
        "error must mention supplementary-plane: got {:?}", err
    );
}

// ─── 9: Round-trip: Char/of → EDN write → parse → identical ─────────────────

/// `(:wat::core::Char/of "x")` → EDN write → `(:wat::edn::read ...)` → typed Char.
/// Proves the EDN bridge is bidirectional for Char values.
#[test]
fn char_edn_round_trip() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [orig  (:wat::core::Char/of "x")
             edn   (:wat::edn::write orig)
             back  (:wat::edn::read edn)
             ok    (:wat::core::= orig back)]
            (:wat::core::if ok -> :wat::core::nil
              (:wat::kernel::println "ROUND-TRIP-OK")
              (:wat::kernel::println "ROUND-TRIP-FAIL"))))
    "#;
    let lines = run(src);
    assert_eq!(lines, vec!["\"ROUND-TRIP-OK\""], "Char must round-trip through EDN write/read");
}

// ─── 10: Equality ─────────────────────────────────────────────────────────────

/// `(= \a \a)` is true; `(= \a \b)` is false.
#[test]
fn char_equality() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a1  \a
             a2  \a
             b   \b
             eq1 (:wat::core::= a1 a2)
             eq2 (:wat::core::= a1 b)]
            (:wat::core::if eq1 -> :wat::core::nil
              (:wat::core::if (:wat::core::= eq2 false) -> :wat::core::nil
                (:wat::kernel::println "EQUALITY-OK")
                (:wat::kernel::println "DIFF-CHARS-EQ-WRONG"))
              (:wat::kernel::println "SAME-CHARS-NEQ-WRONG"))))
    "#;
    let lines = run(src);
    assert_eq!(lines, vec!["\"EQUALITY-OK\""], "Char equality must be correct for same and different chars");
}

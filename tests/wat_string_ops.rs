//! Integration coverage for `:wat::core::string::*` + `:wat::core::regex::*`.
//!
//! Drives every handler through a `:user::main` that writes results
//! line-by-line to stdout; assertions compare the captured stdout.
//! One test per primitive keeps failure messages pointed.

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

fn bool_src(expr: &str) -> String {
    format!(
        r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::if {expr} -> :wat::core::nil
            (:wat::kernel::println "true")
            (:wat::kernel::println "false")))
        "#,
    )
}

fn string_src(expr: &str) -> String {
    format!(
        r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println {expr}))
        "#,
    )
}

// ─── :wat::core::string::contains? / starts-with? / ends-with? ──────────

#[test]
fn contains_hit() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::contains? "hello world" "world")"#)),
        vec!["\"true\""]
    );
}

#[test]
fn contains_miss() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::contains? "hello" "xyz")"#)),
        vec!["\"false\""]
    );
}

#[test]
fn starts_with_hit_and_miss() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::starts-with? "foobar" "foo")"#)),
        vec!["\"true\""]
    );
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::starts-with? "foobar" "bar")"#)),
        vec!["\"false\""]
    );
}

#[test]
fn ends_with_hit_and_miss() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::ends-with? "foobar" "bar")"#)),
        vec!["\"true\""]
    );
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::ends-with? "foobar" "foo")"#)),
        vec!["\"false\""]
    );
}

// ─── :wat::core::string::length ─────────────────────────────────────────

#[test]
fn length_counts_chars_not_bytes() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [n (:wat::core::string::length "héllo")]
            (:wat::core::if (:wat::core::= n 5) -> :wat::core::nil
              (:wat::kernel::println "chars")
              (:wat::kernel::println "bytes"))))
    "#;
    assert_eq!(run(src), vec!["\"chars\"".to_string()]);
}

// ─── :wat::core::string::trim ───────────────────────────────────────────

#[test]
fn trim_strips_whitespace() {
    assert_eq!(
        run(&string_src(r#"(:wat::core::string::trim "   hello   ")"#)),
        vec!["\"hello\""]
    );
}

// ─── :wat::core::string::split / join ───────────────────────────────────

#[test]
fn split_produces_vec() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [pieces
              (:wat::core::string::split "a,b,c" ",")]
            (:wat::kernel::println
              (:wat::core::string::join "|" pieces))))
    "#;
    assert_eq!(run(src), vec!["\"a|b|c\"".to_string()]);
}

#[test]
fn split_empty_separator_rejected() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_
              (:wat::core::string::split "abc" "")]
            ()))
    "#;
    let msg = run_expecting_runtime_err(src);
    assert!(
        msg.contains("separator must not be empty"),
        "expected empty-separator error; got {}",
        msg
    );
}

// ─── :wat::core::regex::matches? ────────────────────────────────────────

#[test]
fn regex_matches_unanchored() {
    assert_eq!(
        run(&bool_src(
            r#"(:wat::core::regex::matches? "[0-9]+" "order #42 shipped")"#
        )),
        vec!["\"true\""]
    );
}

#[test]
fn regex_matches_no_match() {
    assert_eq!(
        run(&bool_src(
            r#"(:wat::core::regex::matches? "^foo$" "foobar")"#
        )),
        vec!["\"false\""]
    );
}

#[test]
fn regex_invalid_pattern_errors() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_ (:wat::core::regex::matches? "[unclosed" "x")]
            ()))
    "#;
    let msg = run_expecting_runtime_err(src);
    assert!(
        msg.contains("invalid regex"),
        "expected invalid regex error; got {}",
        msg
    );
}

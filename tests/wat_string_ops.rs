//! Integration coverage for `:wat::core::string::*` + `:wat::core::regex::*`.
//!
//! Drives every handler through a `:user::main` that writes results
//! line-by-line to stdout; assertions compare the captured stdout.
//! One test per primitive keeps failure messages pointed.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
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
    let bytes = stdout.snapshot_bytes().expect("stdout snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn bool_src(expr: &str) -> String {
    format!(
        r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::if {expr} -> :()
            (:wat::io::IOWriter/println stdout "true")
            (:wat::io::IOWriter/println stdout "false")))
        "#,
    )
}

fn string_src(expr: &str) -> String {
    format!(
        r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout {expr}))
        "#,
    )
}

// ─── :wat::core::string::contains? / starts-with? / ends-with? ──────────

#[test]
fn contains_hit() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::contains? "hello world" "world")"#)),
        vec!["true"]
    );
}

#[test]
fn contains_miss() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::contains? "hello" "xyz")"#)),
        vec!["false"]
    );
}

#[test]
fn starts_with_hit_and_miss() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::starts-with? "foobar" "foo")"#)),
        vec!["true"]
    );
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::starts-with? "foobar" "bar")"#)),
        vec!["false"]
    );
}

#[test]
fn ends_with_hit_and_miss() {
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::ends-with? "foobar" "bar")"#)),
        vec!["true"]
    );
    assert_eq!(
        run(&bool_src(r#"(:wat::core::string::ends-with? "foobar" "foo")"#)),
        vec!["false"]
    );
}

// ─── :wat::core::string::length ─────────────────────────────────────────

#[test]
fn length_counts_chars_not_bytes() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((n :i64) (:wat::core::string::length "héllo")))
            (:wat::core::if (:wat::core::= n 5) -> :()
              (:wat::io::IOWriter/println stdout "chars")
              (:wat::io::IOWriter/println stdout "bytes"))))
    "#;
    assert_eq!(run(src), vec!["chars"]);
}

// ─── :wat::core::string::trim ───────────────────────────────────────────

#[test]
fn trim_strips_whitespace() {
    assert_eq!(
        run(&string_src(r#"(:wat::core::string::trim "   hello   ")"#)),
        vec!["hello"]
    );
}

// ─── :wat::core::string::split / join ───────────────────────────────────

#[test]
fn split_produces_vec() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((pieces :Vec<String>)
              (:wat::core::string::split "a,b,c" ",")))
            (:wat::io::IOWriter/println stdout
              (:wat::core::string::join "|" pieces))))
    "#;
    assert_eq!(run(src), vec!["a|b|c"]);
}

#[test]
fn split_empty_separator_rejected() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((_ :Vec<String>)
              (:wat::core::string::split "abc" "")))
            ()))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout: Arc<dyn WatWriter> = Arc::new(StringIoWriter::new());
    let stderr: Arc<dyn WatWriter> = Arc::new(StringIoWriter::new());
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout),
        Value::io__IOWriter(stderr),
    ];
    let err = invoke_user_main(&world, args).expect_err("empty sep should fail");
    let msg = format!("{:?}", err);
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
        vec!["true"]
    );
}

#[test]
fn regex_matches_no_match() {
    assert_eq!(
        run(&bool_src(
            r#"(:wat::core::regex::matches? "^foo$" "foobar")"#
        )),
        vec!["false"]
    );
}

#[test]
fn regex_invalid_pattern_errors() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((_ :bool) (:wat::core::regex::matches? "[unclosed" "x")))
            ()))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout: Arc<dyn WatWriter> = Arc::new(StringIoWriter::new());
    let stderr: Arc<dyn WatWriter> = Arc::new(StringIoWriter::new());
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout),
        Value::io__IOWriter(stderr),
    ];
    let err = invoke_user_main(&world, args).expect_err("bad regex should fail");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("invalid regex"),
        "expected invalid regex error; got {}",
        msg
    );
}

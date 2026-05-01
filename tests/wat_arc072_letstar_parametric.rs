//! Arc 072 regression — parametric type keywords with `<>` lex
//! cleanly, and whitespace inside `:<...>` produces a clean
//! diagnostic at the lexer layer instead of silently truncating
//! into a mysterious downstream type-check error.
//!
//! Pre-arc-072, the lexer tracked `()` depth but ignored `<>` depth.
//! `:Result<(i64,i64), i64>` (space after the comma) tokenized as
//! `:Result<(i64,i64),` (whitespace truncated the keyword) plus a
//! separate `i64>` symbol — the type parser saw a malformed Result
//! with one arg, the rest dropped. Downstream the type checker
//! surfaced as "fresh-var :?N unsolved" at pattern-arm sites.
//! Probe 018 chased that opaque error through several layers
//! before tracing it back to the lexer.
//!
//! The fix: lexer now tracks `<>` depth alongside `()` for type-
//! head brackets (operator `<` / `>` in keyword paths like
//! `:wat::core::<` are disambiguated by the preceding char — only
//! `<` after an alphanumeric counts toward depth). Whitespace
//! inside an unclosed `<` raises `LexError::UnclosedBracketInKeyword`
//! at the lex layer — the user gets a clean error pointing at the
//! exact byte, not a downstream "fresh var unsolved."
//!
//! The substrate's whitespace rule for type keywords stays strict
//! (per the existing convention — `:Result<i64,String>` not
//! `:Result<i64, String>`). The arc fixes the diagnostic, not the
//! rule.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Result<Vec<String>, String> {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {}", e))?;
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
    invoke_user_main(&world, args).map_err(|e| format!("runtime: {}", e))?;
    let bytes = stdout.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    Ok(lines)
}

/// `:Result<i64,String>` (canonical, no whitespace) lexes, parses,
/// type-checks, and runs end-to-end. The chain proof 018's walker
/// rewrite intends to use.
#[test]
fn letstar_result_no_whitespace_simple_payload() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((wrapped :Result<wat::core::i64,wat::core::String>)
              (Ok 42))
             ((extracted :wat::core::i64)
              (:wat::core::match wrapped -> :wat::core::i64
                ((Ok n) (:wat::core::i64::+ n 1))
                ((Err _) -1))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::i64::to-string extracted))))
    "#;
    match run(src) {
        Ok(lines) => assert_eq!(lines, vec!["43".to_string()]),
        Err(e) => panic!("{}", e),
    }
}

/// `:Result<(i64,i64),i64>` (canonical) — tuple inside parametric.
/// The exact shape that surfaced this arc from proof 018's walker
/// rewrite. Pre-fix: lexer truncated, downstream "fresh var :?71"
/// at the (second pair) call. Post-fix: lexes cleanly, runs.
#[test]
fn letstar_result_no_whitespace_tuple_payload() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((wrapped :Result<(wat::core::i64,wat::core::i64),wat::core::i64>)
              (Ok (:wat::core::tuple 7 11)))
             ((extracted :wat::core::i64)
              (:wat::core::match wrapped -> :wat::core::i64
                ((Ok pair) (:wat::core::second pair))
                ((Err _) -1))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::i64::to-string extracted))))
    "#;
    match run(src) {
        Ok(lines) => assert_eq!(lines, vec!["11".to_string()]),
        Err(e) => panic!("{}", e),
    }
}

/// Whitespace inside `:<...>` now raises a clean lex-layer error
/// instead of silently truncating into a downstream type-check
/// failure. This is the diagnostic improvement that makes proof-018-
/// shape debugging tractable.
#[test]
fn whitespace_inside_angle_brackets_raises_clean_lex_error() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((m :HashMap<String, i64>)
              (:wat::core::HashMap :String :i64)))
            (:wat::io::IOWriter/println stdout "ok")))
    "#;
    let err = run(src).expect_err("expected lex error on `:HashMap<String, i64>`");
    assert!(
        err.contains("whitespace inside unclosed bracket"),
        "expected lex-layer diagnostic, got: {}",
        err
    );
}

/// Operator `<` and `>` in keyword paths must still lex as part of
/// the keyword (they're not bracket openers; they follow `::`). This
/// test confirms the lexer's disambiguation didn't break operators.
#[test]
fn operator_lt_gt_keywords_still_lex() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::if (:wat::core::i64::< 1 2) -> :wat::core::unit
            (:wat::core::if (:wat::core::i64::>= 5 5) -> :wat::core::unit
              (:wat::io::IOWriter/println stdout "ok")
              (:wat::io::IOWriter/println stdout "ge-fail"))
            (:wat::io::IOWriter/println stdout "lt-fail")))
    "#;
    match run(src) {
        Ok(lines) => assert_eq!(lines, vec!["ok".to_string()]),
        Err(e) => panic!("{}", e),
    }
}

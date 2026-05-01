//! End-to-end tests for `:wat::kernel::pipe` — arc 012 slice 1b.
//!
//! Covers the wat-level surface: pipe returns a
//! `:(wat::io::IOWriter,wat::io::IOReader)` 2-tuple, both ends satisfy
//! the existing IOReader / IOWriter primitives, and bytes written to
//! the writer become readable from the reader. No fork involved —
//! the pipe is entirely within the single :user::main thread.
//!
//! EOF-on-writer-dropped behavior is covered in src/io.rs's
//! `pipe_tests` Rust-level tests (which can `drop(w)` explicitly);
//! at the wat level, writer lifetime is scope-bound and tests avoid
//! read-all / EOF paths that would require killing the writer.

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

fn unwrap_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn unwrap_i64(v: Value) -> i64 {
    match v {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Shape ───────────────────────────────────────────────────────────────

#[test]
fn pipe_returns_writer_reader_tuple() {
    // Bind the 2-tuple and destructure via first/second. No I/O —
    // just proves the type shape lands through the checker + runtime.
    let src = r#"

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((pair :(wat::io::IOWriter,wat::io::IOReader))
              (:wat::kernel::pipe))
             ((_w :wat::io::IOWriter) (:wat::core::first pair))
             ((_r :wat::io::IOReader) (:wat::core::second pair)))
            42))
    "#;
    assert_eq!(unwrap_i64(run(src)), 42);
}

// ─── Round-trip ──────────────────────────────────────────────────────────

#[test]
fn pipe_writeln_then_read_line_round_trips() {
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::Option<wat::core::String>)
          (:wat::core::let*
            (((pair :(wat::io::IOWriter,wat::io::IOReader))
              (:wat::kernel::pipe))
             ((w :wat::io::IOWriter) (:wat::core::first pair))
             ((r :wat::io::IOReader) (:wat::core::second pair))
             ((_ :wat::core::i64) (:wat::io::IOWriter/writeln w "hello")))
            (:wat::io::IOReader/read-line r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "hello");
}

#[test]
fn pipe_multiple_writelns_read_line_by_line() {
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::String)
          (:wat::core::let*
            (((pair :(wat::io::IOWriter,wat::io::IOReader))
              (:wat::kernel::pipe))
             ((w :wat::io::IOWriter) (:wat::core::first pair))
             ((r :wat::io::IOReader) (:wat::core::second pair))
             ((_ :wat::core::i64) (:wat::io::IOWriter/writeln w "first"))
             ((_ :wat::core::i64) (:wat::io::IOWriter/writeln w "second"))
             ((a :wat::core::Option<wat::core::String>) (:wat::io::IOReader/read-line r))
             ((b :wat::core::Option<wat::core::String>) (:wat::io::IOReader/read-line r)))
            (:wat::core::match a -> :wat::core::String
              ((Some sa)
               (:wat::core::match b -> :wat::core::String
                 ((Some sb) (:wat::core::string::join "," (:wat::core::vec :wat::core::String sa sb)))
                 (:None     "second-missing")))
              (:None "first-missing"))))
    "#;
    assert_eq!(unwrap_string(run(src)), "first,second");
}

#[test]
fn pipe_write_string_then_read_exact_bytes() {
    // Write a fixed 5-byte string, read exactly 5 bytes back. No EOF,
    // no newline involvement — just byte-level round-trip.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((pair :(wat::io::IOWriter,wat::io::IOReader))
              (:wat::kernel::pipe))
             ((w :wat::io::IOWriter) (:wat::core::first pair))
             ((r :wat::io::IOReader) (:wat::core::second pair))
             ((n :wat::core::i64) (:wat::io::IOWriter/write-string w "hello"))
             ((got :wat::core::Option<Vec<wat::core::u8>>) (:wat::io::IOReader/read r 5)))
            (:wat::core::match got -> :wat::core::i64
              ((Some bytes) n)
              (:None        -1))))
    "#;
    assert_eq!(unwrap_i64(run(src)), 5);
}

// ─── UTF-8 handling matches StringIo ─────────────────────────────────────

#[test]
fn pipe_preserves_utf8_lines() {
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::Option<wat::core::String>)
          (:wat::core::let*
            (((pair :(wat::io::IOWriter,wat::io::IOReader))
              (:wat::kernel::pipe))
             ((w :wat::io::IOWriter) (:wat::core::first pair))
             ((r :wat::io::IOReader) (:wat::core::second pair))
             ((_ :wat::core::i64) (:wat::io::IOWriter/writeln w "héllo")))
            (:wat::io::IOReader/read-line r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "héllo");
}

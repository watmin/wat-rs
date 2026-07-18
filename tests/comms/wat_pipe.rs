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
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::call_beside;
use wat::runtime::Value;

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
    let v = call_beside(file!(), ":my::pipe-returns-writer-reader-tuple").expect("eval");
    assert_eq!(unwrap_i64(v), 42);
}

// ─── Round-trip ──────────────────────────────────────────────────────────

#[test]
fn pipe_writeln_then_read_line_round_trips() {
    let v = call_beside(file!(), ":my::pipe-writeln-round-trips").expect("eval");
    assert_eq!(unwrap_some_string(v), "hello");
}

#[test]
fn pipe_multiple_writelns_read_line_by_line() {
    let v = call_beside(file!(), ":my::pipe-multiple-writelns").expect("eval");
    assert_eq!(unwrap_string(v), "first,second");
}

#[test]
fn pipe_write_string_then_read_exact_bytes() {
    // Write a fixed 5-byte string, read exactly 5 bytes back. No EOF,
    // no newline involvement — just byte-level round-trip.
    let v = call_beside(file!(), ":my::pipe-write-string-exact-bytes").expect("eval");
    assert_eq!(unwrap_i64(v), 5);
}

// ─── UTF-8 handling matches StringIo ─────────────────────────────────────

#[test]
fn pipe_preserves_utf8_lines() {
    let v = call_beside(file!(), ":my::pipe-preserves-utf8").expect("eval");
    assert_eq!(unwrap_some_string(v), "héllo");
}

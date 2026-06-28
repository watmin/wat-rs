//! End-to-end tests for `:wat::io::IOReader` + `:wat::io::IOWriter` —
//! arc 008 slice 2.
//!
//! Covers:
//! - IOReader construction from string / bytes.
//! - read (partial), read-all, read-line (with CRLF handling), rewind.
//! - IOWriter construction + snapshot (to-bytes / to-string).
//! - write (returns count), write-all, writeln (appends \n), flush.
//! - Full round-trip: read from one reader and write to a writer,
//!   then snapshot the writer.
//! - ThreadOwnedCell single-thread ownership — StringIo instances
//!   used within one thread work; we don't test cross-thread panics
//!   here because that requires spawning sub-threads via :wat::kernel::spawn
//!   which is slice-3 territory.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_fn(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let call = format!("({fn_name})");
    let ast = wat::parse_one!(&call).expect("parse compute call");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
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

fn is_option_none(v: &Value) -> bool {
    matches!(v, Value::Option(opt) if opt.is_none())
}

fn bytes_from_vec_u8(v: Value) -> Vec<u8> {
    match v {
        Value::Vec(items) => items
            .iter()
            .map(|it| match it {
                Value::u8(b) => *b,
                other => panic!("expected u8; got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec; got {:?}", other),
    }
}

// ─── IOReader construction + read-line ───────────────────────────────────

#[test]
fn io_reader_from_string_read_line_round_trips() {
    assert_eq!(unwrap_some_string(run_fn(":my::compute-read-line")), "hello");
}

#[test]
fn io_reader_read_line_handles_crlf() {
    assert_eq!(unwrap_some_string(run_fn(":my::compute-read-line-crlf")), "hello");
}

#[test]
fn io_reader_read_line_at_eof_is_none() {
    assert!(is_option_none(&run_fn(":my::compute-read-line-eof")));
}

// ─── IOReader read (byte-level, partial) ─────────────────────────────────

#[test]
fn io_reader_read_returns_up_to_n_bytes() {
    // "hello" is 5 bytes. Read 3, expect [h, e, l].
    match run_fn(":my::compute-read-bytes") {
        Value::Option(opt) => match &*opt {
            Some(v) => {
                let bytes = bytes_from_vec_u8(v.clone());
                assert_eq!(bytes, b"hel".to_vec());
            }
            None => panic!("expected Some; got None"),
        },
        other => panic!("expected Option; got {:?}", other),
    }
}

#[test]
fn io_reader_read_at_eof_is_none() {
    assert!(is_option_none(&run_fn(":my::compute-read-bytes-eof")));
}

// ─── IOReader read-all ──────────────────────────────────────────────────

#[test]
fn io_reader_read_all_returns_everything() {
    let bytes = bytes_from_vec_u8(run_fn(":my::compute-read-all"));
    assert_eq!(bytes, b"hello".to_vec());
}

// ─── IOReader rewind ─────────────────────────────────────────────────────

#[test]
fn io_reader_rewind_restarts_from_beginning() {
    // Read everything, rewind, read again. Second read must succeed.
    let bytes = bytes_from_vec_u8(run_fn(":my::compute-rewind"));
    assert_eq!(bytes, b"again".to_vec());
}

// ─── IOWriter round-trip via to-string ───────────────────────────────────

#[test]
fn io_writer_writeln_then_to_string_round_trips() {
    assert_eq!(
        unwrap_some_string(run_fn(":my::compute-writeln-to-string")),
        "first\nsecond\n"
    );
}

#[test]
fn io_writer_writeln_returns_bytes_written() {
    // "hello" (5 bytes) + "\n" = 6 bytes written.
    assert!(matches!(run_fn(":my::compute-writeln-count"), Value::i64(6)));
}

#[test]
fn io_writer_write_returns_byte_count() {
    // Vec<u8> of 3 bytes written; write returns count.
    assert!(matches!(run_fn(":my::compute-write-bytes"), Value::i64(3)));
}

#[test]
fn io_writer_write_all_then_to_bytes_round_trips() {
    let bytes = bytes_from_vec_u8(run_fn(":my::compute-write-all-to-bytes"));
    assert_eq!(bytes, vec![65, 66, 67]);
}

#[test]
fn io_writer_write_string_does_not_add_newline() {
    // write-string writes bytes as-is; no implicit \n (unlike writeln).
    // Matches the semantics of pre-arc-008 :wat::io::write on real
    // Stdout/Stderr — caller controls newlines.
    assert_eq!(
        unwrap_some_string(run_fn(":my::compute-write-string-no-newline")),
        "hello world"
    );
}

#[test]
fn io_writer_write_string_returns_byte_count() {
    // "héllo" is 6 UTF-8 bytes (é is 2 bytes). This passes only when
    // the lexer preserves multi-byte UTF-8 in string literals — arc
    // 008 slice 3 fixed the byte-at-a-time bug that previously
    // re-encoded each byte as a Latin-1 char.
    assert!(matches!(run_fn(":my::compute-write-string-byte-count"), Value::i64(6)));
}

#[test]
fn io_writer_flush_is_ok_for_string_writer() {
    // flush on an IOWriter backed by an in-memory buffer returns nil.
    assert!(matches!(run_fn(":my::compute-flush"), Value::Unit));
}

// ─── Full round-trip: reader → writer ────────────────────────────────────

#[test]
fn reader_lines_copied_to_writer() {
    // Read two lines from reader, write each to writer with writeln.
    // to-string on writer should show "alpha\nbeta\n".
    assert_eq!(
        unwrap_some_string(run_fn(":my::compute-copy-lines")),
        "alpha\nbeta\n"
    );
}

// ─── Empty cases ─────────────────────────────────────────────────────────

#[test]
fn fresh_writer_to_string_is_empty() {
    assert_eq!(unwrap_some_string(run_fn(":my::compute-fresh-writer-empty")), "");
}

#[test]
fn empty_reader_read_line_is_none() {
    assert!(is_option_none(&run_fn(":my::compute-empty-reader-read-line")));
}

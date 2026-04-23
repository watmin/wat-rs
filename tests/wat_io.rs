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
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "hello\nworld\n")))
            (:wat::io::IOReader/read-line r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "hello");
}

#[test]
fn io_reader_read_line_handles_crlf() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "hello\r\n")))
            (:wat::io::IOReader/read-line r)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "hello");
}

#[test]
fn io_reader_read_line_at_eof_is_none() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:my::drain (r :wat::io::IOReader) -> :Option<String>)
          (:wat::core::let*
            (((_ :Option<String>) (:wat::io::IOReader/read-line r)))
            (:wat::io::IOReader/read-line r)))

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "only-line\n")))
            (:my::drain r)))
    "#;
    assert!(is_option_none(&run(src)));
}

// ─── IOReader read (byte-level, partial) ─────────────────────────────────

#[test]
fn io_reader_read_returns_up_to_n_bytes() {
    // "hello" is 5 bytes. Read 3, expect [h, e, l].
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Option<Vec<u8>>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "hello")))
            (:wat::io::IOReader/read r 3)))
    "#;
    match run(src) {
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
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:my::drain (r :wat::io::IOReader) -> :Option<Vec<u8>>)
          (:wat::core::let*
            (((_ :Option<Vec<u8>>) (:wat::io::IOReader/read r 100)))
            (:wat::io::IOReader/read r 100)))

        (:wat::core::define (:user::main -> :Option<Vec<u8>>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "hi")))
            (:my::drain r)))
    "#;
    assert!(is_option_none(&run(src)));
}

// ─── IOReader read-all ──────────────────────────────────────────────────

#[test]
fn io_reader_read_all_returns_everything() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Vec<u8>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "hello")))
            (:wat::io::IOReader/read-all r)))
    "#;
    let bytes = bytes_from_vec_u8(run(src));
    assert_eq!(bytes, b"hello".to_vec());
}

// ─── IOReader rewind ─────────────────────────────────────────────────────

#[test]
fn io_reader_rewind_restarts_from_beginning() {
    // Read everything, rewind, read again. Second read must succeed.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:my::read-twice (r :wat::io::IOReader) -> :Vec<u8>)
          (:wat::core::let*
            (((_ :Vec<u8>) (:wat::io::IOReader/read-all r))
             ((_ :()) (:wat::io::IOReader/rewind r)))
            (:wat::io::IOReader/read-all r)))

        (:wat::core::define (:user::main -> :Vec<u8>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "again")))
            (:my::read-twice r)))
    "#;
    let bytes = bytes_from_vec_u8(run(src));
    assert_eq!(bytes, b"again".to_vec());
}

// ─── IOWriter round-trip via to-string ───────────────────────────────────

#[test]
fn io_writer_writeln_then_to_string_round_trips() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new))
             ((_ :i64) (:wat::io::IOWriter/writeln w "first"))
             ((_ :i64) (:wat::io::IOWriter/writeln w "second")))
            (:wat::io::IOWriter/to-string w)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "first\nsecond\n");
}

#[test]
fn io_writer_writeln_returns_bytes_written() {
    // "hello" (5 bytes) + "\n" = 6 bytes written.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new)))
            (:wat::io::IOWriter/writeln w "hello")))
    "#;
    assert!(matches!(run(src), Value::i64(6)));
}

#[test]
fn io_writer_write_returns_byte_count() {
    // Vec<u8> of 3 bytes written; write returns count.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new))
             ((bytes :Vec<u8>)
              (:wat::core::vec :u8
                (:wat::core::u8 72)
                (:wat::core::u8 105)
                (:wat::core::u8 33))))
            (:wat::io::IOWriter/write w bytes)))
    "#;
    assert!(matches!(run(src), Value::i64(3)));
}

#[test]
fn io_writer_write_all_then_to_bytes_round_trips() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Vec<u8>)
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new))
             ((bytes :Vec<u8>)
              (:wat::core::vec :u8
                (:wat::core::u8 65)
                (:wat::core::u8 66)
                (:wat::core::u8 67)))
             ((_ :()) (:wat::io::IOWriter/write-all w bytes)))
            (:wat::io::IOWriter/to-bytes w)))
    "#;
    let bytes = bytes_from_vec_u8(run(src));
    assert_eq!(bytes, vec![65, 66, 67]);
}

#[test]
fn io_writer_write_string_does_not_add_newline() {
    // write-string writes bytes as-is; no implicit \n (unlike writeln).
    // Matches the semantics of pre-arc-008 :wat::io::write on real
    // Stdout/Stderr — caller controls newlines.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new))
             ((_ :i64) (:wat::io::IOWriter/write-string w "hello "))
             ((_ :i64) (:wat::io::IOWriter/write-string w "world")))
            (:wat::io::IOWriter/to-string w)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "hello world");
}

#[test]
fn io_writer_write_string_returns_byte_count() {
    // "héllo" is 6 UTF-8 bytes (é is 2 bytes). This passes only when
    // the lexer preserves multi-byte UTF-8 in string literals — arc
    // 008 slice 3 fixed the byte-at-a-time bug that previously
    // re-encoded each byte as a Latin-1 char.
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new)))
            (:wat::io::IOWriter/write-string w "héllo")))
    "#;
    assert!(matches!(run(src), Value::i64(6)));
}

#[test]
fn io_writer_flush_is_ok_for_string_writer() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :())
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new)))
            (:wat::io::IOWriter/flush w)))
    "#;
    assert!(matches!(run(src), Value::Unit));
}

// ─── Full round-trip: reader → writer ────────────────────────────────────

#[test]
fn reader_lines_copied_to_writer() {
    // Read two lines from reader, write each to writer with writeln.
    // to-string on writer should show "alpha\nbeta\n".
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define
          (:my::copy-one
            (r :wat::io::IOReader)
            (w :wat::io::IOWriter)
            -> :i64)
          (:wat::core::match (:wat::io::IOReader/read-line r) -> :i64
            ((Some line) (:wat::io::IOWriter/writeln w line))
            (:None -1)))

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((r :wat::io::IOReader)
              (:wat::io::IOReader/from-string "alpha\nbeta\n"))
             ((w :wat::io::IOWriter) (:wat::io::IOWriter/new))
             ((_ :i64) (:my::copy-one r w))
             ((_ :i64) (:my::copy-one r w)))
            (:wat::io::IOWriter/to-string w)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "alpha\nbeta\n");
}

// ─── Empty cases ─────────────────────────────────────────────────────────

#[test]
fn fresh_writer_to_string_is_empty() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((w :wat::io::IOWriter) (:wat::io::IOWriter/new)))
            (:wat::io::IOWriter/to-string w)))
    "#;
    assert_eq!(unwrap_some_string(run(src)), "");
}

#[test]
fn empty_reader_read_line_is_none() {
    let src = r#"
        (:wat::config::set-capacity-mode! :error)
        (:wat::config::set-dims! 1024)

        (:wat::core::define (:user::main -> :Option<String>)
          (:wat::core::let*
            (((r :wat::io::IOReader) (:wat::io::IOReader/from-string "")))
            (:wat::io::IOReader/read-line r)))
    "#;
    assert!(is_option_none(&run(src)));
}


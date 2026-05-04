//! Arc 049 — newtype value support. End-to-end coverage of:
//! - Constructor `:Type/new(value)` round-trip
//! - Accessor `:Type/0(self)` returns the inner value
//! - Nominal distinction enforced by the type checker
//!   (cannot mix newtype with its inner type)
//! - Newtype as a struct field round-trip

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
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

fn run_expecting_check_error(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("startup should fail with check error");
    format!("{:?}", err)
}

// ─── Construct + access round-trip ────────────────────────────────────

#[test]
fn newtype_construct_and_accessor_roundtrip() {
    let src = r##"
        (:wat::core::newtype :my::trading::Price :f64)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((p :my::trading::Price) (:my::trading::Price/new 100.0))
             ((inner :wat::core::f64) (:my::trading::Price/0 p)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string inner))))
    "##;
    assert_eq!(run(src), vec!["100".to_string()]);
}

// ─── Nominal distinction in argument position ─────────────────────────

#[test]
fn newtype_rejects_inner_type_at_arg_position() {
    let src = r##"
        (:wat::core::newtype :my::trading::Price :f64)

        (:wat::core::define (:my::trading::pretty (p :my::trading::Price) -> :String)
          (:wat::core::f64::to-string (:my::trading::Price/0 p)))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout (:my::trading::pretty 100.0)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("Price") || err.to_lowercase().contains("type"),
        "expected type-mismatch diagnostic mentioning Price; got: {}",
        err
    );
}

// ─── Inverse: newtype rejected where inner expected ───────────────────

#[test]
fn newtype_rejected_where_inner_expected() {
    let src = r##"
        (:wat::core::newtype :my::trading::Price :f64)

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          ;; Pass a Price where an f64 is expected — type-checker should refuse.
          (:wat::core::let*
            (((p :my::trading::Price) (:my::trading::Price/new 100.0))
             ((bogus :f64) (:wat::core::f64::+,2 p 1.0)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string bogus))))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("Price")
            || err.contains("f64")
            || err.to_lowercase().contains("type"),
        "expected type-mismatch diagnostic for Price/f64 mix; got: {}",
        err
    );
}

// ─── Newtype as struct field round-trip ────────────────────────────────

#[test]
fn newtype_as_struct_field_roundtrip() {
    let src = r##"
        (:wat::core::newtype :my::trading::Price :f64)

        (:wat::core::struct :my::Order
          (label :wat::core::String)
          (price :my::trading::Price)
          (qty   :wat::core::i64))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((p :my::trading::Price) (:my::trading::Price/new 99.5))
             ((o :my::Order)          (:my::Order/new "BTC" p 7))
             ((retrieved :my::trading::Price) (:my::Order/price o))
             ((inner :wat::core::f64) (:my::trading::Price/0 retrieved)))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string inner))))
    "##;
    assert_eq!(run(src), vec!["99.5".to_string()]);
}

// ─── Two distinct newtypes over the same inner stay distinct ──────────

#[test]
fn distinct_newtypes_over_same_inner_are_distinct_types() {
    let src = r##"
        (:wat::core::newtype :my::trading::Price :f64)
        (:wat::core::newtype :my::trading::Amount :f64)

        (:wat::core::define (:my::trading::price-pretty (p :my::trading::Price) -> :String)
          (:wat::core::f64::to-string (:my::trading::Price/0 p)))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          ;; Pass an Amount where Price is expected — must fail.
          (:wat::core::let*
            (((a :my::trading::Amount) (:my::trading::Amount/new 50.0)))
            (:wat::io::IOWriter/println stdout (:my::trading::price-pretty a))))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("Price")
            || err.contains("Amount")
            || err.to_lowercase().contains("type"),
        "expected type-mismatch diagnostic Price vs Amount; got: {}",
        err
    );
}

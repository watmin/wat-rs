//! Arc 056 — `:wat::core::sort-by`.
//!
//! User-supplied less-than predicate drives the ordering. Asc vs desc
//! is encoded by which way the predicate compares; key-extraction is
//! the predicate composing inner accessors. Common Lisp tradition.

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

#[test]
fn sort_by_ascending_i64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 3 1 4 1 5 9 2 6))
             ((sorted :wat::core::Vector<wat::core::i64>)
              (:wat::core::sort-by xs
                (:wat::core::lambda ((a :wat::core::i64) (b :wat::core::i64) -> :wat::core::bool)
                  (:wat::core::< a b)))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::string::join ","
                (:wat::core::map sorted
                  (:wat::core::lambda ((n :wat::core::i64) -> :wat::core::String)
                    (:wat::core::i64::to-string n)))))))
    "##;
    assert_eq!(run(src), vec!["1,1,2,3,4,5,6,9".to_string()]);
}

#[test]
fn sort_by_descending_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::f64>) (:wat::core::Vector :wat::core::f64 1.5 0.5 2.5 1.0))
             ((sorted :wat::core::Vector<wat::core::f64>)
              (:wat::core::sort-by xs
                (:wat::core::lambda ((a :wat::core::f64) (b :wat::core::f64) -> :wat::core::bool)
                  (:wat::core::> a b)))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::string::join ","
                (:wat::core::map sorted
                  (:wat::core::lambda ((x :wat::core::f64) -> :wat::core::String)
                    (:wat::core::f64::to-string x)))))))
    "##;
    assert_eq!(run(src), vec!["2.5,1.5,1,0.5".to_string()]);
}

#[test]
fn sort_by_string() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::String>) (:wat::core::Vector :wat::core::String "banana" "apple" "cherry"))
             ((sorted :wat::core::Vector<wat::core::String>)
              (:wat::core::sort-by xs
                (:wat::core::lambda ((a :wat::core::String) (b :wat::core::String) -> :wat::core::bool)
                  (:wat::core::< a b)))))
            (:wat::io::IOWriter/println stdout (:wat::core::string::join "," sorted))))
    "##;
    assert_eq!(run(src), vec!["apple,banana,cherry".to_string()]);
}

#[test]
fn sort_by_empty_vec() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64))
             ((sorted :wat::core::Vector<wat::core::i64>)
              (:wat::core::sort-by xs
                (:wat::core::lambda ((a :wat::core::i64) (b :wat::core::i64) -> :wat::core::bool)
                  (:wat::core::< a b))))
             ((n :wat::core::i64) (:wat::core::length sorted)))
            (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string n))))
    "##;
    assert_eq!(run(src), vec!["0".to_string()]);
}

#[test]
fn sort_by_tuple_first_field_key() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<(wat::core::i64,wat::core::String)>)
              (:wat::core::Vector :(wat::core::i64,wat::core::String)
                (:wat::core::Tuple 30 "alice")
                (:wat::core::Tuple 25 "carol")
                (:wat::core::Tuple 28 "bob")))
             ((sorted :wat::core::Vector<(wat::core::i64,wat::core::String)>)
              (:wat::core::sort-by xs
                (:wat::core::lambda ((a :(wat::core::i64,wat::core::String)) (b :(wat::core::i64,wat::core::String)) -> :wat::core::bool)
                  (:wat::core::< (:wat::core::first a) (:wat::core::first b))))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::string::join ","
                (:wat::core::map sorted
                  (:wat::core::lambda ((p :(wat::core::i64,wat::core::String)) -> :wat::core::String)
                    (:wat::core::second p)))))))
    "##;
    assert_eq!(run(src), vec!["carol,bob,alice".to_string()]);
}

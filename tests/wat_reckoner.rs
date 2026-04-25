//! Arc 053 slice 3 — Reckoner as native wat value.

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
fn reckoner_discrete_construct_dims_labels() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((labels :Vec<String>) (:wat::core::vec :String "up" "down"))
             ((r :wat::holon::Reckoner)
              (:wat::holon::Reckoner/new-discrete "test-rec" 256 100 labels))
             ((d :i64) (:wat::holon::Reckoner/dims r))
             ((label-list :Vec<i64>) (:wat::holon::Reckoner/labels r))
             ((nlabels :i64) (:wat::core::length label-list)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if
                (:wat::core::and (:wat::core::= d 256) (:wat::core::= nlabels 2))
                -> :String "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

#[test]
fn reckoner_observe_then_predict() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((labels :Vec<String>) (:wat::core::vec :String "up" "down"))
             ((r :wat::holon::Reckoner)
              ;; Tiny recalib_interval=1 so discriminants update after every observe.
              (:wat::holon::Reckoner/new-discrete "rec" 256 1 labels))
             ((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((u1 :()) (:wat::holon::Reckoner/observe r v 0 1.0))
             ((u2 :()) (:wat::holon::Reckoner/observe r v 1 1.0))
             ((pred :(Vec<(i64,f64)>,Option<i64>,f64,f64))
              (:wat::holon::Reckoner/predict r v))
             ((conviction :f64) (:wat::core::third pred)))
            ;; Predict returns a tuple — we just verify the call ran
            ;; and conviction is a valid f64 (>= 0). Discriminants may
            ;; not be fully resolved after two observations; we don't
            ;; assert on score shape.
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::>= conviction 0.0) -> :String "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

#[test]
fn reckoner_continuous_construct() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((r :wat::holon::Reckoner)
              (:wat::holon::Reckoner/new-continuous "cont" 256 100 0.0 16))
             ((d :i64) (:wat::holon::Reckoner/dims r)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= d 256) -> :String "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

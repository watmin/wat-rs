//! Arc 053 slices 4 + 5 — Engram + EngramLibrary as native wat values.

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
fn library_construct_empty() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((lib :wat::holon::EngramLibrary) (:wat::holon::EngramLibrary/new 10000))
             ((n :wat::core::i64) (:wat::holon::EngramLibrary/len lib)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= n 0) -> :wat::core::String "empty" "non-empty"))))
    "##;
    assert_eq!(run(src), vec!["empty".to_string()]);
}

#[test]
fn library_add_subspace_then_count() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((lib :wat::holon::EngramLibrary) (:wat::holon::EngramLibrary/new 10000))
             ((sub :wat::holon::OnlineSubspace) (:wat::holon::OnlineSubspace/new 10000 4))
             ((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ;; Train at least once so the subspace is non-trivial.
             ((r :wat::core::f64) (:wat::holon::OnlineSubspace/update sub v))
             ((u :wat::core::unit) (:wat::holon::EngramLibrary/add lib "pattern-a" sub))
             ((n :wat::core::i64) (:wat::holon::EngramLibrary/len lib))
             ((found :wat::core::bool) (:wat::holon::EngramLibrary/contains lib "pattern-a"))
             ((missing :wat::core::bool) (:wat::holon::EngramLibrary/contains lib "absent")))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if
                (:wat::core::and (:wat::core::= n 1)
                  (:wat::core::and found (:wat::core::not missing))) -> :wat::core::String
                "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

#[test]
fn library_match_returns_named_pairs() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((lib :wat::holon::EngramLibrary) (:wat::holon::EngramLibrary/new 10000))
             ((sub :wat::holon::OnlineSubspace) (:wat::holon::OnlineSubspace/new 10000 4))
             ((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((r :wat::core::f64) (:wat::holon::OnlineSubspace/update sub v))
             ((u :wat::core::unit) (:wat::holon::EngramLibrary/add lib "alpha" sub))
             ;; Match against the same vector — should return 1 pair (name, residual).
             ((matches :Vec<(wat::core::String,wat::core::f64)>)
              (:wat::holon::EngramLibrary/match-vec lib v 5 5))
             ((nmatches :wat::core::i64) (:wat::core::length matches)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= nmatches 1) -> :wat::core::String "one-match" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["one-match".to_string()]);
}

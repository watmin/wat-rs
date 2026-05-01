//! `:wat::std::stat::*` — mean, variance, stddev.
//!
//! Surfaced by holon-lab-trading arc 026 slice 9 + slice 10 (Hurst
//! R/S, DFA, variance ratio all want windowed stats). Universal
//! enough to live in core stdlib. Population convention (numpy
//! default `ddof=0`); :wat::core::Option<wat::core::f64> for all three with None on empty
//! input (matches f64::min-of / max-of's reduction-empty pattern).

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
fn mean_known_input() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::f64>) (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0))
             ((m :wat::core::Option<wat::core::f64>) (:wat::std::stat::mean xs))
             ((v :wat::core::f64)
              (:wat::core::match m -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["3".to_string()]);
}

#[test]
fn mean_empty_is_none() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::f64>) (:wat::core::Vector :wat::core::f64))
             ((m :wat::core::Option<wat::core::f64>) (:wat::std::stat::mean xs))
             ((label :wat::core::String)
              (:wat::core::match m -> :wat::core::String
                ((:wat::core::Some _) "some") (:wat::core::None "none"))))
            (:wat::io::IOWriter/println stdout label)))
    "##;
    assert_eq!(run(src), vec!["none".to_string()]);
}

#[test]
fn variance_population_known_input() {
    // {1, 2, 3, 4, 5}: mean=3, var = ((1-3)² + (2-3)² + 0 + (4-3)² + (5-3)²) / 5
    //                       = (4+1+0+1+4)/5 = 2.0.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::f64>) (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0))
             ((v :wat::core::f64)
              (:wat::core::match (:wat::std::stat::variance xs) -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["2".to_string()]);
}

#[test]
fn variance_single_point_zero() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::f64>) (:wat::core::Vector :wat::core::f64 7.0))
             ((v :wat::core::f64)
              (:wat::core::match (:wat::std::stat::variance xs) -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))))
            (:wat::io::IOWriter/println stdout (:wat::core::f64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["0".to_string()]);
}

#[test]
fn stddev_known_input() {
    // {1, 2, 3, 4, 5}: variance=2, stddev = sqrt(2) ≈ 1.4142...
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((xs :wat::core::Vector<wat::core::f64>) (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0))
             ((sd :wat::core::f64)
              (:wat::core::match (:wat::std::stat::stddev xs) -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> sd 1.41) -> :wat::core::String
                "ok" "bad"))))
    "##;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

//! Arc 056 carry-along — `:wat::core::not=` + Enum equality.
//!
//! Clojure-tradition inequality. Shares the polymorphic-compare
//! inference rules with `=`; the runtime is `not(=)`. Also fills the
//! prior gap where `=` couldn't compare two `Value::Enum` values
//! (added an Enum arm to `values_equal`).

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
fn not_eq_i64_true_when_different() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout
            (:wat::core::if (:wat::core::not= 3 5) -> :String
              "yes" "no")))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

#[test]
fn not_eq_i64_false_when_same() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout
            (:wat::core::if (:wat::core::not= 7 7) -> :String
              "yes" "no")))
    "##;
    assert_eq!(run(src), vec!["no".to_string()]);
}

#[test]
fn not_eq_f64_cross_numeric_coerce() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout
            (:wat::core::if (:wat::core::not= 3 3.0) -> :String
              "yes" "no")))
    "##;
    assert_eq!(run(src), vec!["no".to_string()]);
}

#[test]
fn eq_on_enum_unit_variants() {
    let src = r##"
        (:wat::core::enum :my::Color :Red :Blue :Green)
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((a :my::Color) :my::Color::Red)
             ((b :my::Color) :my::Color::Red)
             ((c :my::Color) :my::Color::Blue))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::and
                                (:wat::core::= a b)
                                (:wat::core::not= a c))
                              -> :String
                "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

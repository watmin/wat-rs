//! Arc 053 slice 1 — Vector-tier algebra primitives.
//!
//! Coverage: vector-bind, vector-bundle, vector-blend, vector-permute
//! over `Value::Vector` inputs (post-arc-052).

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
fn vector_bind_roundtrip() {
    // bind(a, b) == bind(a, b) — deterministic.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "a")))
             ((vb :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "b")))
             ((c1 :wat::holon::Vector) (:wat::holon::vector-bind va vb))
             ((c2 :wat::holon::Vector) (:wat::holon::vector-bind va vb)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= c1 c2) -> :String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["yes".to_string()]);
}

#[test]
fn vector_bundle_singleton_returns_input() {
    // Bundle of a single vector returns ~the input (sign of the only contributor).
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((bundled :wat::holon::Vector)
              (:wat::holon::vector-bundle (:wat::core::vec :wat::holon::Vector va)))
             ;; Cosine should be ~1.0 (same sign pattern).
             ((c :f64) (:wat::holon::cosine va bundled)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> c 0.99) -> :String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["near-1".to_string()]);
}

#[test]
fn vector_blend_weighted() {
    // blend(a, a, 1.0, 0.0) should equal a.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((vb :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "y")))
             ((blended :wat::holon::Vector) (:wat::holon::vector-blend va vb 1.0 0.0))
             ((c :f64) (:wat::holon::cosine va blended)))
            ;; Pure a-weight should give very high cosine to a.
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::> c 0.95) -> :String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["near-1".to_string()]);
}

#[test]
fn vector_permute_changes_vector() {
    // permute(v, k) for k != 0 should differ from v.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((shifted :wat::holon::Vector) (:wat::holon::vector-permute va 5)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= va shifted) -> :String "same" "differs"))))
    "##;
    assert_eq!(run(src), vec!["differs".to_string()]);
}

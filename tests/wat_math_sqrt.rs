//! Carry-along — `:wat::std::math::sqrt`.
//!
//! Surfaced by holon-lab-trading arc 026 slice 4 (Bollinger's
//! RollingStddev needs `var.sqrt()`). Same shape as ln/exp/sin/cos —
//! single-method f64 unary; mirrors the existing dispatch.

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
fn sqrt_perfect_square() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout
            (:wat::core::f64::to-string (:wat::std::math::sqrt 16.0))))
    "##;
    assert_eq!(run(src), vec!["4".to_string()]);
}

#[test]
fn sqrt_of_zero() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout
            (:wat::core::f64::to-string (:wat::std::math::sqrt 0.0))))
    "##;
    assert_eq!(run(src), vec!["0".to_string()]);
}

#[test]
fn sqrt_round_trip_with_square() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((x :f64) 7.5)
             ((rt :f64) (:wat::std::math::sqrt (:wat::core::* x x))))
            (:wat::io::IOWriter/println stdout
              (:wat::core::f64::to-string rt))))
    "##;
    assert_eq!(run(src), vec!["7.5".to_string()]);
}

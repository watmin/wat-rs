//! Arc 056 carry-along — `:wat::core::not=` + Enum equality.
//!
//! Clojure-tradition inequality. Shares the polymorphic-compare
//! inference rules with `=`; the runtime is `not(=)`. Also fills the
//! prior gap where `=` couldn't compare two `Value::Enum` values
//! (added an Enum arm to `values_equal`).

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::thread_io::{install_ambient_stdio, uninstall_ambient_stdio, AmbientStdio};

fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(read_fd));
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));
    (reader, writer)
}

fn drain_lines(reader: &Arc<dyn WatReader>) -> Vec<String> {
    let bytes = reader
        .read_all(wat::span::Span::unknown())
        .expect("read-all");
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

fn run(src: &str) -> Vec<String> {
    let _ = uninstall_ambient_stdio();
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    invoke_user_main(&world, Vec::new()).expect("main");
    let _ = uninstall_ambient_stdio();
    drain_lines(&stdout_capture)
}

#[test]
fn not_eq_i64_true_when_different() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println
            (:wat::core::if (:wat::core::not= 3 5) -> :wat::core::String
              "yes" "no")))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

#[test]
fn not_eq_i64_false_when_same() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println
            (:wat::core::if (:wat::core::not= 7 7) -> :wat::core::String
              "yes" "no")))
    "##;
    assert_eq!(run(src), vec!["\"no\"".to_string()]);
}

#[test]
fn not_eq_f64_cross_numeric_coerce() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println
            (:wat::core::if (:wat::core::not= 3 3.0) -> :wat::core::String
              "yes" "no")))
    "##;
    assert_eq!(run(src), vec!["\"no\"".to_string()]);
}

#[test]
fn eq_on_enum_unit_variants() {
    let src = r##"
        (:wat::core::enum :my::Color :Red :Blue :Green)
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a :my::Color::Red
             b :my::Color::Red
             c :my::Color::Blue]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::and
                                (:wat::core::= a b)
                                (:wat::core::not= a c))
                              -> :wat::core::String
                "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

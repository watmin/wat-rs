//! Arc 056 — `:wat::core::sort-by`.
//!
//! User-supplied less-than predicate drives the ordering. Asc vs desc
//! is encoded by which way the predicate compares; key-extraction is
//! the predicate composing inner accessors. Common Lisp tradition.

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
fn sort_by_ascending_i64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::i64 3 1 4 1 5 9 2 6)
             sorted
              (:wat::core::sort-by xs
                (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
                  (:wat::core::< a b)))]
            (:wat::kernel::println
              (:wat::core::string::join ","
                (:wat::core::map sorted
                  (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::String
                    (:wat::core::i64::to-string n)))))))
    "##;
    assert_eq!(run(src), vec!["\"1,1,2,3,4,5,6,9\"".to_string()]);
}

#[test]
fn sort_by_descending_f64() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::f64 1.5 0.5 2.5 1.0)
             sorted
              (:wat::core::sort-by xs
                (:wat::core::fn [a <- :wat::core::f64 b <- :wat::core::f64] -> :wat::core::bool
                  (:wat::core::> a b)))]
            (:wat::kernel::println
              (:wat::core::string::join ","
                (:wat::core::map sorted
                  (:wat::core::fn [x <- :wat::core::f64] -> :wat::core::String
                    (:wat::core::f64::to-string x)))))))
    "##;
    assert_eq!(run(src), vec!["\"2.5,1.5,1,0.5\"".to_string()]);
}

#[test]
fn sort_by_string() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::String "banana" "apple" "cherry")
             sorted
              (:wat::core::sort-by xs
                (:wat::core::fn [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::bool
                  (:wat::core::< a b)))]
            (:wat::kernel::println (:wat::core::string::join "," sorted))))
    "##;
    assert_eq!(run(src), vec!["\"apple,banana,cherry\"".to_string()]);
}

#[test]
fn sort_by_empty_vec() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::i64)
             sorted
              (:wat::core::sort-by xs
                (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
                  (:wat::core::< a b)))
             n (:wat::core::length sorted)]
            (:wat::kernel::println n)))
    "##;
    assert_eq!(run(src), vec!["0".to_string()]);
}

#[test]
fn sort_by_tuple_first_field_key() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs
              (:wat::core::Vector :(wat::core::i64,wat::core::String)
                (:wat::core::Tuple 30 "alice")
                (:wat::core::Tuple 25 "carol")
                (:wat::core::Tuple 28 "bob"))
             sorted
              (:wat::core::sort-by xs
                (:wat::core::fn [a <- :(wat::core::i64,wat::core::String) b <- :(wat::core::i64,wat::core::String)] -> :wat::core::bool
                  (:wat::core::< (:wat::core::first a) (:wat::core::first b))))]
            (:wat::kernel::println
              (:wat::core::string::join ","
                (:wat::core::map sorted
                  (:wat::core::fn [p <- :(wat::core::i64,wat::core::String)] -> :wat::core::String
                    (:wat::core::second p)))))))
    "##;
    assert_eq!(run(src), vec!["\"carol,bob,alice\"".to_string()]);
}

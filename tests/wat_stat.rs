//! `:wat::std::stat::*` — mean, variance, stddev.
//!
//! Surfaced by holon-lab-trading arc 026 slice 9 + slice 10 (Hurst
//! R/S, DFA, variance ratio all want windowed stats). Universal
//! enough to live in core stdlib. Population convention (numpy
//! default `ddof=0`); :wat::core::Option<wat::core::f64> for all three with None on empty
//! input (matches f64::min-of / max-of's reduction-empty pattern).

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
fn mean_known_input() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0)
             m (:wat::std::stat::mean xs)
             v
              (:wat::core::match m -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))]
            (:wat::kernel::println (:wat::core::f64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["\"3\"".to_string()]);
}

#[test]
fn mean_empty_is_none() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::f64)
             m (:wat::std::stat::mean xs)
             label
              (:wat::core::match m -> :wat::core::String
                ((:wat::core::Some _) "some") (:wat::core::None "none"))]
            (:wat::kernel::println label)))
    "##;
    assert_eq!(run(src), vec!["\"none\"".to_string()]);
}

#[test]
fn variance_population_known_input() {
    // {1, 2, 3, 4, 5}: mean=3, var = ((1-3)² + (2-3)² + 0 + (4-3)² + (5-3)²) / 5
    //                       = (4+1+0+1+4)/5 = 2.0.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0)
             v
              (:wat::core::match (:wat::std::stat::variance xs) -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))]
            (:wat::kernel::println (:wat::core::f64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["\"2\"".to_string()]);
}

#[test]
fn variance_single_point_zero() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::f64 7.0)
             v
              (:wat::core::match (:wat::std::stat::variance xs) -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))]
            (:wat::kernel::println (:wat::core::f64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["\"0\"".to_string()]);
}

#[test]
fn stddev_known_input() {
    // {1, 2, 3, 4, 5}: variance=2, stddev = sqrt(2) ≈ 1.4142...
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [xs (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0)
             sd
              (:wat::core::match (:wat::std::stat::stddev xs) -> :wat::core::f64
                ((:wat::core::Some x) x) (:wat::core::None -1.0))]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> sd 1.41) -> :wat::core::String
                "ok" "bad"))))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

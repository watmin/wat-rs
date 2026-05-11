//! Arc 053 slice 3 — Reckoner as native wat value.

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
fn reckoner_discrete_construct_dims_labels() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [labels
              (:wat::core::Vector :wat::holon::HolonAST
                (:wat::holon::Atom "up")
                (:wat::holon::Atom "down"))
             r
              (:wat::holon::Reckoner/new-discrete "test-rec" 10000 100 labels)
             d (:wat::holon::Reckoner/dims r)
             label-list (:wat::holon::Reckoner/labels r)
             nlabels (:wat::core::length label-list)]
            (:wat::kernel::println
              (:wat::core::if
                (:wat::core::and (:wat::core::= d 10000) (:wat::core::= nlabels 2))
                -> :wat::core::String "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

#[test]
fn reckoner_observe_then_predict() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [labels
              (:wat::core::Vector :wat::holon::HolonAST
                (:wat::holon::Atom "up")
                (:wat::holon::Atom "down"))
             r
              ;; Tiny recalib_interval=1 so discriminants update after every observe.
              (:wat::holon::Reckoner/new-discrete "rec" 10000 1 labels)
             v (:wat::holon::encode (:wat::holon::Atom "x"))
             u1 (:wat::holon::Reckoner/observe r v 0 1.0)
             u2 (:wat::holon::Reckoner/observe r v 1 1.0)
             pred
              (:wat::holon::Reckoner/predict r v)
             conviction (:wat::core::third pred)]
            ;; Predict returns a tuple — we just verify the call ran
            ;; and conviction is a valid f64 (>= 0). Discriminants may
            ;; not be fully resolved after two observations; we don't
            ;; assert on score shape.
            (:wat::kernel::println
              (:wat::core::if (:wat::core::>= conviction 0.0) -> :wat::core::String "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

#[test]
fn reckoner_continuous_construct() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [r
              (:wat::holon::Reckoner/new-continuous "cont" 10000 100 0.0 16)
             d (:wat::holon::Reckoner/dims r)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= d 10000) -> :wat::core::String "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

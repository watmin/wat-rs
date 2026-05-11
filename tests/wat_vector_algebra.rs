//! Arc 053 slice 1 — Vector-tier algebra primitives.
//!
//! Coverage: vector-bind, vector-bundle, vector-blend, vector-permute
//! over `Value::Vector` inputs (post-arc-052).

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
fn vector_bind_roundtrip() {
    // bind(a, b) == bind(a, b) — deterministic.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "a"))
             vb (:wat::holon::encode (:wat::holon::Atom "b"))
             c1 (:wat::holon::vector-bind va vb)
             c2 (:wat::holon::vector-bind va vb)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= c1 c2) -> :wat::core::String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

#[test]
fn vector_bundle_singleton_returns_input() {
    // Bundle of a single vector returns ~the input (sign of the only contributor).
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "x"))
             bundled
              (:wat::holon::vector-bundle (:wat::core::Vector :wat::holon::Vector va))
             ;; Cosine should be ~1.0 (same sign pattern).
             c (:wat::holon::cosine va bundled)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["\"near-1\"".to_string()]);
}

#[test]
fn vector_blend_weighted() {
    // blend(a, a, 1.0, 0.0) should equal a.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "x"))
             vb (:wat::holon::encode (:wat::holon::Atom "y"))
             blended (:wat::holon::vector-blend va vb 1.0 0.0)
             c (:wat::holon::cosine va blended)]
            ;; Pure a-weight should give very high cosine to a.
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> c 0.95) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["\"near-1\"".to_string()]);
}

#[test]
fn vector_permute_changes_vector() {
    // permute(v, k) for k != 0 should differ from v.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "x"))
             shifted (:wat::holon::vector-permute va 5)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= va shifted) -> :wat::core::String "same" "differs"))))
    "##;
    assert_eq!(run(src), vec!["\"differs\"".to_string()]);
}

//! Arc 053 slices 4 + 5 — Engram + EngramLibrary as native wat values.

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
fn library_construct_empty() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [lib (:wat::holon::EngramLibrary/new 10000)
             n (:wat::holon::EngramLibrary/len lib)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= n 0) -> :wat::core::String "empty" "non-empty"))))
    "##;
    assert_eq!(run(src), vec!["\"empty\"".to_string()]);
}

#[test]
fn library_add_subspace_then_count() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [lib (:wat::holon::EngramLibrary/new 10000)
             sub (:wat::holon::OnlineSubspace/new 10000 4)
             v (:wat::holon::encode (:wat::holon::Atom "x"))
             ;; Train at least once so the subspace is non-trivial.
             r (:wat::holon::OnlineSubspace/update sub v)
             u (:wat::holon::EngramLibrary/add lib "pattern-a" sub)
             n (:wat::holon::EngramLibrary/len lib)
             found (:wat::holon::EngramLibrary/contains lib "pattern-a")
             missing (:wat::holon::EngramLibrary/contains lib "absent")]
            (:wat::kernel::println
              (:wat::core::if
                (:wat::core::and (:wat::core::= n 1)
                  (:wat::core::and found (:wat::core::not missing))) -> :wat::core::String
                "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

#[test]
fn library_match_returns_named_pairs() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [lib (:wat::holon::EngramLibrary/new 10000)
             sub (:wat::holon::OnlineSubspace/new 10000 4)
             v (:wat::holon::encode (:wat::holon::Atom "x"))
             r (:wat::holon::OnlineSubspace/update sub v)
             u (:wat::holon::EngramLibrary/add lib "alpha" sub)
             ;; Match against the same vector — should return 1 pair (name, residual).
             matches
              (:wat::holon::EngramLibrary/match-vec lib v 5 5)
             nmatches (:wat::core::length matches)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= nmatches 1) -> :wat::core::String "one-match" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["\"one-match\"".to_string()]);
}

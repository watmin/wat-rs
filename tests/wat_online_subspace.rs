//! Arc 053 slice 2 — OnlineSubspace as native wat value.

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
fn subspace_construct_dim_k_n_zero() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [s (:wat::holon::OnlineSubspace/new 10000 16)
             d (:wat::holon::OnlineSubspace/dim s)
             k (:wat::holon::OnlineSubspace/k s)
             n (:wat::holon::OnlineSubspace/n s)]
            (:wat::kernel::println
              (:wat::core::if
                (:wat::core::and (:wat::core::= d 10000)
                  (:wat::core::and (:wat::core::= k 16) (:wat::core::= n 0))) -> :wat::core::String
                "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

#[test]
fn subspace_update_increments_n_and_returns_residual() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [s (:wat::holon::OnlineSubspace/new 10000 4)
             v (:wat::holon::encode (:wat::holon::Atom "x"))
             residual (:wat::holon::OnlineSubspace/update s v)
             n (:wat::holon::OnlineSubspace/n s)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= n 1) -> :wat::core::String "incremented" "stuck"))))
    "##;
    assert_eq!(run(src), vec!["\"incremented\"".to_string()]);
}

#[test]
fn subspace_eigenvalues_returns_k_floats() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [s (:wat::holon::OnlineSubspace/new 10000 8)
             eigs (:wat::holon::OnlineSubspace/eigenvalues s)
             len (:wat::core::length eigs)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= len 8) -> :wat::core::String "k-eigs" "wrong-len"))))
    "##;
    assert_eq!(run(src), vec!["\"k-eigs\"".to_string()]);
}

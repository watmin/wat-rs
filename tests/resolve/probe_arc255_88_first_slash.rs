//! Stone 255.88. A slashed spelling's first `/` is the only partition.
//! `foo/bar` binds and is read. `u/a/b` is callable. The pathological
//! program prints 42. A macro's internal `completely/insane` does not
//! capture the caller's. `wat.core//` reads as namespace `wat.core`, name `/`.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_file};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::scope::Identifier;
use wat::services::{install_ambient_stdio, take_ambient_stdio, AmbientStdio};

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
        .read_all(wat::rust_caller_span!())
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

fn run_file(fixture_path: &str) -> Vec<String> {
    let _ = take_ambient_stdio();
    let world = startup_from_file(fixture_path).expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    invoke_user_main(&world, Vec::new()).expect("main");
    let _ = take_ambient_stdio();
    drain_lines(&stdout_capture)
}

#[test]
fn the_first_slash_is_the_only_partition() {
    let slash = Identifier::bare("wat.core//");
    assert_eq!(slash.namespace(), "wat.core");
    assert_eq!(slash.method(), "/");
    let out = run_file("tests/resolve/probe_arc255_88_first_slash.wat");
    assert_eq!(out, vec!["0", "7", "42", "1", "99"]);
}

//! Arc 255.77 — cutover stone 3, "the UUID type goes home". Pins
//! `:wat::telemetry::framing-floor-of`'s numbers for each of the four FIXED-VALUE branches
//! (i64/f64/Uuid/bool), measured against the UNMODIFIED draw (`c16915636`) before this stone's
//! rename: 38, 42, 55, 24 (in that order — see the co-located `.wat` fixture). The brief's
//! STOP-2: a pinned number changing means the trap fired (the rename broke the `ast-name`
//! string compare and the branch silently fell to `:else 0`) — this test is the gate that would
//! have caught it, and the fixture's `:wat::core::Uuid` field spelling was itself one of this
//! stone's own corpus conversions (`wat-scripts/fixes/uuid-type-goes-home.wat`), so this file
//! exercises the POST-rename spelling while still asserting the PRE-rename numbers.
use wat::freeze::{invoke_user_main, startup_from_file};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::services::{install_ambient_stdio, take_ambient_stdio, AmbientStdio};
use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;

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

#[test]
fn framing_floor_of_pinned_numbers_hold_across_the_uuid_rename() {
    let _ = take_ambient_stdio();
    let world = startup_from_file("tests/types/probe_arc255_77_framing_floor_pin.wat")
        .expect("startup");
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
    let lines = drain_lines(&stdout_capture);
    assert_eq!(
        lines,
        vec!["38", "42", "55", "24"],
        "framing-floor-of's pinned numbers (i64, f64, Uuid, bool, in that order) must be \
         IDENTICAL before and after the uuid-type-goes-home rename + the type-equal? trap cure"
    );
}

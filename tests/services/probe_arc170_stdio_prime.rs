//! Arc 170 stdio-as-defservice — PHASE 1, the P1 UNIT PROOF.
//!
//! The three stdio streams are reborn as `defservice`s (the PRIMES,
//! `:wat::kernel::{stdout,stderr,stdin}-svc'`), coexisting with the hand-rolled path. This probe
//! proves the primed StdOut'/StdIn' services round-trip over a CONTROLLED fd (a pipe pair built here
//! in Rust), driven ENTIRELY through the generated client face — the same face any caller uses:
//!
//!   - `primed_stdout_write_line_lands_bytes` — start `stdout-svc'` on a pipe write-fd, `connect'`,
//!     `write-line` two lines; the exact bytes land on the pipe read-end (the fd was born inside the
//!     service's kernel `::init` via `IOWriter/from-fd`, dup-then-own).
//!   - `primed_stdin_read_line_returns_line` — feed one line into a pipe, start `stdin-svc'` on the
//!     read-fd, `read-line` → `ReadLineResponse::Line "…"`.
//!   - `primed_stdin_eof_is_matchable` — close the write-end (no writers → EOF), `read-line` →
//!     `ReadLineResponse::Eof` (the no-hidden-failures upgrade: EOF is a MATCHABLE value, not a panic
//!     that kills the serve loop — R55/R57).
//!
//! The wat driver logic lives in the co-located fixture `probe_arc170_stdio_prime.wat`
//! (`:user::run-stdout` / `:user::run-stdin`, each taking the fd as an i64 arg — the fd number is a
//! pure i64, born in Rust, passed in). The fixture is `:user::`-namespaced: it cannot define
//! `:wat::kernel::` services nor call the kernel-restricted `from-fd` directly; it reaches `from-fd`
//! legitimately only THROUGH the real stdlib service's generated `::init`.
//!
//! Run: cargo test --release -p wat --test services probe_arc170_stdio_prime

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};
use wat::span::Span;

fn probe_span() -> Span {
    Span::new(Arc::new("probe_arc170_stdio_prime".to_string()), 1, 1)
}

/// A fresh anonymous pipe. Returns (read-end, write-end) as owned fds (Drop closes each).
fn make_pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [0i32; 2];
    let rc = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(rc, 0, "libc::pipe() failed: {}", std::io::Error::last_os_error());
    // SAFETY: pipe() returned 0, so fds[0]/fds[1] are freshly-owned fds.
    unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) }
}

/// Read exactly `n` bytes from `fd` (the bytes are already present in the pipe — the primed
/// write-line round-trips synchronously, so both writes completed before the wat call returned).
fn read_exact_n(fd: i32, n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    let mut got = 0usize;
    while got < n {
        let r = unsafe { libc::read(fd, buf[got..].as_mut_ptr() as *mut libc::c_void, n - got) };
        assert!(r > 0, "libc::read returned {r} after {got}/{n} bytes: {}", std::io::Error::last_os_error());
        got += r as usize;
    }
    buf
}

#[test]
fn primed_stdout_write_line_lands_bytes() {
    let world = startup_beside(file!()).expect("arc 170 fixture freezes");
    let run_stdout = world
        .symbols()
        .get(":user::run-stdout")
        .expect(":user::run-stdout in fixture")
        .clone();

    let (r, w) = make_pipe();
    // The service dups `w` inside its `::init` (from-fd) during /start; `w` must stay open across the
    // call. write-line round-trips synchronously, so both lines are in the pipe on return.
    let acks = apply_function(
        run_stdout,
        vec![Value::i64(w.as_raw_fd() as i64)],
        world.symbols(),
        probe_span(),
    )
    .expect("run-stdout raised");
    assert!(
        matches!(acks, Value::i64(2)),
        "expected 2 WriteLineResponse::Ok acks; got {acks:?}"
    );

    let expected = b"primed-line-1\nprimed-line-2\n";
    let got = read_exact_n(r.as_raw_fd(), expected.len());
    assert_eq!(
        got, expected,
        "primed StdOut' write-line bytes must land on the pipe read-end (from-fd dup wrote through)"
    );
    // r, w drop here (OwnedFd → close).
}

#[test]
fn primed_stdin_read_line_returns_line() {
    let world = startup_beside(file!()).expect("arc 170 fixture freezes");
    let run_stdin = world
        .symbols()
        .get(":user::run-stdin")
        .expect(":user::run-stdin in fixture")
        .clone();

    let (r, w) = make_pipe();
    let msg = b"stdin-line-1\n";
    let wrote = unsafe { libc::write(w.as_raw_fd(), msg.as_ptr() as *const libc::c_void, msg.len()) };
    assert_eq!(wrote, msg.len() as isize, "pipe write failed");

    let got = apply_function(
        run_stdin,
        vec![Value::i64(r.as_raw_fd() as i64)],
        world.symbols(),
        probe_span(),
    )
    .expect("run-stdin raised");
    match got {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "stdin-line-1",
            "primed StdIn' read-line must return the fed line (newline-trimmed)"
        ),
        other => panic!("expected ReadLineResponse::Line \"stdin-line-1\"; got {other:?}"),
    }
    // keep `w` open across the call (else a premature EOF); drop at scope end.
    drop(w);
    drop(r);
}

#[test]
fn primed_stdin_eof_is_matchable() {
    let world = startup_beside(file!()).expect("arc 170 fixture freezes");
    let run_stdin = world
        .symbols()
        .get(":user::run-stdin")
        .expect(":user::run-stdin in fixture")
        .clone();

    let (r, w) = make_pipe();
    // Close the write-end BEFORE the read → no writers → the service's read-frame sees EOF, which
    // surfaces as the matchable ReadLineResponse::Eof (NOT a panic that kills the serve loop).
    drop(w);
    let got = apply_function(
        run_stdin,
        vec![Value::i64(r.as_raw_fd() as i64)],
        world.symbols(),
        probe_span(),
    )
    .expect("run-stdin raised");
    match got {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "EOF",
            "primed StdIn' EOF must surface as the matchable ReadLineResponse::Eof value"
        ),
        other => panic!("expected ReadLineResponse::Eof (\"EOF\"); got {other:?}"),
    }
    drop(r);
}

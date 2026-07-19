//! Stone 259-killed — STOP-1 seam test: truncated-frame path coverage.
//!
//! When a peer writes partial bytes (no newline) then closes the write end,
//! the framer accumulates the partial content, sees EOF (`n == 0` from the
//! io_uring read), and `comms::process::Receiver::recv()` returns
//! `Err(RecvError::Disconnected)` — no hang, no silent value.
//!
//! This was previously tested via `probe_ipc_framing_negatives.rs`
//! `truncated_frame_is_rejected_by_recv_prime` which spawned a WAT process
//! child using the annihilated `print-raw'` verb. With `print-raw'` removed,
//! this seam test directly exercises `comms::process::Receiver<String>` over
//! a raw pipe — no WAT runtime, no spawned process.
//!
//! Modeled on `tests/nursery/probe_arc259_comms_recv_multiline_frame.rs`.

use std::os::fd::{FromRawFd, OwnedFd};
use wat::comms::{RecvError, process::sender_receiver_from_split_fds};

/// Writing partial bytes (no `\n`) then closing the write end → `Disconnected`.
///
/// The framer never sees a newline so no `Frame(end)` is produced; when the
/// read side gets EOF (`n == 0`), `recv()` returns `Err(RecvError::Disconnected)`.
/// This proves the path does NOT hang and does NOT silently yield a value.
#[test]
fn truncated_frame_eof_returns_disconnected() {
    let mut fds = [0i32; 2];
    assert_eq!(
        unsafe { libc::pipe(fds.as_mut_ptr()) },
        0,
        "pipe(2) must succeed"
    );

    // Write a partial EDN value — no closing `}`, no `\n`.
    // rune:lint(no-inlined-edn) — input under test: a partial EDN byte sequence written then closed to exercise the truncated-frame path.
    let partial = b"{:a 1";
    let n = unsafe {
        libc::write(fds[1], partial.as_ptr() as *const libc::c_void, partial.len())
    };
    assert_eq!(n as usize, partial.len(), "write partial bytes");

    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let (sender, receiver) =
        sender_receiver_from_split_fds::<String>(read_fd, write_fd).expect("comms pair");

    // Close the write end — the receiver will see EOF on the next read.
    drop(sender);

    // recv() must return Disconnected, not block and not yield a value.
    match receiver.recv() {
        Ok(s) => panic!(
            "truncated frame: recv must return Err(Disconnected) when the write end \
             closes mid-frame (no newline); got Ok({s:?})"
        ),
        Err(RecvError::Disconnected) => { /* correct */ }
        Err(other) => panic!(
            "truncated frame: expected Err(RecvError::Disconnected); got Err({other:?})"
        ),
    }
}

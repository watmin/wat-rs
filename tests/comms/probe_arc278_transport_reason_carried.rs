//! Arc 278 no-hidden-failures — the TRANSPORT-tier twin of Mechanism A
//! (service-reply `Reply::Failed{cause}`) and the crash-reason plumbing
//! (`PeerRecvError::Crashed(String)`). `RecvError` (`src/comms/mod.rs`) had
//! no slot for a reason: a raw transport failure (io errno / invalid utf8 /
//! undecodable frame / malformed frame) collapsed via
//! `map_err(|_| RecvError::Disconnected)` to a mute `Disconnected` — a
//! caller could not tell a clean EOF from a wire that broke with a reason.
//!
//! RED at HEAD: writing an invalid-UTF-8 frame (bytes that cannot decode,
//! terminated with `\n` so the framer sees a complete frame) makes
//! `comms::process::Receiver::recv()` return a MUTE `Err(RecvError::Disconnected)`
//! — the caller cannot tell this apart from a genuine clean peer close.
//! GREEN once the frame-scan malformed / decode-error paths bind the
//! detail into `RecvError::Failed(reason)`.
//!
//! Modeled on `tests/comms/probe_truncated_frame_disconnects.rs`'s raw-pipe
//! harness (raw fds, no WAT runtime, no spawned process).

use std::os::fd::{FromRawFd, OwnedFd};
use wat::comms::{RecvError, process::sender_receiver_from_split_fds};

/// Writing an invalid-UTF-8, newline-terminated frame → the receiver's
/// `recv()` error must be `RecvError::Failed(reason)` with the decode
/// detail in `reason` (NOT a bare `Disconnected` / "peer closed" mask).
#[test]
fn invalid_utf8_frame_surfaces_failed_with_reason() {
    let mut fds = [0i32; 2];
    assert_eq!(
        unsafe { libc::pipe(fds.as_mut_ptr()) },
        0,
        "pipe(2) must succeed"
    );

    // Invalid UTF-8 (a lone continuation byte, 0x80, cannot start or
    // continue any valid UTF-8 sequence) followed by the frame terminator.
    let bad_frame: &[u8] = &[0x80, 0x81, b'\n'];
    let n = unsafe {
        libc::write(fds[1], bad_frame.as_ptr() as *const libc::c_void, bad_frame.len())
    };
    assert_eq!(n as usize, bad_frame.len(), "write invalid-UTF-8 frame");

    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let (sender, receiver) =
        sender_receiver_from_split_fds::<String>(read_fd, write_fd).expect("comms pair");
    // Keep the write end open — this is NOT a clean-EOF scenario; the
    // frame itself is malformed while the peer is still "alive". A clean
    // close (dropping sender first) is the no-regression case covered by
    // `probe_truncated_frame_disconnects.rs` and stays `Disconnected`.
    let _keep_alive = sender;

    match receiver.recv() {
        Ok(s) => panic!(
            "invalid-UTF-8 frame: recv must return Err(Failed(reason)), not decode to a \
             value; got Ok({s:?})"
        ),
        Err(RecvError::Failed(reason)) => {
            assert!(
                // rune:lint(loose-assert) — asserts RecvError::Failed's reason NAMES the utf-8
                // decode detail; the exact text wraps std::str::Utf8Error's Display (an impl
                // detail, not under test), not a deterministic value that owes assert_eq!.
                reason.to_lowercase().contains("utf-8") || reason.to_lowercase().contains("utf8"),
                "RecvError::Failed's reason must CONTAIN the utf-8/decode detail, not a generic \
                 message; got: {reason:?}"
            );
        }
        Err(other) => panic!(
            "THE LAW (wat never hides a failure) — transport-tier twin: an invalid-UTF-8 frame \
             must surface as Err(RecvError::Failed(reason)) carrying the decode detail, not a \
             mute mask. Got Err({other:?})"
        ),
    }
}

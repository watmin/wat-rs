//! Arc 259 — disconfirming probe: the comms fd reader must VALUE-FRAME, not
//! split on the first newline.
//!
//! THE GAP (the second framer): `read_framed_edn` (edn::render) value-frames the
//! ambient/channel WatReader path, but the comms io_uring path
//! (`comms/process.rs` `take_frame`, line 849) splits on the FIRST `'\n'` —
//! it assumes wat-edn is single-line (process.rs:51). So a multi-line EDN value
//! crossing a process peer (a child `pprintln`-ing a pretty map) is mis-framed:
//! `recv'` reads only the first physical line `{`.
//!
//! THE CONTRACT (one frame-finder, both readers): extract `next_complete_frame`
//! (accumulate lines until the buffer parses a complete EDN value, line-granular,
//! `DEFAULT_MAX_FRAME_BYTES`-bounded) and route BOTH `read_framed_edn` and the
//! comms `take_frame` through it. Then the comms `recv` value-frames too.
//!
//! `Receiver<String>` is the probe vehicle: `String::from_wire` is raw
//! passthrough, so `recv()` returns exactly the framed bytes — we observe the
//! framing directly (truncated `{` vs the whole multi-line value).
//!
//! RED at HEAD: `take_frame` splits the first `'\n'` → `recv()` yields `"{"`.
//! GREEN after the one-framer extraction: `recv()` yields `"{\n  :a 1\n}"`.

use std::os::fd::{FromRawFd, OwnedFd};
use wat::comms::process::sender_receiver_from_split_fds;

#[test]
fn comms_recv_value_frames_a_multiline_edn_value() {
    let mut fds = [0i32; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0, "pipe(2)");

    // Raw-write a MULTI-LINE EDN value to the pipe — exactly what a child's
    // `pprintln` puts on its stdout. Bypass the compact Sender; this is the
    // raw byte stream the comms reader must frame.
    // rune:lint(no-inlined-edn) — input under test: a multi-line EDN value raw-written to the pipe as the byte stream the comms reader must frame.
    let msg = "{\n  :a 1\n}\n";
    let n = unsafe {
        libc::write(fds[1], msg.as_ptr() as *const libc::c_void, msg.len())
    };
    assert_eq!(n as usize, msg.len(), "write the whole multi-line frame");

    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let (sender, receiver) =
        sender_receiver_from_split_fds::<String>(read_fd, write_fd).expect("comms pair");
    drop(sender); // close the write end → recv sees the frame, then EOF

    // RED at HEAD: the first-`\n` split yields just "{".
    // GREEN: value-framed → the whole multi-line value (terminating \n stripped).
    match receiver.recv() {
        Ok(s) => assert_eq!(
            // rune:lint(no-inlined-edn) — is the EDN tooling correct: the framer's exact bytes are under test; assert_edn_eq is whitespace-blind and would not catch a mangled frame.
            s, "{\n  :a 1\n}",
            "comms recv must value-frame the WHOLE multi-line EDN value, \
             not split on the first newline (got the truncated prefix)"
        ),
        Err(e) => panic!("recv failed: {:?}", e),
    }
}

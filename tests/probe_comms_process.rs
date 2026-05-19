//! Arc 214 Slice 3 Stone A smoke probe — verify io_uring bytes round-trip.
//!
//! Six tests:
//!   1. pair() constructs successfully
//!   2. single-frame round-trip preserves bytes
//!   3. FIFO ordering across multiple sends
//!   4. sender drop wakes recv with Err(RecvError)
//!   5. accumulator correctly splits two frames received in one read
//!   6. large frame spans multiple io_uring reads
//!
//! Stone A is NOT cascade-aware (Stone B). Tests do NOT exercise
//! substrate shutdown — they exercise io_uring + framing only.

use std::thread;
use std::time::Duration;

use wat::comms::process::pair;
use wat::comms::RecvError;

#[test]
fn probe_slice3a_pair_constructs_successfully() {
    // Verifies the libc::pipe → OwnedFd wrapping path works at all.
    let result = pair();
    assert!(result.is_ok(), "pair() must return Ok; got {:?}", result.err());
    let (_tx, _rx) = result.expect("pair");
    // Drop closes both fds; no fd leak. (Verified statically by OwnedFd
    // Drop impl; not asserted at runtime.)
}

#[test]
fn probe_slice3a_single_frame_round_trip() {
    // Verifies the core contract: bytes sent are bytes received.
    let (tx, rx) = pair().expect("pair");
    tx.send(b"hello").expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent frame");
    assert_eq!(got, b"hello", "received bytes must equal sent bytes");
}

#[test]
fn probe_slice3a_fifo_ordering_preserved_across_sends() {
    // Verifies that N sends followed by N recvs preserve order.
    let (tx, rx) = pair().expect("pair");
    tx.send(b"first").expect("send 1");
    tx.send(b"second").expect("send 2");
    tx.send(b"third").expect("send 3");
    assert_eq!(rx.recv().expect("recv 1"), b"first");
    assert_eq!(rx.recv().expect("recv 2"), b"second");
    assert_eq!(rx.recv().expect("recv 3"), b"third");
}

#[test]
fn probe_slice3a_sender_drop_wakes_recv_with_err() {
    // Verifies that dropping the Sender causes recv to return Err(RecvError)
    // (EOF on the pipe; io_uring Read returns 0). The Sender is dropped
    // on a separate thread to ensure recv() is genuinely blocked when the
    // drop happens.
    let (tx, rx) = pair().expect("pair");
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        drop(tx);
    });
    let result = rx.recv();
    assert_eq!(
        result,
        Err(RecvError),
        "recv must return Err(RecvError) after sender drop (EOF)"
    );
    handle.join().expect("sender-drop thread");
}

#[test]
fn probe_slice3a_accumulator_splits_two_frames_from_one_read() {
    // Verifies the accumulator correctness: when the sender writes two
    // newline-framed payloads in one libc::write call (kernel delivers
    // both atomically), the first recv() returns frame 1 and the second
    // recv() returns frame 2 WITHOUT another io_uring read.
    //
    // This exercises the take_frame fast path on the second call.
    let (tx, rx) = pair().expect("pair");
    // Two sends in quick succession; the kernel typically merges them
    // into one available chunk on the read side (PIPE_BUF atomicity).
    tx.send(b"alpha").expect("send 1");
    tx.send(b"beta").expect("send 2");
    let first = rx.recv().expect("recv 1");
    let second = rx.recv().expect("recv 2");
    assert_eq!(first, b"alpha", "first recv must return first frame");
    assert_eq!(second, b"beta", "second recv must return second frame");
}

#[test]
fn probe_slice3a_large_frame_spans_multiple_io_uring_reads() {
    // Verifies that a frame larger than the io_uring read buffer (4096)
    // is correctly assembled across multiple loop iterations of recv().
    // Build a 10_000-byte payload (no newlines per Stone A framing
    // constraint).
    let (tx, rx) = pair().expect("pair");
    let payload: Vec<u8> = (0..10_000u32).map(|i| (i % 26) as u8 + b'a').collect();
    // Sender on a separate thread because a 10001-byte write may block
    // if the pipe buffer is full (typical pipe buffer = 64KB so this is
    // fine; the thread split also exercises the recv-blocks-then-wakes
    // path).
    let payload_clone = payload.clone();
    let send_handle = thread::spawn(move || {
        tx.send(&payload_clone).expect("send large");
    });
    let got = rx.recv().expect("recv large");
    assert_eq!(got.len(), payload.len(), "received length must match sent");
    assert_eq!(got, payload, "received bytes must equal sent bytes");
    send_handle.join().expect("sender thread");
}

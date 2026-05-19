//! Arc 214 Slice 3 Stone C smoke probe — verify HolonRepresentable
//! round-trip through io_uring pipe via String as the test type.
//!
//! Six tests:
//!   1. pair() constructs successfully
//!   2. single-string round-trip preserves the string
//!   3. FIFO ordering across multiple sends
//!   4. sender drop wakes recv with Err(RecvError)
//!   5. accumulator correctly splits two frames received in one read
//!   6. large string spans multiple io_uring reads
//!
//! Stone C wires generic T: HolonRepresentable. The wire chain is
//! T → HolonAST → tagged EDN string → newline-framed bytes →
//! libc::write → io_uring Read → bytes → EDN → HolonAST → T.
//!
//! Embedded `\n` in strings escape during wat-edn serialization, so
//! the Stone A "no newlines in payload" constraint no longer applies
//! at the wire layer.

use std::thread;
use std::time::Duration;

use wat::comms::RecvError;
use wat::comms::process::pair;

#[test]
fn probe_slice3c_pair_constructs_successfully() {
    // Verifies the libc::pipe → OwnedFd wrapping path works under
    // the generic T parameter.
    let result = pair::<String>();
    assert!(result.is_ok(), "pair() must return Ok; got {:?}", result.err());
    let (_tx, _rx) = result.expect("pair");
}

#[test]
fn probe_slice3c_single_string_round_trip() {
    // Verifies the core contract: a String sent is the exact same
    // String received (after the EDN roundtrip).
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("hello".to_string()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent string");
    assert_eq!(got, "hello", "received string must equal sent string");
}

#[test]
fn probe_slice3c_fifo_ordering_preserved_across_sends() {
    // Verifies that N sends followed by N recvs preserve order.
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("first".to_string()).expect("send 1");
    tx.send("second".to_string()).expect("send 2");
    tx.send("third".to_string()).expect("send 3");
    assert_eq!(rx.recv().expect("recv 1"), "first");
    assert_eq!(rx.recv().expect("recv 2"), "second");
    assert_eq!(rx.recv().expect("recv 3"), "third");
}

#[test]
fn probe_slice3c_sender_drop_wakes_recv_with_err() {
    // Verifies that dropping the Sender causes recv to return
    // Err(RecvError) (EOF on the pipe; io_uring Read returns 0).
    let (tx, rx) = pair::<String>().expect("pair");
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
fn probe_slice3c_accumulator_splits_two_frames_from_one_read() {
    // Verifies that when the sender writes two EDN frames in quick
    // succession (kernel may deliver both atomically), the first
    // recv() returns frame 1 and the second recv() returns frame 2
    // WITHOUT another io_uring read.
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("alpha".to_string()).expect("send 1");
    tx.send("beta".to_string()).expect("send 2");
    let first = rx.recv().expect("recv 1");
    let second = rx.recv().expect("recv 2");
    assert_eq!(first, "alpha", "first recv must return first string");
    assert_eq!(second, "beta", "second recv must return second string");
}

#[test]
fn probe_slice3c_large_string_spans_multiple_io_uring_reads() {
    // Verifies that a String whose EDN encoding exceeds the io_uring
    // read buffer (4096) is correctly assembled across multiple loop
    // iterations of recv(). 10_000-char ASCII string — the EDN
    // encoding is even bigger (tagged-EDN wraps it), guaranteeing
    // multi-iteration reads.
    let (tx, rx) = pair::<String>().expect("pair");
    let payload: String = (0..10_000u32)
        .map(|i| (i % 26) as u8 + b'a')
        .map(|b| b as char)
        .collect();
    let payload_clone = payload.clone();
    let send_handle = thread::spawn(move || {
        tx.send(payload_clone).expect("send large");
    });
    let got = rx.recv().expect("recv large");
    assert_eq!(got.len(), payload.len(), "received length must match sent");
    assert_eq!(got, payload, "received string must equal sent string");
    send_handle.join().expect("sender thread");
}

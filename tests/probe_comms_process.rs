//! Arc 214 Slice 3 process-tier smoke probes.
//!
//! Tests organized by stone (prefix tracks the stone that owns the contract):
//!
//! `probe_slice3c_*` (6 tests; Stone C — HolonRepresentable wire chain):
//!   1. pair() constructs successfully
//!   2. single-string round-trip preserves the string
//!   3. FIFO ordering across multiple sends
//!   4. sender drop wakes recv with Err(RecvError)
//!   5. accumulator correctly splits two frames received in one read
//!   6. large string spans multiple io_uring reads
//!
//! `probe_slice3d1_*` (10 tests; Stone D1 — mechanical methods + traits):
//!   1-3. try_recv: Empty, Disconnected, success
//!   4. len reports accumulator frame count
//!   5-6. Sender::close, Receiver::close consume the endpoint
//!   7. Sender clone shares the write-end fd
//!   8. Receiver clone has fresh accumulator + shares pipe fd
//!   9-10. CommSender / CommReceiver trait dispatch
//!
//! The Stone C wire chain (T → HolonAST → tagged EDN string →
//! newline-framed bytes → libc::write → io_uring Read → bytes → EDN →
//! HolonAST → T) carries through all tests. Embedded `\n` in strings
//! escape during wat-edn serialization, so the Stone A "no newlines in
//! payload" constraint no longer applies at the wire layer.

use std::thread;
use std::time::Duration;

use wat::comms::{CommReceiver, CommSender, RecvError, SendError, TryRecvError};
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

// ─── Stone D1 probes ──────────────────────────────────────────────────────────

#[test]
fn probe_slice3d1_try_recv_empty_returns_empty() {
    // Verifies try_recv reports Empty when no data is ready and no
    // shutdown is firing. _tx kept alive so the channel stays
    // connected (Empty, not Disconnected).
    let (_tx, rx) = pair::<String>().expect("pair");
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

#[test]
fn probe_slice3d1_try_recv_disconnected_after_sender_drop() {
    // Verifies try_recv reports Disconnected (not Empty) after all
    // senders drop — callers need this distinction to avoid infinite
    // retry loops.
    let (tx, rx) = pair::<String>().expect("pair");
    drop(tx);
    // Give the kernel a moment to propagate the close.
    thread::sleep(Duration::from_millis(20));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn probe_slice3d1_try_recv_succeeds_when_data_ready() {
    // Verifies try_recv returns the value when data is ready.
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("hello".to_string()).expect("send");
    // Give the kernel a moment to flush the write.
    thread::sleep(Duration::from_millis(20));
    let result = rx.try_recv();
    assert_eq!(result, Ok("hello".to_string()));
}

#[test]
fn probe_slice3d1_len_reports_accumulator_frames() {
    // Verifies len() returns the count of complete frames in the
    // accumulator. We don't assert exact intermediate len values
    // (kernel-scheduling dependent) — we verify recv values are
    // correct and len observably tracks consumption.
    let (tx, rx) = pair::<String>().expect("pair");
    assert_eq!(rx.len(), 0, "fresh receiver has empty accumulator");
    tx.send("one".to_string()).expect("send 1");
    tx.send("two".to_string()).expect("send 2");
    // After both recvs, accumulator is drained → len 0.
    assert_eq!(rx.recv().expect("recv 1"), "one");
    assert_eq!(rx.recv().expect("recv 2"), "two");
    assert_eq!(rx.len(), 0, "accumulator empty after both recvs");
}

#[test]
fn probe_slice3d1_sender_close_consumes_endpoint() {
    // Verifies Sender::close consumes self and returns Ok(()).
    let (tx, rx) = pair::<String>().expect("pair");
    let result = tx.close();
    assert!(result.is_ok(), "Sender::close must return Ok(())");
    drop(rx);
}

#[test]
fn probe_slice3d1_receiver_close_consumes_endpoint() {
    // Verifies Receiver::close consumes self and returns Ok(()).
    let (tx, rx) = pair::<String>().expect("pair");
    let result = rx.close();
    assert!(result.is_ok(), "Receiver::close must return Ok(())");
    drop(tx);
}

#[test]
fn probe_slice3d1_sender_clone_shares_write_end() {
    // Verifies cloned senders both write to the same channel: both
    // values arrive on the receiver. Cloned senders share the kernel
    // pipe via libc::dup.
    let (tx, rx) = pair::<String>().expect("pair");
    let tx2 = tx.clone();
    tx.send("from tx".to_string()).expect("send via tx");
    tx2.send("from tx2".to_string()).expect("send via tx2");
    let first = rx.recv().expect("recv 1");
    let second = rx.recv().expect("recv 2");
    let mut got = [first, second];
    got.sort();
    assert_eq!(got, ["from tx".to_string(), "from tx2".to_string()]);
}

#[test]
fn probe_slice3d1_receiver_clone_competes_for_frames() {
    // Verifies cloned receivers share the same kernel pipe: a clone
    // can independently recv frames sent on the channel. The clone
    // has a FRESH empty accumulator (not inherited from the original).
    // Proves the fd-dup semantic: clone reads from the same pipe, not
    // a copy of it.
    let (tx, rx) = pair::<String>().expect("pair");
    let rx2 = rx.clone();

    // Send one frame. rx2 (the clone) receives it — proving it shares
    // the pipe. rx does NOT see the frame (it was consumed by rx2).
    tx.send("shared".to_string()).expect("send");
    // Give the kernel a moment to flush the write.
    thread::sleep(Duration::from_millis(20));

    // rx2 (clone) can recv the frame from the shared pipe.
    let got = rx2.recv().expect("recv via rx2 (clone)");
    assert_eq!(got, "shared", "clone must recv from the shared pipe");

    // rx's accumulator is fresh (empty) — clone did NOT inherit original's state.
    assert_eq!(rx.len(), 0, "original's accumulator stays empty; clone is independent");

    // Clean up.
    drop(tx);
    drop(rx);
}

#[test]
fn probe_slice3d1_comm_sender_trait_dispatch() {
    // Verifies CommSender<T> trait impl works — generic fn over
    // CommSender dispatches correctly to our concrete Sender<T>.
    fn generic_send<S: CommSender<String>>(tx: &S, value: String) -> Result<(), SendError<String>> {
        tx.send(value)
    }
    let (tx, rx) = pair::<String>().expect("pair");
    generic_send(&tx, "via trait".to_string()).expect("send via trait");
    let got = rx.recv().expect("recv");
    assert_eq!(got, "via trait");
}

#[test]
fn probe_slice3d1_comm_receiver_trait_dispatch() {
    // Verifies CommReceiver<T> trait impl works — generic fn over
    // CommReceiver dispatches correctly to our concrete Receiver<T>.
    fn generic_recv<R: CommReceiver<String>>(rx: &R) -> Result<String, RecvError> {
        rx.recv()
    }
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("via trait".to_string()).expect("send");
    let got = generic_recv(&rx).expect("recv via trait");
    assert_eq!(got, "via trait");
}

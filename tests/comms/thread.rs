//! Arc 214 Slice 2 smoke probe — verify thread tier round-trip + cascade.
//!
//! Ten tests covering: round-trip (unbounded + bounded), sender-drop,
//! try_recv (empty + disconnected), Clone semantics (sender + receiver),
//! Select firing + index ordering, close multi-clone behavior.
//!
//! SHUTDOWN_RX is NOT initialized in these tests (bootstrap fallback path).
//! The cascade-aware recv falls back to bare crossbeam recv, which is correct
//! for the test environment — the contract is verified structurally (the
//! select! pattern is in the code) rather than by triggering shutdown.

use std::thread;

use wat::comms::{ReceiverIndex, RecvError, SelectOutcome, TryRecvError};
use wat::comms::thread::{bounded, pair, Select};

#[test]
fn probe_slice2_unbounded_round_trip() {
    // Verifies the most basic contract: a value sent is a value received.
    let (tx, rx) = pair::<i64>();
    tx.send(42).expect("send must succeed on live channel");
    assert_eq!(rx.recv().expect("recv must return the sent value"), 42);
}

#[test]
fn probe_slice2_bounded_round_trip() {
    // Verifies bounded construction + len tracking across two enqueue/dequeue cycles.
    let (tx, rx) = bounded::<i64>(4);
    tx.send(1).expect("send 1");
    tx.send(2).expect("send 2");
    assert_eq!(rx.len(), 2, "two values enqueued; len must be 2");
    assert_eq!(rx.recv().expect("recv 1"), 1, "FIFO: first value out is 1");
    assert_eq!(rx.recv().expect("recv 2"), 2, "FIFO: second value out is 2");
    assert_eq!(rx.len(), 0, "channel must be empty after consuming both values");
}

#[test]
fn probe_slice2_sender_drop_triggers_recv_err() {
    // Verifies that dropping ALL senders causes recv to return Err(RecvError)
    // rather than hanging — the cascade-aware recv must not block on a dead channel.
    let (tx, rx) = pair::<i64>();
    drop(tx);
    assert_eq!(
        rx.recv(),
        Err(RecvError),
        "recv on disconnected channel must return Err(RecvError)"
    );
}

#[test]
fn probe_slice2_try_recv_empty_returns_empty() {
    // Verifies non-blocking recv correctly reports Empty when no value is ready.
    // _tx kept alive so the channel stays connected (Empty, not Disconnected).
    let (_tx, rx) = pair::<i64>();
    assert_eq!(
        rx.try_recv(),
        Err(TryRecvError::Empty),
        "try_recv on empty connected channel must return Empty"
    );
}

#[test]
fn probe_slice2_try_recv_disconnected_after_sender_drop() {
    // Verifies that Disconnected is returned (not Empty) after all senders drop —
    // callers need this distinction to avoid infinite retry loops.
    let (tx, rx) = pair::<i64>();
    drop(tx);
    assert_eq!(
        rx.try_recv(),
        Err(TryRecvError::Disconnected),
        "try_recv after sender drop must return Disconnected, not Empty"
    );
}

#[test]
fn probe_slice2_clone_sender_multi_producer() {
    // Verifies that cloned senders share the same channel: both values arrive.
    // Two producers on separate threads; ordering is nondeterministic so we sort.
    let (tx, rx) = pair::<i64>();
    let tx2 = tx.clone();
    thread::spawn(move || {
        tx.send(1).expect("thread 1 send");
    });
    thread::spawn(move || {
        tx2.send(2).expect("thread 2 send");
    });
    let a = rx.recv().expect("recv first value");
    let b = rx.recv().expect("recv second value");
    let mut got = [a, b];
    got.sort();
    assert_eq!(got, [1, 2], "both values must arrive regardless of ordering");
}

#[test]
fn probe_slice2_clone_receiver_exactly_one_gets_frame() {
    // Verifies that cloned receivers compete for messages: exactly ONE of the
    // two clones gets the value; the other sees Empty (not a duplicate recv).
    // Test shape is serial (non-blocking try_recv competition) — the name
    // reflects the actual contract exercised, not multi-threaded concurrency.
    let (tx, rx) = pair::<i64>();
    let rx2 = rx.clone();
    tx.send(99).expect("send");
    let from_a = rx.try_recv();
    let from_b = rx2.try_recv();
    assert!(
        matches!(
            (from_a, from_b),
            (Ok(99), Err(TryRecvError::Empty)) | (Err(TryRecvError::Empty), Ok(99))
        ),
        "exactly one receiver must get the value; the other must see Empty"
    );
}

#[test]
fn probe_slice2_select_picks_fired_receiver() {
    // Verifies that Select returns the correct ReceiverIndex and value when
    // exactly one of two registered receivers has a queued message.
    let (tx_a, rx_a) = pair::<i64>();
    let (_tx_b, rx_b) = pair::<i64>();
    tx_a.send(7).expect("send to rx_a");
    let mut sel: Select<i64> = Select::new();
    let idx_a = sel.recv(&rx_a);
    // registered to give Select a second arm; returned index intentionally unused
    let _idx_b = sel.recv(&rx_b);
    match sel.select() {
        SelectOutcome::Recv { index, result } => {
            assert_eq!(index, idx_a, "fired index must match the receiver that had data");
            assert_eq!(result, Ok(7), "result must carry the sent value");
        }
        SelectOutcome::Shutdown => panic!("unexpected Shutdown — SHUTDOWN_RX not initialized in tests"),
        SelectOutcome::SubstrateError(e) => panic!("unexpected SubstrateError: {e}"),
    }
}

#[test]
fn probe_slice2_select_indices_match_registration_order() {
    // Verifies that ReceiverIndex reflects registration order (0, 1, 2)
    // independent of crossbeam's internal arm index — which may differ when
    // SHUTDOWN_RX occupies arm 0 internally.
    let (_tx_a, rx_a) = pair::<i64>();
    let (_tx_b, rx_b) = pair::<i64>();
    let (_tx_c, rx_c) = pair::<i64>();
    let mut sel: Select<i64> = Select::new();
    let idx_a = sel.recv(&rx_a);
    let idx_b = sel.recv(&rx_b);
    let idx_c = sel.recv(&rx_c);
    assert_eq!(idx_a, ReceiverIndex(0), "first registered receiver must be index 0");
    assert_eq!(idx_b, ReceiverIndex(1), "second registered receiver must be index 1");
    assert_eq!(idx_c, ReceiverIndex(2), "third registered receiver must be index 2");
}

#[test]
fn probe_slice2_close_idempotent_with_clones() {
    // Verifies that closing one clone of a multi-clone Sender does not close the
    // channel — the remaining clone can still send; recv still succeeds.
    use wat::comms::CommSender;
    let (tx, rx) = pair::<i64>();
    let tx2 = tx.clone();
    // Close the first clone; channel stays alive because tx2 still exists.
    // close() is infallible (returns ()); no .expect() needed.
    CommSender::close(tx);
    tx2.send(5).expect("remaining clone must still be able to send");
    assert_eq!(
        rx.recv().expect("recv after partial close"),
        5,
        "value sent by surviving clone must arrive"
    );
}

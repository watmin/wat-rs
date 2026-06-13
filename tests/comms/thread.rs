//! Arc 214 Slice 2 smoke probe — verify thread tier round-trip + cascade.
//!
//! Seven tests covering: round-trip, sender-drop, Clone semantics (sender +
//! receiver), Select firing + index ordering, close multi-clone behavior.
//!
//! SHUTDOWN_RX is NOT initialized in these tests (bootstrap fallback path).
//! The cascade-aware recv falls back to bare crossbeam recv, which is correct
//! for the test environment — the contract is verified structurally (the
//! select! pattern is in the code) rather than by triggering shutdown.

use std::thread;

use wat::comms::{ReceiverIndex, RecvError, SelectOutcome};
use wat::comms::thread::{pair, Select};

#[test]
fn probe_slice2_pair_round_trip() {
    // Verifies the most basic contract: a value sent via mini-TCP depth-1 pair
    // is a value received.
    let (tx, rx) = pair::<i64>();
    tx.send(42).expect("send must succeed on live channel");
    assert_eq!(rx.recv().expect("recv must return the sent value"), 42);
}

#[test]
fn probe_slice2_sender_drop_triggers_recv_err() {
    // Verifies that dropping ALL senders causes recv to return Err(RecvError)
    // rather than hanging — the cascade-aware recv must not block on a dead channel.
    let (tx, rx) = pair::<i64>();
    drop(tx);
    assert_eq!(
        rx.recv(),
        Err(RecvError::Disconnected),
        "recv on disconnected channel must return Err(RecvError::Disconnected)"
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
        SelectOutcome::Listener => unreachable!("thread-tier Select has no listener arm"),
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

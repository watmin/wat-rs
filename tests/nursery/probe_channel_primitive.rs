//! Arc 213 χ-1 smoke probe — comms::thread channel basic semantics.
//! Cascade-awareness verified by χ-4's 50-trial replication proof under
//! real runtime conditions; this probe verifies the channel itself
//! behaves as a channel.
//!
//! Arc 214 Stone 6.1: the χ-1 bounded wrapper (typed_channel quarry) is dead;
//! this probe now exercises `comms::thread::pair` directly (the same
//! depth-1 backing `make-channel` uses). Arc 253 2-state collapse:
//! `try_recv` returns `Option<T>` (`None` = empty OR disconnected).

use wat::comms::{RecvError};
use wat::comms::thread::pair;

#[test]
fn probe_chi1_depth1_round_trip() {
    let (tx, rx) = pair::<i32>();
    tx.send(42).expect("send");
    assert_eq!(rx.recv().expect("recv"), 42);
}

#[test]
fn probe_chi1_sender_drop_triggers_recv_err() {
    let (tx, rx) = pair::<i32>();
    drop(tx);
    assert!(matches!(rx.recv(), Err(RecvError)));
}

#[test]
fn probe_chi1_try_recv_empty_returns_none() {
    // Arc 253 2-state collapse: Empty and Disconnected both → None.
    let (_tx, rx) = pair::<i32>();
    assert!(matches!(rx.try_recv(), None));
}

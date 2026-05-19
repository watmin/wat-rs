//! Arc 213 χ-1 smoke probe — wat::channel wrapper basic semantics.
//! Cascade-awareness verified by χ-4's 50-trial replication proof under
//! real runtime conditions; this probe verifies the wrapper itself
//! behaves as a channel.

use wat::typed_channel::{unbounded, RecvError, TryRecvError};

#[test]
fn probe_chi1_unbounded_round_trip() {
    let (tx, rx) = unbounded::<i32>();
    tx.send(42).expect("send");
    assert_eq!(rx.recv().expect("recv"), 42);
}

#[test]
fn probe_chi1_sender_drop_triggers_recv_err() {
    let (tx, rx) = unbounded::<i32>();
    drop(tx);
    assert!(matches!(rx.recv(), Err(RecvError)));
}

#[test]
fn probe_chi1_try_recv_empty_returns_empty() {
    let (_tx, rx) = unbounded::<i32>();
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));
}

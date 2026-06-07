//! Arc 213 χ-1 smoke probe — wat::typed_channel wrapper basic semantics.
//! Cascade-awareness verified by χ-4's 50-trial replication proof under
//! real runtime conditions; this probe verifies the wrapper itself
//! behaves as a channel.
//!
//! Arc 254.0: `unbounded()` was annihilated (depth-1 doctrine — there is one
//! channel and its depth is always 1). These probes now exercise the surviving
//! `bounded(1)` primitive that `make-channel` is built on.

use wat::typed_channel::{bounded, RecvError, TryRecvError};

#[test]
fn probe_chi1_depth1_round_trip() {
    let (tx, rx) = bounded::<i32>(1);
    tx.send(42).expect("send");
    assert_eq!(rx.recv().expect("recv"), 42);
}

#[test]
fn probe_chi1_sender_drop_triggers_recv_err() {
    let (tx, rx) = bounded::<i32>(1);
    drop(tx);
    assert!(matches!(rx.recv(), Err(RecvError)));
}

#[test]
fn probe_chi1_try_recv_empty_returns_empty() {
    let (_tx, rx) = bounded::<i32>(1);
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));
}

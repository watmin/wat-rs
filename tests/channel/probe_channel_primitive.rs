//! Arc 213 χ-1 smoke probe — comms::thread channel basic semantics.
//! Cascade-awareness verified by χ-4's 50-trial replication proof under
//! real runtime conditions; this probe verifies the channel itself
//! behaves as a channel.
//!
//! Arc 214 Stone 6.1: the χ-1 bounded wrapper (typed_channel quarry) is dead;
//! this probe now exercises `comms::thread::pair` directly (the same
//! depth-1 backing the thread-tier channel constructors use). Arc 214 ε:
//! `RecvError` is a two-variant enum — a sender-drop EOF surfaces as
//! `RecvError::Disconnected`.

use wat::comms::RecvError;
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
    // EOF / all-senders-dropped is the data arm → Disconnected (NOT Shutdown).
    // Note: `Err(RecvError)` here would bind a catch-all, not match the enum —
    // the variant must be named for this to be a real assertion.
    assert!(matches!(rx.recv(), Err(RecvError::Disconnected)));
}

// `probe_chi1_try_recv_empty_returns_none` was RETIRED at arc 214 ε:
// `try_recv` was annihilated. A non-blocking empty→None probe has no
// blocking-`recv` equivalent (recv would park on an empty channel), so the
// subject is gone, not relocated.

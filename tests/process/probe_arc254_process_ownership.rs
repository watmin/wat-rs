//! Arc 254 — de-risk probe for stone 254.3 (process-tier ownership).
//!
//! THE TRAP: `comms::process::Receiver<T>` is `Send + !Sync` — it owns a
//! per-receiver io_uring ring (`RefCell<IoUring>`). The wat channel handle is
//! today `Arc<ReceiverInner>` (a shared Value; clone = Arc-clone of the SAME
//! receiver). `Arc<!Sync>` is `!Send`, so a process receiver CANNOT be
//! Arc-shared across threads.
//!
//! THE MODEL this probe proves out for 254.3: process receivers are
//! SINGLE-OWNER and MOVE into the consumer thread; a single owner drains
//! every frame losslessly and in order. Dup-clone fan-out was DISCONFIRMED
//! as a safe primitive (see the note at the bottom of this file) — multi-reader
//! fan-in is served by `select` over N distinct single-owner channels, not by
//! cloning one receiver. (The thread tier stays Arc/crossbeam-shareable.)
//!
//! Stand-in payload: `String` (its `EdnRepresentable` impl already exists).
//! The ownership facts this probe establishes are payload-independent, so
//! String is sufficient here.

use wat::comms::process::{pair, Receiver, Sender};

// Compile-time witness: process endpoints are `Send` — they MOVE into threads.
fn assert_send<T: Send>() {}

#[test]
fn process_endpoints_are_send() {
    assert_send::<Sender<String>>();
    assert_send::<Receiver<String>>();
}

#[test]
fn process_receiver_single_owner_move_across_thread() {
    // The model: Receiver MOVES into the consumer thread (single owner there);
    // Sender stays with the producer. No Arc, no sharing.
    let (tx, rx): (Sender<String>, Receiver<String>) = pair().expect("pipe pair");

    let consumer = std::thread::spawn(move || {
        // `rx` is owned by THIS thread. recv() blocks until a value or EOF.
        rx.recv().expect("recv value")
    });

    tx.send("hello-from-parent".to_string()).expect("send");
    let got = consumer.join().expect("join");
    assert_eq!(got, "hello-from-parent");
}

#[test]
fn process_receiver_single_owner_drain_is_complete_and_ordered() {
    // THE SAFE MODEL: a SINGLE owner recv's repeatedly until EOF and observes
    // EVERY frame, in order. The per-receiver accumulator buffers extra bytes a
    // greedy read pulls off the pipe, and the SAME owner drains them on the next
    // recv — so single-owner drain is lossless and ordered.
    let (tx, rx): (Sender<String>, Receiver<String>) = pair().expect("pipe pair");

    tx.send("one".to_string()).expect("send one");
    tx.send("two".to_string()).expect("send two");
    drop(tx); // disconnect → recv returns Err(RecvError) once drained

    let mut got = Vec::new();
    while let Ok(v) = rx.recv() {
        got.push(v);
    }
    assert_eq!(got, vec!["one".to_string(), "two".to_string()]);
}

// DISCONFIRMED (de-risk finding, 2026-06-06): dup-clone is NOT a safe fan-out
// primitive. With two clones competing on one pipe, clone A's recv greedily
// reads BOTH framed payloads off the kernel pipe (`one\ntwo\n`) in one read,
// returns "one", and buffers "two" in its PRIVATE accumulator; if A is then
// dropped having recv'd once, "two" is STRANDED (lost) and clone B sees EOF.
// Frames are therefore NOT fairly distributed across clones, and a clone
// dropped with a non-empty accumulator loses buffered frames.
//   => 254.3 design: process channels are SINGLE-OWNER. Clone-as-fan-out is
//      not exposed. Multi-reader fan-in is served by `select` over N distinct
//      single-owner channels, not by cloning one receiver.

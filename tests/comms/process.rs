//! Arc 214 Slice 3 process-tier smoke probes.
//!
//! Tests organized by stone (prefix tracks the stone that owns the contract):
//!
//! `probe_slice3c_*` (6 tests; Stone C — EdnRepresentable wire chain):
//!   1. pair() constructs successfully
//!   2. single-string round-trip preserves the string
//!   3. FIFO ordering across multiple sends
//!   4. sender drop wakes recv with Err(RecvError)
//!   5. accumulator correctly splits two frames received in one read
//!   6. large string spans multiple io_uring reads
//!
//! `probe_slice3d1_*` (Stone D1 — mechanical methods + traits):
//!   - len reports accumulator frame count
//!   - Sender::close, Receiver::close consume the endpoint
//!   - (retired) Sender::clone — Clone impl removed; single-writer by design
//!   - Receiver clone has fresh accumulator + shares pipe fd
//!   - CommSender / CommReceiver trait dispatch
//!   - (retired at arc 214 ε) the three try_recv probes — try_recv annihilated;
//!     a non-blocking-None probe has no blocking-recv equivalent
//!
//! `probe_slice3d2_*` (2 tests; Stone D2 — Select<'a, T> cascade-aware fan-in):
//!   1. select picks the fired receiver (correct ReceiverIndex + value)
//!   2. ReceiverIndex matches registration order (0, 1, 2)
//!
//! The Stone C wire chain (T → HolonAST → tagged EDN string →
//! newline-framed bytes → libc::write → io_uring Read → bytes → EDN →
//! HolonAST → T) carries through all tests. Embedded `\n` characters
//! in strings are escaped by wat-edn serialization; the wire layer never
//! sees a literal newline except as a frame delimiter.

use std::thread;

use wat::comms::{CommReceiver, CommSender, ReceiverIndex, RecvError, SelectOutcome, SendError};
use wat::comms::process::{pair, Select};

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
fn probe_slice3c_recv_returns_err_after_sender_drop() {
    // Verifies that recv returns Err(RecvError) when the sender side of
    // the pipe has been closed (EOF; io_uring Read returns 0).
    //
    // Lock-step via the wire: drop(tx) is synchronous (libc::close(2)
    // state-changes the pipe at close-time); the subsequent recv() sees
    // EOF immediately via io_uring's POLL_ADD + Read sequence. No timing
    // assumption involved.
    //
    // The earlier shape (spawn a thread, sleep 50ms, drop tx, recv on
    // main) pretended to test "drop wakes a parked recv" but actually
    // tested the same contract this simpler shape proves — the substrate
    // doesn't expose kernel-side introspection for "is this recv parked",
    // so the parked-then-woken scenario isn't deterministically testable
    // at this layer. Per `feedback_lock_step_via_pipe`: sleep is a guess;
    // we use the wire.
    let (tx, rx) = pair::<String>().expect("pair");
    drop(tx);
    let result = rx.recv();
    assert_eq!(
        result,
        Err(RecvError::Disconnected),
        "recv must return Err(RecvError::Disconnected) after sender drop (EOF — the data arm)"
    );
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

// The three `probe_slice3d1_try_recv_*` probes were RETIRED at arc 214 ε:
// `try_recv` was annihilated (it was the last non-blocking poll in the io_uring
// data path, dead with zero callers). A non-blocking-None probe cannot be mapped
// onto blocking `recv` (recv would park forever on no-data / return Err on EOF),
// so the subject is gone, not relocated — the tests go with it. `recv`'s
// data + EOF contracts are covered by `probe_slice3c_single_string_round_trip`
// and `probe_slice3c_recv_returns_err_after_sender_drop`.

#[test]
fn probe_slice3d1_len_reports_accumulator_frames() {
    // Verifies len() returns the count of complete frames in the
    // accumulator.
    //
    // Intermediate len after ONE recv:
    // After send("one") + send("two"), one recv is issued. recv does one
    // io_uring Read. If the kernel delivered both frames in the pipe buffer
    // before the Read (very likely for small sends — both writes complete
    // before our Read syscall), the accumulator after take_frame holds
    // frame 2 → len == 1. If the kernel only has frame 1's bytes in the
    // buffer at Read time, accumulator is empty after take_frame → len == 0.
    // The invariant we CAN assert: len <= 1 (at most one leftover frame)
    // and len >= 0 (trivially). The exact value is kernel-scheduling
    // dependent. We assert the bound and verify correct consumption below.
    let (tx, rx) = pair::<String>().expect("pair");
    assert_eq!(rx.len(), 0, "fresh receiver has empty accumulator");
    tx.send("one".to_string()).expect("send 1");
    tx.send("two".to_string()).expect("send 2");
    // One recv — consume frame 1; frame 2 may or may not be in accumulator.
    assert_eq!(
        rx.recv().expect("recv must succeed — frame 1 is in the kernel pipe"),
        "one",
        "first frame must be 'one'"
    );
    assert!(rx.len() <= 1, "accumulator holds at most one leftover frame after one recv");
    // After the second recv, accumulator must be fully drained.
    assert_eq!(rx.recv().expect("recv 2"), "two");
    assert_eq!(rx.len(), 0, "accumulator empty after consuming both frames");
}

#[test]
fn probe_slice3d1_sender_close_consumes_endpoint() {
    // Verifies Sender::close consumes self (infallible; no Result to check).
    // The contract: move semantics enforce single-close at compile time;
    // Drop closes the OwnedFd; no runtime error path exists.
    let (tx, rx) = pair::<String>().expect("pair");
    tx.close(); // consumes tx; if this compiles, the contract holds
    drop(rx);
}

#[test]
fn probe_slice3d1_receiver_close_consumes_endpoint() {
    // Verifies Receiver::close consumes self (infallible; no Result to check).
    // The contract: move semantics enforce single-close at compile time;
    // Drop closes the OwnedFd; no runtime error path exists.
    let (tx, rx) = pair::<String>().expect("pair");
    rx.close(); // consumes rx; if this compiles, the contract holds
    drop(tx);
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
    // No sleep: send() returns after libc::write(2) completes; bytes
    // are in the kernel pipe buffer; the next recv on either clone
    // sees them. Lock-step via the wire.

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

// ─── Stone D2 probes ──────────────────────────────────────────────────────────

#[test]
fn probe_slice3d2_select_picks_fired_receiver() {
    // Verifies Select returns the correct ReceiverIndex + value when
    // exactly one of two registered receivers has a queued frame.
    let (tx_a, rx_a) = pair::<String>().expect("pair a");
    let (_tx_b, rx_b) = pair::<String>().expect("pair b");
    tx_a.send("hello-a".to_string()).expect("send to rx_a");
    // No sleep: tx_a.send() returns after libc::write(2) completes;
    // bytes are in the kernel pipe buffer. Select's submit_and_wait(1)
    // BLOCKS on kernel events; POLL_ADD on rx_a's fd fires immediately
    // (POLLIN already set). Lock-step via the wire.
    let mut sel: Select<String> = Select::new();
    let idx_a = sel.recv(&rx_a);
    // Register rx_b too so Select genuinely has two arms;
    // returned index intentionally unused.
    let _idx_b = sel.recv(&rx_b);
    match sel.select() {
        Ok(SelectOutcome::Recv { index, result }) => {
            assert_eq!(index, idx_a, "fired index must match the receiver with data");
            assert_eq!(result, Ok("hello-a".to_string()), "result must carry the sent value");
        }
        Ok(SelectOutcome::Shutdown) => panic!("unexpected Shutdown"),
        Ok(SelectOutcome::Listener) => panic!("unexpected Listener — no listener arm registered"),
        Err(e) => panic!("unexpected io_uring substrate error: {e}"),
    }
}

#[test]
fn probe_slice3d2_select_indices_match_registration_order() {
    // Verifies ReceiverIndex reflects registration order (0, 1, 2)
    // independent of any io_uring internal token scheme.
    let (_tx_a, rx_a) = pair::<String>().expect("pair a");
    let (_tx_b, rx_b) = pair::<String>().expect("pair b");
    let (_tx_c, rx_c) = pair::<String>().expect("pair c");
    let mut sel: Select<String> = Select::new();
    let idx_a = sel.recv(&rx_a);
    let idx_b = sel.recv(&rx_b);
    let idx_c = sel.recv(&rx_c);
    assert_eq!(idx_a, ReceiverIndex(0), "first registered receiver must be index 0");
    assert_eq!(idx_b, ReceiverIndex(1), "second registered receiver must be index 1");
    assert_eq!(idx_c, ReceiverIndex(2), "third registered receiver must be index 2");
}

/// Arc 209 C0b.3a-i reactor unit test — listener arm fires on pending connection.
///
/// Binds a non-blocking abstract-namespace UDS `UnixListener`, registers it
/// with `Select::listener(fd)`, then spawns a thread to connect. Verifies
/// that `select()` returns `SelectOutcome::Listener` (not `Shutdown` or `Recv`)
/// and that a subsequent non-blocking `accept()` succeeds.
#[test]
fn select_listener_arm_fires_on_pending_connection() {
    use std::os::fd::AsRawFd;
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
    let addr = SocketAddr::from_abstract_name(b"wat.arc209.c0b3ai.test").unwrap();
    let listener = UnixListener::bind_addr(&addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    let t = std::thread::spawn(move || {
        let _c = UnixStream::connect_addr(&addr).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
    });
    let mut sel: Select<String> = Select::new();
    sel.listener(listener.as_raw_fd());
    match sel.select().expect("select") {
        SelectOutcome::Listener => {
            let _ = listener.accept().expect("accept the pending conn");
        }
        SelectOutcome::Shutdown => panic!("expected Listener; got Shutdown"),
        SelectOutcome::Recv { .. } => panic!("expected Listener; got Recv"),
    }
    t.join().unwrap();
}

// ─── Stone C0b.2e-i-0 — Value round-trip over comms::process ────────────────

/// Arc 209 Stone C0b.2e-i-0 gate test — `Value` round-trips over a
/// `comms::process` `Sender<Value>` / `Receiver<Value>` pair via the
/// plain-EDN wire (`EdnRepresentable`).
///
/// At HEAD before this stone, `Value` was not `EdnRepresentable` so
/// `pair::<Value>()` would not compile. This test is the new-capability
/// gate: it must be GREEN after the trait split, and it proves `Value`
/// is now a legal wire `T` at the comms layer.
#[test]
fn probe_arc209_c0b2ei0_value_round_trip_over_process_pair() {
    use wat::value::Value;

    let (tx, rx) = pair::<Value>().expect("pair::<Value>() must succeed");

    // i64 scalar
    let v_i64 = Value::i64(42);
    tx.send(v_i64.clone()).expect("send i64");
    let got_i64 = rx.recv().expect("recv i64");
    assert_eq!(got_i64, v_i64, "i64 Value must round-trip");

    // bool
    let v_bool = Value::bool(true);
    tx.send(v_bool.clone()).expect("send bool");
    let got_bool = rx.recv().expect("recv bool");
    assert_eq!(got_bool, v_bool, "bool Value must round-trip");

    // String
    let v_str = Value::String(std::sync::Arc::new("hello-wire".to_string()));
    tx.send(v_str.clone()).expect("send String");
    let got_str = rx.recv().expect("recv String");
    assert_eq!(got_str, v_str, "String Value must round-trip");

    // Unit (nil)
    let v_unit = Value::Unit;
    tx.send(v_unit.clone()).expect("send Unit");
    let got_unit = rx.recv().expect("recv Unit");
    assert_eq!(got_unit, v_unit, "Unit Value must round-trip");
}

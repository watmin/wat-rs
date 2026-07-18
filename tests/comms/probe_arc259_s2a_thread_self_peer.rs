//! Arc 259 (The Forced Hand) Stone S2a — the ThreadProg self-peer model, on the
//! UNIFIED pipes-only `Peer` (FM-2-bis disconfirming probe).
//!
//! S2a reshapes the `:thread` tier of `spawn-program'` from the platform
//! APPLY-LOOP (recv Value → apply a 1-ary fn → send the result) to the
//! SELF-PEER HANDOFF: the prog is `[self] -> nil`, handed its OWN pipes-only
//! peer ONCE, and drives it with `recv'`/`send'` directly. A thread shares the
//! parent's ambient stdio, so it CANNOT use stdio for its data channel; it must
//! be handed an explicit `(rx, tx)` self-peer (the capability grant; the
//! principled exception to "every peer is a stdio `:user::main`" — the process
//! tier keeps the stdio model).
//!
//! ## The unified peer (no bespoke `ThreadSelf'`, no mirror projection)
//!
//! The worker's self-peer is the SAME pipes-only `Peer<S,R>` type as everything
//! else: `send'`→S, `recv'`→R, UNIFORM (`<send-type, recv-type>` for every peer).
//! The worker is `Peer'<O,I>` — the param-swap of the parent's `Thread'<I,O>`
//! (parent sends I / recvs O; worker recvs I / sends O). For the echo here
//! I=O=i64, so the worker is `Peer'<i64,i64>`.
//!
//! S2a is the runtime MODEL + the verb head; the strict forced-hand TYPING
//! (wrong-payload rejected) + the parent-side unification + RAII-`close` land in
//! S2b/S2c.
//!
//! Run: `cargo test --release -p wat --test comms probe_arc259_s2a_thread_self_peer`

use wat::freeze::call_beside;
use wat::runtime::Value;

/// LOAD-BEARING (RUNTIME): the thread prog drives its OWN pipes-only self-peer.
///
/// The prog `(fn [self] (send' self (recv' self)))` echoes via its self-peer:
/// recv the parent's 42, send it straight back, return nil. The parent (holding
/// the `Thread'` handle) sends 42, recvs the echo, closes. Sequencing on the
/// depth-1 channels: parent send(42) → worker recv(42) → worker send(42) →
/// parent recv(42) → worker returns → peer reaped by RAII Drop.
#[test]
fn s2a_thread_prog_drives_self_peer() {
    let got = match call_beside(file!(), ":user::compute").expect("compute eval") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    };
    assert_eq!(got, 42, "thread prog echoes 42 through its pipes-only Peer' self-peer");
}

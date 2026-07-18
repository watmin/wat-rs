//! Arc 209 Stone C0b.1 — thread-tier connection (`listener'` / `connect'` / `accept'`).
//!
//! THE SHAPE (DESIGN-STONE-C0b.1): a service `listener'`s (mints a crossbeam rendezvous,
//! returns `(Listener', Address')`); a client `connect'`s to the `Address'` (mints the
//! connection pairs, keeps + wraps its `Peer'` end locally, ships the server's raw halves over
//! the rendezvous); the service `accept'`s on the `Listener'` (receives those halves, wraps its
//! `Peer'` end locally). No `Peer'` cell crosses a thread — each side wraps on its own thread
//! (custody holds). `send'`/`recv'` already handle bare `Peer'`.
//!
//! The miniature service: the spawned program captures the `Listener'`, `accept'`s one client,
//! reads a number, and replies it doubled. The client `connect'`s, sends 5, reads 10 — a
//! protected scalar round-tripped over a rendezvous both sides walked into from their own
//! threads.
//!
//! Run:
//!   cargo test --release -p wat --test comms probe_arc209_c0b1_thread_connection -- --test-threads=1

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn thread_connection_listen_accept_connect_round_trips() {
    let got = call_beside(file!(), ":user::compute").unwrap_or_else(|e| panic!(
        "thread connection verbs are ABSENT: listener'/connect'/accept' are unsupported. \
         Eval error: {e:?}"
    ));
    assert!(
        matches!(got, Value::i64(10)),
        "thread connection round-trip returned the wrong value: expected i64 10 (5 * 2); got {got:?}"
    );
}

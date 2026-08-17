//! Arc 272 step 6a — the child mints the rendezvous and hands the capability to the parent over
//! the LINEAGE channel (the self-peer). The ZERO-MUTEX handoff: perfect knowledge transmitted,
//! lock-step, no name.
//!
//! The shared-memory partition decides how the program gets its listener: a thread captures an
//! in-memory listener (shared); a process autobinds its OWN (separate memory, step 2b — kernel-minted,
//! no name) and SENDS its address back to the parent. The parent's `recv'` blocks until that send
//! lands, then `connect'`s — `docs/ZERO-MUTEX.md`: "synchronization IS the channel handoff." c0b3aii
//! already runs this handshake with a bare `1` READY marker; here the marker IS the capability.
//!
//! This isolates 6a's one new bit (NOT the poll' service loop — that's proven by c0b3aii):
//!   - child: `(listener' (process) :i64 :i64)` autobind → `Bound`; self-peer typed to carry
//!     `Address'`; `(send' self (Bound/address b))` — hand over the capability; `accept'` the parent;
//!     round-trip n→n+100.
//!   - parent: `(spawn-program' (process) <forms>)` → `svc`; `(recv' svc)` → the minted `Address'`;
//!     `(connect' addr)`; `send' 5`; `recv'` → 105.
//!
//! GREEN as of 258.5a + 272 6a-i. Two composing fixes made the no-ascription handoff work: (1)
//! arc 258.5a — `connect'` UNIFIES its arg, so the fresh 1-arg `(recv' svc)` result binds to
//! `Address'` from the consumer ("the type lives in the channel"; no `-> :T`); (2) arc 272 6a-i —
//! `Address'` crosses as a portable `#wat.kernel/Address [bytes]` tag (decode via
//! `from_socket_name_bytes`), so `recv'` reconstructs the capability with no runtime type hint.
//! No `-> :T` ascription anywhere — that arrow stays killed. `spawn-program'` stays 2-arg throughout —
//! the listener never touches the spawn surface.
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc272_6a_capability_handoff

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn child_mints_and_hands_capability_over_lineage_channel() {
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105: the child autobound its own listener (no name), sent the Address' capability \
         to the parent over the self-peer (lock-step), the parent recv'd it and dialed it, round-trip \
         5 -> 105; spawn-program' stayed 2-arg; got {got:?}"
    );
}

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
//! `Address'` crosses as a portable `#wat-edn.cap/address [bytes]` tag (decode via
//! `from_socket_name_bytes`), so `recv'` reconstructs the capability with no runtime type hint.
//! No `-> :T` ascription anywhere — that arrow stays killed. `spawn-program'` stays 2-arg throughout —
//! the listener never touches the spawn surface.
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc272_6a_capability_handoff

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc  (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  ;; the child mints its OWN rendezvous: autobind, no name (step 2b).
                  [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                   addr (:wat::spawn::Bound/address b)
                   ;; the self-peer carries the Address' capability child->parent (S = Address').
                   self (:wat::program::self-peer
                          :wat::kernel::Address'<wat::core::i64,wat::core::i64> :wat::core::i64)
                   ;; hand the parent the capability — the lock-step handoff (it now has perfect knowledge).
                   _    (:wat::kernel::send' self addr)
                   ;; accept the parent's dial on our own listener; round-trip n -> n+100.
                   c    (:wat::kernel::accept' (:wat::spawn::Bound/listener b))
                   n    (:wat::kernel::recv' c)
                   _    (:wat::kernel::send' c (:wat::core::+ n 100))]
                  nil))))
     ;; recv' the child's minted capability over the lineage channel (blocks until the child sends it).
     ;; 1-arg — NO `-> :T` ascription (that arrow is enqueued for the kill, arc 258 IO cluster). The
     ;; type must flow from the channel: the spawn-program' handle should carry the child's self-peer
     ;; type so `recv'` yields Address' here, and `connect'` confirms it. RED at HEAD = that inference
     ;; isn't wired (the dep this 6a is blocked on) + Address' has no EDN decode arm.
     addr (:wat::kernel::recv' svc)
     ;; dial the capability — the child is guaranteed listening (it sent AFTER listen()).
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c 5)
     got  (:wat::kernel::recv' c)]
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn child_mints_and_hands_capability_over_lineage_channel() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (6a: capability-over-lineage handoff)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105: the child autobound its own listener (no name), sent the Address' capability \
         to the parent over the self-peer (lock-step), the parent recv'd it and dialed it, round-trip \
         5 -> 105; spawn-program' stayed 2-arg; got {got:?}"
    );
}

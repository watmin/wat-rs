//! Arc 209 (defservice, re-grounded) — the CONNECTION PRIMITIVE probe.
//!
//! THE GAP (disconfirming probe — RED at HEAD by design):
//!
//! defservice provisions a client by minting a NET-NEW, connected `Peer'` pair
//! (crossbeam / pipe / socket) ADJACENT to the admin channel: the service keeps the
//! server end in its `select'` set; the grantee gets the client end as its
//! service-client; deprovision drops the server end from the TCO loop. ("programs ≠
//! channels" — builder, 2026-06-12.)
//!
//! But the entire primed concurrency surface today is exactly seven verbs —
//! `spawn-program'` / `spawn-thread'` / `spawn-process'` / `send'` / `recv'` /
//! `select'` / `close'`. The ONLY way to obtain a `Peer'` is to SPAWN a program.
//! There is no verb (and no production Rust fn) that mints two connected `Peer'` ends
//! WITHOUT spawning. The building blocks exist internally — `comms::thread::pair()`
//! (raw crossbeam), the `kernel::peer::Peer` struct (`PEER_TYPE_PATH`), and
//! `spawn_thread_peer`'s internal wiring — but no standalone pair constructor is
//! exposed. (`make_thread_peer_pair_for_test` builds a LEGACY `ThreadPeer`, not a
//! unified `Peer'`, and is test-only.)
//!
//! This probe attempts the smallest real version of provision: mint a connected
//! `Peer'` pair, round-trip a request/response over it, and `select'` over the server
//! end. At HEAD it is RED on EXACTLY one line — the `peer-pair'` mint — with every
//! other primitive (`send'`/`recv'`/`select'`) present and working.
//!
//! NAMING IS PROVISIONAL: `:wat::kernel::peer-pair'` is a placeholder for the
//! capability (mint-connected-pair-without-spawn). Stone C0 finalizes the verb name
//! and signature; update this probe to match when it ships. The probe proves the
//! CAPABILITY is absent now and becomes the green proof of the provision mechanic.
//!
//! GREEN means: a service can hand a client a live channel to ITSELF without spawning
//! a new program — the door `:remote` eventually generalizes (connect over a wire vs.
//! spawn). Build Stone C only after this is green.
//!
//! Run:
//!   cargo test --release -p wat --test channel probe_arc209_connection_primitive -- --test-threads=1

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// The provision round-trip over a NON-SPAWNED, connected `Peer'` pair.
///
/// `peer-pair'` mints `(server, client)`; the client sends a request; the service
/// `select'`s over `[server]`, receives it, replies with `req * 2`; the client
/// `recv'`s the reply. Asserts the reply is `84` (round-trip + select' over a
/// provisioned peer).
///
/// GREEN once `peer-pair'` and the Stone 259 ServiceEvent migration both ship.
/// Stone 259: `select'` returns `ServiceEvent<I,O>`; match on `:Message{idx, req}`.
#[test]
fn connection_primitive_mints_a_connected_peer_pair_without_spawning() {
    // Stone 259: select' returns ServiceEvent<I,O> (was Tuple<i64,O>).
    // The request message is in :Message{idx, msg}.
    // just-eval (rubric): the expression lives in the co-located fixture's
    // zero-arg `:user::compute`, driven via `call_beside_value` — no inline wat driver.
    match call_beside_value(file!(), ":user::compute") {
        Ok(Value::i64(84)) => { /* green — the provision mechanic works end-to-end */ }
        Ok(other) => panic!(
            "connection primitive round-trip returned the wrong value: expected i64 84, got {other:?}"
        ),
        Err(e) => panic!(
            "connection primitive is ABSENT (the gap this probe names): minting a connected \
             Peer' pair without spawning is unsupported. Build the `peer-pair'`-class primitive \
             (expose comms::thread::pair into two Peer' ends; pipe for process; socket for remote), \
             then this round-trips to 84. Eval error: {e}"
        ),
    }
}

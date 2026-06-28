//! Arc 209 C0b.2b — the socket-backed `Peer'` + `socket-pair'` (the wat socket-peer creator).
//!
//! The process connection tier needs a `Peer'` backed by a socket (not crossbeam, not
//! pipes+pidfd) — one bidirectional `UnixStream`, `send'`/`recv'` as EDN over the socket fd,
//! leaning on the existing `comms::process` io_uring reactor (Sender/Receiver are fd-generic).
//!
//! THE REACH-STUMBLE: to test a socket peer we reached for a wat way to *create* one and found it
//! missing — so we build it, not inject around it. `socket-pair'` is the minimal creator,
//! mirroring C0's `peer-pair'` (a same-process connected pair): `socketpair(2)` → two connected
//! socket peers. `(socket-pair' :S :R) -> (:Tuple Peer'<S,R> Peer'<R,S>)`.
//!
//! THE GATE: `socket-pair'` mints a connected socket-peer pair; `send' a 5` round-trips to
//! `recv' b` → 5, over the socket, EDN-framed, io_uring-driven. `send'`/`recv'` must dispatch on
//! the socket peer kind exactly as they do on the thread/process kinds.
//!
//! Run SERIALLY: cargo test --release -p wat --test comms probe_arc209_c0b2b_socket_peer -- --test-threads=1

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn socket_pair_mints_socket_peers_that_round_trip_over_the_socket() {
    let world = startup_beside(file!())
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5 round-tripped over the socket peer (send' a 5 → recv' b); got {got:?}"
    );
}

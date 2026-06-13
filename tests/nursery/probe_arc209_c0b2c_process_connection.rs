//! Arc 209 C0b.2c — the PROCESS connection tier: `listener'`/`connect'`/`accept'`
//! over an abstract-namespace AF_UNIX socket, producing `SocketPeer'`s.
//!
//! C0b.1/C0b.1b built the connection verbs thread-tier ONLY (crossbeam rendezvous).
//! C0b.2a made the `listener'` host load-bearing — `(listener' (process) …)` is a clean
//! CHECK ERROR today (the process tier is unbuilt). C0b.2b built the socket-backed
//! `SocketPeer'` + `socket-pair'` and proved io_uring drives an AF_UNIX socket like a pipe.
//! C0b.2c fills the `(process)` arm of all three verbs.
//!
//! THE MECHANISM (mirrors `probe_arc209_c0b_uds_abstract_spike`, but through the wat verbs):
//!   - `(listener' (process) :S :R)` → `Tuple[SocketListener'<S,R>, SocketAddress'<S,R>]`
//!     (bind an abstract name + listen; return the listener + the dial-able name).
//!   - `(connect' addr)` over a `SocketAddress'` → `SocketPeer'<S,R>` (connect_addr; the
//!     connection queues in the backlog before accept runs).
//!   - `(accept' listener)` over a `SocketListener'` → `SocketPeer'<R,S>` (accept the queued conn).
//!   - `send'`/`recv'` already dispatch on `SocketPeer'` (C0b.2b).
//!
//! THE GATE: a single-process, single-thread request/response round-trip. The "service" owns a
//! protected scalar (10); the "client" sends 5; the service replies 15. Deadlock-free: connect
//! queues before accept dequeues; the small messages fit the socket buffer; no thread join.
//!
//! RED at HEAD: `(listener' (process) …)` is a C0b.2a check error → `startup_from_source` fails.
//! GREEN once C0b.2c ships the `(process)` arms + the `SocketListener'`/`SocketAddress'` types.
//!
//! Run SERIALLY: cargo test --release -p wat --test nursery probe_arc209_c0b2c_process_connection -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; Process-tier connection: listen on an abstract UDS, connect, accept a per-connection
;; SocketPeer', round-trip a protected scalar over the socket. Proves listener'/connect'/accept'
;; on the (process) host + send'/recv' over the accepted/connected socket peers.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair     (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
     listener (:wat::core::first pair)
     addr     (:wat::core::second pair)
     client   (:wat::kernel::connect' addr)
     server   (:wat::kernel::accept' listener)
     _        (:wat::kernel::send' client 5)
     got      (:wat::kernel::recv' server)
     _        (:wat::kernel::send' server (:wat::core::+ got 10))
     reply    (:wat::kernel::recv' client)]
    reply))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn process_listener_connect_accept_round_trips_over_abstract_uds() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.2c: process connection verbs)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(15)),
        "expected 15 round-tripped over the process-tier socket connection \
         (client sends 5 → server adds to its protected 10 → replies 15); got {got:?}"
    );
}

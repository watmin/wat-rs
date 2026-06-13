//! Arc 209 C0b.2c — the PROCESS connection tier: `listener'`/`connect'`/`accept'`
//! over an abstract-namespace AF_UNIX socket, producing `SocketPeer'`s.
//!
//! C0b.1/C0b.1b built the connection verbs thread-tier ONLY (crossbeam rendezvous).
//! C0b.2a made the `listener'` host load-bearing — `(listener' (process) …)` is a clean
//! CHECK ERROR today (the process tier is unbuilt). C0b.2b built the socket-backed
//! `SocketPeer'` + `socket-pair'` and proved io_uring drives an AF_UNIX socket like a pipe.
//! C0b.2c filled the `(process)` arm of all three verbs.
//!
//! C0b.2d SUPERSEDED the mint-and-return form (`listener' (process) :S :R` → Tuple) with
//! `socket-address'` + bind-addr: `(listener' (process) addr)` takes a `SocketAddress'` opaque
//! (from `(socket-address' name :S :R)`) and returns JUST `SocketListener'<S,R>`. This probe
//! is updated in-strike to the named form.
//!
//! THE MECHANISM (same-process named connection via `socket-address'`):
//!   - `(socket-address' name :S :R)` → `SocketAddress'<S,R>` (typed address from a String name).
//!   - `(listener' (process) addr)` → `SocketListener'<S,R>` (bind the given address, listen).
//!   - `(connect' addr)` over a `SocketAddress'` → `SocketPeer'<S,R>` (connect_addr; the
//!     connection queues in the backlog before accept runs).
//!   - `(accept' listener)` over a `SocketListener'` → `SocketPeer'<R,S>` (accept the queued conn).
//!   - `send'`/`recv'` already dispatch on `SocketPeer'` (C0b.2b).
//!
//! THE GATE: a single-process, single-thread request/response round-trip. The "service" owns a
//! protected scalar (10); the "client" sends 5; the service replies 15. Deadlock-free: connect
//! queues before accept dequeues; the small messages fit the socket buffer; no thread join.
//!
//! Run SERIALLY: cargo test --release -p wat --test nursery probe_arc209_c0b2c_process_connection -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; Process-tier connection (named form — C0b.2d): socket-address' constructs a typed address
;; from a String name; listener' binds it; connect'/accept' rendezvous by the same address.
;; Round-trips a protected scalar (10) → client sends 5 → service replies 15.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [addr     (:wat::kernel::socket-address' "wat.arc209.c0b2c.svc" :wat::core::i64 :wat::core::i64)
     listener (:wat::kernel::listener' (:wat::spawn::process) addr)
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

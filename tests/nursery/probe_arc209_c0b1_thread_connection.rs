//! Arc 209 Stone C0b.1 — thread-tier connection (`listener'` / `connect'` / `accept'`).
//!
//! RED at HEAD by design: the three connection verbs do not exist. This is the gate for the
//! first C0b strike — the host-parametric connection layer, thread clause (crossbeam rendezvous).
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
//! threads. GREEN proves the whole connection shape on the proven (crossbeam) tier.
//!
//! Run:
//!   cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_expr(expr: &str) -> Result<Value, String> {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("trivial startup should succeed");
    let ast = wat::parse_one!(expr).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn thread_connection_listen_accept_connect_round_trips() {
    // service: capture the Listener', accept one client, double its number.
    // client (this body): connect to the Address', send 5, recv 10.
    let expr = r#"(:wat::core::let
        [pair  (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
         l     (:wat::kernel::Bound/listener pair)
         addr  (:wat::kernel::Bound/address pair)
         svc   (:wat::kernel::spawn-program' (:wat::spawn::thread)
                  (:wat::core::fn [_admin <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                    (:wat::core::let
                      [conn (:wat::kernel::accept' l)
                       n    (:wat::kernel::recv' conn)
                       _    (:wat::kernel::send' conn (:wat::core::* n 2))]
                      nil)))
         conn  (:wat::kernel::connect' addr)
         _     (:wat::kernel::send' conn 5)
         reply (:wat::kernel::recv' conn)]
        reply)"#;

    match eval_expr(expr) {
        Ok(Value::i64(10)) => { /* green — the connection shape works end to end on the thread tier */ }
        Ok(other) => panic!(
            "thread connection round-trip returned the wrong value: expected i64 10, got {other:?}"
        ),
        Err(e) => panic!(
            "thread connection verbs are ABSENT (the gap this probe names): listener'/connect'/\
             accept' are unsupported. Build C0b.1 (crossbeam rendezvous; client mints + wraps its \
             end, ships the server's halves, accept' wraps the server end) → this round-trips a \
             protected scalar (5 * 2 = 10). Eval error: {e}"
        ),
    }
}

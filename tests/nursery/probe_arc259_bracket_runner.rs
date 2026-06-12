//! Arc 259 S3.2a — `:wat::bracket::runner-loop`: the multi-message server, the
//! foundation the brackets pool stands on.
//!
//! A `spawn-program` peer that serves a STREAM of work: `recv' item → work-fn →
//! send' result`, NAMED-RECURSING until its channel drains (recv' raises → the
//! runner exits cleanly). Today's peers are single-shot (recv once, send once,
//! return); the runner serves arbitrarily many messages. wat has TCO for named-defn
//! tail-calls (arc 003: "any gen_server-shaped driver, constant stack"), so the loop
//! is safe at any item count — no stack growth.
//!
//! RED at HEAD: `:wat::bracket::runner-loop` does not exist.
//!
//! Run SERIALLY (spawns a thread):
//!   `cargo test --release -p wat --test nursery probe_arc259_bracket_runner -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute_i64(body: &str) -> i64 {
    let src = format!(
        "{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// One runner with a doubling work-fn, served a STREAM of 3 items: 1→2, 2→4, 3→6.
/// The parent sums the three results (12) — proving the peer served MULTIPLE
/// messages (a single-shot peer could not). The peer drops at scope-exit → RAII
/// drain → the runner's `recv'` raises → it exits; no `close'`, no hang.
#[test]
fn runner_serves_a_stream_of_messages() {
    let v = run_compute_i64(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
           (:wat::core::let \
             [peer (:wat::kernel::spawn-program' (:wat::spawn::thread) \
                     (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                       (:wat::bracket::runner-loop self \
                         (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2))))) \
              _a (:wat::kernel::send' peer 1) a (:wat::kernel::recv' peer) \
              _b (:wat::kernel::send' peer 2) b (:wat::kernel::recv' peer) \
              _c (:wat::kernel::send' peer 3) c (:wat::kernel::recv' peer)] \
             (:wat::core::+ a (:wat::core::+ b c))))",
    );
    assert_eq!(v, 12, "runner served 3 messages: 1->2, 2->4, 3->6; sum 12");
}

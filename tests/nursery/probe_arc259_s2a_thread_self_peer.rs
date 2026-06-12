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
//! ## Why this is RED at HEAD
//!
//! At HEAD the `:thread` arm runs the apply-loop: it calls the prog with the
//! MESSAGE value (42), not a self-peer — AND `send'`/`recv'` reject the `Peer'`
//! head (they know only `Thread'`/`Process'`), so the prog fails at check. Either
//! way: RED. Post-S2a the prog is handed a `:wat::kernel::Peer'` and echoes
//! through it; the parent reads 42.
//!
//! S2a is the runtime MODEL + the verb head; the strict forced-hand TYPING
//! (wrong-payload rejected) + the parent-side unification + RAII-`close` land in
//! S2b/S2c.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_s2a`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute_i64(src: &str) -> i64 {
    let src = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env)
        .expect("compute eval (RED at HEAD: the :thread arm still runs the apply-loop)")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// LOAD-BEARING (RUNTIME): the thread prog drives its OWN pipes-only self-peer.
///
/// The prog `(fn [self] (send' self (recv' self)))` echoes via its self-peer:
/// recv the parent's 42, send it straight back, return nil. The parent (holding
/// the `Thread'` handle) sends 42, recvs the echo, closes. Sequencing on the
/// depth-1 channels: parent send(42) → worker recv(42) → worker send(42) →
/// parent recv(42) → worker returns → parent close'.
#[test]
fn s2a_thread_prog_drives_self_peer() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [peer (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
                                   (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                                     (:wat::kernel::send' self (:wat::kernel::recv' self))))
                            _ (:wat::kernel::send' peer 42)
                            got (:wat::kernel::recv' peer)
                            _ (:wat::kernel::close' peer)]
            got))
    "#;
    assert_eq!(
        run_compute_i64(src),
        42,
        "thread prog echoes 42 through its pipes-only Peer' self-peer"
    );
}

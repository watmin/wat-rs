//! Arc 259 (The Forced Hand) Stone S2c-i — the per-tier kernel primitive
//! `:wat::kernel::spawn-thread'` (FM-2-bis disconfirming probe).
//!
//! S2c-i extracts the per-tier spawn primitives out of the monolithic
//! `spawn-program'` (`:tier env prog`, tier-keyword dispatch) into standalone
//! 1-arg verbs — `spawn-thread'` (prog) + `spawn-process'` (forms) — that the
//! coming host-type `defclause` (S2c-ii) will dispatch to. No tier keyword, no
//! env arg: just "spawn this thread prog, give me the peer." Additive — the
//! monolithic `spawn-program'` stays live until S2d migrates + cuts it.
//!
//! `spawn-thread'` takes a self-peer prog (the S2a model): the worker is handed
//! its own `Peer'<S,R>` once and drives it. The parent gets back a `Thread'<I,O>`.
//!
//! ## Why this is RED at HEAD
//!
//! `:wat::kernel::spawn-thread'` is not a registered verb at HEAD — the call
//! fails to resolve / type-check. Post-S2c-i it spawns a thread peer and the
//! round-trip returns 42.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_s2ci`

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
        .expect("compute eval (RED at HEAD: spawn-thread' is not a registered verb)")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// LOAD-BEARING (RUNTIME): a thread peer spawned via the 2-arg `spawn-thread'`
/// primitive (prog + init-fn; no tier keyword, no env). The self-peer prog echoes;
/// the parent drives it via the returned `Thread'` handle.
#[test]
fn s2ci_spawn_thread_prime_round_trip() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [peer (:wat::kernel::spawn-thread'
                                   (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                                     (:wat::kernel::send' self (:wat::kernel::recv' self)))
                                   (:wat::core::fn [] -> :wat::Record (:wat::program::EmptyEnv)))
                            _ (:wat::kernel::send' peer 42)
                            got (:wat::kernel::recv' peer)
                            _ (:wat::kernel::close' peer)]
            got))
    "#;
    assert_eq!(
        run_compute_i64(src),
        42,
        "spawn-thread' spawns a thread peer; the self-peer prog echoes 42"
    );
}

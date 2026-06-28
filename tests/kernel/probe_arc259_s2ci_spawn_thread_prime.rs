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
//! Run: `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_s2ci_spawn_thread_prime`
//!
//! WAT fixture: tests/kernel/probe_arc259_s2ci_spawn_thread_prime.wat (co-located sibling)

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_compute_i64() -> i64 {
    let world = startup_beside(file!()).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env)
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// LOAD-BEARING (RUNTIME): a thread peer spawned via `spawn-program'` with the
/// `(thread)` host key. Post S2d, the user path goes through `spawn-program'`;
/// `spawn-thread'` is internal-only. The self-peer prog echoes; RAII reaps.
#[test]
fn s2ci_spawn_thread_prime_round_trip() {
    assert_eq!(
        run_compute_i64(),
        42,
        "spawn-program' (thread) spawns a thread peer; the self-peer prog echoes 42"
    );
}

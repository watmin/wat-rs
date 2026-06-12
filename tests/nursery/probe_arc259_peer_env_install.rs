//! Arc 259 — the per-peer env install (the spawn re-stamp): every peer gets its OWN
//! escape hatch, not just the root main.
//!
//! Today the env is installed only at the freeze seam (root `:user::main`). A
//! spawned thread peer runs its prog with NO env in its thread-local, so
//! `(:wat::program::env)` inside a peer fails "no env installed". This stone installs
//! a fresh env in the spawned thread BEFORE the prog runs:
//!   - `wat.started-at`   INHERITED (same process boot)
//!   - `wat.process-id`   INHERITED (same process)
//!   - `wat.os-thread-id` RE-STAMPED — the spawned thread's own `gettid`
//!   - `wat.peer-kind`    `:thread` (a thread peer shares the address space)
//!   - `wat.peer-started-at` = now (the thread's start)
//!
//! The proof flows back over the CHANNEL (an assertion inside a peer is swallowed by
//! the closure's catch_unwind; only what the peer sends reaches the parent).
//!
//! RED at HEAD: the peer can't read a nonexistent env → it dies before sending → the
//! parent's cascade-aware `recv'` raises → compute errors.
//!
//! Run SERIALLY (spawn probes flake under parallel load):
//!   `cargo test --release -p wat --test nursery probe_arc259_peer_env_install -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Spawn a thread peer whose prog sends `body` (an i64 read from its own env) back;
/// return what the parent receives. Panics (RED) if the peer dies before sending.
fn peer_sends_i64(body: &str) -> i64 {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
           (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread) \
                                    (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                                      (:wat::kernel::send' self {body}))) \
                             got (:wat::kernel::recv' peer) \
                             _ (:wat::kernel::close' peer)] \
             got)) \
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval (RED at HEAD: peer has no env → dies → recv' raises)")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// The peer reads its OWN os-thread-id from its OWN env — a real tid (> 0),
/// distinct from the parent (test) thread's tid.
#[test]
fn thread_peer_reads_its_own_os_thread_id() {
    let parent_tid = unsafe { libc::gettid() } as i64;
    let peer_tid = peer_sends_i64("(:wat::program::Env/wat.os-thread-id (:wat::program::env))");
    assert!(peer_tid > 0, "peer's os-thread-id is a real tid (> 0); got {peer_tid}");
    assert_ne!(
        peer_tid, parent_tid,
        "the spawned peer's tid differs from the parent thread's (its OWN env)"
    );
}

/// The peer's `wat.peer-kind` is `:thread` (it shares the address space) — finally
/// exercising the `:thread` variant the root main never stamps.
#[test]
fn thread_peer_kind_is_thread() {
    let got = peer_sends_i64(
        "(:wat::core::if \
           (:wat::core::= (:wat::program::Env/wat.peer-kind (:wat::program::env)) \
                          :wat::program::PeerKind::thread) -> :wat::core::i64 \
           111 222)",
    );
    assert_eq!(got, 111, "a thread peer's wat.peer-kind is :thread");
}

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
//! Wat source lives in the co-located sibling fixture `probe_arc259_peer_env_install.wat`,
//! slurped via `startup_beside(file!())`.
//!
//! Run SERIALLY (spawn probes flake under parallel load):
//!   `cargo nextest run --release -E 'test(thread_peer)'`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// The peer reads its OWN os-thread-id from its OWN env — a real tid (> 0),
/// distinct from the parent (test) thread's tid.
#[test]
fn thread_peer_reads_its_own_os_thread_id() {
    let parent_tid = unsafe { libc::gettid() } as i64;
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:probe::compute-a)").expect("parse");
    let peer_tid = match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval (RED at HEAD: peer has no env → dies → recv' raises)")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    };
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
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:probe::compute-b)").expect("parse");
    let got = match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    };
    assert_eq!(got, 111, "a thread peer's wat.peer-kind is :thread");
}

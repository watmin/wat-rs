//! Arc 259 S3.5a-0 — the thread-peer crash-reason IPC flaw.
//!
//! THE FLAW: the unified `Peer` contract is violated on the thread transport. When a
//! PROCESS peer's body crashes, the reason travels over the pipe — `ProcessPeerBundle::recv`
//! reads the Err channel (the child's fd 2) on Ok-EOF → `Crashed(reason)`, and `recv'`
//! surfaces that reason (`runtime.rs:22419-22441`). When a THREAD peer's body crashes, the
//! panic is caught and DISCARDED (`spawn.rs:455-458`, `let _ =`); there is no crash channel,
//! and `recv'` maps the disconnect with `|_|` to a generic "peer closed / thread exited"
//! (`runtime.rs:22382`). The failure MESSAGE is silently lost on one transport.
//!
//! Program-compliant fix: give the thread peer a crash channel (the crossbeam analog of the
//! process Err channel); on a caught panic the worker sends the reason; `Thread::recv` reads
//! it on output-EOF → `Crashed(reason)`, and `recv'` surfaces it — exactly like the process
//! peer. (This also gives brackets' cascade-abort the failure message.)
//!
//! RED at HEAD: a thread peer that crashes with a known sentinel message — `recv'` raises, but
//! the reason does NOT contain the sentinel (it is discarded). GREEN once the reason travels.
//!
//! Run SERIALLY (spawns a thread):
//!   `cargo test --release -p wat --test nursery probe_arc259_thread_crash_reason -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Environment;

/// Eval `compute`, which MUST raise (the peer crashed); return the raised error's text.
fn compute_raise_text(body: &str) -> String {
    let src = format!("{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(v) => panic!("expected compute to RAISE (the thread peer crashed); got Ok({v:?})"),
        Err(e) => format!("{e:?}"),
    }
}

/// A thread peer whose body crashes with `BOOM-SENTINEL-9173`. `recv'` raises — and the raised
/// reason MUST carry the sentinel (the crash reason travelled over the pipe), exactly as a
/// process peer's would. RED at HEAD: the thread tier discards the reason → generic message.
#[test]
fn thread_peer_surfaces_crash_reason_over_recv() {
    let err = compute_raise_text(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
           (:wat::core::let \
             [p (:wat::kernel::spawn-program' (:wat::spawn::thread) \
                  (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                    (:wat::kernel::assertion-failed! \"BOOM-SENTINEL-9173\" :wat::core::None :wat::core::None))) \
              _ (:wat::kernel::recv' p)] \
             0))",
    );
    assert!(
        err.contains("BOOM-SENTINEL-9173"),
        "thread peer `recv'` must surface the crash reason over the pipe (like the process peer); \
         the message was discarded. got: {err}"
    );
}

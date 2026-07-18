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

use wat::freeze::call_beside;

/// Call `:user::compute` in the co-located fixture world; return the raised error's text.
/// `compute` MUST raise — the thread peer crashes; the caller asserts on the reason.
fn compute_raise_text() -> String {
    match call_beside(file!(), ":user::compute") {
        Ok(v) => panic!("expected compute to RAISE (the thread peer crashed); got Ok({v:?})"),
        Err(e) => format!("{e:?}"),
    }
}

/// A thread peer whose body crashes with `BOOM-SENTINEL-9173`. `recv'` raises — and the raised
/// reason MUST carry the sentinel (the crash reason travelled over the pipe), exactly as a
/// process peer's would. RED at HEAD: the thread tier discards the reason → generic message.
#[test]
fn thread_peer_surfaces_crash_reason_over_recv() {
    let err = compute_raise_text();
    assert!(
        err.contains("BOOM-SENTINEL-9173"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "thread peer `recv'` must surface the crash reason over the pipe (like the process peer); \
         the message was discarded. got: {err}"
    );
}

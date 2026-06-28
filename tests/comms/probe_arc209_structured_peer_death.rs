//! Arc 209 C0b PREREQUISITE — structured peer death: the prime crash path must carry the
//! STRUCTURED `Failure`, not a flattened message String.
//!
//! Builds on arc 259 S3.5a-0 (`probe_arc259_thread_crash_reason`), which proved the crash
//! *message* travels over the thread peer's crash channel. This probe goes one level deeper:
//! the structured `AssertionPayload` fields — `actual` and `expected` — must ALSO survive.
//!
//! THE REGRESSION: a death carries `(message, Option<AssertionPayload>)`
//! (`extract_panic_payload`, `runtime.rs:18840`), and the `AssertionPayload` holds `actual` +
//! `expected`. But the thread death path DISCARDS the structure — `spawn.rs:472` is
//! `let (message, _assertion) = extract_panic_payload(payload); let _ = crash_tx.send(message)`.
//! Only the message String goes down the `Receiver<String>` crash channel. The old channel
//! `recv` returned `Vector<ThreadDiedError>` (structured); the prime `recv'` regressed it.
//!
//! GREEN once the structured `Failure` flows through the prime crash path.
//!
//! Run SERIALLY (spawns a thread):
//!   `cargo test --release -p wat --test comms probe_arc209_structured_peer_death -- --test-threads=1`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

/// Eval `compute` from the co-located fixture; `compute` MUST raise (the peer crashed).
/// Returns the raised error's text for assertion.
fn compute_raise_text() -> String {
    let world = startup_beside(file!())
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(v) => panic!("expected compute to RAISE (the thread peer crashed); got Ok({v:?})"),
        Err(e) => format!("{e:?}"),
    }
}

/// A thread peer dies via `assertion-failed!` carrying a structured `actual` + `expected`.
/// `recv'` raises — and the raised reason MUST carry BOTH structured fields, not just the
/// message.
#[test]
fn thread_peer_recv_surfaces_structured_actual_and_expected() {
    let err = compute_raise_text();
    // Baseline (already shipped by arc 259 S3.5a-0): the message survives.
    assert!(
        err.contains("structured-death-marker"),
        "regression: the crash MESSAGE must still travel. got: {err}"
    );
    // The new bar: the STRUCTURED actual + expected must survive too.
    assert!(
        err.contains("ACTUAL-42173"),
        "the structured `actual` field must survive the crash path (it is discarded at \
         spawn.rs:472 today). got: {err}"
    );
    assert!(
        err.contains("EXPECTED-99731"),
        "the structured `expected` field must survive the crash path. got: {err}"
    );
}

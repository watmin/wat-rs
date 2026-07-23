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

use wat::freeze::call_beside;

/// Eval `compute` from the co-located fixture. Arc 278 recv'-wall: the crashed peer surfaces as a
/// matchable `RecvOutcome::Lost` VALUE (never a raise — a raise unwinds past the reader). The fixture
/// MATCHES ::Lost and RETURNS the Lost cause's `Failure/message` (the crash-channel envelope). We
/// assert `is_ok` (it matched Lost as a value) + that the returned reason carries all three fields.
fn compute_reason_text() -> String {
    let result = call_beside(file!(), ":user::compute");
    let text = format!("{result:?}");
    assert!(
        result.is_ok(),
        "the thread peer crash must surface as a matchable RecvOutcome::Lost VALUE (never a raise); \
         got Err: {text}"
    );
    assert!(
        // rune:lint(loose-assert) — distinguishing the value-based RecvOutcome marker (::Lost vs the
        // "UNEXPECTED-*" sentinels) among alternatives; the full reason text is machine-specific.
        !text.contains("UNEXPECTED"),
        "the crash must match RecvOutcome::Lost (not ::Message/::Closed); got: {text}"
    );
    text
}

/// A thread peer dies via `assertion-failed!` carrying a structured `actual` + `expected`.
/// The Lost cause's `Failure/message` (the #wat.kernel/AssertionFailure envelope) MUST carry BOTH
/// structured fields, not just the message.
#[test]
fn thread_peer_recv_surfaces_structured_actual_and_expected() {
    let text = compute_reason_text();
    // Baseline (already shipped by arc 259 S3.5a-0): the message survives.
    assert!(
        text.contains("structured-death-marker"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "regression: the crash MESSAGE must still travel. got: {text}"
    );
    // The new bar: the STRUCTURED actual + expected must survive too.
    assert!(
        text.contains("ACTUAL-42173"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "the structured `actual` field must survive the crash path. got: {text}"
    );
    assert!(
        text.contains("EXPECTED-99731"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "the structured `expected` field must survive the crash path. got: {text}"
    );
}

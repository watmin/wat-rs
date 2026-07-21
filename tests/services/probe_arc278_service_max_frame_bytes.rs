//! Arc 278 Stone 1a — the per-service hard frame limit `FOO` (`:max-frame-bytes`) + the honest
//! over-FOO disposition. See docs/arc/2026/06/278-rules-engine/DESIGN-service-io-budgets.md
//! ("Two budget layers").
//!
//! ## Mechanism 1 — the FOO knob
//! A `defservice` DECLARES `:max-frame-bytes N`; it threads from the process child-main's
//! `listener'` → `SocketListener{max_frame_bytes}` → the accepted-connection receivers
//! (`sender_receiver_from_fd_with_budget`). Undeclared services keep the 512 KiB
//! `DEFAULT_MAX_FRAME_BYTES`. PER-SERVICE, not a global raise.
//!   - `large_foo_accepts_a_600kib_request`: a service declaring 1 MiB accepts a ~600 KiB request.
//!     At HEAD (512 KiB default, no knob) this MUTES — the knob is what makes it arrive.
//!
//! ## Mechanism 2 — the honest over-FOO disposition (reply + evict + keep serving)
//! An over-FOO frame is a 400-class CLIENT error, not a 500-class crash. `poll'` routes
//! `RecvError::FrameTooLarge` → the NEW `ServiceEvent::Rejected{idx, cause}`; the serve loop TELLS
//! that client (`Reply::Failed{cause}` via a NON-BLOCKING `try-send'` — the deadlock guard), EVICTS
//! just that connection (discarding the un-read oversized residual), and KEEPS SERVING everyone
//! else. NOT the mute `Closed`, NOT the terminal `Lost` (whose `eprintln` is wat's panic — a
//! client-triggerable service crash / DoS).
//!   - `small_foo_over_budget_fails_with_the_reason`: the caller's op fails carrying the reason
//!     ("too large" / "exceeded" / "max-frame-bytes"), NOT the mute "peer closed / channel
//!     disconnected".
//!   - `small_foo_service_survives_an_over_budget_frame`: a fresh in-budget request on a new
//!     connection succeeds after a different connection sent an over-FOO frame — one dumb client
//!     cannot DoS the shared service.
//!
//! Over-FOO is a PROCESS-tier concept (byte frames); the thread tier has no frames.

use wat::freeze::call_beside;
use wat::runtime::Value;

/// mechanism 1: a service declaring `:max-frame-bytes 1048576` (1 MiB) on PROCESS accepts a
/// ~600 KiB request. At HEAD (512 KiB default, no knob) this MUTES — the per-service FOO is what
/// makes the legit bulk write arrive (the arena's ~600 KiB write).
#[test]
fn large_foo_accepts_a_600kib_request() {
    let got = call_beside(file!(), ":user::large-foo-accepts")
        .unwrap_or_else(|e| panic!("large-FOO ~600 KiB request must SUCCEED, got raise: {e:?}"));
    assert!(
        matches!(got, Value::i64(7)),
        "expected PutResponse.ok == 7 for a ~600 KiB request under a 1 MiB FOO; got {got:?}"
    );
}

/// mechanism 2: a service declaring `:max-frame-bytes 4096` rejects a >4 KiB request — the caller's
/// op FAILS carrying the REASON (a catchable `Reply::Failed`), NOT the mute "peer closed / channel
/// disconnected". The over-FOO connection is closed; the reason reaches the caller.
#[test]
fn small_foo_over_budget_fails_with_the_reason() {
    let result = call_beside(file!(), ":user::small-foo-rejects");
    let err = result.expect_err(
        "a >4 KiB request under a 4096-byte FOO must FAIL (over-FOO reject), not succeed",
    );
    let msg = format!("{err:?}");
    assert!(
        // rune:lint(loose-assert) — a property over a per-run-variable diagnostic (the reason
        // rides a recv'-surfaced Reply::Failed carrying a source location); we assert the reason
        // SUBSTANCE (names the frame cap) is present AND the mute mask is absent — the
        // legitimately-loose case the lint documents, not a fixed value.
        (msg.contains("too large") || msg.contains("exceeded") || msg.contains("max-frame-bytes"))
            && !msg.contains("channel disconnected"),
        "THE LAW (wat never hides a failure): an over-FOO reject must carry the frame REASON to the \
         caller (e.g. 'request too large — exceeded this service's max-frame-bytes limit'), a \
         catchable Reply::Failed — NOT a mute 'peer closed / channel disconnected'. Got: {msg}"
    );
}

/// mechanism 2: the service SURVIVES an over-FOO frame — a fresh in-budget request on a new
/// connection succeeds after a different connection sent an over-FOO frame. One dumb client cannot
/// DoS the shared service (this is what routing to the TERMINAL `Lost` arm would have broken).
#[test]
fn small_foo_service_survives_an_over_budget_frame() {
    let got = call_beside(file!(), ":user::small-foo-survives")
        .expect("service must SURVIVE an over-FOO frame — a fresh in-budget request must succeed");
    assert!(
        matches!(got, Value::i64(7)),
        "fresh in-budget request after an over-FOO frame must return 7 (service alive); got {got:?}"
    );
}

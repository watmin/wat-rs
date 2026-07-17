//! Arc 278 — wat NEVER HIDES A FAILURE (see DESIGN-no-hidden-failures.md).
//!
//! A `journal'` service forked to a PROCESS receives a client message it cannot decode (a `Log` whose
//! `message` is the user record `:probe::Note`, absent from the forked child's baked type registry).
//! BEFORE the fix, the child died with a rich, located reason —
//!   "poll' (process tier): client message decode failed: ... unknown tag #probe/Note (body shape:
//!    map); no matching struct or enum in the type registry"
//! — that was written to an ALREADY-CLOSED err pipe (EPIPE) and LOST; the caller's `write-logs` `recv'`
//! raised a MUTE "recv failed: peer closed / channel disconnected".
//!
//! THE LAW: the caller's error must CARRY the reason. This differs from
//! `probe_arc272_rs2_crash_surfaces_to_client`, which only asserts the crash *raises* (is_err) — a mute
//! raise passes that. Here we assert the raise carries the REASON. GREEN via Mechanism A (the
//! protocol-tier completion of the outcome-enum model): `poll'` returns a `ServiceEvent::Malformed`
//! carrying the cause instead of raising; the serve loop replies `Reply::Failed{cause}` to the caller
//! and keeps serving; `recv'` surfaces `Reply::Failed` as a raise carrying the cause.
//!
//! Run: cargo test --release -p wat --test services dead_child_speaks

use wat::freeze::call_beside;

#[test]
fn a_forked_service_that_cannot_decode_a_message_speaks_its_reason_to_the_caller() {
    // The undecodable message MUST raise (not hang, not fake a value) — and, crucially, the raise MUST
    // carry the child's real reason, not a mute mask.
    let result = call_beside(file!(), ":user::compute");
    let err = result.expect_err(
        "write-logs of an undecodable payload across a process fork must RAISE (the child cannot decode it)",
    );
    let msg = format!("{err:?}");
    assert!(
        // rune:lint(loose-assert) — the raised error embeds a per-run-variable source location
        // (edn_shim.rs:LINE:COL); we assert the diagnostic SUBSTANCE (that it names the undecodable
        // tag / decode failure) is present — a property over a variable message, the legitimately-loose
        // case the lint documents, not a deterministic value that owes an exact assert_eq!.
        msg.contains("unknown tag")
            || msg.contains("decode failed")
            || msg.contains("no matching struct or enum"),
        "THE LAW (wat never hides a failure): the caller's error must carry the child's real reason \
         (e.g. 'unknown tag #probe/Note ... no matching struct or enum in the type registry'). \
         Instead it surfaced a MUTE mask: {msg}"
    );
}

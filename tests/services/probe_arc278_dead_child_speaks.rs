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

use wat::freeze::call_beside_value;

#[test]
fn a_forked_service_that_cannot_decode_a_message_speaks_its_reason_to_the_caller() {
    // EXACT DATA-EQUALITY (no `.contains`): the fixture MATCHES the RecvOutcome as a VALUE (a raise
    // would unwind past the reader — the mask the wall kills) and returns a structured :probe::Outcome
    // whose in-wat `reason-names-decode-failure?` bool proves THE LAW — the caller's error carries the
    // child's real reason ("...no matching struct or enum in the type registry"), never a mute mask.
    // The golden #probe.Outcome/Lost [true] is captured (UPDATE_EDN=1), never hand-authored; the
    // per-run-variable Failure location stays in wat, only its boolean RESULT crosses. Mirrors the
    // canonical gate probe_arc278_recv_outcome_wall. "wat stdio is edn — it's always data" (builder).
    let v = call_beside_value(file!(), ":user::compute").unwrap_or_else(|e| {
        panic!(
            "the undecodable payload across a process fork must surface as a matchable \
             RecvOutcome::Lost VALUE (never a raise, which would unwind past the reader); got Err: {e:?}"
        )
    });
    let edn = ::wat_edn::write(&wat::edn_shim::value_to_edn_with(&v, None).expect("the probe's value must encode"));
    wat::assert_edn_matches_file!(
        edn,
        "dead_child_speaks__forked_service_speaks_its_reason.edn",
        "THE LAW (wat never hides a failure): the forked child's decode failure must MATCH \
         RecvOutcome::Lost (never the mute ::Closed/::Message) and CARRY its real reason"
    );
}

//! Arc 209 C0b PREREQUISITE — structured peer death: the prime crash path must carry the
//! STRUCTURED `Failure`, not a flattened message String.
//!
//! Builds on arc 259 S3.5a-0 (`probe_arc259_thread_crash_reason`), which proved the crash
//! *message* travels over the thread peer's crash channel. This probe goes one level deeper:
//! the structured `AssertionPayload` fields — `actual` and `expected` — must ALSO survive.
//!
//! Arc 278 no-hidden-failures (the string-wrap annihilation, deeper): the thread crash channel
//! now carries a STRUCTURED `Vector<LociDiedError>` (parity with the process tier — same bare
//! `[#wat.kernel.LociDiedError/…]` EDN line `loci_died_error_from_reason` bridges), NOT the
//! flattened `#wat.kernel/AssertionFailure {…}` envelope String. So `message`, `actual`, and
//! `expected` ride in the `Failure` RECORD's own fields — the fixture reads them STRUCTURALLY
//! off `Failure/message` / `Failure/actual` / `Failure/expected` and joins them with "|". This
//! probe asserts that EXACT structured value: the fields survived as DATA, not scraped out of a
//! stringified blob. (Before the fix, actual/expected survived ONLY because they were embedded
//! in the envelope String — the resurrected string-wrap this fix kills.)
//!
//! Run SERIALLY (spawns a thread):
//!   `cargo test --release -p wat --test comms probe_arc209_structured_peer_death -- --test-threads=1`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Eval `compute` from the co-located fixture. Arc 278 recv'-wall: the crashed peer surfaces as a
/// matchable `RecvOutcome::Lost` VALUE (never a raise — a raise unwinds past the reader). The
/// fixture MATCHES ::Lost → ::Panic → its `Some(Failure)` and RETURNS the three surviving fields
/// (`Failure/message` | `Failure/actual` | `Failure/expected`) joined with "|".
fn compute_reason_text() -> String {
    match call_beside_value(file!(), ":user::compute").expect("compute should run") {
        Value::String(s) => (*s).clone(),
        other => panic!(
            "the thread peer crash must surface as a matchable RecvOutcome::Lost VALUE \
             carrying a structured Failure; got: {other:?}"
        ),
    }
}

/// A thread peer dies via `assertion-failed!` carrying a structured `message` + `actual` +
/// `expected`. All three MUST survive the crash path STRUCTURALLY — read off the `Failure`
/// record's own fields, not scraped from a stringified envelope. Because they are the user's
/// own literal strings (no host-specific path/pid/timestamp), the whole joined value is exact.
#[test]
fn thread_peer_recv_surfaces_structured_actual_and_expected() {
    let text = compute_reason_text();
    assert_eq!(
        text, "structured-death-marker|ACTUAL-42173|EXPECTED-99731",
        "the structured message/actual/expected must survive the crash path as DATA \
         (read off the Failure record's fields), not scraped from a stringified envelope"
    );
}

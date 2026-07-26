//! Arc 272 record-state rs-2 (crash half) — a far-side crash SURFACES to the client as a raise, not a
//! hang or a fake value. The graceful/crash duality of the service contract: `(<svc>/stop c)` returns the
//! final state on a clean terminate; a crashed handler makes the client's call RAISE the crash reason.
//!
//! This is the EXISTING substrate crash-surfacing (peer.rs:110-123 thread crash channel; runtime.rs:23771
//! process Err channel → `PeerRecvError::Crashed` → `#wat.kernel/ProcessPanics`), exercised THROUGH the
//! generated client face: an op handler that crashes → the service dies → the client's `recv'` of the
//! reply raises the reason (deadlock-free — recv' checks the crash channel on output-EOF, never hangs).
//! gen_server-faithful: calling a dead server raises.
//!
//! Likely GREEN at HEAD already (the client face + crash-surfacing both exist) — this LOCKS that contract
//! as a tested invariant for the final-state feature; it must stay green after the `:Stop` work lands.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs2_crash_surfaces_to_client

use wat::freeze::call_beside_value;

#[test]
fn far_side_crash_raises_to_the_client_not_hang_or_fake() {
    // arc 291 4b-ii: State is now a defstruct; :durable mints ::Record; start takes ::Record.
    // Wat source lives in the co-located fixture: probe_arc272_rs2_crash_surfaces_to_client.wat
    //
    // Arc 278 recv'-wall: the far-side crash surfaces as a matchable RecvOutcome::Lost VALUE (never a
    // raise — a raise unwinds past the reader). The fixture MATCHES the outcome and RETURNS a marker
    // ("LOST:<administrative msg>" on the crash, "MESSAGE"/"CLOSED" otherwise). The crash must NOT
    // hang and must NOT fake a value — it must surface as ::Lost, distinct from ::Message/::Closed.
    let result = call_beside_value(file!(), ":user::compute");
    let text = format!("{result:?}");
    assert!(
        result.is_ok(),
        "the far-side crash must surface as a matchable RecvOutcome::Lost VALUE (never a raise, which \
         would unwind past the reader); got Err: {text}"
    );
    assert!(
        // rune:lint(loose-assert) — distinguishing the value-based RecvOutcome marker ("LOST:<reason>"
        // vs "MESSAGE"/"CLOSED"); the appended reason text is machine-specific.
        text.contains("LOST:"),
        "expected the far-side crash to surface to the client as RecvOutcome::Lost (recv' surfaces \
         the crash), not ::Message/::Closed; instead the call returned: {text}"
    );
}

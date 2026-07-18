//! Arc 278 no-hidden-failures — transport-tier twin, RED gate 2 (the crash half, locked).
//!
//! `probe_arc272_rs2_crash_surfaces_to_client` only asserts `is_err()` on a THREAD-locus service
//! whose handler genuinely panics — a mute raise would pass that assertion. This probe strengthens
//! the check on the PROCESS locus (the transport this arc's remaining hole is about): a process
//! service whose handler GENUINELY panics (a real runtime panic mid-handler via
//! `assertion-failed!` — NOT a decode rejection, which is Mechanism A's path, covered by
//! `probe_arc278_dead_child_speaks`) must make the caller's error CONTAIN the crash reason, not
//! just be `is_err`.
//!
//! Existing substrate crash-surfacing for the process tier: `ProcessPeerBundle::recv` reads the
//! Err channel (child's fd 2) on Ok-EOF → `PeerRecvError::Crashed(reason)` (`kernel/spawn.rs:316-327`,
//! `classify_peer_error`/`classify_peer_death`), and `recv'` surfaces it (`runtime.rs:26159-26172`).
//! That mechanism belongs to the OWNER's spawn handle (the `Process<I,O>`/`Thread<I,O>` returned by
//! `/start`, holding the dedicated fd-2 err pipe wired at fork). This test instead calls through
//! `c = connect'(Handle/addr h)` — a SEPARATE peer, matching `probe_arc272_rs2_crash_surfaces_to_client`'s
//! shape exactly (that is the test this brief asked to strengthen).
//!
//! STOP-2 FINDING (2026-07-18, this session): this gate IS RED at HEAD, and stays RED after the
//! transport-tier `RecvError::Failed` work, for a STRUCTURAL reason, not an EPIPE-timing bug. The
//! unified `Peer` struct a `connect'`-ed client holds (`kernel/peer.rs:206-210`) has exactly two
//! fields — `tx` and `rx` — NO crash-channel field at all (unlike `Thread<I,O>`/`Process<I,O>`,
//! which each carry a dedicated `crash`/`err` receiver). Verified empirically: a THREAD-locus
//! variant of this exact scenario (co-located scratch probe, since deleted) shows the SAME mute
//! `"recv failed: peer closed / channel disconnected"` on the client `recv'`, even though the raw
//! crash payload IS observed via eprintln-terminal on the panicking thread itself
//! (`#wat.kernel/AssertionFailure {... BOOM-SENTINEL-... }`). So the gap is not process-tier-specific
//! and not a timing race: when the WHOLE spawned unit (thread or process) dies from an unhandled
//! panic, every live `connect'`-ed client just sees its socket/channel close — there is no reason
//! to thread through `RecvError` here because the actual transport event IS a genuine clean EOF
//! (`RecvError::Disconnected`), not a decodable wire failure. Delivering the reason to `c` would
//! require the dying unit to broadcast its crash payload to every live client connection before
//! exiting — a new cross-cutting mechanism touching the defservice codegen
//! (`wat/service.wat`'s serve loop) and the panic-catching boundary, well outside this strike's
//! blast radius (`src/comms/*`, `src/channel/transfer.rs`, `src/kernel/spawn.rs`, `src/runtime.rs`
//! recv' surfacing only). Per the brief's own STOP-2 clause, surfacing this mechanism — not
//! guessing a fix — is the correct action. Ignored so the scoped gate stays green; the orchestrator
//! decides whether this becomes its own follow-on strike.
//!
//! Run (RED, ignored): cargo test --release -p wat --test services process_crash_reason_carried -- --ignored

use wat::freeze::call_beside;

#[test]
#[ignore = "STOP-2 (arc 278 transport-tier twin, 2026-07-18): a connect'd client Peer has NO \
            crash channel (kernel/peer.rs Peer{tx,rx} only) — the crash reason is structurally \
            unreachable here, not an EPIPE timing bug. See module doc for the full finding."]
fn a_process_handler_that_genuinely_panics_carries_its_reason_to_the_client() {
    let result = call_beside(file!(), ":user::compute");
    let err = result.expect_err(
        "expected the process handler's genuine panic to RAISE to the client (recv' surfaces \
         the crash reason via the Err/fd-2 channel), not hang or fake a value",
    );
    let msg = format!("{err:?}");
    assert!(
        // rune:lint(loose-assert) — asserts the crash reason carries the embedded sentinel; the
        // full error Debug embeds a per-run-variable source location, not a deterministic value
        // that owes assert_eq!.
        msg.contains("BOOM-SENTINEL-PROCESS-4471"),
        "THE LAW (wat never hides a failure) — transport-tier twin: a process handler's genuine \
         crash must surface its REASON to the caller (the sentinel embedded in the \
         assertion-failed! message), not just an is_err() mute raise. Got: {msg}"
    );
}

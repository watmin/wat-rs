//! Arc 278 #15 (mute-kill floor) — the REJECT-AND-KEEP-SERVING facet, at the comms layer.
//!
//! A defservice is a network service facing untrusted clients who WILL send over-budget frames
//! (dumb or hostile). The service must reject the bad frame WITH A REASON and KEEP SERVING — one
//! dumb client must not be able to wedge the connection (nor, at the service layer, DoS the whole
//! service). At the transport, that means: after a receiver rejects an over-budget frame
//! (`RecvError::FrameTooLarge`, which already carries its Display — "frame exceeded cap …"), it must
//! DRAIN the oversized frame to its terminating newline and RE-ALIGN, so the NEXT in-budget frame
//! reads.
//!
//! RED at HEAD: `take_frame` returns `Err(FrameTooLarge)` WITHOUT draining the accumulator (it only
//! `split_off`s on a *good* frame — src/comms/process.rs), so every subsequent `recv()` re-scans the
//! SAME oversized bytes → `FrameTooLarge` forever. The channel is STUCK: the in-budget frame queued
//! behind the bad one is unreachable. That wedged connection IS the dumb-client vulnerability #15
//! forbids. GREEN once the `FrameTooLarge` path drains-to-the-newline and continues.
//!
//! Modeled on tests/comms/probe_arc278_transport_reason_carried.rs (raw comms pair, no WAT runtime,
//! no spawned process). Hang-safe: `recv()` fast-paths `take_buffered_frame()?` before any blocking
//! read (src/comms/process.rs:640), so recv #2 returns from the retained accumulator immediately.

use wat::comms::{process::pair_with_budget, RecvError};

/// After a receiver rejects an over-budget frame, it must recover and read the next in-budget frame.
#[test]
fn over_budget_frame_is_rejected_with_reason_then_the_channel_recovers() {
    // A 64-byte per-message cap.
    let (sender, receiver) = pair_with_budget::<String>(64).expect("budgeted comms pair");

    // A bad client's over-budget frame (100 chars → framed > 64 bytes), then a good in-budget frame.
    sender.send("X".repeat(100)).expect("send over-budget frame");
    sender.send("ok".to_string()).expect("send in-budget frame");
    // Not a clean-EOF scenario — the peer stays alive; the frames are the whole story.
    let _keep_alive = &sender;

    // recv #1 — the over-budget frame is REJECTED WITH A REASON (not decoded, not mute).
    // `FrameTooLarge`'s Display carries the cap detail — the SPEAK floor at the comms layer.
    match receiver.recv() {
        Err(RecvError::FrameTooLarge) => { /* expected: reasoned rejection */ }
        other => panic!(
            "over-budget frame: recv #1 must be Err(FrameTooLarge) (a reasoned rejection), got {other:?}"
        ),
    }

    // recv #2 — THE LAW (reject-and-keep-serving): the receiver must have DRAINED the oversized frame
    // and RE-ALIGNED; the next in-budget frame must read. RED at HEAD: the channel is stuck on the
    // un-drained oversized bytes → FrameTooLarge again, never "ok".
    match receiver.recv() {
        Ok(s) => assert_eq!(
            s, "ok",
            "after rejecting an over-budget frame, the next in-budget frame must read (drain-realign)"
        ),
        other => panic!(
            "THE LAW — an over-budget frame must be REJECT-AND-KEEP-SERVING, not a wedged channel: \
             after rejecting the oversized frame the receiver must drain it to the newline, re-align, \
             and read the next in-budget frame. Got {other:?} — the channel is STUCK (the oversized \
             bytes were never drained; take_frame doesn't discard on FrameTooLarge). This is the #15 \
             mute-kill floor's keep-serving half: one dumb client's frame must not wedge the wire."
        ),
    }
}

//! Arc 278 item (c) — RED gate: SELF-SCHEDULING DEFSERVICES.
//! See docs/arc/2026/06/278-rules-engine/DESIGN-self-scheduling-defservices.md.
//!
//! A `defservice` arms a `-tick` internal op (leading dash = reactor-internal, NOT on the surface),
//! carried in the handler's `Outcome` as an `Alarm`; it fires on the service's own timer (env-grab
//! tier → both loci), re-arms itself to `target`, and advances a durable counter — while a client
//! `poll` still replies (the reactor serves between ticks). RED NOW: no `Alarm`/`ReplyAndArm`/
//! `NoReplyAndArm`; the serve loop threads `clients`, not `selectables`; the leading dash is dropped
//! by kebab->pascal → the fixture cannot even type-check, `call_beside_value` raises. GREEN when the stone
//! lands (count == target, poll replies). The mechanism is proven hand-rolled in
//! wat-scripts/scratch-pad/probe-self-scheduling-loop.wat — this gate proves the GENERATED serve loop.
//!
//! STATUS 2026-08-30: GREEN, both loci. The stone landed; what remained after it was a fixture that
//! released its own service handle, not a substrate gap. See the un-ignore notes below.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// (thread locus) A self-armed `-tick` fires + re-arms to `target` (3), and `poll` still replies.
/// Returns the polled count; GREEN iff it equals the target 3 (fired thrice, re-armed each time,
/// on the service's own select loop, and the reactor served the poll between ticks).
// UN-IGNORED 2026-08-30. This stood #[ignore]d for 38 days on a cause that was never measured.
// Its reason named "the remove-at idx-shift (service.wat:958/961) evicting the client peer" —
// inferred from the symptom `recv': peer closed`, never verified. Three things were wrong with it:
// `remove-at` is at service.wat:1591/1594 (the cited lines had drifted ~630 and now hold unrelated
// handle-name minting); the self-scheduling mechanism reaches target at BOTH loci; and the eviction
// reproduces with NO timer armed at all, so the timer was never involved.
//
// What actually kept it red was the FIXTURE releasing its own service: `drive-ticker` drove from
// the let's BODY, i.e. tail position, which ends the scope holding the handle before the call runs.
// See the comment at drive-ticker in the .wat beside this file.
#[test]
fn self_tick_fires_rearms_and_reactor_serves_thread() {
    let got = call_beside_value(file!(), ":user::self-tick-rearms-thread").unwrap_or_else(|e| {
        panic!("the self-scheduling `-tick` must fire + re-arm and poll must reply; got raise: {e:?}")
    });
    match got {
        Value::i64(n) => assert_eq!(
            n, 3,
            "the `-tick` must fire + re-arm to target=3 while poll still replies; got {n} \
             (a -1 = poll got RequestTooLarge; a <3 = the timer did not re-arm to target; the \
             self-scheduling capability is not built)"
        ),
        other => panic!("expected the polled Count(3); got {other:?}"),
    }
}

/// (process locus) Identical, but the service is forked to a process — the `-tick` timer must arm at
/// the PROCESS tier (env-grab: the service's own kind), proving the capability is loci-agnostic.
// UN-IGNORED 2026-08-30 — see the note on the thread-locus twin above. The process tier was
// measured too: the -tick arms at the service's own tier (env-grab) and reaches target.
#[test]
fn self_tick_fires_rearms_and_reactor_serves_process() {
    let got = call_beside_value(file!(), ":user::self-tick-rearms-process").unwrap_or_else(|e| {
        panic!("the process-tier self-scheduling `-tick` must fire + re-arm and poll must reply; \
                got raise: {e:?}")
    });
    match got {
        Value::i64(n) => assert_eq!(
            n, 3,
            "the process-tier `-tick` must fire + re-arm to target=3 (env-grab arms it at the \
             process tier) while poll still replies; got {n}"
        ),
        other => panic!("expected the polled Count(3); got {other:?}"),
    }
}

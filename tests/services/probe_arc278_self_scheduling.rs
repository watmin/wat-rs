//! Arc 278 item (c) — RED gate: SELF-SCHEDULING DEFSERVICES.
//! See docs/arc/2026/06/278-rules-engine/DESIGN-self-scheduling-defservices.md.
//!
//! A `defservice` arms a `-tick` internal op (leading dash = reactor-internal, NOT on the surface),
//! carried in the handler's `Outcome` as an `Alarm`; it fires on the service's own timer (env-grab
//! tier → both loci), re-arms itself to `target`, and advances a durable counter — while a client
//! `poll` still replies (the reactor serves between ticks). RED NOW: no `Alarm`/`ReplyAndArm`/
//! `NoReplyAndArm`; the serve loop threads `clients`, not `selectables`; the leading dash is dropped
//! by kebab->pascal → the fixture cannot even type-check, `call_beside` raises. GREEN when the stone
//! lands (count == target, poll replies). The mechanism is proven hand-rolled in
//! wat-scripts/scratch-pad/probe-self-scheduling-loop.wat — this gate proves the GENERATED serve loop.

use wat::freeze::call_beside;
use wat::runtime::Value;

/// (thread locus) A self-armed `-tick` fires + re-arms to `target` (3), and `poll` still replies.
/// Returns the polled count; GREEN iff it equals the target 3 (fired thrice, re-armed each time,
/// on the service's own select loop, and the reactor served the poll between ticks).
#[ignore = "arc 278 item (c) RED gate — BLOCKED on Stone 2 (the self-scheduling design ABOVE the \
            multiplexer), NOT the poll'/timer gap: Stone 1 CLOSED that — `after` now builds a UNIFIED \
            Peer' that joins `poll'` by construction (proven, both tiers, in \
            wat-scripts/scratch-pad/probe-timer-as-peer.wat; the tier-open `Timer'` is retired). Still \
            RED because Stone 2 is unbuilt: no `Alarm`/`ReplyAndArm`/`NoReplyAndArm`, the serve loop \
            threads `clients` not `selectables`, and the leading-dash `-tick` is not synthesized → the \
            fixture cannot type-check. Un-ignore when Stone 2 lands (count == target, poll replies)."]
#[test]
fn self_tick_fires_rearms_and_reactor_serves_thread() {
    let got = call_beside(file!(), ":user::self-tick-rearms-thread").unwrap_or_else(|e| {
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
#[ignore = "arc 278 item (c) RED gate — BLOCKED on Stone 2 (the self-scheduling design ABOVE the \
            multiplexer), NOT the poll'/timer gap: Stone 1 CLOSED that — `after` now builds a UNIFIED \
            Peer' that joins `poll'` by construction (proven, both tiers, in \
            wat-scripts/scratch-pad/probe-timer-as-peer.wat; the tier-open `Timer'` is retired). Still \
            RED because Stone 2 is unbuilt: no `Alarm`/`ReplyAndArm`/`NoReplyAndArm`, the serve loop \
            threads `clients` not `selectables`, and the leading-dash `-tick` is not synthesized → the \
            fixture cannot type-check. Un-ignore when Stone 2 lands (count == target, poll replies)."]
#[test]
fn self_tick_fires_rearms_and_reactor_serves_process() {
    let got = call_beside(file!(), ":user::self-tick-rearms-process").unwrap_or_else(|e| {
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

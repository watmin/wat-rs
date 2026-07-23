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
// TRACKED, item-(c): the arc-278 widening lands the CHECK (superset-O selectables type-check), but the
// generated serve loop's Stone 2-A runtime — Alarm→timer arm + `-tick` fire + re-arm — crashes mid-tick
// (`send': channel disconnected`); the timer is still in the wrong location (DESIGN-self-scheduling-
// defservices.md: `after` → a unified `Peer'<nil,O>`). NOT a masked regression — an UNBUILT stone,
// being built next; the mechanism is proven hand-rolled in wat-scripts/scratch-pad/probe-self-scheduling-loop.wat.
#[test]
#[ignore = "item-c: the -tick op-ref colon fix (UnboundSymbol) landed; remaining = the remove-at idx-shift \
            (service.wat:958/961) evicting the client peer, + the send'-wall makes the failure legible (DESIGN-send-outcome-wall.md)"]
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
#[test]
#[ignore = "item-c: the -tick op-ref colon fix (UnboundSymbol) landed; remaining = the remove-at idx-shift \
            (service.wat:958/961) evicting the client peer, + the send'-wall makes the failure legible (DESIGN-send-outcome-wall.md)"]
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

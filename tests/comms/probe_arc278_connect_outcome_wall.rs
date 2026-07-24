//! Arc 278 peer-lifecycle Strike 4 — the `connect'` OUTCOME WALL (the LAST peer wall).
//!
//! `:wat::kernel::connect'` used to RETURN a bare `Peer'<S,R>` and RAISE on its
//! *handleable* failures (ECONNREFUSED / no listener / rendezvous gone, the
//! `OnlyThisPeer` identity reject, `peer_cred` read + socket-wrap io error). Per the
//! peer-lifecycle LAW (2026-07-23) — *"we deliver an enum for code to handle exceptions
//! with; raise is uncatchable on purpose, a thing that must never happen"* — every
//! handleable failure is now a matchable `:wat::kernel::ConnectOutcome<S,R>` variant
//! (PARAMETRIC + Impure, mirroring `AcceptOutcome<R,S>` — `Connected` holds a live
//! `Peer'`):
//!   Connected [peer <- Peer'<S,R>]  — dialed + admitted (the happy path)
//!   Refused   [cause <- Failure]    — ECONNREFUSED / no listener / rendezvous gone (RETRYABLE)
//!   Rejected  [cause <- Failure]    — OnlyThisPeer identity check failed (NOT retryable)
//!   Failed    [cause <- Failure]    — peer_cred read / socket-wrap io error
//! The must-never-happen raises (arity, address-type-mismatch, the in-process
//! malformed-address substrate bug) stay raises.
//!
//! The returned Value is asserted STRUCTURALLY (`Value::Enum` field extraction),
//! never a loose `format!("{:?}").contains(...)`.
//!
//! RED before the wall: `connect'` returned a bare `Peer'` opaque (`Value::RustOpaque`)
//! on success and RAISED on the handleable failures, so `as_connect_outcome` panics
//! ("not a ConnectOutcome enum value") on the happy path and `connect-refused` raised
//! rather than yielding `Refused`. GREEN after.
//!
//! The `Rejected[cause]` (OnlyThisPeer identity mismatch) and `Failed[cause]` (peer_cred
//! / socket-wrap io) variants are process-tier-only and not cheaply reachable
//! single-threaded — they need a real UDS server that isn't the address minter / a
//! broken accepted socket under the forked test binary. They are constructed by the SAME
//! `connect_outcome_{rejected,failed}` helpers (identical enum-value construction,
//! differing only in variant + a `message_only_failure` payload) and mapped from the
//! `!connect_admits` / `peer_cred` / socket-wrap arms of `SocketAddress::connect`
//! (address.rs). No live probe fakes them here (the brief forbids faking a hard-to-reach
//! path — the accept'/close' precedent).
//!
//! Run: cargo test --release -p wat --test comms probe_arc278_connect_outcome_wall -- --test-threads=1

use std::sync::Arc;
use wat::freeze::call_beside;
use wat::runtime::{EnumValue, Value};

/// Extract a `:wat::kernel::ConnectOutcome` enum value, asserting the type path.
fn as_connect_outcome(v: &Value) -> &Arc<EnumValue> {
    match v {
        Value::Enum(ev) => {
            assert_eq!(
                ev.type_path, ":wat::kernel::ConnectOutcome",
                "connect' must return ConnectOutcome; got type_path {:?} (variant {:?})",
                ev.type_path, ev.variant_name
            );
            ev
        }
        other => panic!(
            "connect' must return a ConnectOutcome enum value, not a bare Peer' opaque / a raise; got {:?}",
            other
        ),
    }
}

// ─── happy dial → Connected[peer] (single-threaded; runs in the floor) ─────────

/// A thread-tier `connect'` queues a connect-request in the bounded(1) rendezvous
/// slot (non-blocking) against a LIVE listener and returns the client Peer' →
/// `ConnectOutcome::Connected[peer]`.
///
/// RED before the wall: `connect'` returned a bare `Peer'` opaque (`Value::RustOpaque`),
/// so `as_connect_outcome` panics ("not a ConnectOutcome enum value"). GREEN after.
#[test]
fn connect_to_live_listener_yields_connected() {
    let v = call_beside(file!(), ":user::connect-happy")
        .unwrap_or_else(|e| panic!("connect' should eval to a ConnectOutcome, not raise: {e:?}"));
    let ev = as_connect_outcome(&v);
    assert_eq!(
        ev.variant_name, "Connected",
        "a dial into a live rendezvous is Connected; got {:?}",
        ev.variant_name
    );
    assert_eq!(ev.fields.len(), 1, "Connected carries one field (peer); got {:?}", ev.fields);
    // The payload is a LIVE peer — the wrapped `PEER_TYPE_PATH` opaque, not a plain value.
    match &ev.fields[0] {
        Value::RustOpaque(inner) => assert_eq!(
            inner.type_path,
            wat::kernel::spawn::PEER_TYPE_PATH,
            "Connected.peer must be a Peer' opaque; got type_path {:?}",
            inner.type_path
        ),
        other => panic!("Connected.peer must be a live Peer' opaque; got {:?}", other),
    }
}

// ─── retryable transport → Refused[cause] (single-threaded; runs in the floor) ──

/// `connect'` on an address whose listener (the only rendezvous Receiver) was dropped
/// before the dial → crossbeam send Disconnected → `ConnectOutcome::Refused[cause]`, a
/// retryable transport failure, NOT a raise the dialer unwinds past.
///
/// RED before the wall: `connect'` RAISED ("rendezvous send failed — listener was
/// dropped") instead of returning `Refused`, so `call_beside` returns `Err` and the
/// `unwrap_or_else` panics. GREEN after.
#[test]
fn connect_to_dropped_listener_yields_refused() {
    let v = call_beside(file!(), ":user::connect-refused")
        .unwrap_or_else(|e| panic!("connect' on a dropped listener must yield Refused, not raise: {e:?}"));
    let ev = as_connect_outcome(&v);
    assert_eq!(
        ev.variant_name, "Refused",
        "connect' on a dropped-listener rendezvous is Refused; got {:?}",
        ev.variant_name
    );
    assert_eq!(ev.fields.len(), 1, "Refused carries one field (cause <- Failure); got {:?}", ev.fields);
    // The cause is a structured `:wat::kernel::Failure`, not a flat String.
    match &ev.fields[0] {
        Value::Enum(_) | Value::Aggregate(_) => {}
        other => panic!("Refused.cause must be a structured Failure; got {:?}", other),
    }
}

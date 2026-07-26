//! Arc 278 peer-lifecycle Strike 3 — the `accept'` OUTCOME WALL.
//!
//! `:wat::kernel::accept'` used to RETURN a bare `Peer'<R,S>` and RAISE on its
//! *handleable* failures (rendezvous dropped/shutdown, decode error, `select`
//! error, `peer_cred` read fail). Per the peer-lifecycle LAW (2026-07-23) — *"we
//! deliver an enum for code to handle exceptions with; raise is uncatchable on
//! purpose, a thing that must never happen"* — every handleable failure is now a
//! matchable `:wat::kernel::AcceptOutcome<R,S>` variant (PARAMETRIC + Impure,
//! mirroring `RecvOutcome<O>` — `Accepted` holds a live `Peer'`):
//!   Accepted [peer <- Peer'<R,S>]  — an AUTHORIZED peer connected (the happy path)
//!   Closed   []                    — the rendezvous shut down / address dropped (clean)
//!   Failed   [cause <- Failure]    — decode / select / peer_cred / socket-wrap io error
//! The security bounce stays INTERNAL (drop + re-poll — `Rejected` is deliberately
//! CUT), and the must-never-happen raises (arity, listener-type-mismatch, the
//! in-process malformed-connect-request substrate bug) stay raises.
//!
//! The returned Value is asserted STRUCTURALLY (`Value::Enum` field extraction),
//! never a loose `format!("{:?}").contains(...)`.
//!
//! RED before the wall: `accept'` returned a bare `Peer'` opaque (`Value::RustOpaque`)
//! on success and RAISED on the handleable failures, so `as_accept_outcome` panics
//! ("not an AcceptOutcome enum value") on the happy path and `accept-closed` raised
//! rather than yielding `Closed`. GREEN after.
//!
//! The `Failed[cause]` variant (a decode / select / peer_cred / socket-wrap io error)
//! is not cheaply reachable single-threaded — it needs a corrupted wire / a broken
//! accepted socket under the forked test binary. It is constructed by the SAME
//! `accept_outcome_failed` helper (identical enum-value construction, differing only
//! in variant + a `message_only_failure` payload) and mapped from the select /
//! peer_cred / socket-wrap arms of `SocketListener::accept` (listener.rs). No live
//! probe fakes it here (the brief forbids faking a hard-to-reach path).
//!
//! Run: cargo test --release -p wat --test comms probe_arc278_accept_outcome_wall -- --test-threads=1

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::{EnumValue, Value};

/// Extract a `:wat::kernel::AcceptOutcome` enum value, asserting the type path.
fn as_accept_outcome(v: &Value) -> &Arc<EnumValue> {
    match v {
        Value::Enum(ev) => {
            assert_eq!(
                ev.type_path, ":wat::kernel::AcceptOutcome",
                "accept' must return AcceptOutcome; got type_path {:?} (variant {:?})",
                ev.type_path, ev.variant_name
            );
            ev
        }
        other => panic!(
            "accept' must return an AcceptOutcome enum value, not a bare Peer' opaque / a raise; got {:?}",
            other
        ),
    }
}

// ─── happy path → Accepted[peer] (single-threaded; runs in the floor) ──────────

/// A thread-tier `connect'` queues a connect-request in the bounded(1) rendezvous
/// slot (non-blocking) and returns the client Peer'; `accept'` then dequeues it and
/// wraps the authorized server Peer' → `AcceptOutcome::Accepted[peer]`.
///
/// RED before the wall: `accept'` returned a bare `Peer'` opaque (`Value::RustOpaque`),
/// so `as_accept_outcome` panics ("not an AcceptOutcome enum value"). GREEN after.
#[test]
fn accept_authorized_peer_yields_accepted() {
    let v = call_beside_value(file!(), ":user::accept-happy")
        .unwrap_or_else(|e| panic!("accept' should eval to an AcceptOutcome, not raise: {e:?}"));
    let ev = as_accept_outcome(&v);
    assert_eq!(
        ev.variant_name, "Accepted",
        "a queued authorized connect-request is Accepted; got {:?}",
        ev.variant_name
    );
    assert_eq!(ev.fields.len(), 1, "Accepted carries one field (peer); got {:?}", ev.fields);
    // The payload is a LIVE peer — the wrapped `PEER_TYPE_PATH` opaque, not a plain value.
    match &ev.fields[0] {
        Value::RustOpaque(inner) => assert_eq!(
            inner.type_path,
            wat::kernel::spawn::PEER_TYPE_PATH,
            "Accepted.peer must be a Peer' opaque; got type_path {:?}",
            inner.type_path
        ),
        other => panic!("Accepted.peer must be a live Peer' opaque; got {:?}", other),
    }
}

// ─── clean terminal → Closed[] (single-threaded; runs in the floor) ────────────

/// `accept'` on a listener whose address (the only rendezvous Sender) was dropped
/// before the accept → crossbeam recv Disconnected → `AcceptOutcome::Closed[]`, a
/// clean terminal, NOT a raise the server loop unwinds past.
///
/// RED before the wall: `accept'` RAISED ("rendezvous recv failed — address was
/// dropped or shutdown") instead of returning `Closed`, so `call_beside_value` returns
/// `Err` and the `unwrap_or_else` panics. GREEN after.
#[test]
fn accept_on_dropped_rendezvous_yields_closed() {
    let v = call_beside_value(file!(), ":user::accept-closed")
        .unwrap_or_else(|e| panic!("accept' on a dropped rendezvous must yield Closed, not raise: {e:?}"));
    let ev = as_accept_outcome(&v);
    assert_eq!(
        ev.variant_name, "Closed",
        "accept' on a dropped-address rendezvous is Closed; got {:?}",
        ev.variant_name
    );
    assert_eq!(ev.fields.len(), 0, "Closed is reason-free (no fields); got {:?}", ev.fields);
}

//! Arc 278 — THE ACCEPTANCE GATE for the call context: an OPT-IN third arm parameter
//! `[s ctx req]` on a `defservice` op handler, carrying a five-field pure context
//! (`:wat::service::CallCtx` — a PLACEHOLDER name, STOP-5: an intueri cast is owed), with a
//! stable monotonic caller id minted in the generated serve loop and travelling WITH its peer
//! (STOP-2: never a parallel position-keyed vector).
//!
//! See docs/arc/2026/06/278-rules-engine/BRIEF-the-call-context.md +
//! DESIGN-STONE-the-call-context.md. Modelled on
//! tests/services/probe_arc278_per_op_request_too_large.{rs,wat} (connect/round-trip shape).
//!
//! Three things, and the third is the one that matters most (the brief, verbatim):
//!   1. A 3-param arm receives a POPULATED ctx (namespace == the service fqdn, operation == the
//!      op's own kebab name, caller-id present).
//!   2. A 2-param arm in the SAME service still works — proving OPT-IN, not migration.
//!   3. ★ THE STABILITY GATE — the id survives an eviction: connect three clients, disconnect
//!      the MIDDLE one, then have a SURVIVOR call an op and assert it still sees its ORIGINAL
//!      id. A position-keyed implementation passes every other test and fails only this one.
//!
//! Run: cargo test --release -p wat --test probe_arc278_call_context

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// (1a) The 3-param `whoami` arm's ctx carries a present caller-id and the op's own name
/// (`operation` == "whoami", spliced as a compile-time literal from the macro's own `op-str`).
#[test]
fn ctx_populated_caller_id_and_operation() {
    let got = call_beside_value(file!(), ":user::ctx-populated-id-and-op")
        .unwrap_or_else(|e| panic!("ctx-populated-id-and-op raised: {e:?}"));
    match got {
        Value::Tuple(items) => {
            assert_eq!(items.len(), 2, "expected a 2-tuple (caller-id, operation); got {items:?}");
            match (&items[0], &items[1]) {
                (Value::i64(caller_id), Value::String(operation)) => {
                    assert!(
                        *caller_id >= 0,
                        "the FIRST connection's caller-id must be a present, non-negative \
                         monotonic id; got {caller_id}"
                    );
                    assert_eq!(
                        operation.as_str(),
                        "whoami",
                        "ctx.operation must equal the op's own kebab name (a compile-time \
                         literal splice of the macro's `op-str`); got {operation:?}"
                    );
                }
                other => panic!("expected (i64, String); got {other:?}"),
            }
        }
        other => panic!("expected a Tuple(caller-id, operation); got {other:?}"),
    }
}

/// (1b) ctx.namespace equals the SERVICE's own fqdn (`:probe::callctx3svc`) — a compile-time
/// literal splice of the macro's own `fqdn-kw`, kept as its own test (a keyword, not packable
/// alongside the i64/String pair above without a third accessor round-trip).
#[test]
fn ctx_namespace_is_the_service_fqdn() {
    let got = call_beside_value(file!(), ":user::ctx-namespace-is-fqdn")
        .unwrap_or_else(|e| panic!("ctx-namespace-is-fqdn raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: ctx.namespace must equal the service's own fqdn :probe::callctx3svc; \
         got {got:?}"
    );
}

/// (2) A 2-param `[s req]` arm (`ping`) in the SAME service still works, untouched — the proof
/// that the third ctx param is OPT-IN per op, not a service-wide migration.
#[test]
fn two_param_arm_still_works_opt_in_not_migration() {
    let got = call_beside_value(file!(), ":user::two-param-arm-still-works")
        .unwrap_or_else(|e| panic!("two-param-arm-still-works raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: the ordinary 2-param `ping` arm, in the SAME service as the 3-param \
         `whoami` ctx arm, must keep working unmodified; got {got:?}"
    );
}

/// (3) ★ THE STABILITY GATE — the defect most likely to ship green. Connect three clients
/// (c1, c2, c3, ids 0/1/2 by construction — each `connect'` is a blocking handshake, so the
/// mint order is deterministic), evict the MIDDLE one (c2), then have c3 (the survivor whose
/// POSITION in `selectables` shifts down when c2 is removed) call `whoami` again.
///
/// A position-keyed implementation reports c3's CURRENT index (1, having shifted down from 2)
/// instead of its ORIGINAL minted id (2) — passing every other test in this file and failing
/// only here. Two independent assertions: id-before == id-after (stability across the
/// eviction), AND id-after == 2 (the analytically correct third-ever-minted id, not merely
/// "whatever id-before happened to be").
#[test]
fn stability_gate_survivor_keeps_original_id_across_middle_eviction() {
    let got = call_beside_value(file!(), ":user::stability-gate")
        .unwrap_or_else(|e| panic!("stability-gate raised: {e:?}"));
    match got {
        Value::Tuple(items) => {
            assert_eq!(items.len(), 2, "expected a 2-tuple (id-before, id-after); got {items:?}");
            match (&items[0], &items[1]) {
                (Value::i64(id_before), Value::i64(id_after)) => {
                    assert_eq!(
                        id_before, id_after,
                        "the survivor's caller-id must be IDENTICAL before and after the \
                         middle client's eviction — a position-keyed id would shift once the \
                         middle client (idx 1) is removed and the survivor (originally idx 2) \
                         slides down to idx 1; got before={id_before} after={id_after}"
                    );
                    assert_eq!(
                        *id_after, 2,
                        "the survivor was the THIRD client ever connected (ids 0/1/2 by \
                         construction), so its stable id must be 2 regardless of the middle \
                         client's later eviction — a position-keyed scheme would report 1 \
                         (its post-eviction array index) instead; got {id_after}"
                    );
                }
                other => panic!("expected (i64, i64); got {other:?}"),
            }
        }
        other => panic!("expected a Tuple(id-before, id-after); got {other:?}"),
    }
}

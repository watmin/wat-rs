//! Arc 278 — THE ACCEPTANCE GATE for ctx-is-mandatory: EVERY public `defservice` op arm now
//! takes the MANDATORY third param `[s ctx req]` (`:wat::service::Invocation`), and every
//! internal (`-`) op arm takes `[s ctx]` (`:wat::service::SelfInvocation`) — no longer opt-in.
//!
//! See docs/arc/2026/06/278-rules-engine/BRIEF-ctx-is-mandatory.md +
//! DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md (which SUPERSEDES BRIEF-the-call-context.md /
//! DESIGN-STONE-the-call-context.md — this file used to be that strike's gate; it is now this
//! one's, upgraded arm-by-arm rather than replaced). Modelled on
//! tests/services/probe_arc278_per_op_request_too_large.{rs,wat} (connect/round-trip shape).
//!
//! Four things, and the second is the one that matters most (STOP-0's own words — the test that
//! would have caught it, and nothing else in the suite could):
//!   1. A public arm receives a POPULATED `Invocation` (namespace == the service fqdn,
//!      operation == the op's own kebab name, conn-id present).
//!   2. ★ An internal arm receives a POPULATED `SelfInvocation`, read THROUGH the ctx binder.
//!   3. A 2-param public op arm is now a LOCATED COMPILE ERROR naming the op (negative fixture:
//!      probe_arc278_call_context_two_param_public_arm.wat.bad).
//!   4. ★ THE STABILITY GATE — the id survives an eviction: connect three clients, disconnect
//!      the MIDDLE one, then have a SURVIVOR call an op and assert it still sees its ORIGINAL
//!      id. A position-keyed implementation passes every other test and fails only this one.
//!
//! Run: cargo test --release -p wat --test probe_arc278_call_context

use wat::freeze::{call_beside_value, startup_from_file};
use wat::macros::MacroErrorKind;
use wat::{MacroError, StartupError};
use wat::runtime::Value;

/// (1a) The 3-param `whoami` arm's ctx carries a present conn-id and the op's own name
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

/// A second public op (`ping`) in the SAME service also carries ctx unconditionally — not the
/// "opt-in per op" proof it used to be (that framing died with the opt-in design), just a check
/// that ctx isn't special-cased to whichever op happens to be declared first.
#[test]
fn second_public_arm_also_works() {
    let got = call_beside_value(file!(), ":user::second-public-arm-also-works")
        .unwrap_or_else(|e| panic!("second-public-arm-also-works raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: the second public `ping` arm, in the SAME service as `whoami`, must \
         work with its own mandatory ctx; got {got:?}"
    );
}

/// (2) ★ THE test — an INTERNAL arm receives a POPULATED `SelfInvocation`, read through the ctx
/// binder. This is the test STOP-0 named directly: before the fix, the internal branch's
/// `let-bindings` took the param VECTOR's shape but only the OLD 2-item [s-binder, state]
/// contents, so a second binder (`ctx`) was silently dropped — a body that referenced it would
/// have type-checked and then read... nothing, because the binder was never bound. This test
/// proves the opposite: `-mark`'s `ctx` carries `operation == "-mark"` and
/// `namespace == :probe::callctx3svc`, both read THROUGH the ctx binder and round-tripped out via
/// durable state (the only channel an internal op has — it has no client to reply to).
#[test]
fn internal_arm_receives_populated_self_invocation() {
    let got = call_beside_value(file!(), ":user::internal-arm-ctx-populated")
        .unwrap_or_else(|e| panic!("internal-arm-ctx-populated raised: {e:?}"));
    match got {
        Value::Tuple(items) => {
            assert_eq!(
                items.len(),
                2,
                "expected a 2-tuple (operation-is-dash-mark, namespace-is-fqdn); got {items:?}"
            );
            match (&items[0], &items[1]) {
                (Value::bool(op_ok), Value::bool(ns_ok)) => {
                    assert!(
                        *op_ok,
                        "the internal `-mark` arm's SelfInvocation ctx.operation must equal \
                         \"-mark\" (STOP-0: this is exactly the binder that used to be silently \
                         dropped)"
                    );
                    assert!(
                        *ns_ok,
                        "the internal `-mark` arm's SelfInvocation ctx.namespace must equal the \
                         service's own fqdn :probe::callctx3svc"
                    );
                }
                other => panic!("expected (bool, bool); got {other:?}"),
            }
        }
        other => panic!("expected a Tuple(bool, bool); got {other:?}"),
    }
}

/// (3) A public op arm declared `[s req]` (2 params, the OLD opt-in shape) is now a LOCATED
/// compile error naming the op — ctx is mandatory, arity is never a dispatch fallback (STOP-2).
/// Negative fixture, loaded via `startup_from_file`; RED is pass.
#[test]
fn two_param_public_arm_is_a_located_compile_error() {
    let err = startup_from_file(
        "tests/services/probe_arc278_call_context_two_param_public_arm.wat.bad",
    )
    .expect_err("a public op arm declared [s req] (2 params) must fail to load");

    // Destructure to the REASON rather than string-matching the rendering. Two things this buys,
    // and the second is why neither a `contains` nor an `.edn` golden was right here:
    //
    //  1. It is STRONGER. `msg.contains("public op 'ping'")` passes on a message that names the op
    //     and then says anything at all. The reason is the contract this strike owes its caller —
    //     name the op, state the expected shape, report what was got — so it is asserted verbatim.
    //
    //  2. It is STABLE. The rendered error's inner span points at the `macro-error` call site
    //     INSIDE `wat/service.wat` (line ~1159 today), which moves on every unrelated edit to the
    //     most-churned file in the arc. A whole-value `.edn` golden would pin that span and go red
    //     for reasons it does not care about — a gate that trains re-baselining. The `reason` is a
    //     span-free scalar, so `assert_eq!` on it is the lint's own prescription for a scalar, with
    //     no exemption needed. A `rune:lint(loose-assert)` would NOT have been honest here: the
    //     lint exempts values that vary per RUN (pid/path/hash/timestamp); this one is fully
    //     deterministic per tree and merely brittle, which is a different thing.
    let StartupError::Macro(MacroError { kind: MacroErrorKind::ProgramBodyEvalFailed { cause, .. }, .. }) = &err
    else {
        panic!("expected StartupError::Macro(ProgramBodyEvalFailed); got {err:?}")
    };
    let MacroErrorKind::MalformedTemplate { reason } = &cause.kind else {
        panic!("expected the inner cause to be MalformedTemplate; got {:?}", cause.kind)
    };
    assert_eq!(
        reason,
        "defservice: public op 'ping' must have shape [s ctx req] (3 params); got 2 params",
        "the arity refusal must name the op, state the expected shape, AND report what it got"
    );
}

/// (4) ★ THE STABILITY GATE — the defect most likely to ship green. Connect three clients
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

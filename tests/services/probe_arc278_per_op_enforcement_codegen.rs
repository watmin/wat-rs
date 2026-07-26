//! Arc 278 #16.2 — RED gate: the per-op `:max-request-bytes` ENFORCEMENT CODEGEN.
//! See docs/arc/2026/06/278-rules-engine/DESIGN-service-io-budgets.md (STATUS: "16.2 the per-op
//! ENFORCEMENT codegen") + the CURARE CHECKPOINT in DESIGN-no-hidden-failures.md.
//!
//! The op declares `:max-request-bytes 200` on the SURFACE; the `:impls` body returns bare `:Ok`
//! (no hand-rolled measure — that was the feasibility probe, option a). Enforcement must come from
//! the serve-loop codegen (16.2). RED NOW: no codegen → the over-cap request gets the body's `:Ok`
//! (test 1 → -1). GREEN AFTER: the codegen measures + returns `RequestTooLarge` before the body.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// (1) An over-op-budget request must be flagged `RequestTooLarge{bytes, cap}` BY THE CODEGEN —
/// the `:impls` body returns bare `:Ok`, so a RequestTooLarge reply proves the serve-loop enforced
/// it. The entrypoint returns `bytes` (> the 200 cap) on the RequestTooLarge arm, `-1` on the Ok arm.
/// RED until 16.2 lands (the body's `:Ok` comes back → -1).
#[test]
fn codegen_flags_over_op_request_the_body_does_not() {
    let got = call_beside_value(file!(), ":user::over-op-codegen-flags").unwrap_or_else(|e| {
        panic!("the over-op request must be flagged RequestTooLarge by the codegen, got raise: {e:?}")
    });
    match got {
        Value::i64(bytes) => assert!(
            bytes > 200,
            "the codegen-flagged RequestTooLarge.bytes must exceed the 200-byte cap (the exact \
             encoded request size); got {bytes} — a -1 means the Ok arm matched (the codegen did \
             NOT enforce; the bare-:Ok body came back for the over-cap request)"
        ),
        other => panic!(
            "expected RequestTooLarge{{bytes,cap}} from the codegen → bytes>200; got {other:?}"
        ),
    }
}

/// (2) The SAME connection recovers IN PLACE: after an over-op request is flagged `RequestTooLarge`
/// (connection KEPT — arrived <= FOO, wire synced, a normal reply, no eviction), a follow-up
/// in-budget request ON THE SAME peer returns `Ok`. Guards that 16.2's RequestTooLarge path does
/// not close the connection. Returns `1` if the follow-up succeeded on the same peer, else `-1`.
#[test]
fn same_connection_survives_the_codegen_flag() {
    let got = call_beside_value(file!(), ":user::same-conn-recovers-after-codegen").unwrap_or_else(|e| {
        panic!("the in-budget follow-up on the SAME connection must succeed, got raise: {e:?}")
    });
    match got {
        Value::i64(n) => assert!(
            n > 0,
            "the in-budget follow-up on the SAME connection must return Ok (1); a -1 means the \
             connection did not survive the RequestTooLarge flag; got {n}"
        ),
        other => panic!("expected Ok on the SAME connection after the RequestTooLarge flag; got {other:?}"),
    }
}

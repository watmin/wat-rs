//! Arc 278 #16 (c) — FEASIBILITY: the per-op `:max-request-bytes` mechanism, hand-rolled.
//! See docs/arc/2026/06/278-rules-engine/DESIGN-service-io-budgets.md ("The response contract",
//! "Two budget layers" — the per-op limit, matchable, connection LIVES) + the SESSION-END
//! breadcrumb in DESIGN-no-hidden-failures.md (#16/#17 (c): checker-forced, serve-loop-enforced,
//! match-forced budget).
//!
//! This is the disconfirming/foundation probe for option (c): it proves 16.2's MECHANISM composes
//! on the substrate before the checker+codegen strike — an op whose `<Op>Response` is an ENUM
//! with `RequestTooLarge{bytes, cap}`, the body measuring `:wat::edn::write` length and returning
//! the NAMED variant over a small cap. GREEN here means the strike (16.0 annotation → 16.1 checker
//! wall → 16.2 serve-loop codegen) only needs to MOVE this measure+construct out of the body and
//! into the auto-enforced path — the mechanism itself is sound.

use wat::freeze::call_beside;
use wat::runtime::Value;

/// (1) An over-op-budget request returns the MATCHABLE `RequestTooLarge{bytes, cap}` variant —
/// a value the client `match`es (NOT a raise, NOT a lumped `cause` string). The entrypoint returns
/// `bytes` (> the 200 cap) on the RequestTooLarge arm, `-1` on the Ok arm.
#[test]
fn over_op_request_returns_a_matchable_too_large_variant() {
    let got = call_beside(file!(), ":user::over-op-returns-matchable").unwrap_or_else(|e| {
        panic!("over-op request must return a matchable RequestTooLarge variant, got raise: {e:?}")
    });
    match got {
        Value::i64(bytes) => assert!(
            bytes > 200,
            "the matched RequestTooLarge.bytes must exceed the 200-byte cap (the exact encoded \
             request size); got {bytes} — a -1 would mean the Ok arm matched (the over-cap request \
             was NOT flagged)"
        ),
        other => panic!(
            "expected RequestTooLarge{{bytes,cap}} matched → bytes>200 (a matchable value); got {other:?}"
        ),
    }
}

/// (2) The SAME connection recovers IN PLACE: after an over-op request returns `RequestTooLarge`
/// (connection KEPT — the request arrived <= FOO, wire synced, so it is a normal reply, no
/// eviction), a follow-up in-budget request ON THE SAME peer returns `Ok`. This is the whole point
/// of the per-op tier vs the transport `FOO` kick: recoverable in place, not closed.
#[test]
fn same_connection_recovers_in_place_after_too_large() {
    let got = call_beside(file!(), ":user::same-conn-recovers").unwrap_or_else(|e| {
        panic!("the in-budget follow-up on the SAME connection must succeed, got raise: {e:?}")
    });
    match got {
        Value::i64(n) => assert!(
            n > 0,
            "the in-budget Ok.n (the encoded request size) must be >0 — a -1 means the follow-up on \
             the SAME connection did NOT get Ok (the connection did not survive the RequestTooLarge); \
             got {n}"
        ),
        other => panic!("expected Ok{{n}} on the SAME connection after RequestTooLarge; got {other:?}"),
    }
}

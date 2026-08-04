//! Arc 278 BRIEF-client-validates-locally — the generated `defservice` client method must
//! refuse an over-budget request LOCALLY (against the surface's own `:max-request-bytes`), and
//! must NOT `recv` after a send it never made.
//!
//! See docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-client-validates-locally.md.
//!
//! STOP-1's discriminator: a fat request refused locally and one refused by the server's own
//! per-op guard return the byte-identical `RequestTooLarge{bytes,cap}` — so `probe_arc278_
//! client_validates_locally.wat`'s fixture deliberately sizes its poison payload OVER the
//! service's `:max-frame-bytes` (FOO) too, making that exact value UNREACHABLE via the wire (FOO
//! would intercept first, pre-decode, and evict the connection). Getting `RequestTooLarge` back
//! at all, AND the same connection surviving a follow-up in-budget request, is the actual proof
//! nothing was sent — an actually-sent poison frame would have tripped FOO and killed `c`.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// The client refuses an over-budget request locally: no send, no recv, the SAME
/// `RequestTooLarge{bytes,cap}` value a server would have sent (cap names the surface's own
/// `:max-request-bytes` contract, 100 — never the service's `:max-frame-bytes`, 2048). Then the
/// SAME connection completes an ordinary in-budget request — impossible had the poison request
/// actually reached the wire, since it is sized over FOO and would have gotten `c` evicted.
#[test]
fn over_budget_refused_locally_then_connection_survives() {
    let got = call_beside_value(file!(), ":user::over-budget-refused-locally-then-connection-survives")
        .unwrap_or_else(|e| {
            panic!(
                "a locally-refused over-budget request followed by an in-budget request on the \
                 SAME connection must both succeed (the fixture's own in-wat assertions name \
                 exactly which shape broke); got raise: {e:?}"
            )
        });
    assert!(
        matches!(got, Value::i64(7)),
        "expected the follow-up PutResponse.ok == 7 (proof the connection survived the earlier \
         local refusal untouched); got {got:?}"
    );
}

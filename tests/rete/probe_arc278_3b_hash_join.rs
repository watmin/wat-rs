//! Arc 278 stone 3b — disconfirming probe: `HashJoinNode` (the two-sided equality join). RED at HEAD.
//!
//! THE HEART: a two-condition rule joining on `?loc`. The HashJoinNode crosses Tokens (left, from the
//! root-join's beta-memory) against Elements (right, from the WindSpeed alpha), unifying when the shared
//! `?loc` agrees. The cold-and-windy join, end to end.
//!
//!   (:Temperature (?loc <- :location) (?t <- :celsius))
//!   (:WindSpeed    (?loc <- :location) (?w <- :kph))
//!
//! - MATCH (same loc): one joined Token with ?loc/?t/?w all bound.
//! - NO JOIN (diff loc): zero tokens at the HashJoinNode (the ?loc keys disagree — the join drops it).
//!
//! RED at HEAD: `fire-rules` does root-join seeding only (3a) → the HashJoinNode's beta-memory is empty.
//!
//! Run: cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside;
use wat::runtime::Value;

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_produces_one_token_on_same_loc
//   src/rete/kernel.rs #[cfg(test)]::hash_join_drops_on_mismatched_loc
//   src/rete/kernel.rs #[cfg(test)]::hash_join_no_cross_loc_leakage
#[test]
#[ignore]
fn join_produces_one_token_on_matching_loc() {
    let got = call_beside(file!(), ":user::htoks-length-oslo").expect("eval");
    assert_eq!(got, Value::i64(1), "Temp+Wind at the same loc → one joined Token; got {got:?}");
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_produces_one_token_on_same_loc
#[test]
#[ignore]
fn joined_token_unifies_both_conditions() {
    assert_eq!(call_beside(file!(), ":user::oslo-t-binding").expect("eval"),
        Value::Option(Arc::new(Some(Value::i64(15)))), "?t bound from Temperature");
    assert_eq!(call_beside(file!(), ":user::oslo-w-binding").expect("eval"),
        Value::Option(Arc::new(Some(Value::i64(45)))), "?w bound from WindSpeed");
    assert_eq!(call_beside(file!(), ":user::oslo-loc-binding").expect("eval"),
        Value::Option(Arc::new(Some(Value::String(Arc::new("Oslo".to_string()))))), "?loc unified");
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_drops_on_mismatched_loc
#[test]
#[ignore]
fn join_drops_on_mismatched_loc() {
    let got = call_beside(file!(), ":user::htoks-length-bergen").expect("eval");
    assert_eq!(got, Value::i64(0), "Temp(Oslo)+Wind(Bergen) → no joined Token; got {got:?}");
}

// HAZARD #1 — cross-product leakage. 2 Temps × 2 Winds across 2 locations must yield EXACTLY the 2 same-loc
// joins (Oslo×Oslo, Bergen×Bergen), NOT 4 (a naive cross ignoring ?loc) and NOT 0 (a bad compatibility check).
//
// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_no_cross_loc_leakage
#[test]
#[ignore]
fn join_no_cross_loc_leakage() {
    let got = call_beside(file!(), ":user::htoks-length-2x2").expect("eval");
    assert_eq!(got, Value::i64(2), "2 Temps × 2 Winds / 2 locs → exactly 2 same-loc joins (not 4, not 0); got {got:?}");
}

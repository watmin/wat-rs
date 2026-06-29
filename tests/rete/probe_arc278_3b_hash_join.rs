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
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// Build the fired session for a given WindSpeed location, then isolate the HashJoinNode's Tokens (`htoks`).
fn setup(wind_loc: &str) -> String {
    format!("\
   c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
   c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
   rule  (:wat::rete::Rule \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector))\
   sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
   sess1 (:wat::rete::insert sess0 (:user::Temperature 15 \"Oslo\"))\
   sess2 (:wat::rete::insert sess1 (:user::WindSpeed 45 \"{wind_loc}\"))\
   fired (:wat::rete::fire-rules sess2)\
   network (:wat::rete::Session/network fired)\
   bmem  (:wat::rete::Session/beta-memory fired)\
   hjid  (:wat::core::Option/expect -> :wat::core::i64 \
            (:wat::core::get \
              (:wat::core::filter \
                (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool \
                  (:wat::core::= (:wat::rete::node-kind-label \
                                   (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) \"n\")) \
                                 \"HashJoinNode\")) \
                (:wat::core::PersistentMap/keys network)) \
              0) \"hjid\")\
   htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector \
            ((:wat::core::Some pv) pv) \
            (:wat::core::None (:wat::core::PersistentVector)))")
}

fn ev(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_produces_one_token_on_same_loc
//   src/rete/kernel.rs #[cfg(test)]::hash_join_drops_on_mismatched_loc
//   src/rete/kernel.rs #[cfg(test)]::hash_join_no_cross_loc_leakage
#[test]
#[ignore]
fn join_produces_one_token_on_matching_loc() {
    let got = ev(&format!("(:wat::core::let [{}] (:wat::core::length htoks))", setup("Oslo")));
    assert_eq!(got, Value::i64(1), "Temp+Wind at the same loc → one joined Token; got {got:?}");
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_produces_one_token_on_same_loc
#[test]
#[ignore]
fn joined_token_unifies_both_conditions() {
    let s = setup("Oslo");
    let binds = format!("(:wat::core::let [{s} \
        tok (:wat::core::Option/expect -> :wat::rete::Token (:wat::core::get htoks 0) \"tok\") \
        b   (:wat::rete::Token/bindings tok)]");
    assert_eq!(ev(&format!("{binds} (:wat::core::PersistentMap/get b \"?t\"))")),
        Value::Option(Arc::new(Some(Value::i64(15)))), "?t bound from Temperature");
    assert_eq!(ev(&format!("{binds} (:wat::core::PersistentMap/get b \"?w\"))")),
        Value::Option(Arc::new(Some(Value::i64(45)))), "?w bound from WindSpeed");
    assert_eq!(ev(&format!("{binds} (:wat::core::PersistentMap/get b \"?loc\"))")),
        Value::Option(Arc::new(Some(Value::String(Arc::new("Oslo".to_string()))))), "?loc unified");
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_drops_on_mismatched_loc
#[test]
#[ignore]
fn join_drops_on_mismatched_loc() {
    let got = ev(&format!("(:wat::core::let [{}] (:wat::core::length htoks))", setup("Bergen")));
    assert_eq!(got, Value::i64(0), "Temp(Oslo)+Wind(Bergen) → no joined Token; got {got:?}");
}

// HAZARD #1 — cross-product leakage. 2 Temps × 2 Winds across 2 locations must yield EXACTLY the 2 same-loc
// joins (Oslo×Oslo, Bergen×Bergen), NOT 4 (a naive cross ignoring ?loc) and NOT 0 (a bad compatibility check).
const SETUP_2X2: &str = "\
   c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
   c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
   rule  (:wat::rete::Rule \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector))\
   s0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
   s1 (:wat::rete::insert s0 (:user::Temperature 15 \"Oslo\"))\
   s2 (:wat::rete::insert s1 (:user::Temperature 10 \"Bergen\"))\
   s3 (:wat::rete::insert s2 (:user::WindSpeed 45 \"Oslo\"))\
   s4 (:wat::rete::insert s3 (:user::WindSpeed 50 \"Bergen\"))\
   fired (:wat::rete::fire-rules s4)\
   network (:wat::rete::Session/network fired)\
   bmem  (:wat::rete::Session/beta-memory fired)\
   hjid  (:wat::core::Option/expect -> :wat::core::i64 \
            (:wat::core::get \
              (:wat::core::filter \
                (:wat::core::fn [k <- :wat::core::i64] -> :wat::core::bool \
                  (:wat::core::= (:wat::rete::node-kind-label \
                                   (:wat::core::Option/expect -> :wat::core::Record (:wat::core::PersistentMap/get network k) \"n\")) \
                                 \"HashJoinNode\")) \
                (:wat::core::PersistentMap/keys network)) 0) \"hjid\")\
   htoks (:wat::core::match (:wat::core::PersistentMap/get bmem hjid) -> :wat::core::PersistentVector \
            ((:wat::core::Some pv) pv) \
            (:wat::core::None (:wat::core::PersistentVector)))";

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::hash_join_no_cross_loc_leakage
#[test]
#[ignore]
fn join_no_cross_loc_leakage() {
    let got = ev(&format!("(:wat::core::let [{SETUP_2X2}] (:wat::core::length htoks))"));
    assert_eq!(got, Value::i64(2), "2 Temps × 2 Winds / 2 locs → exactly 2 same-loc joins (not 4, not 0); got {got:?}");
}

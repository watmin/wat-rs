//! Arc 278 stone 4a — production-fire (token → RHS → derived fact).
//!
//! After the equality-join network matches, a Token reaching the ProductionNode FIREs the rule's RHS:
//! evaluate `(:weather::ColdAndWindy ?loc)` with the token's bindings into a derived
//! `:weather::ColdAndWindy` record, stored in production-memory.
//! Live mouths: `compile-all`, `insert`, `fire-rules`, `query`.
//!
//!   :when  [(:weather::Temperature (?loc <- :location) (?t <- :celsius))
//!           (:weather::WindSpeed    (?loc <- :location) (?w <- :kph))]
//!   :then  [(:weather::ColdAndWindy :location ?loc)]
//!
//! - MATCH (same loc): the join yields one Token → the RHS fires → ONE ColdAndWindy("Oslo") in production-memory.
//! - NO JOIN (diff loc): zero tokens at the ProductionNode → zero derived facts.
//! - 2×2 (no leakage): 2 Temps × 2 Winds / 2 locs → exactly 2 same-loc joins → exactly 2 derived facts.
//!
//! Run: cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn compile_cw_fires_nothing() {
    let got = call_beside_value(file!(), ":user::compile-cw-fires-nothing").expect("eval");
    assert_eq!(got, Value::i64(0), "compile+fire with no facts derives nothing; got {got:?}");
}

#[test]
fn fired_oslo_builds() {
    let _ = call_beside_value(file!(), ":test::fired-oslo").expect("eval");
}

#[test]
fn fired_bergen_builds() {
    let _ = call_beside_value(file!(), ":test::fired-bergen").expect("eval");
}

#[test]
fn production_fires_one_fact_on_matching_loc() {
    let got = call_beside_value(file!(), ":user::pfacts-length-oslo").expect("eval");
    assert_eq!(got, Value::i64(1), "Temp+Wind at the same loc → the RHS fires once → one derived fact; got {got:?}");
}

#[test]
fn derived_fact_is_cold_and_windy_at_oslo() {
    // The derived fact is a :weather::ColdAndWindy whose location field came from the token's ?loc binding.
    assert_eq!(call_beside_value(file!(), ":user::fact-type-oslo").expect("eval"),
        Value::String(Arc::new("weather::ColdAndWindy".to_string())), "derived fact is a ColdAndWindy");
    assert_eq!(call_beside_value(file!(), ":user::fact-location-oslo").expect("eval"),
        Value::String(Arc::new("Oslo".to_string())), "location bound from the unified ?loc");
}

#[test]
fn no_fire_on_mismatched_loc() {
    let got = call_beside_value(file!(), ":user::pfacts-length-bergen").expect("eval");
    assert_eq!(got, Value::i64(0), "Temp(Oslo)+Wind(Bergen) → no token reaches the production → no derived fact; got {got:?}");
}

// HAZARD — one fact per activation, no cross-product. 2 Temps × 2 Winds / 2 locs → exactly the 2 same-loc
// joins → exactly 2 derived facts (NOT 4 from a blind cross, NOT 1 from a clobbered accumulator).
#[test]
fn no_cross_loc_leakage() {
    let got = call_beside_value(file!(), ":user::pfacts-length-2x2").expect("eval");
    assert_eq!(got, Value::i64(2), "2 Temps × 2 Winds / 2 locs → exactly 2 same-loc joins → 2 derived facts (not 4, not 1); got {got:?}");
}

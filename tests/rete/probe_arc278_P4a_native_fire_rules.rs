//! Arc 278 stone P4a — disconfirming probe: the native Rust `fire-rules'` (cascade fixpoint) is
//! OBSERVATIONALLY EQUIVALENT to the wat oracle `fire-rules`. RED at HEAD (`fire-rules'` is UnknownFunction).
//!
//! `fire-rules'` is to `fire-once'` what the wat `fire-rules` is to `fire-once`: a fixpoint that lets derived
//! facts re-enter the network until no new fact is produced. The contract is OBSERVABLE — `query(fire-rules' s,
//! T) == query(fire-rules s, T)` for every type T — NOT raw Session equality (P4b restructures memories by
//! design). The cascade case is the canary: a fact DERIVED by a lower rule must unlock a higher rule across
//! rounds (forward chaining), and native must match wat exactly.
//!
//! Run: cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn ev(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

// ─── Single rule: fire-rules' on a one-round derivation == fire-rules ──────────────

fn single_rule(wind_loc: &str) -> String {
    format!("\
   c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))\
   c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))\
   rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   rule  (:wat::rete::Rule' \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector rhs1))\
   s0    (:wat::rete::compile (:wat::core::PersistentVector rule))\
   s1    (:wat::rete::insert s0 (:weather::Temperature 15 \"Oslo\"))\
   s2    (:wat::rete::insert s1 (:weather::WindSpeed 45 \"{wind_loc}\"))")
}

fn single_count(wind_loc: &str, fire_verb: &str) -> Value {
    ev(&format!(
        "(:wat::core::let [{} fired ({fire_verb} s2)] \
           (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy)))",
        single_rule(wind_loc)
    ))
}

#[test]
fn native_matches_wat_single_rule_match() {
    let native = single_count("Oslo", ":wat::rete::fire-rules'");
    let wat = single_count("Oslo", ":wat::rete::fire-rules-spec");
    assert_eq!(native, wat, "native fire-rules' must agree with wat fire-rules (Oslo); {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(1), "the match derives exactly one ColdAndWindy; got {native:?}");
}

#[test]
fn native_matches_wat_single_rule_no_match() {
    let native = single_count("Bergen", ":wat::rete::fire-rules'");
    let wat = single_count("Bergen", ":wat::rete::fire-rules-spec");
    assert_eq!(native, wat, "native must agree with wat on no-join; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(0), "mismatched loc → no derived fact; got {native:?}");
}

// ─── Cascade: a fact DERIVED by ruleA unlocks ruleB across rounds (THE canary) ────
// ruleA: Temperature + WindSpeed (same loc) → ColdAndWindy(loc)
// ruleB: ColdAndWindy(loc)                  → WeatherAlert(loc)   [fires on a DERIVED fact]
const CASCADE: &str = "\
   ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))\
   ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))\
   rhsA  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   ruleA (:wat::rete::Rule' \"cw\" (:wat::core::PersistentVector ca1 ca2) (:wat::core::PersistentVector rhsA))\
   cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))\
   rhsB  (:wat::core::quote (:wat::rete::insert (:weather::WeatherAlert ?loc)))\
   ruleB (:wat::rete::Rule' \"alert\" (:wat::core::PersistentVector cb1) (:wat::core::PersistentVector rhsB))\
   s0    (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))\
   s1    (:wat::rete::insert s0 (:weather::Temperature 15 \"Oslo\"))\
   s2    (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))";

fn cascade_count(ty: &str, fire_verb: &str) -> Value {
    ev(&format!(
        "(:wat::core::let [{CASCADE} fired ({fire_verb} s2)] \
           (:wat::core::length (:wat::rete::query fired {ty})))"
    ))
}

#[test]
fn native_matches_wat_cascade_first_rule() {
    let native = cascade_count(":weather::ColdAndWindy", ":wat::rete::fire-rules'");
    let wat = cascade_count(":weather::ColdAndWindy", ":wat::rete::fire-rules-spec");
    assert_eq!(native, wat, "native must agree on round-1 derivation; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(1), "ruleA derives one ColdAndWindy; got {native:?}");
}

#[test]
fn native_matches_wat_cascade_second_rule() {
    // The forward-chain canary: WeatherAlert is derived ONLY if the round-1 ColdAndWindy re-entered the
    // network and triggered ruleB. If fire-rules' didn't cascade, native would be 0 while wat is 1.
    let native = cascade_count(":weather::WeatherAlert", ":wat::rete::fire-rules'");
    let wat = cascade_count(":weather::WeatherAlert", ":wat::rete::fire-rules-spec");
    assert_eq!(native, wat, "native must cascade derived→higher-rule like wat; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(1), "ruleB fires on the DERIVED ColdAndWindy → one WeatherAlert; got {native:?}");
}

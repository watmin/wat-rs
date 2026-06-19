//! Arc 278 stone P2 — disconfirming probe: the native Rust `fire-once'` is OBSERVATIONALLY EQUIVALENT to the
//! wat oracle `fire-once`. RED at HEAD (`fire-once'` is UnknownFunction).
//!
//! The differential harness for the perf close: for every input session, the native single-pass fire produces
//! the SAME derived facts as the wat oracle's single pass — `query(fire-once' s) == query(fire-once s)`. NOT
//! raw Session equality (P3 restructures the memories by design); the durable contract is the derived facts.
//!
//! Run: cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::Record::def :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::ColdAndWindy [location <- :wat::core::String])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

// A staged (not-yet-fired) cold-and-windy session: hand-built rule, Temp(Oslo,15) + Wind(<loc>,45).
fn staged(wind_loc: &str) -> String {
    format!("\
   c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))\
   c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))\
   rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   rule  (:wat::rete::Rule \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector rhs1))\
   s0    (:wat::rete::compile (:wat::core::PersistentVector rule))\
   s1    (:wat::rete::insert s0 (:weather::Temperature 15 \"Oslo\"))\
   s2    (:wat::rete::insert s1 (:weather::WindSpeed 45 \"{wind_loc}\"))")
}

fn ev(expr: &str) -> Value {
    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

// count of derived ColdAndWindy facts after firing `s2` with the given fire verb (fire-once' or fire-once).
fn derived_count(wind_loc: &str, fire_verb: &str) -> Value {
    ev(&format!(
        "(:wat::core::let [{} fired ({fire_verb} s2)] \
           (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy)))",
        staged(wind_loc)
    ))
}

#[test]
fn native_matches_wat_on_a_match() {
    let native = derived_count("Oslo", ":wat::rete::fire-once'");
    let wat = derived_count("Oslo", ":wat::rete::fire-once");
    assert_eq!(native, wat, "native fire-once' must agree with wat fire-once (Oslo); native {native:?} vs wat {wat:?}");
    assert_eq!(native, Value::i64(1), "the match derives exactly one ColdAndWindy; got {native:?}");
}

#[test]
fn native_matches_wat_on_no_match() {
    let native = derived_count("Bergen", ":wat::rete::fire-once'");
    let wat = derived_count("Bergen", ":wat::rete::fire-once");
    assert_eq!(native, wat, "native must agree with wat on the no-join case; native {native:?} vs wat {wat:?}");
    assert_eq!(native, Value::i64(0), "mismatched loc → no derived fact; got {native:?}");
}

#[test]
fn native_derives_the_right_fact() {
    // The native-derived fact is a ColdAndWindy at "Oslo" (content, not just count).
    let setup = format!(
        "(:wat::core::let [{} fired (:wat::rete::fire-once' s2) \
           cw (:wat::core::Option/expect -> :wat::Record \
                (:wat::core::get (:wat::rete::query fired :weather::ColdAndWindy) 0) \"cw\")]",
        staged("Oslo")
    );
    assert_eq!(ev(&format!("{setup} (:wat::core::type cw))")),
        Value::String(Arc::new("weather::ColdAndWindy".to_string())), "native derives a ColdAndWindy");
    assert_eq!(ev(&format!("{setup} (:weather::ColdAndWindy/location cw))")),
        Value::String(Arc::new("Oslo".to_string())), "native binds ?loc = Oslo through the join");
}

// 2×2: 2 Temps × 2 Winds / 2 locs → exactly the 2 same-loc joins → 2 derived; native must match wat.
const STAGED_2X2: &str = "\
   c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))\
   c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))\
   rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   rule  (:wat::rete::Rule \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector rhs1))\
   s0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
   s1 (:wat::rete::insert s0 (:weather::Temperature 15 \"Oslo\"))\
   s2 (:wat::rete::insert s1 (:weather::Temperature 10 \"Bergen\"))\
   s3 (:wat::rete::insert s2 (:weather::WindSpeed 45 \"Oslo\"))\
   s4 (:wat::rete::insert s3 (:weather::WindSpeed 50 \"Bergen\"))";

#[test]
fn native_no_cross_loc_leakage() {
    let native = ev(&format!("(:wat::core::let [{STAGED_2X2} fired (:wat::rete::fire-once' s4)] \
        (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy)))"));
    let wat = ev(&format!("(:wat::core::let [{STAGED_2X2} fired (:wat::rete::fire-once s4)] \
        (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy)))"));
    assert_eq!(native, wat, "native must agree with wat on the 2×2; native {native:?} vs wat {wat:?}");
    assert_eq!(native, Value::i64(2), "exactly 2 same-loc joins → 2 derived (not 4, not 0); got {native:?}");
}

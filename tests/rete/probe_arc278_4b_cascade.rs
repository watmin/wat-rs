//! Arc 278 stone 4b — disconfirming probe: cascade-to-fixpoint. RED at HEAD.
//!
//! The second slice of stone 4. A derived fact must RE-ENTER the network so a rule that consumes it can fire.
//! A 2-rule chain proves it: rule A derives ColdAndWindy from Temp+Wind; rule B fires on ColdAndWindy and
//! derives WeatherAlert. Cold-and-windy alone (4a) can't show cascade — nothing consumes ColdAndWindy.
//!
//!   A :when [(:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20))
//!            (:weather::WindSpeed    (?loc <- :location) (?w <- :kph)     (:wat::core::> ?w 30))]
//!     :then (:wat::rete::insert (:weather::ColdAndWindy ?loc))
//!   B :when [(:weather::ColdAndWindy (?loc <- :location))]
//!     :then (:wat::rete::insert (:weather::WeatherAlert ?loc))
//!
//! - CASCADE (Temp+Wind same loc): A derives ColdAndWindy → it re-enters → B fires → WeatherAlert. The fixpoint
//!   has exactly ONE ColdAndWindy + ONE WeatherAlert (no re-derivation inflation across rounds).
//! - NO TRIGGER (diff loc): A never fires → no ColdAndWindy → B never fires → zero derived facts (and the
//!   fixpoint terminates without spinning).
//!
//! RED at HEAD: `fire-rules` is single-pass (4a) — ColdAndWindy lands in production-memory but never re-enters,
//! so rule B never fires and WeatherAlert is never derived.
//!
//! Run: cargo test --release -p wat --test probe_arc278_4b_cascade -- --include-ignored

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// Build the fired session for a given WindSpeed location, then gather ALL derived facts across every
// ProductionNode (production-memory values flattened into one PV<:wat::core::Record>) as `derived`.
fn setup(wind_loc: &str) -> String {
    format!("\
   ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))\
   ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))\
   ra1   (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   ruleA (:wat::rete::Rule \"A\" (:wat::core::PersistentVector ca1 ca2) (:wat::core::PersistentVector ra1))\
   cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))\
   rb1   (:wat::core::quote (:wat::rete::insert (:weather::WeatherAlert ?loc)))\
   ruleB (:wat::rete::Rule \"B\" (:wat::core::PersistentVector cb1) (:wat::core::PersistentVector rb1))\
   sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))\
   sess1 (:wat::rete::insert sess0 (:weather::Temperature 15 \"Oslo\"))\
   sess2 (:wat::rete::insert sess1 (:weather::WindSpeed 45 \"{wind_loc}\"))\
   fired (:wat::rete::fire-rules sess2)\
   pmem  (:wat::rete::Session/production-memory fired)\
   derived (:wat::core::foldl \
              (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector] \
                -> :wat::core::PersistentVector \
                (:wat::core::foldl \
                  (:wat::core::fn [a <- :wat::core::PersistentVector f <- :wat::core::Record] \
                    -> :wat::core::PersistentVector \
                    (:wat::core::PersistentVector/conj a f)) \
                  acc pv)) \
              (:wat::core::PersistentVector) \
              (:wat::core::PersistentMap/values pmem))")
}

// Count derived facts of a given record type (e.g. "weather::WeatherAlert" — :wat::core::type drops the colon).
fn count_of(setup_block: &str, type_fqdn: &str) -> Value {
    ev(&format!("(:wat::core::let [{setup_block}] \
        (:wat::core::length \
          (:wat::core::into (:wat::core::PersistentVector) \
            (:wat::core::filter \
              (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool \
                (:wat::core::= (:wat::core::type f) \"{type_fqdn}\")) \
              derived))))"))
}

fn ev(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

#[test]
fn cascade_fires_rule_b_on_rule_a_output() {
    // THE HEART of 4b: rule B fires on the fact rule A derived. RED at HEAD (single-pass — B never fires).
    let got = count_of(&setup("Oslo"), "weather::WeatherAlert");
    assert_eq!(got, Value::i64(1), "B should fire on A's derived ColdAndWindy → one WeatherAlert; got {got:?}");
}

#[test]
fn rule_a_still_derives_its_fact() {
    let got = count_of(&setup("Oslo"), "weather::ColdAndWindy");
    assert_eq!(got, Value::i64(1), "A derives exactly one ColdAndWindy (no cross-round re-derivation inflation); got {got:?}");
}

#[test]
fn fixpoint_total_is_exactly_two_derived() {
    // The closure is {ColdAndWindy, WeatherAlert} — exactly 2, proving no inflation (each derived once) and
    // the loop reached a fixpoint rather than spinning.
    let got = ev(&format!("(:wat::core::let [{}] (:wat::core::length derived))", setup("Oslo")));
    assert_eq!(got, Value::i64(2), "fixpoint closure = ColdAndWindy + WeatherAlert = 2; got {got:?}");
}

#[test]
fn no_cascade_without_the_root_fact() {
    // Temp(Oslo)+Wind(Bergen) → A never fires → no ColdAndWindy → B never fires → zero derived, and the
    // fixpoint terminates (no spin).
    let got = ev(&format!("(:wat::core::let [{}] (:wat::core::length derived))", setup("Bergen")));
    assert_eq!(got, Value::i64(0), "no root match → no cascade → zero derived facts; got {got:?}");
}

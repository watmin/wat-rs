//! Arc 278 stone 4c — disconfirming probe: truth maintenance / retraction. RED at HEAD.
//!
//! A retracted fact must drop every derived fact whose support depended on it, transitively. On the
//! re-run-from-scratch oracle this is pure replay — once the fact model keeps INPUT distinct from DERIVED.
//!
//! Two-rule chain (reused from 4b): A: Temp+Wind(same loc)→ColdAndWindy; B: ColdAndWindy→WeatherAlert.
//!
//! RED at HEAD, two ways:
//!  - fact-model: 4b's fire-rules returns Session.facts = the whole closure (input + derived), so a derived
//!    ColdAndWindy leaks into facts. Part A asserts facts holds only INPUT types → RED.
//!  - retract: `:wat::rete::retract` does not exist yet → Part B/C/D raise at eval → RED.
//!
//! Run: cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// The 2-rule chain compiled into sess0 (bind it, then thread inserts onto it).
const RULES: &str = "\
   ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))\
   ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))\
   ra1   (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   ruleA (:wat::rete::Rule \"A\" (:wat::core::PersistentVector ca1 ca2) (:wat::core::PersistentVector ra1))\
   cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))\
   rb1   (:wat::core::quote (:wat::rete::insert (:weather::WeatherAlert ?loc)))\
   ruleB (:wat::rete::Rule \"B\" (:wat::core::PersistentVector cb1) (:wat::core::PersistentVector rb1))\
   sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))";

fn ev(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

// length of (derived facts of `type_fqdn`) flattened from production-memory of the bound `fired` session.
fn derived_of(ty: &str) -> String {
    format!(
        "(:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter \
           (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) \"{ty}\")) \
           (:wat::core::foldl \
             (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector] \
               -> :wat::core::PersistentVector \
               (:wat::core::foldl (:wat::core::fn [a <- :wat::core::PersistentVector x <- :wat::core::Record] \
                 -> :wat::core::PersistentVector (:wat::core::PersistentVector/conj a x)) acc pv)) \
             (:wat::core::PersistentVector) \
             (:wat::core::PersistentMap/values (:wat::rete::Session/production-memory fired))))))"
    )
}

// length of (facts of `type_fqdn`) in the bound `fired` session's INPUT facts.
fn facts_of(ty: &str) -> String {
    format!(
        "(:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter \
           (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) \"{ty}\")) \
           (:wat::rete::Session/facts fired))))"
    )
}

// ── Part A — the fact-model fix: fire keeps INPUT distinct from DERIVED ──────────

#[test]
fn fire_keeps_input_facts_distinct_from_derived() {
    // assert Temp+Wind at Oslo, fire. Session.facts must hold the 2 INPUT facts and NO derived ColdAndWindy.
    let setup = format!(
        "(:wat::core::let [{RULES}\
           s1 (:wat::rete::insert sess0 (:weather::Temperature 15 \"Oslo\"))\
           s2 (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))\
           fired (:wat::rete::fire-rules s2)]"
    );
    // input facts present (Temperature kept):
    assert_eq!(ev(&format!("{setup} {})", facts_of("weather::Temperature"))), Value::i64(1),
        "input Temperature stays in Session.facts");
    // derived facts NOT leaked into Session.facts (they live in production-memory):
    assert_eq!(ev(&format!("{setup} {})", facts_of("weather::ColdAndWindy"))), Value::i64(0),
        "derived ColdAndWindy must NOT be in Session.facts (input distinct from derived)");
    // but it IS derived (in production-memory):
    assert_eq!(ev(&format!("{setup} {})", derived_of("weather::ColdAndWindy"))), Value::i64(1),
        "ColdAndWindy is derived into production-memory");
}

// ── Part B — retraction drops the derived consequence ───────────────────────────

#[test]
fn retract_removes_derived_consequence() {
    let setup = format!(
        "(:wat::core::let [{RULES}\
           s1 (:wat::rete::insert sess0 (:weather::Temperature 15 \"Oslo\"))\
           s2 (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))\
           f0 (:wat::rete::fire-rules s2)\
           s3 (:wat::rete::retract f0 (:weather::Temperature 15 \"Oslo\"))\
           fired (:wat::rete::fire-rules s3)]"
    );
    assert_eq!(ev(&format!("{setup} {})", derived_of("weather::ColdAndWindy"))), Value::i64(0),
        "retracting Temperature drops the ColdAndWindy it supported");
}

// ── Part C — retraction cascades transitively (CW supported WA) ──────────────────

#[test]
fn retract_cascades_transitively() {
    let setup = format!(
        "(:wat::core::let [{RULES}\
           s1 (:wat::rete::insert sess0 (:weather::Temperature 15 \"Oslo\"))\
           s2 (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))\
           f0 (:wat::rete::fire-rules s2)\
           s3 (:wat::rete::retract f0 (:weather::Temperature 15 \"Oslo\"))\
           fired (:wat::rete::fire-rules s3)]"
    );
    // WA depended on CW which depended on Temp → retracting Temp takes the whole chain down.
    assert_eq!(ev(&format!("{setup} {})", derived_of("weather::WeatherAlert"))), Value::i64(0),
        "transitive TM: WeatherAlert (derived from derived ColdAndWindy) is gone too");
}

// ── Part D — retraction is precise: independent derivations survive ──────────────

#[test]
fn retract_leaves_independent_derivations() {
    let setup = format!(
        "(:wat::core::let [{RULES}\
           s1 (:wat::rete::insert sess0 (:weather::Temperature 15 \"Oslo\"))\
           s2 (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))\
           s3 (:wat::rete::insert s2 (:weather::Temperature 10 \"Bergen\"))\
           s4 (:wat::rete::insert s3 (:weather::WindSpeed 50 \"Bergen\"))\
           f0 (:wat::rete::fire-rules s4)\
           s5 (:wat::rete::retract f0 (:weather::Temperature 15 \"Oslo\"))\
           fired (:wat::rete::fire-rules s5)]"
    );
    // Oslo's CW drops; Bergen's CW survives (its support is intact).
    assert_eq!(ev(&format!("{setup} {})", derived_of("weather::ColdAndWindy"))), Value::i64(1),
        "only the Oslo derivation drops; Bergen's ColdAndWindy survives (precise TM)");
}

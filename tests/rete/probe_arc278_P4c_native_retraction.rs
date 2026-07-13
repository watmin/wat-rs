//! Arc 278 stone P4c — native truth maintenance: `retract` + native `fire-rules'` == the wat oracle.
//!
//! GROUNDING (the realization behind this stone): `retract` is engine-agnostic — it edits `Session.facts`
//! (remove-by-value, stage-only). Truth maintenance then falls out of REPLAY: the native delta engine
//! (`fire-rules'`, P4b) re-derives the closure from the reduced input, so a retracted fact's consequences
//! simply are not re-derived — transitively and precisely. P4b already made that replay LINEAR. So there is
//! NO separate "incremental support-store retract cascade" to build in the value-semantics surface (each
//! `fire` rebuilds from facts; the support store only buys O(delta) retract for a PERSISTENT cross-fire
//! streaming engine, a deferred surface). This probe is the proof: native TM == oracle TM, scenario for
//! scenario. The differential is the gate; there is no new engine code.
//!
//! Run: cargo test --release -p wat --test probe_arc278_P4c_native_retraction

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// A: Temp+Wind(same loc)→ColdAndWindy; B: ColdAndWindy→WeatherAlert (the 4c chain).
const RULES: &str = "\
   ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))\
   ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))\
   ra1   (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   ruleA (:wat::rete::Rule :name \"A\" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))\
   cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))\
   rb1   (:wat::core::quote (:wat::rete::insert (:weather::WeatherAlert ?loc)))\
   ruleB (:wat::rete::Rule :name \"B\" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))\
   sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))";

fn ev(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}")).value_owned()
}

// derived count of `ty` after firing-with-`fire_verb` the staged-then-retracted session described by `body`.
// `body` binds `fired`. Returns the count via query-by-type-string.
fn count(setup: &str, ty: &str) -> Value {
    ev(&format!("(:wat::core::let [{setup}] (:wat::core::length (:wat::rete::query-by-type-string fired \"{ty}\")))"))
}

/// Single retract: drop a support → its derived ColdAndWindy is gone. native == wat.
#[test]
fn native_retract_drops_consequence_like_wat() {
    for verb in [":wat::rete::fire-rules'", ":wat::rete::fire-rules-spec"] {
        let setup = format!("{RULES}\
           s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
           s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\"))\
           f0 ({verb} s2)\
           s3 (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
           fired ({verb} s3)");
        assert_eq!(count(&setup, "weather::ColdAndWindy"), Value::i64(0),
            "[{verb}] retracting Temperature drops the ColdAndWindy it supported");
    }
}

/// Transitive: retract Temp → CW gone → WA (derived from CW) gone too. native == wat.
#[test]
fn native_retract_cascades_transitively_like_wat() {
    for verb in [":wat::rete::fire-rules'", ":wat::rete::fire-rules-spec"] {
        let setup = format!("{RULES}\
           s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
           s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\"))\
           f0 ({verb} s2)\
           s3 (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
           fired ({verb} s3)");
        assert_eq!(count(&setup, "weather::WeatherAlert"), Value::i64(0),
            "[{verb}] transitive TM: WeatherAlert (from derived ColdAndWindy) is gone too");
    }
}

/// Precise: retract Oslo's Temp; Bergen's independent derivation survives. native == wat (== 1).
#[test]
fn native_retract_is_precise_like_wat() {
    let mut results = vec![];
    for verb in [":wat::rete::fire-rules'", ":wat::rete::fire-rules-spec"] {
        let setup = format!("{RULES}\
           s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
           s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\"))\
           s3 (:wat::rete::insert s2 (:weather::Temperature :celsius 10 :location \"Bergen\"))\
           s4 (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location \"Bergen\"))\
           f0 ({verb} s4)\
           s5 (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
           fired ({verb} s5)");
        let cw = count(&setup, "weather::ColdAndWindy");
        let wa = count(&setup, "weather::WeatherAlert");
        assert_eq!(cw, Value::i64(1), "[{verb}] only Oslo drops; Bergen's ColdAndWindy survives (precise TM)");
        assert_eq!(wa, Value::i64(1), "[{verb}] Bergen's WeatherAlert survives");
        results.push((cw, wa));
    }
    assert_eq!(results[0], results[1], "native fire-rules' TM must equal wat fire-rules TM exactly");
}

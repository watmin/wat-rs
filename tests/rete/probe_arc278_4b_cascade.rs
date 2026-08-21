//! Arc 278 stone 4b — cascade-to-fixpoint: a derived fact re-enters the network.
//!
//! A 2-rule chain proves it: rule A derives ColdAndWindy from Temp+Wind; rule B fires on ColdAndWindy and
//! derives WeatherAlert. Live mouths: `compile-all`, `insert`, `fire-rules`, `query`.
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
//! Run: cargo test --release -p wat --test probe_arc278_4b_cascade -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn compile_ab_fires_nothing() {
    let got = call_beside_value(file!(), ":user::compile-ab-fires-nothing").expect("eval");
    assert_eq!(got, Value::i64(0), "two-rule compile+fire with no facts derives nothing; got {got:?}");
}

#[test]
fn cascade_fired_session_builds() {
    let _ = call_beside_value(file!(), ":test::cascade-fired-session").expect("eval");
}

#[test]
fn cascade_fires_rule_b_on_rule_a_output() {
    // THE HEART of 4b: rule B fires on the fact rule A derived.
    let got = call_beside_value(file!(), ":user::weatheralert-count-oslo").expect("eval");
    assert_eq!(got, Value::i64(1), "B should fire on A's derived ColdAndWindy → one WeatherAlert; got {got:?}");
}

#[test]
fn rule_a_still_derives_its_fact() {
    let got = call_beside_value(file!(), ":user::coldandwindy-count-oslo").expect("eval");
    assert_eq!(got, Value::i64(1), "A derives exactly one ColdAndWindy (no cross-round re-derivation inflation); got {got:?}");
}

#[test]
fn fixpoint_total_is_exactly_two_derived() {
    // The closure is {ColdAndWindy, WeatherAlert} — exactly 2, proving no inflation (each derived once) and
    // the loop reached a fixpoint rather than spinning.
    let got = call_beside_value(file!(), ":user::derived-length-oslo").expect("eval");
    assert_eq!(got, Value::i64(2), "fixpoint closure = ColdAndWindy + WeatherAlert = 2; got {got:?}");
}

#[test]
fn no_cascade_without_the_root_fact() {
    // Temp(Oslo)+Wind(Bergen) → A never fires → no ColdAndWindy → B never fires → zero derived, and the
    // fixpoint terminates (no spin).
    let got = call_beside_value(file!(), ":user::derived-length-bergen").expect("eval");
    assert_eq!(got, Value::i64(0), "no root match → no cascade → zero derived facts; got {got:?}");
}

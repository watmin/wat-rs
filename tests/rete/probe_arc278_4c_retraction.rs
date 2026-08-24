//! Arc 278 stone 4c — truth maintenance / retraction.
//!
//! A retracted fact drops every derived fact whose support depended on it, transitively. On the
//! re-run-from-scratch oracle this is pure replay — the fact model keeps INPUT distinct from DERIVED.
//! Two-rule chain (reused from 4b): A: Temp+Wind(same loc)→ColdAndWindy; B: ColdAndWindy→WeatherAlert.
//! Live mouths: `compile-all`, `insert`, `fire-rules`, `retract`, `query`.
//!
//! - fact-model: `Session.facts` holds INPUT types only; derived ColdAndWindy lives in production-memory.
//! - retract: drops the consequence transitively and precisely (independent derivations survive).
//!
//! Run: cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// ── Part A — the fact-model fix: fire keeps INPUT distinct from DERIVED ──────────

#[test]
fn compile_ab_rules_fires_nothing() {
    assert_eq!(call_beside_value(file!(), ":user::compile-ab-rules-fires-nothing").expect("eval"), Value::i64(0),
        "two-rule compile+fire with no facts derives nothing");
}

#[test]
fn seed_oslo_then_fire_cw() {
    assert_eq!(call_beside_value(file!(), ":user::seed-oslo-then-fire-cw").expect("eval"), Value::i64(1),
        "Oslo Temp+Wind fire derives one ColdAndWindy");
}

#[test]
fn seed_bergen_then_fire_cw() {
    assert_eq!(call_beside_value(file!(), ":user::seed-bergen-then-fire-cw").expect("eval"), Value::i64(1),
        "Bergen Temp+Wind fire derives one ColdAndWindy");
}

#[test]
fn fire_keeps_input_facts_distinct_from_derived() {
    // assert Temp+Wind at Oslo, fire. Session.facts must hold the 2 INPUT facts and NO derived ColdAndWindy.
    // input facts present (Temperature kept):
    assert_eq!(call_beside_value(file!(), ":user::part-a-temperature-in-facts").expect("eval"), Value::i64(1),
        "input Temperature stays in Session.facts");
    // derived facts NOT leaked into Session.facts (they live in production-memory):
    assert_eq!(call_beside_value(file!(), ":user::part-a-coldandwindy-in-facts").expect("eval"), Value::i64(0),
        "derived ColdAndWindy must NOT be in Session.facts (input distinct from derived)");
    // but it IS derived (in production-memory):
    assert_eq!(call_beside_value(file!(), ":user::part-a-coldandwindy-derived").expect("eval"), Value::i64(1),
        "ColdAndWindy is derived into production-memory");
}

// ── Part B — retraction drops the derived consequence ───────────────────────────

#[test]
fn retract_removes_derived_consequence() {
    assert_eq!(call_beside_value(file!(), ":user::part-b-coldandwindy-derived-after-retract").expect("eval"), Value::i64(0),
        "retracting Temperature drops the ColdAndWindy it supported");
}

// ── Part C — retraction cascades transitively (CW supported WA) ──────────────────

#[test]
fn retract_cascades_transitively() {
    // WA depended on CW which depended on Temp → retracting Temp takes the whole chain down.
    assert_eq!(call_beside_value(file!(), ":user::part-c-weatheralert-derived-after-retract").expect("eval"), Value::i64(0),
        "transitive TM: WeatherAlert (derived from derived ColdAndWindy) is gone too");
}

// ── Part D — retraction is precise: independent derivations survive ──────────────

#[test]
fn retract_leaves_independent_derivations() {
    // Oslo's CW drops; Bergen's CW survives (its support is intact).
    assert_eq!(call_beside_value(file!(), ":user::part-d-coldandwindy-derived-after-retract-oslo").expect("eval"), Value::i64(1),
        "only the Oslo derivation drops; Bergen's ColdAndWindy survives (precise TM)");
}

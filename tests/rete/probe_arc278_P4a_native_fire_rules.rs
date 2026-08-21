//! Arc 278 stone P4a — disconfirming probe: native `fire-rules` (cascade fixpoint) is
//! observationally equivalent to `fire-rules$oracle`. Dual-impl: unprimed is native.
//!
//! `fire-rules` is to `fire-once` what `fire-rules$oracle` is to `fire-once$oracle`: a fixpoint that lets derived
//! facts re-enter the network until no new fact is produced. The contract is OBSERVABLE —
//! `query(fire-rules s, T) == query(fire-rules$oracle s, T)` for every type T — NOT raw Session equality
//! (P4b restructures memories by design). The cascade case is the canary: a fact DERIVED by a lower
//! rule must unlock a higher rule across rounds (forward chaining), and native must match the oracle.
//!
//! Run: cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): wind_loc / query type and the fire verb are each small-valued and every
// combination a #[test] needs is a fixed, enumerable named entry in the co-located fixture.
fn call(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("eval raised: {e:?}"))
}

// ─── Single rule: fire-rules on a one-round derivation == fire-rules$oracle ──────────────

#[test]
fn compile_cw_fires_nothing() {
    assert_eq!(call(":user::compile-cw-fires-nothing"), Value::i64(0), "single-rule compile+fire with no facts derives nothing");
}

#[test]
fn compile_ab_fires_nothing() {
    assert_eq!(call(":user::compile-ab-fires-nothing"), Value::i64(0), "two-rule compile+fire with no facts derives nothing");
}

#[test]
fn seed_oslo_session_builds() {
    let _ = call(":user::seed-oslo-session");
}

#[test]
fn native_matches_wat_single_rule_match() {
    let native = call(":user::single-native-oslo");
    let wat = call(":user::single-wat-oslo");
    assert_eq!(native, wat, "native fire-rules must agree with fire-rules$oracle (Oslo); {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(1), "the match derives exactly one ColdAndWindy; got {native:?}");
}

#[test]
fn native_matches_wat_single_rule_no_match() {
    let native = call(":user::single-native-bergen");
    let wat = call(":user::single-wat-bergen");
    assert_eq!(native, wat, "native must agree with wat on no-join; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(0), "mismatched loc → no derived fact; got {native:?}");
}

// ─── Cascade: a fact DERIVED by ruleA unlocks ruleB across rounds (THE canary) ────
// ruleA: Temperature + WindSpeed (same loc) → ColdAndWindy(loc)
// ruleB: ColdAndWindy(loc)                  → WeatherAlert(loc)   [fires on a DERIVED fact]

#[test]
fn native_matches_wat_cascade_first_rule() {
    let native = call(":user::cascade-native-cw");
    let wat = call(":user::cascade-wat-cw");
    assert_eq!(native, wat, "native must agree on round-1 derivation; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(1), "ruleA derives one ColdAndWindy; got {native:?}");
}

#[test]
fn native_matches_wat_cascade_second_rule() {
    // The forward-chain canary: WeatherAlert is derived ONLY if the round-1 ColdAndWindy re-entered the
    // network and triggered ruleB. If fire-rules didn't cascade, native would be 0 while wat is 1.
    let native = call(":user::cascade-native-wa");
    let wat = call(":user::cascade-wat-wa");
    assert_eq!(native, wat, "native must cascade derived→higher-rule like wat; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(1), "ruleB fires on the DERIVED ColdAndWindy → one WeatherAlert; got {native:?}");
}

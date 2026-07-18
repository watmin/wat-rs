//! Arc 278 stone P2 — disconfirming probe: the native Rust `fire-once'` is OBSERVATIONALLY EQUIVALENT to the
//! wat oracle `fire-once`. RED at HEAD (`fire-once'` is UnknownFunction).
//!
//! The differential harness for the perf close: for every input session, the native single-pass fire produces
//! the SAME derived facts as the wat oracle's single pass — `query(fire-once' s) == query(fire-once s)`. NOT
//! raw Session equality (P3 restructures the memories by design); the durable contract is the derived facts.
//!
//! Run: cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): wind_loc and the fire verb are each 2-valued, so every scenario is a fixed,
// enumerable named entry in the co-located fixture — driven via call_beside.
fn call(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).unwrap_or_else(|e| panic!("eval raised: {e:?}"))
}

#[test]
fn native_matches_wat_on_a_match() {
    let native = call(":user::count-native-oslo");
    let wat = call(":user::count-wat-oslo");
    assert_eq!(native, wat, "native fire-once' must agree with wat fire-once (Oslo); native {native:?} vs wat {wat:?}");
    assert_eq!(native, Value::i64(1), "the match derives exactly one ColdAndWindy; got {native:?}");
}

#[test]
fn native_matches_wat_on_no_match() {
    let native = call(":user::count-native-bergen");
    let wat = call(":user::count-wat-bergen");
    assert_eq!(native, wat, "native must agree with wat on the no-join case; native {native:?} vs wat {wat:?}");
    assert_eq!(native, Value::i64(0), "mismatched loc → no derived fact; got {native:?}");
}

#[test]
fn native_derives_the_right_fact() {
    // The native-derived fact is a ColdAndWindy at "Oslo" (content, not just count).
    assert_eq!(call(":user::native-fact-type"),
        Value::String(Arc::new("weather::ColdAndWindy".to_string())), "native derives a ColdAndWindy");
    assert_eq!(call(":user::native-fact-location"),
        Value::String(Arc::new("Oslo".to_string())), "native binds ?loc = Oslo through the join");
}

#[test]
fn native_no_cross_loc_leakage() {
    // 2×2: 2 Temps × 2 Winds / 2 locs → exactly the 2 same-loc joins → 2 derived; native must match wat.
    let native = call(":user::count-native-2x2");
    let wat = call(":user::count-wat-2x2");
    assert_eq!(native, wat, "native must agree with wat on the 2×2; native {native:?} vs wat {wat:?}");
    assert_eq!(native, Value::i64(2), "exactly 2 same-loc joins → 2 derived (not 4, not 0); got {native:?}");
}

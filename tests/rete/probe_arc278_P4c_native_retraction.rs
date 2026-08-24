//! Arc 278 stone P4c — native truth maintenance: `retract` + native `fire-rules` == `fire-rules$oracle`.
//!
//! GROUNDING (the realization behind this stone): `retract` is engine-agnostic — it edits `Session.facts`
//! (remove-by-value, stage-only). Truth maintenance then falls out of REPLAY: the native delta engine
//! (`fire-rules`, P4b) re-derives the closure from the reduced input, so a retracted fact's consequences
//! simply are not re-derived — transitively and precisely. P4b already made that replay LINEAR. So there is
//! NO separate "incremental support-store retract cascade" to build in the value-semantics surface (each
//! `fire` rebuilds from facts; the support store only buys O(delta) retract for a PERSISTENT cross-fire
//! streaming engine, a deferred surface). This probe is the proof: native TM == oracle TM, scenario for
//! scenario. The differential is the gate; there is no new engine code.
//!
//! Run: cargo test --release -p wat --test probe_arc278_P4c_native_retraction

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): the fire verb (native fire-rules vs oracle fire-rules$oracle) is 2-valued and
// every scenario a #[test] needs is a fixed, enumerable named entry in the co-located fixture.
fn call(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("eval raised: {e:?}"))
}

/// Single retract: drop a support → its derived ColdAndWindy is gone. native == wat.
#[test]
fn native_retract_drops_consequence_like_wat() {
    for (verb, fn_name) in [
        ("fire-rules", ":user::native-retract-drops-cw"),
        ("fire-rules$oracle", ":user::oracle-retract-drops-cw"),
    ] {
        assert_eq!(call(fn_name), Value::i64(0),
            "[{verb}] retracting Temperature drops the ColdAndWindy it supported");
    }
}

/// Transitive: retract Temp → CW gone → WA (derived from CW) gone too. native == wat.
#[test]
fn native_retract_cascades_transitively_like_wat() {
    for (verb, fn_name) in [
        ("fire-rules", ":user::native-retract-cascade-wa"),
        ("fire-rules$oracle", ":user::oracle-retract-cascade-wa"),
    ] {
        assert_eq!(call(fn_name), Value::i64(0),
            "[{verb}] transitive TM: WeatherAlert (from derived ColdAndWindy) is gone too");
    }
}

/// Precise: retract Oslo's Temp; Bergen's independent derivation survives. native == wat (== 1).
#[test]
fn native_retract_is_precise_like_wat() {
    let mut results = vec![];
    for (verb, cw_fn, wa_fn) in [
        ("fire-rules", ":user::native-retract-precise-cw", ":user::native-retract-precise-wa"),
        ("fire-rules$oracle", ":user::oracle-retract-precise-cw", ":user::oracle-retract-precise-wa"),
    ] {
        let cw = call(cw_fn);
        let wa = call(wa_fn);
        assert_eq!(cw, Value::i64(1), "[{verb}] only Oslo drops; Bergen's ColdAndWindy survives (precise TM)");
        assert_eq!(wa, Value::i64(1), "[{verb}] Bergen's WeatherAlert survives");
        results.push((cw, wa));
    }
    assert_eq!(results[0], results[1], "native fire-rules TM must equal fire-rules$oracle TM exactly");
}

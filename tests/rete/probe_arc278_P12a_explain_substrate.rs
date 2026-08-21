//! Arc 278 — P12a: the EXPLAIN substrate. `fire-rules-explain` + the `Explained {session, support}` index.
//! RED at HEAD (`fire-rules-explain` / `Explained` / `Support` are unknown); GREEN when P12a lands.
//! Contract: DESIGN-STONE-P12a-explain-substrate.md.
//!
//! Proves the OPT-IN diagnostic fire captures the support graph at the substrate level — WITHOUT needing the
//! wat `explain` walk (P12b). Three layered assertions on the cold-and-windy cascade
//! (Temperature ⋈ WindSpeed → ColdAndWindy → WeatherAlert):
//!   1. CLOSURE FIDELITY — explain mode derives the SAME facts as the fast path (it only adds provenance).
//!   2. INDEX POPULATED — the support map has one entry per derived fact (ColdAndWindy + WeatherAlert = 2).
//!   3. CHAINS CAPTURED — each entry's producing token carries its real `matches` support chain
//!      (ColdAndWindy's token: Temp+Wind = 2 edges; WeatherAlert's: ColdAndWindy = 1 edge; sum = 3).
//!
//! `Explained` is EPHEMERAL — re-derived per explain, never serialized; the snapshot stays `{facts, rules}`.
//! Run: cargo test --release -p wat --test probe_arc278_P12a_explain_substrate -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn compile_weather_fires_nothing() {
    let n = call_beside_value(file!(), ":user::compile-weather-fires-nothing").expect("compute should run");
    assert!(matches!(n, Value::i64(0)), "compile+fire with no facts derives no ColdAndWindy; got {n:?}");
}

/// 1. CLOSURE FIDELITY — explain mode derives the same facts as the fast path: `Explained/session` is a real
///    fired session, and the ColdAndWindy closure count is 1 (diagnostics add provenance, never change WHAT fires).
#[test]
fn fire_rules_explain_preserves_the_closure() {
    let n = call_beside_value(file!(), ":user::closure-fidelity-coldandwindy-count").expect("compute should run");
    assert!(matches!(n, Value::i64(1)), "explain mode must derive the same ColdAndWindy closure as the fast path (1); got {n:?}");
}

/// 2. INDEX POPULATED — the support map has one entry per derived fact: ColdAndWindy + WeatherAlert = 2.
#[test]
fn support_index_has_an_entry_per_derived_fact() {
    let n = call_beside_value(file!(), ":user::support-index-length").expect("compute should run");
    assert!(matches!(n, Value::i64(2)), "support index must have one entry per derived fact (ColdAndWindy, WeatherAlert = 2); got {n:?}");
}

/// 3. CHAINS CAPTURED — each entry's producing token carries its real `matches` support chain. Sum of chain
///    lengths over all support entries: ColdAndWindy's token has 2 edges (Temperature, WindSpeed), WeatherAlert's
///    has 1 (ColdAndWindy) → 3. This proves the index stores the real provenance, not just fact keys.
#[test]
fn support_tokens_carry_their_full_chains() {
    let n = call_beside_value(file!(), ":user::support-chains-total-length").expect("compute should run");
    assert!(matches!(n, Value::i64(3)), "support tokens must carry their real chains (ColdAndWindy 2 + WeatherAlert 1 = 3); got {n:?}");
}

/// 4. `$oracle` explain matches native support cardinality (the grid's missing cell).
#[test]
fn explain_oracle_matches_native_support_length() {
    let native = call_beside_value(file!(), ":user::support-index-length").expect("native");
    let oracle = call_beside_value(file!(), ":user::support-index-length-oracle").expect("oracle");
    assert_eq!(native, oracle, "fire-rules-explain$oracle support length must equal native; native={native:?} oracle={oracle:?}");
}

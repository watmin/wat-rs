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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// The shared lifecycle prefix: collect → compile → insert ×2 → fire-rules-explain, binding the
/// `Explained` result to `ex`. `body` is spliced in with `ex` in scope.
fn run_with_explained(body: &str) -> Value {
    let compute = format!(
        "(:wat::core::let\n\
          [rules   (:wat::rete::collect-rules :weather)\n\
           session (:wat::rete::compile rules)\n\
           session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location \"Oslo\"))\n\
           session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location \"Oslo\"))\n\
           ex      (:wat::rete::fire-rules-explain session)]\n\
          {body})"
    );
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(&compute).expect("parse compute");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

/// 1. CLOSURE FIDELITY — explain mode derives the same facts as the fast path: `Explained/session` is a real
/// fired session, and the ColdAndWindy closure count is 1 (diagnostics add provenance, never change WHAT fires).
#[test]
fn fire_rules_explain_preserves_the_closure() {
    let n = run_with_explained(
        "(:wat::core::length\n\
           (:wat::rete::query-by-type-string (:wat::rete::Explained/session ex) \"weather::ColdAndWindy\"))");
    assert!(matches!(n, Value::i64(1)), "explain mode must derive the same ColdAndWindy closure as the fast path (1); got {n:?}");
}

/// 2. INDEX POPULATED — the support map has one entry per derived fact: ColdAndWindy + WeatherAlert = 2.
#[test]
fn support_index_has_an_entry_per_derived_fact() {
    let n = run_with_explained(
        "(:wat::core::PersistentMap/length (:wat::rete::Explained/support ex))");
    assert!(matches!(n, Value::i64(2)), "support index must have one entry per derived fact (ColdAndWindy, WeatherAlert = 2); got {n:?}");
}

/// 3. CHAINS CAPTURED — each entry's producing token carries its real `matches` support chain. Sum of chain
/// lengths over all support entries: ColdAndWindy's token has 2 edges (Temperature, WindSpeed), WeatherAlert's
/// has 1 (ColdAndWindy) → 3. This proves the index stores the real provenance, not just fact keys.
#[test]
fn support_tokens_carry_their_full_chains() {
    let n = run_with_explained(
        "(:wat::core::foldl\n\
           (:wat::core::fn [acc <- :wat::core::i64  sv <- :wat::rete::Support]\n\
             -> :wat::core::i64\n\
             (:wat::core::i64::+ acc\n\
               (:wat::core::length (:wat::rete::Token/matches (:wat::rete::Support/token sv)))))\n\
           0\n\
           (:wat::core::PersistentMap/values (:wat::rete::Explained/support ex)))");
    assert!(matches!(n, Value::i64(3)), "support tokens must carry their real chains (ColdAndWindy 2 + WeatherAlert 1 = 3); got {n:?}");
}

//! Arc 278 — Stone 7-strat-native: STRATIFIED negation in the NATIVE kernel + the DIFFERENTIAL.
//! The wat ORACLE (`fire-rules-spec` → `fire-stratified`) already orders strata so a `:not` over a
//! DERIVED fact fires only after its producer stratum closes (built this arc: `bb6fb0f9`). The native
//! fast path (`fire-rules` → `fire-rules'` → `eval_fire_rules_native`) is still raw `fire_fixpoint_delta`
//! — a single fixpoint, no stratification — so a negation-over-derived rule fires a round too early and
//! leaks an extra derivation.
//!
//! The canonical case (`wat-scripts/fixes/rete-truth-maintenance-probes/neg.wat`, Clara-validated by
//! `neg.clj`): A(1),A(2); `mark-bad` derives Bad for k=2; `ok` = A with NO Bad. Correct = {Bad:1, Ok:1}
//! (only k=1 has no Bad). Native raw gives Ok=2 (the `ok` node fires for k=2 before Bad(2) is derived).
//!
//! The full differential chain (the acceptance spec):
//!     clj+clara  ──▶  wat+rete (`fire-rules-spec`)  ──▶  wat+rust-rete (`fire-rules`)
//! When all three agree we are in a good state. This probe pins the two wat boundaries; `neg.clj` pins
//! the Clara boundary externally.
//!
//! RED at HEAD: `native_ok` returns 2 (oracle returns 1) → `differential_stratified_negation` fails on the
//! native==oracle assertion. GREEN when native stratification lands (native mirrors the oracle: Ok=1).
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_7strat_native_differential

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// Fire `A(1),A(2)` through `fire_fn` and return (Bad count, Ok count).
/// The world (records + the two rules) is the co-located `.wat` fixture, loaded via `startup_beside`.
fn counts(fire_fn: &str) -> Result<(i64, i64), String> {
    let run = format!(
        "(:wat::core::let\n\
          [rules   (:wat::rete::collect-rules :n)\n\
           s0      (:wat::rete::compile rules)\n\
           s1      (:wat::rete::insert s0 (:n::A 1))\n\
           s2      (:wat::rete::insert s1 (:n::A 2))\n\
           fired   (:wat::rete::{fire_fn} s2)]\n\
          (:wat::core::PersistentVector\n\
            (:wat::core::length (:wat::rete::query-by-type-string fired \"n::Bad\"))\n\
            (:wat::core::length (:wat::rete::query-by-type-string fired \"n::Ok\"))))"
    );
    let w = startup_beside(file!()).map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    let out = eval_in_frozen(&ast, &w, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned();
    match &out {
        Value::wat__core__PersistentVector(v) => {
            let g = |i: usize| match v.get(i) {
                Some(Value::i64(n)) => Ok(*n),
                other => Err(format!("slot {i}: expected i64; got {other:?}")),
            };
            Ok((g(0)?, g(1)?))
        }
        other => Err(format!("expected PV; got {other:?}")),
    }
}

/// The oracle (`fire-rules-spec` → `fire-stratified`) is the reference within wat; it already stratifies.
#[test]
fn oracle_stratified_negation_is_ok1() {
    let (bad, ok) = counts("fire-rules-spec").expect("oracle fire");
    assert_eq!((bad, ok), (1, 1), "oracle: A(1),A(2) → Bad=1, Ok=1 (only k=1 has no Bad); got Bad={bad} Ok={ok}");
}

/// THE DIFFERENTIAL — native `fire-rules` must equal the oracle (and Clara's Bad=1/Ok=1).
/// RED at HEAD: native raw `fire_fixpoint_delta` leaks Ok=2 (no stratification). GREEN when native stratifies.
#[test]
fn differential_stratified_negation() {
    let oracle = counts("fire-rules-spec").expect("oracle fire");
    let native = counts("fire-rules").expect("native fire");
    assert_eq!(native, oracle, "native==oracle on stratified negation; native={native:?} oracle={oracle:?}");
    assert_eq!(native, (1, 1), "native: Bad=1, Ok=1 (== Clara's neg.clj); got {native:?}");
}

//! Arc 278 — Stone 7-strat-native: STRATIFIED negation in the NATIVE kernel + the DIFFERENTIAL.
//! The wat ORACLE (`fire-rules-spec` → `fire-stratified`) orders strata so a `:not` over a DERIVED fact
//! fires only after its producer stratum closes (built `bb6fb0f9`). The native fast path
//! (`fire-rules` → `fire-rules'` → `eval_fire_rules_native`) gained the same stratification natively
//! (`bdbf3021`) — a PARALLEL port, not a flag on the oracle.
//!
//! The worlds + drivers live in the co-located `.wat` fixture; the driver is a fn parameterized by the
//! fire verb (the only thing the differential varies), so the `.rs` only names the entry point.
//!
//! The full differential chain (the acceptance spec):
//!     clj+clara  ──▶  wat+rete (`fire-rules-spec`)  ──▶  wat+rust-rete (`fire-rules`)
//! When all three agree we are in a good state; `neg.clj` pins the Clara boundary externally
//! (`wat-scripts/fixes/rete-truth-maintenance-probes/`).
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_7strat_native_differential

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// Evaluate `(:<ns>::run-counts :wat::rete::<fire_fn>)` against the co-located fixture and return its
/// first `n` per-type counts. All wat lives in the `.wat`; this only names the entry + the fire verb.
fn run_counts(ns: &str, fire_fn: &str, n: usize) -> Result<Vec<i64>, String> {
    let call = format!("(:{ns}::run-counts :wat::rete::{fire_fn})");
    let w = startup_beside(file!()).map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&call).map_err(|e| format!("parse: {e:?}"))?;
    let out = eval_in_frozen(&ast, &w, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned();
    match &out {
        Value::wat__core__PersistentVector(v) => (0..n)
            .map(|i| match v.get(i) {
                Some(Value::i64(x)) => Ok(*x),
                other => Err(format!("slot {i}: expected i64; got {other:?}")),
            })
            .collect(),
        other => Err(format!("expected PV; got {other:?}")),
    }
}

/// The oracle (`fire-rules-spec` → `fire-stratified`) is the reference within wat; it already stratifies.
#[test]
fn oracle_stratified_negation_is_ok1() {
    let c = run_counts("n", "fire-rules-spec", 2).expect("oracle fire");
    assert_eq!(c, vec![1, 1], "oracle: A(1),A(2) → Bad=1, Ok=1 (only k=1 has no Bad); got {c:?}");
}

/// THE DIFFERENTIAL — native `fire-rules` must equal the oracle (and Clara's Bad=1/Ok=1).
#[test]
fn differential_stratified_negation() {
    let oracle = run_counts("n", "fire-rules-spec", 2).expect("oracle fire");
    let native = run_counts("n", "fire-rules", 2).expect("native fire");
    assert_eq!(native, oracle, "native==oracle on stratified negation; native={native:?} oracle={oracle:?}");
    assert_eq!(native, vec![1, 1], "native: Bad=1, Ok=1 (== Clara's neg.clj); got {native:?}");
}

/// THE HARDER DIFFERENTIAL — 3 strata, facts threaded across TWO negation layers.
/// Guards the native stratified driver's cross-stratum acc-facts reconstruction (the one deviation from
/// a line-for-line port) beyond the minimal 2-stratum case — the R18 lesson made a test.
#[test]
fn differential_three_stratum_negation() {
    let oracle = run_counts("n3", "fire-rules-spec", 3).expect("oracle fire");
    let native = run_counts("n3", "fire-rules", 3).expect("native fire");
    assert_eq!(native, oracle, "native==oracle on 3-stratum negation; native={native:?} oracle={oracle:?}");
    assert_eq!(native, vec![1, 2, 1], "native: Bad=1, Warn=2, Safe=1 (only k=2 is Safe); got {native:?}");
}

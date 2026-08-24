//! Arc 278 stone 7-strat-native — STRATIFIED negation in the native kernel + the differential.
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! The oracle (`fire-rules$oracle` → `fire-stratified`) orders strata so a `:not` over a DERIVED fact
//! fires only after its producer stratum closes. Native `fire-rules` stratifies the same way — a
//! PARALLEL port, not a flag on the oracle.
//!
//! The worlds + drivers live in the co-located `.wat` fixture; the driver only names the fire verb via a
//! thin zero-arg wrapper entry point (the only thing the differential varies).
//!
//! The full differential chain (the acceptance spec):
//!     clj+clara  ──▶  wat+rete (`fire-rules$oracle`)  ──▶  wat+rust-rete (`fire-rules`)
//! When all three agree we are in a good state; `neg.clj` pins the Clara boundary externally
//! (`wat-scripts/fixes/rete-truth-maintenance-probes/`).
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_7strat_native_differential

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Call the named entry fn and return its first `n` per-type counts. All wat lives in the `.wat`.
fn run_counts(entry: &str, n: usize) -> Result<Vec<i64>, String> {
    let out = call_beside_value(file!(), entry).map_err(|e| format!("eval: {e:?}"))?;
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

/// The oracle (`fire-rules$oracle` → `fire-stratified`) is the reference within wat; it already stratifies.
#[test]
fn oracle_stratified_negation_is_ok1() {
    let c = run_counts(":user::n-oracle-counts", 2).expect("oracle fire");
    assert_eq!(c, vec![1, 1], "oracle: A(1),A(2) → Bad=1, Ok=1 (only k=1 has no Bad); got {c:?}");
}

/// THE DIFFERENTIAL — native `fire-rules` must equal the oracle (and Clara's Bad=1/Ok=1).
#[test]
fn differential_stratified_negation() {
    let oracle = run_counts(":user::n-oracle-counts", 2).expect("oracle fire");
    let native = run_counts(":user::n-native-counts", 2).expect("native fire");
    assert_eq!(native, oracle, "native==oracle on stratified negation; native={native:?} oracle={oracle:?}");
    assert_eq!(native, vec![1, 1], "native: Bad=1, Ok=1 (== Clara's neg.clj); got {native:?}");
}

/// THE HARDER DIFFERENTIAL — 3 strata, facts threaded across TWO negation layers.
/// Guards the native stratified driver's cross-stratum acc-facts reconstruction (the one deviation from
/// a line-for-line port) beyond the minimal 2-stratum case — the R18 lesson made a test.
#[test]
fn differential_three_stratum_negation() {
    let oracle = run_counts(":user::n3-oracle-counts", 3).expect("oracle fire");
    let native = run_counts(":user::n3-native-counts", 3).expect("native fire");
    assert_eq!(native, oracle, "native==oracle on 3-stratum negation; native={native:?} oracle={oracle:?}");
    assert_eq!(native, vec![1, 2, 1], "native: Bad=1, Warn=2, Safe=1 (only k=2 is Safe); got {native:?}");
}

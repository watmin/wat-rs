//! Arc 278 stone 7-exists — `:exists` (existential, the NegationNode sibling); `fire-rules` == `fire-rules$oracle`.
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! `(:wat::rete::exists <inner>)` passes a token iff ≥1 element matches the inner condition for its bindings,
//! binds NOTHING, and fires the token EXACTLY ONCE regardless of how many match (no multiplicity — the key
//! difference from a join). It is NegationNode's filter predicate flipped: negation passes iff ZERO compatible;
//! exists passes iff ≥1 compatible. Present → 1; absent → 0; three readings still fire once.
//!
//! Run: cargo test --release -p wat --test probe_arc278_7exists_native_differential

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn count(entry: &str) -> Result<i64, String> {
    match call_beside_value(file!(), entry).map_err(|e| format!("eval: {e:?}"))? {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

#[test]
fn compile_watched_fires_nothing() {
    assert_eq!(count(":user::compile-watched-fires-nothing").expect("eval"), 0,
        "compile+fire with no facts derives no Watched");
}

/// 1 — DIFFERENTIAL, exists passes (≥1 match): native == oracle, both 1.
#[test]
fn differential_exists_present() {
    let native = count(":user::native-one-reading").expect("native");
    let oracle = count(":user::oracle-one-reading").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (present); native={native} oracle={oracle}");
    assert_eq!(native, 1, "≥1 reading → 1; got {native}");
}

/// 2 — DIFFERENTIAL, exists blocks (zero matches): native == oracle, both 0.
#[test]
fn differential_exists_absent() {
    let native = count(":user::native-station-only").expect("native");
    let oracle = count(":user::oracle-station-only").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (absent); native={native} oracle={oracle}");
    assert_eq!(native, 0, "no readings → 0; got {native}");
}

/// 3 — DIFFERENTIAL, NO MULTIPLICITY (the existential property): 3 readings → fires ONCE, not 3.
/// This is what separates `:exists` from a join — the token passes once iff ≥1, never multiplied.
#[test]
fn differential_exists_no_multiplicity() {
    let native = count(":user::native-three-readings").expect("native");
    let oracle = count(":user::oracle-three-readings").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (3 readings); native={native} oracle={oracle}");
    assert_eq!(native, 1, "3 readings → fires ONCE (existential, not a join); got {native}");
}

/// 4 — DIFFERENTIAL, the shared-var filter: a reading elsewhere does not satisfy exists for Oslo.
#[test]
fn differential_exists_shared_var() {
    let native = count(":user::native-reading-elsewhere").expect("native");
    let oracle = count(":user::oracle-reading-elsewhere").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (elsewhere); native={native} oracle={oracle}");
    assert_eq!(native, 0, "reading@Bergen ≠ Oslo → 0; got {native}");
}

/// 5 — the NATIVE engine alone honors exists (headline: native passes-once / blocks, no under/over-derive).
#[test]
fn native_exists_passes_once_and_blocks() {
    assert_eq!(count(":user::native-one-reading").expect("native"), 1, "native ≥1 → 1");
    assert_eq!(count(":user::native-station-only").expect("native"), 0, "native none → 0");
    assert_eq!(count(":user::native-three-readings").expect("native"), 1, "native 3 → 1 (once)");
}

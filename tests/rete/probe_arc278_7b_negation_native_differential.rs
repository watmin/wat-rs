//! Arc 278 stone 7-b — negation (`:not`/NegationNode) in the native kernel; `fire-rules` == `fire-rules$oracle`.
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! For a `:not` rule the two mouths agree. Negation passes a token iff NO fact matches the negated
//! condition for its bindings. Native and oracle: absent (1), present-matching (0), present-different (1).
//!
//! Run: cargo test --release -p wat --test probe_arc278_7b_negation_native_differential

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn count(entry: &str) -> Result<i64, String> {
    match call_beside_value(file!(), entry).map_err(|e| format!("eval: {e:?}"))? {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

/// 1 — DIFFERENTIAL, negation passes (absent): native == oracle, both 1.
#[test]
fn differential_negation_absent() {
    let native = count(":user::native-absent").expect("native");
    let oracle = count(":user::oracle-absent").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (absent); native={native} oracle={oracle}");
    assert_eq!(native, 1, "absent → 1; got {native}");
}

/// 2 — DIFFERENTIAL, negation blocks (present-matching): native == oracle, both 0.
#[test]
fn differential_negation_present_matching() {
    let native = count(":user::native-present-matching").expect("native");
    let oracle = count(":user::oracle-present-matching").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (present-matching); native={native} oracle={oracle}");
    assert_eq!(native, 0, "present-matching → 0; got {native}");
}

/// 3 — DIFFERENTIAL, the shared-var join-filter (present-different): native == oracle, both 1.
#[test]
fn differential_negation_present_different() {
    let native = count(":user::native-present-different").expect("native");
    let oracle = count(":user::oracle-present-different").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (present-different); native={native} oracle={oracle}");
    assert_eq!(native, 1, "Maintenance@Bergen ≠ Oslo → 1; got {native}");
}

/// 4 — the NATIVE engine alone honors negation (the headline: native filters, not under-derives).
#[test]
fn native_negation_passes_and_blocks() {
    assert_eq!(count(":user::native-absent").expect("native"), 1, "native absent → 1");
    assert_eq!(count(":user::native-present-matching").expect("native"), 0, "native present-match → 0");
}

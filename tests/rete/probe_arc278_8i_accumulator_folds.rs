//! Arc 278 stone 8-i — the wat accumulator fold library (`:wat::rete::acc::*`), standalone.
//!
//! The accumulators are PURE WAT FOLDS over a `PV<Element>` (an Element = `(:wat::rete::Element :fact fact :bindings bindings)`);
//! value-folds read a BOUND `?var` (a string key) from each element's bindings map. `mean` = `sum / count`
//! (composition). Empty-case return types (illegal states unrepresentable):
//!   count / sum      → BARE value (0 on empty — always concrete; never Option)
//!   distinct / all   → BARE PV   ([] on empty)
//!   group-by         → BARE PM   ({} on empty)
//!   min / max / mean → Option    (None on empty — there is no minimum/maximum/mean of nothing)
//! Option is min/max/mean only. Live mouths: `acc::count`, `acc::sum`, `acc::min`, `acc::max`,
//! `acc::mean`, `acc::distinct`, `acc::all`, `acc::group-by`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_8i_accumulator_folds

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Some(i64) helper.
fn is_some_i64(v: &Value, n: i64) -> bool {
    matches!(v, Value::Option(o) if matches!(&**o, Some(Value::i64(m)) if *m == n))
}

/// count → BARE 3 (length is always concrete; never Option).
#[test]
fn count_folds() {
    assert!(matches!(call_beside_value(file!(), ":user::count-folds").unwrap(), Value::i64(3)), "count = 3 (bare)");
}

/// sum ?bytes → BARE 600 (empty sum = 0; never Option).
#[test]
fn sum_folds() {
    assert!(matches!(call_beside_value(file!(), ":user::sum-folds").unwrap(), Value::i64(600)), "sum = 600 (bare)");
}

/// min ?bytes → Some(100).
#[test]
fn min_folds() {
    assert!(is_some_i64(&call_beside_value(file!(), ":user::min-folds").unwrap(), 100), "min = 100");
}

/// max ?bytes → Some(300).
#[test]
fn max_folds() {
    assert!(is_some_i64(&call_beside_value(file!(), ":user::max-folds").unwrap(), 300), "max = 300");
}

/// mean ?bytes → Some(200) — THE composition: sum(600)/count(3).
#[test]
fn mean_is_sum_over_count() {
    assert!(is_some_i64(&call_beside_value(file!(), ":user::mean-is-sum-over-count").unwrap(), 200), "mean = 600/3 = 200");
}

/// distinct ?port → BARE vec of length 2 (80, 443 — the duplicate 80 collapses).
#[test]
fn distinct_folds() {
    let v = call_beside_value(file!(), ":user::distinct-folds").unwrap();
    assert!(matches!(v, Value::i64(2)), "distinct ports = 2; got {v:?}");
}

/// all → BARE vec of length 3 (the gathered facts).
#[test]
fn all_folds() {
    let v = call_beside_value(file!(), ":user::all-folds").unwrap();
    assert!(matches!(v, Value::i64(3)), "all facts = 3; got {v:?}");
}

/// group-by ?port → BARE map with 2 keys (80 → [a,c], 443 → [b]).
#[test]
fn group_by_folds() {
    let v = call_beside_value(file!(), ":user::group-by-folds").unwrap();
    assert!(matches!(v, Value::i64(2)), "group-by → 2 keys; got {v:?}");
}

/// EMPTY: count over an empty set → BARE 0 (count always concrete — never None).
#[test]
fn count_empty_is_zero() {
    assert!(matches!(call_beside_value(file!(), ":user::count-empty-is-zero").unwrap(), Value::i64(0)), "count [] = 0 (bare)");
}

/// EMPTY: min over an empty set → None (no token — there is no minimum of nothing).
#[test]
fn min_empty_is_none() {
    let v = call_beside_value(file!(), ":user::min-empty-is-none").unwrap();
    assert!(matches!(&v, Value::Option(o) if o.is_none()), "min [] = None; got {v:?}");
}

//! Arc 056 carry-along — `:wat::core::not=` + Enum equality.
//!
//! Clojure-tradition inequality. Shares the polymorphic-compare
//! inference rules with `=`; the runtime is `not(=)`. Also fills the
//! prior gap where `=` couldn't compare two `Value::Enum` values
//! (added an Enum arm to `values_equal`).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

fn unwrap_bool(v: Value) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

#[test]
fn not_eq_i64_true_when_different() {
    assert!(unwrap_bool(run_expr(":t::test1-not-eq-true")));
}

#[test]
fn not_eq_i64_false_when_same() {
    assert!(!unwrap_bool(run_expr(":t::test2-not-eq-false")));
}

#[test]
fn not_eq_f64_cross_numeric_is_category_aware() {
    // SUPERSEDED (296 Wave B2, finding 9): this test used to assert arc-237
    // Stone 237.8a — cross-numeric coercion for equality DELETED, so
    // `(:wat::core::not= 3 3.0)` was a TypeMismatch (same-type-only relational
    // intrinsic). Arc 300 Stone C5 deliberately REVERSED 237.8a to match
    // eval/clj semantics (category-aware numeric comparison); Stone C5b
    // (`1f1873e1`) rebuilt the same path. Ground truth measured live:
    // `(:wat::core::not= 3 3.0)` => true, `(:wat::core::= 3 3.0)` => false.
    // Mixed-numeric `not=`/`=` now type-check (Ok, not a CheckError) and
    // evaluate category-aware — the opposite contract of the retired test.
    assert!(
        unwrap_bool(run_expr(":t::test3-not-eq-cross-numeric")),
        "expected (:wat::core::not= 3 3.0) => true (category-aware, C5)"
    );
    assert!(
        !unwrap_bool(run_expr(":t::test3b-eq-cross-numeric")),
        "expected (:wat::core::= 3 3.0) => false (different numeric categories)"
    );
}

#[test]
fn eq_on_enum_unit_variants() {
    assert!(
        unwrap_bool(run_expr(":t::test4-enum-eq")),
        "expected enum eq/not= to return true"
    );
}

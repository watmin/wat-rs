//! FM-2-bis probe for Stone 237.8c — equality grid (Shape B: per-Type leaves + structural engine).
//!
//! Recipe (DESIGN-STONE-237.8c, four-questions verdict Shape B):
//!   - Mint `:wat::core::f64::=` / `:f64::not=` as type-locked f64 aliases into the
//!     structural engine (matching the existing `:i64::=` pattern).
//!   - Keep polymorphic `=`/`not=` STRUCTURAL (values_equal-backed) — equality is the
//!     justified asymmetry (universal + recursive + subtype-compatible).
//!   - Rename `infer_comparison` -> `infer_equality`; delete the dead cross-numeric
//!     arm in `values_equal`.
//!
//! ROW STATUS (initial):
//!   - REGRESSION (preserve behavior; GREEN at HEAD): polymorphic `=`/`not=` over
//!     scalars + composites; cross-numeric `(= 1 2.0)` and cross-type `(= 1 "x")`
//!     are check errors (THE DECISION + existing infer_comparison).
//!     [arc 300 C5 update: cross-numeric `=` now type-checks — see
//!     `regression_cross_numeric_now_type_checks` below; cross-type stays a check
//!     error, unchanged, out of C5's scope.]
//!   - MINT-CONFIRMERS (RED at HEAD; `:f64::=`/`:f64::not=` do not exist yet;
//!     `#[ignore]`'d): un-ignored by sonnet after the f64 equality leaves are minted.
//!
//! Run: cargo test --release --test probe_arc237_8c_equality_grid

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn eval_bool(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval")
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — preserve the polymorphic structural `=`/`not=`. GREEN at HEAD.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_eq_scalars() {
    assert_eq!(eval_bool(":user::eq-scalars-i64-eq"), Value::bool(true));
    assert_eq!(eval_bool(":user::eq-scalars-i64-neq"), Value::bool(false));
    assert_eq!(eval_bool(":user::eq-scalars-f64-eq"), Value::bool(true));
    assert_eq!(eval_bool(":user::eq-scalars-str-eq"), Value::bool(true));
    assert_eq!(eval_bool(":user::eq-scalars-bool-neq"), Value::bool(false));
}

#[test]
fn regression_eq_composites_recursive() {
    // Structural recursive equality over vectors (values_equal engine).
    assert_eq!(eval_bool(":user::eq-composites-equal"), Value::bool(true));
    assert_eq!(eval_bool(":user::eq-composites-diff-len"), Value::bool(false));
}

#[test]
fn regression_not_eq() {
    assert_eq!(eval_bool(":user::not-eq-neq"), Value::bool(true));
    assert_eq!(eval_bool(":user::not-eq-eq"), Value::bool(false));
}

#[test]
fn regression_cross_numeric_now_type_checks() {
    // arc 300 C5 RETIRED 237.8a's comparison-side reject (the same reversal C4 made
    // for arithmetic): mixed-numeric `=` now type-checks (`infer_equality`'s
    // `both_numeric` arm), matching eval + clj. `(= 1 2.0)` still EVALS to `false`
    // (category-aware `=`, C4's contract, unchanged) — C5 only makes the check accept
    // it as well-formed. Formerly `regression_cross_numeric_is_check_error`.
    let r = startup_from_file("tests/types/probe_arc237_8c_equality_grid_cross_numeric.wat");
    assert!(r.is_ok(), "cross-numeric `=` now type-checks (arc 300 C5); got: {:?}", r);
}

#[test]
fn regression_cross_type_is_check_error() {
    let r = startup_from_file("tests/types/probe_arc237_8c_equality_grid_cross_type.wat.bad");
    assert!(r.is_err(), "cross-type `=` must be a check error");
}

// Stone 237.8d — the per-Type alias mint tests (`:f64::=` / `:f64::not=`) were
// removed here. Those aliases are HARD CUT: equality is a RELATIONAL intrinsic;
// the canonical uniform `:wat::core::=` / `:wat::core::not=` are the only paths.
// The regression_* tests above exercise `=`/`not=` over f64 via the uniform ops.

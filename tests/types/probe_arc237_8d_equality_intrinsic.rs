//! FM-2-bis probe for Stone 237.8d — equality is a RELATIONAL intrinsic; the
//! grid residue (per-Type `::=` aliases) is HARD CUT.
//!
//! The reversal (see `docs/DISPATCH.md`): the clause matcher checks each arg
//! against a fixed named type independently and never unifies arg0 with arg1;
//! equality IS that cross-arg unification (`infer_equality` `unify(a,b)`, ∀T) —
//! so it is an intrinsic, not a clause. The mid-arc grid minted fake per-Type
//! leaves (`:i64::=`/`:f64::=`/…) that all alias to the one structural engine;
//! they contradict the doctrine and are cut here. `=`/`not=` impl is UNCHANGED.
//!
//! ROW STATUS:
//!   - REGRESSION (GREEN at HEAD + after): uniform `=`/`not=` over scalars,
//!     composites, and RECORDS (the ∀T relational case the cut must not regress);
//!     cross-numeric / cross-type stay check errors.
//!     [arc 300 C5 update: cross-numeric `=` now type-checks — see
//!     `regression_cross_numeric_now_type_checks` below; cross-type stays a check
//!     error, unchanged, out of C5's scope.]
//!   - CUT-CONFIRMERS (RED at HEAD — the aliases still resolve; `#[ignore]`'d):
//!     un-ignored by sonnet after the four aliases are removed, then GREEN.
//!
//! Run: cargo test --release --test probe_arc237_8d_equality_intrinsic

use wat::freeze::{call_beside, startup_from_file};
use wat::runtime::Value;

fn eval_bool(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("eval")
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — uniform `=`/`not=` over every type. GREEN at HEAD and after.
// The impl (infer_equality + eval_eq + values_equal) is untouched by 237.8d.
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
    assert_eq!(eval_bool(":user::eq-composites-equal"), Value::bool(true));
    assert_eq!(eval_bool(":user::eq-composites-diff-len"), Value::bool(false));
}

#[test]
fn regression_not_eq() {
    assert_eq!(eval_bool(":user::not-eq-neq"), Value::bool(true));
    assert_eq!(eval_bool(":user::not-eq-eq"), Value::bool(false));
}

#[test]
fn regression_eq_records_is_the_relational_case() {
    // The ∀T relational case: both args are :my::Pt; `infer_equality` unifies
    // their types; `values_equal` compares the records structurally. THIS is
    // why equality cannot be a finite clause list — and the cut must NOT
    // regress it.
    assert_eq!(eval_bool(":user::eq-records-equal"), Value::bool(true));
    assert_eq!(eval_bool(":user::eq-records-diff"), Value::bool(false));
}

#[test]
fn regression_cross_numeric_now_type_checks() {
    // arc 300 C5 RETIRED 237.8a's comparison-side reject: mixed-numeric `=` now
    // type-checks (`infer_equality`'s `both_numeric` arm), matching eval + clj.
    // `(= 1 2.0)` still EVALS to `false` (category-aware `=`, C4's contract,
    // unchanged). Formerly `regression_cross_numeric_is_check_error`.
    let r = startup_from_file(
        "tests/types/probe_arc237_8d_equality_intrinsic_cross_numeric.wat.bad",
    );
    assert!(r.is_ok(), "cross-numeric `=` now type-checks (arc 300 C5); got: {:?}", r);
}

#[test]
fn regression_cross_type_is_check_error() {
    let r = startup_from_file(
        "tests/types/probe_arc237_8d_equality_intrinsic_cross_type.wat.bad",
    );
    assert!(r.is_err(), "cross-type `=` must be a check error");
}

// ═══════════════════════════════════════════════════════════════════════════
// CUT-CONFIRMERS — the four per-Type equality aliases must NOT resolve after
// 237.8d. RED at HEAD (they still resolve) → `#[ignore]`. Un-ignore after the
// cut; then the unknown keyword fails check and these go GREEN.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cut_i64_eq_gone() {
    let r = startup_from_file(
        "tests/types/probe_arc237_8d_equality_intrinsic_cut_i64_eq.wat.bad",
    );
    assert!(r.is_err(), ":i64::= must be cut (unknown keyword)");
}

#[test]
fn cut_i64_not_eq_gone() {
    let r = startup_from_file(
        "tests/types/probe_arc237_8d_equality_intrinsic_cut_i64_not_eq.wat.bad",
    );
    assert!(r.is_err(), ":i64::not= must be cut");
}

#[test]
fn cut_f64_eq_gone() {
    let r = startup_from_file(
        "tests/types/probe_arc237_8d_equality_intrinsic_cut_f64_eq.wat.bad",
    );
    assert!(r.is_err(), ":f64::= must be cut");
}

#[test]
fn cut_f64_not_eq_gone() {
    let r = startup_from_file(
        "tests/types/probe_arc237_8d_equality_intrinsic_cut_f64_not_eq.wat.bad",
    );
    assert!(r.is_err(), ":f64::not= must be cut");
}

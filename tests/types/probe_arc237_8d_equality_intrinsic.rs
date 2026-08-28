//! FM-2-bis probe for Stone 237.8d — equality is a RELATIONAL intrinsic; the
//! grid residue (per-Type `::=` aliases) was HARD CUT, then RESTORED.
//!
//! The 237.8d reversal (see `docs/DISPATCH.md`): the clause matcher checks each arg
//! against a fixed named type independently and never unifies arg0 with arg1;
//! equality IS that cross-arg unification (`infer_equality` `unify(a,b)`, ∀T) —
//! so it is an intrinsic, not a clause. The mid-arc grid minted fake per-Type
//! leaves (`:i64::=`/`:f64::=`/…) that all alias to the one structural engine;
//! 237.8d judged them a contradiction of the doctrine and cut them. `=`/`not=`
//! impl is UNCHANGED throughout — untouched by either the cut or the restore.
//!
//! `DESIGN-STONE-per-type-equality-restored.md` (2026-08-05, builder's ruling)
//! REVERSED the cut in part: 237.8d applied "fake per-Type leaf, not a defclause
//! leaf" to equality but not to its twin, ordering (`:i64::>` etc are the same
//! shape and were kept). The asymmetry, not the relational-intrinsic argument,
//! was the defect — equality-as-intrinsic still stands, untouched.
//!
//! ROW STATUS:
//!   - REGRESSION (GREEN at HEAD + after): uniform `=`/`not=` over scalars,
//!     composites, and RECORDS (the ∀T relational case the restore must not
//!     regress); cross-numeric / cross-type stay check errors.
//!     [arc 300 C5 update: cross-numeric `=` now type-checks — see
//!     `regression_cross_numeric_now_type_checks` below; cross-type stays a check
//!     error, unchanged, out of C5's scope.]
//!   - RESTORED (GREEN — the four heads resolve, type-check, AND evaluate
//!     correctly; formerly CUT-CONFIRMERS asserting `r.is_err()` against
//!     `*_cut_*.wat.bad` fixtures. Those fixtures are gone: in this project
//!     `.wat.bad` means "the checker must reject this file" — a file the checker
//!     now ACCEPTS cannot honestly carry that extension. The coverage is not
//!     lost: the value-level tests below run the ops through the co-located
//!     `.wat` (a real `.wat`, via `call_beside_value`) and assert the returned
//!     booleans, which strictly subsumes "it type-checks.")
//!
//! Run: cargo test --release --test probe_arc237_8d_equality_intrinsic

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn eval_bool(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval")
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
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":wat::core::="
            && param == "#2"
            && expected == ":wat::core::i64"
            && got == ":wat::core::String"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// RESTORED — value-level proof (not just "checks"). Same co-located fixture +
// call_beside_value pattern the REGRESSION block above uses. Includes the f64
// NaN case: `eval_f64_compare` is already NaN-correct, so `NaN != NaN` falls
// out for free — this is the load-bearing property, not a guard to add.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn restored_i64_eq() {
    assert_eq!(eval_bool(":user::restored-i64-eq-true"), Value::bool(true));
    assert_eq!(eval_bool(":user::restored-i64-eq-false"), Value::bool(false));
}

#[test]
fn restored_i64_not_eq() {
    assert_eq!(eval_bool(":user::restored-i64-not-eq-true"), Value::bool(true));
    assert_eq!(eval_bool(":user::restored-i64-not-eq-false"), Value::bool(false));
}

#[test]
fn restored_f64_eq() {
    assert_eq!(eval_bool(":user::restored-f64-eq-true"), Value::bool(true));
    assert_eq!(eval_bool(":user::restored-f64-eq-false"), Value::bool(false));
}

#[test]
fn restored_f64_not_eq_including_nan() {
    assert_eq!(eval_bool(":user::restored-f64-not-eq-true"), Value::bool(true));
    // IEEE 754: NaN != NaN is `true` — a defined answer, not a domain hole.
    // `eval_f64_compare` already gets this right; do not special-case it.
    assert_eq!(eval_bool(":user::restored-f64-nan-not-eq-itself"), Value::bool(true));
}

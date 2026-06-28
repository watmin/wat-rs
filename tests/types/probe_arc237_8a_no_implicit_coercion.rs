//! FM-2-bis probe for Stone 237.8a — empirically prove THE DECISION's gap +
//! the regression contract for same-type arithmetic + comparison.
//!
//! THE DECISION (locked, `feedback_no_implicit_coercion`): no implicit numeric
//! coercion across the substrate. Today `infer_arithmetic` promotes
//! (i64, f64) → f64 silently; `infer_comparison:13158` accepts cross-numeric
//! `(< 1 2.0)` silently. After 237.8a both must REJECT at check time.
//!
//! Per-Type leaves + the variadic ergonomic surfaces (`:wat::core::+` etc.)
//! STAY (per the cliffnotes Headline state + DESIGN-STONE-237.7-intrinsic-kill).
//! Only the cross-type machinery dies.
//!
//! ROW STATUS:
//!   - 6 rows GREEN AT HEAD `169c5e07`+ (regression contract — same-type
//!     arithmetic + comparison + non-numeric comparison work today and post)
//!   - 3 rows `#[ignore]`d AT HEAD (disconfirming: cross-type today SUCCEEDS;
//!     the assertion `result.is_err()` FAILS at HEAD because the substrate
//!     silently coerces). Sonnet's stone work removes the `#[ignore]`
//!     annotations after tightening the substrate; the un-ignore is the contract.
//!
//! Run: cargo test --release --test probe_arc237_8a_no_implicit_coercion

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn eval_value(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for no_implicit_coercion fixture");
    let ast = wat::parse_one!(&format!("({fn_name})")).expect("parse fn call");
    eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned()
}

// ─── Same-type arithmetic — regression contract ─────────────────────────────

#[test]
fn arith_i64_same_type_works() {
    assert_eq!(
        eval_value(":user::arith-i64-same"),
        Value::i64(3),
        "i64 + i64 → i64 — preserved under THE DECISION",
    );
}

#[test]
fn arith_f64_same_type_works() {
    assert_eq!(
        eval_value(":user::arith-f64-same"),
        Value::f64(3.0),
        "f64 + f64 → f64 — preserved under THE DECISION",
    );
}

#[test]
fn arith_variadic_same_type_three_args_works() {
    // The variadic ergonomic surface STAYS under same-type discipline.
    assert_eq!(
        eval_value(":user::arith-variadic-same"),
        Value::i64(6),
        "variadic same-type fold preserved (3-arg + over i64)",
    );
}

// ─── Same-type comparison — regression contract ─────────────────────────────

#[test]
fn comparison_i64_same_type_works() {
    assert_eq!(
        eval_value(":user::cmp-i64-same"),
        Value::bool(true),
        "i64 < i64 → bool — preserved",
    );
}

#[test]
fn comparison_f64_same_type_works() {
    assert_eq!(
        eval_value(":user::cmp-f64-same"),
        Value::bool(true),
        "f64 < f64 → bool — preserved",
    );
}

#[test]
fn comparison_string_same_type_works() {
    // Non-numeric comparison — unaffected by THE DECISION (string=string).
    assert_eq!(
        eval_value(":user::cmp-str-same"),
        Value::bool(true),
        "string = string preserved — non-numeric path unaffected",
    );
}

// ─── Cross-type — disconfirming AT HEAD; un-ignore in Stone 237.8a ──────────
// Today these all SUCCEED silently (the falsehood THE DECISION rejects).
// Post-237.8a: they must reject at check (`infer_arithmetic` + `infer_comparison`
// tightened); `result.is_err()` flips to TRUE; the `#[ignore]` annotation
// comes off.

#[test]
fn arith_i64_f64_mixed_rejected_at_check() {
    let result = startup_from_file(
        "tests/types/probe_arc237_8a_no_implicit_coercion_arith_i64_f64_bad.wat",
    );
    assert!(
        result.is_err(),
        "i64 + f64 MUST reject at check (no implicit coercion); got: {:?}",
        result,
    );
}

#[test]
fn arith_f64_i64_mixed_rejected_at_check() {
    let result = startup_from_file(
        "tests/types/probe_arc237_8a_no_implicit_coercion_arith_f64_i64_bad.wat",
    );
    assert!(
        result.is_err(),
        "f64 + i64 MUST reject at check (no implicit coercion); got: {:?}",
        result,
    );
}

#[test]
fn comparison_i64_f64_mixed_rejected_at_check() {
    let result = startup_from_file(
        "tests/types/probe_arc237_8a_no_implicit_coercion_cmp_i64_f64_bad.wat",
    );
    assert!(
        result.is_err(),
        "i64 < f64 MUST reject at check (no implicit coercion in comparison); got: {:?}",
        result,
    );
}

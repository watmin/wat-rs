//! FM-2-bis probe for Stone 237.8b — recipe-lock + numeric grid via wat-defclause.
//!
//! Four LOAD-BEARING GATES (must settle empirically at HEAD before BRIEFing):
//!   Gate 1 — defclause supports `&` rest-binders in args-vec
//!   Gate 2 — defclause first-match dispatches by arg-`<-`-Type (not just :guard)
//!   Gate 3 — 0-ary clause body literal `0` infers as `:wat::core::i64`
//!   Gate 4 — per-Type ordering primitives correctness (including f64 NaN)
//!
//! Plus regression contract (preserve existing behavior) + mint-confirmers
//! (new behavior; `#[ignore]`'d at HEAD; un-ignored by sonnet post-stone).
//!
//! ROW STATUS (initial):
//!   - GATES: run at HEAD; if any fails RED, reshape strategy / defer 8b
//!     until defclause extension lands.
//!   - REGRESSION: pass at HEAD via existing `infer_arithmetic` /
//!     `eval_arithmetic_variadic` / per-Type variadic wat fns / 237.3
//!     i64 ordering aliases. Preserved post-stone via defclause.
//!   - MINT-CONFIRMERS: fail at HEAD (primitives don't exist); `#[ignore]`'d.
//!     Sonnet's stone work mints them; un-ignore after substrate edits.
//!   - CROSS-TYPE REJECTION: pass at HEAD via 8a's tightening (handler-level).
//!     Post-stone: rejection now via defclause `:NoMatchingClause`.
//!
//! Run: cargo test --release --test probe_arc237_8b_defclause_arithmetic

//! Wat source: tests/function/probe_arc237_8b_defclause_arithmetic.wat
//! Negative fixtures: probe_arc237_8b_gate2_cross_bad.wat, probe_arc237_8b_regression_cross_plus_bad.wat,
//!   probe_arc237_8b_regression_cross_lt_bad.wat, probe_arc237_8b_zero_ary_minus_bad.wat.

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for 8b defclause-arithmetic fixture");
    let ast = wat::parse_one!(&format!("({fn_name})")).expect("parse named fn call");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

// ═══════════════════════════════════════════════════════════════════════════
// GATES — load-bearing for 8b strategy
// ═══════════════════════════════════════════════════════════════════════════

/// Gate 1 — defclause supports `&` rest-binders; 1+2+3+4 = 10.
#[test]
fn gate_1_defclause_supports_rest_binder() {
    assert_eq!(run(":user::gate-1-sum-all"), Value::i64(10), "GATE 1: 1+2+3+4 = 10 via & rest-binder fold");
}

/// Gate 2 — first-match dispatches by arg-<--Type; i64 arg → "i64" clause fires.
#[test]
fn gate_2_defclause_dispatches_by_arg_type() {
    match run(":user::gate-2-dispatch") {
        Value::String(s) => assert_eq!(s.as_ref(), "i64", "GATE 2: i64 clause should fire for i64 arg"),
        other => panic!("GATE 2: expected Value::String('i64'); got {:?}", other),
    }
}

/// Gate 2-cross — cross-type (i64, f64) call must reject (no matching clause).
#[test]
fn gate_2_cross_no_matching_clause() {
    let result = startup_from_file("tests/function/probe_arc237_8b_gate2_cross_bad.wat");
    assert!(result.is_err(), "GATE 2-cross: (i64, f64) mixed args MUST reject; got Ok");
}

/// Gate 3 — 0-ary clause body literal `0` infers as :i64.
#[test]
fn gate_3_zero_ary_literal_infers_i64() {
    assert_eq!(run(":user::gate-3-zero-ary"), Value::i64(0), "GATE 3: 0-ary returns literal 0");
}

/// Gate 4a — i64 ordering primitives correctness (existing 237.3 aliases).
#[test]
fn gate_4a_i64_ordering_works() {
    assert_eq!(run(":user::gate-4a-lt"), Value::bool(true), "i64::< 1 2 → true");
    assert_eq!(run(":user::gate-4a-gt"), Value::bool(true), "i64::> 5 3 → true");
}

/// Gate 4b — f64 NaN handling: 1.0 < NaN is false per IEEE 754.
#[test]
fn gate_4b_f64_nan_ordering() {
    assert_eq!(run(":user::gate-4b-nan"), Value::bool(false), "1.0 < NaN MUST return false per IEEE 754");
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — preserve existing behavior
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_arith_i64_2ary_works() {
    assert_eq!(run(":user::regression-i64-2ary"), Value::i64(3));
}

#[test]
fn regression_arith_f64_2ary_works() {
    assert_eq!(run(":user::regression-f64-2ary"), Value::f64(3.0));
}

#[test]
fn regression_arith_variadic_3args_works() {
    assert_eq!(run(":user::regression-variadic-3"), Value::i64(6));
}

#[test]
fn regression_arith_minus_1ary_negate_i64() {
    // 1-ary `-` is negation: 0 - x
    assert_eq!(run(":user::regression-minus-negate"), Value::i64(-5));
}

#[test]
fn regression_ordering_i64_lt_works() {
    assert_eq!(run(":user::regression-lt"), Value::bool(true));
}

#[test]
fn regression_cross_type_plus_rejected() {
    let result = startup_from_file("tests/function/probe_arc237_8b_regression_cross_plus_bad.wat");
    assert!(result.is_err(), "cross-type i64+f64 MUST reject; got Ok");
}

#[test]
fn regression_cross_type_lt_rejected() {
    let result = startup_from_file("tests/function/probe_arc237_8b_regression_cross_lt_bad.wat");
    assert!(result.is_err(), "cross-type i64<f64 MUST reject; got Ok");
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT-CONFIRMERS — new primitives minted in Stone 237.8b
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mint_i64_lte_works() {
    assert_eq!(run(":user::mint-i64-lte-boundary"), Value::bool(true), "i64::<= 5 5 → true (boundary)");
    assert_eq!(run(":user::mint-i64-lte-false"), Value::bool(false));
}

#[test]
fn mint_f64_ordering_basic() {
    assert_eq!(run(":user::mint-f64-lt"), Value::bool(true));
    assert_eq!(run(":user::mint-f64-gte"), Value::bool(true));
}

#[test]
fn mint_i64_not_eq_renamed() {
    assert_eq!(run(":user::mint-not-eq"), Value::bool(true));
}

#[test]
fn mint_arith_zero_ary_plus_identity() {
    assert_eq!(run(":user::mint-plus-zero-ary"), Value::i64(0), "0-ary + returns i64 0 (Lisp identity)");
}

#[test]
fn mint_arith_zero_ary_star_identity() {
    assert_eq!(run(":user::mint-star-zero-ary"), Value::i64(1), "0-ary * returns i64 1 (Lisp identity)");
}

#[test]
fn mint_arith_zero_ary_minus_errors() {
    let result = startup_from_file("tests/function/probe_arc237_8b_zero_ary_minus_bad.wat");
    assert!(result.is_err(), "0-ary `-` MUST error (no clause for it); got Ok");
}

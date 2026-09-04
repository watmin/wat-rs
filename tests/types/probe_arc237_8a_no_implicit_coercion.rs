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

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn eval_value(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval")
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

// ─── Cross-type ARITHMETIC — arc 300 C4 RETIRED 237.8a's reject ─────────────
// 237.8a rejected all mixed arithmetic as a workaround for the unsolved N-ary
// problem. Arc 300 C4 solved N-ary (heterogeneous N-ary → an honest
// NoMatchingClause gap) and adopted 2-ary mixed contagion — "clojure's
// expressability on rust's platform": `(+ 1 2.0)` => 3.0 (float wins). So these
// mixed-ARITHMETIC fixtures now TYPE-CHECK (=> f64). Comparison was a separate
// thread; arc 300 C5 closed it too (see `comparison_i64_f64_mixed_coerces`
// below) — checker now matches eval + clj for mixed-numeric comparison.

#[test]
fn arith_i64_f64_mixed_coerces_to_f64() {
    // arc 300 C4: (+ i64 f64) => f64 — the fixture declares `-> f64` and now type-checks.
    let result = startup_from_file(
        "tests/types/probe_arc237_8a_no_implicit_coercion_arith_i64_f64.wat",
    );
    assert!(
        result.is_ok(),
        "i64 + f64 now coerces to f64 (arc 300 C4 retired 237.8a's reject); got: {:?}",
        result,
    );
}

#[test]
fn arith_f64_i64_mixed_coerces_to_f64() {
    // arc 300 C4: (+ f64 i64) => f64 (both operand orders).
    let result = startup_from_file(
        "tests/types/probe_arc237_8a_no_implicit_coercion_arith_f64_i64.wat",
    );
    assert!(
        result.is_ok(),
        "f64 + i64 now coerces to f64 (arc 300 C4 retired 237.8a's reject); got: {:?}",
        result,
    );
}

#[test]
fn comparison_i64_f64_mixed_coerces() {
    // arc 300 C5: mixed-numeric comparison now type-checks (consistency with C4's
    // arithmetic reversal + eval + clj — `(< 1 2.0)` => true both at check and eval).
    // Retires 237.8a's comparison-side cross-numeric reject; see `infer_ordering`'s
    // `both_numeric` arm (src/check.rs).
    let result = startup_from_file(
        "tests/types/probe_arc237_8a_no_implicit_coercion_cmp_i64_f64.wat",
    );
    assert!(
        result.is_ok(),
        "i64 < f64 now coerces/type-checks (arc 300 C5 retired 237.8a's comparison reject); got: {:?}",
        result,
    );
}

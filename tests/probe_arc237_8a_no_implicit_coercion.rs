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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn eval_value(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute").value_owned()
}

fn try_startup(src: &str) -> Result<(), String> {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Same-type arithmetic — regression contract ─────────────────────────────

#[test]
fn arith_i64_same_type_works() {
    assert_eq!(
        eval_value(
            r#"(:wat::core::define (:user::compute -> :wat::core::i64)
                 (:wat::core::+ 1 2))"#
        ),
        Value::i64(3),
        "i64 + i64 → i64 — preserved under THE DECISION",
    );
}

#[test]
fn arith_f64_same_type_works() {
    assert_eq!(
        eval_value(
            r#"(:wat::core::define (:user::compute -> :wat::core::f64)
                 (:wat::core::+ 1.0 2.0))"#
        ),
        Value::f64(3.0),
        "f64 + f64 → f64 — preserved under THE DECISION",
    );
}

#[test]
fn arith_variadic_same_type_three_args_works() {
    // The variadic ergonomic surface STAYS under same-type discipline.
    assert_eq!(
        eval_value(
            r#"(:wat::core::define (:user::compute -> :wat::core::i64)
                 (:wat::core::+ 1 2 3))"#
        ),
        Value::i64(6),
        "variadic same-type fold preserved (3-arg + over i64)",
    );
}

// ─── Same-type comparison — regression contract ─────────────────────────────

#[test]
fn comparison_i64_same_type_works() {
    assert_eq!(
        eval_value(
            r#"(:wat::core::define (:user::compute -> :wat::core::bool)
                 (:wat::core::< 1 2))"#
        ),
        Value::bool(true),
        "i64 < i64 → bool — preserved",
    );
}

#[test]
fn comparison_f64_same_type_works() {
    assert_eq!(
        eval_value(
            r#"(:wat::core::define (:user::compute -> :wat::core::bool)
                 (:wat::core::< 1.0 2.0))"#
        ),
        Value::bool(true),
        "f64 < f64 → bool — preserved",
    );
}

#[test]
fn comparison_string_same_type_works() {
    // Non-numeric comparison — unaffected by THE DECISION (string=string).
    assert_eq!(
        eval_value(
            r#"(:wat::core::define (:user::compute -> :wat::core::bool)
                 (:wat::core::= "a" "a"))"#
        ),
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
#[ignore = "Stone 237.8a: remove this #[ignore] when the substrate tightens — i64+f64 must reject at check"]
fn arith_i64_f64_mixed_rejected_at_check() {
    let result = try_startup(
        r#"(:wat::core::define (:user::compute -> :wat::core::f64)
             (:wat::core::+ 1 2.0))"#,
    );
    assert!(
        result.is_err(),
        "i64 + f64 MUST reject at check (no implicit coercion); got: {:?}",
        result,
    );
}

#[test]
#[ignore = "Stone 237.8a: remove this #[ignore] when the substrate tightens — f64+i64 must reject at check"]
fn arith_f64_i64_mixed_rejected_at_check() {
    let result = try_startup(
        r#"(:wat::core::define (:user::compute -> :wat::core::f64)
             (:wat::core::+ 1.0 2))"#,
    );
    assert!(
        result.is_err(),
        "f64 + i64 MUST reject at check (no implicit coercion); got: {:?}",
        result,
    );
}

#[test]
#[ignore = "Stone 237.8a: remove this #[ignore] when the substrate tightens — comparison cross-numeric must reject at check"]
fn comparison_i64_f64_mixed_rejected_at_check() {
    let result = try_startup(
        r#"(:wat::core::define (:user::compute -> :wat::core::bool)
             (:wat::core::< 1 2.0))"#,
    );
    assert!(
        result.is_err(),
        "i64 < f64 MUST reject at check (no implicit coercion in comparison); got: {:?}",
        result,
    );
}

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
//!   - CUT-CONFIRMERS (RED at HEAD — the aliases still resolve; `#[ignore]`'d):
//!     un-ignored by sonnet after the four aliases are removed, then GREEN.
//!
//! Run: cargo test --release --test probe_arc237_8d_equality_intrinsic

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

/// Define a 0-ary `:user::compute` returning bool with `body`, eval it.
fn eval_bool_expr(body: &str) -> Result<Value, String> {
    eval_bool_with_prelude("", body)
}

/// Same, with a `preamble` (e.g. a record def) spliced before `compute`.
fn eval_bool_with_prelude(preamble: &str, body: &str) -> Result<Value, String> {
    let src = format!(
        "{preamble}\n(:wat::core::defn :user::compute [] -> :wat::core::bool {body})"
    );
    let full = with_nil_main(&src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

/// True if startup (parse + check) accepts a bool-returning compute with `body`.
fn checks_ok(body: &str) -> bool {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::bool {body})"
    );
    let full = with_nil_main(&src);
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new())).is_ok()
}

const PT: &str =
    "(:wat::core::defrecord :my::Pt [x <- :wat::core::i64  y <- :wat::core::i64])";

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — uniform `=`/`not=` over every type. GREEN at HEAD and after.
// The impl (infer_equality + eval_eq + values_equal) is untouched by 237.8d.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_eq_scalars() {
    assert_eq!(eval_bool_expr("(:wat::core::= 1 1)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::= 1 2)").unwrap(), Value::bool(false));
    assert_eq!(eval_bool_expr("(:wat::core::= 1.0 1.0)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr(r#"(:wat::core::= "a" "a")"#).unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::= true false)").unwrap(), Value::bool(false));
}

#[test]
fn regression_eq_composites_recursive() {
    assert_eq!(eval_bool_expr("(:wat::core::= [1 2 3] [1 2 3])").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::= [1 2] [1 2 3])").unwrap(), Value::bool(false));
}

#[test]
fn regression_not_eq() {
    assert_eq!(eval_bool_expr("(:wat::core::not= 1 2)").unwrap(), Value::bool(true));
    assert_eq!(eval_bool_expr("(:wat::core::not= 1 1)").unwrap(), Value::bool(false));
}

#[test]
fn regression_eq_records_is_the_relational_case() {
    // The ∀T relational case: both args are :my::Pt; `infer_equality` unifies
    // their types; `values_equal` compares the records structurally. THIS is
    // why equality cannot be a finite clause list — and the cut must NOT
    // regress it.
    assert_eq!(
        eval_bool_with_prelude(PT, "(:wat::core::= (:my::Pt 0 0) (:my::Pt 0 0))").unwrap(),
        Value::bool(true)
    );
    assert_eq!(
        eval_bool_with_prelude(PT, "(:wat::core::= (:my::Pt 0 0) (:my::Pt 0 9))").unwrap(),
        Value::bool(false)
    );
}

#[test]
fn regression_cross_numeric_is_check_error() {
    assert!(!checks_ok("(:wat::core::= 1 2.0)"), "cross-numeric `=` must be a check error");
}

#[test]
fn regression_cross_type_is_check_error() {
    assert!(!checks_ok(r#"(:wat::core::= 1 "x")"#), "cross-type `=` must be a check error");
}

// ═══════════════════════════════════════════════════════════════════════════
// CUT-CONFIRMERS — the four per-Type equality aliases must NOT resolve after
// 237.8d. RED at HEAD (they still resolve) → `#[ignore]`. Un-ignore after the
// cut; then the unknown keyword fails check and these go GREEN.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cut_i64_eq_gone() {
    assert!(!checks_ok("(:wat::core::i64::= 1 1)"), ":i64::= must be cut (unknown keyword)");
}

#[test]
fn cut_i64_not_eq_gone() {
    assert!(!checks_ok("(:wat::core::i64::not= 1 2)"), ":i64::not= must be cut");
}

#[test]
fn cut_f64_eq_gone() {
    assert!(!checks_ok("(:wat::core::f64::= 1.0 1.0)"), ":f64::= must be cut");
}

#[test]
fn cut_f64_not_eq_gone() {
    assert!(!checks_ok("(:wat::core::f64::not= 1.0 2.0)"), ":f64::not= must be cut");
}

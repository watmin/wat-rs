//! Arc 278 stone 6b-i — `eval-test`, the runtime evaluator for `where`/`:test` predicates.
//!
//! `(:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :bool`
//! evaluates a boolean expr against a token's merged bindings (`?var → value`). Comparisons, computed
//! operands, string predicates, AND user fns (the whole point: "use their own fn for filtering").
//! A non-bool result is a TypeMismatch (a `where` is a predicate). Live mouth: `eval-test`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_6b_eval_test

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// 1 — a true comparison over bindings → true.
#[test]
fn comparison_true() {
    assert!(matches!(call_beside_value(file!(), ":user::comparison-true"), Ok(Value::bool(true))), "5>3 → true");
}

/// 2 — a false comparison over bindings → false.
#[test]
fn comparison_false() {
    assert!(matches!(call_beside_value(file!(), ":user::comparison-false"), Ok(Value::bool(false))), "3>5 → false");
}

/// 3 — a pure intrinsic predicate (string::starts-with?) over a string binding.
#[test]
fn string_predicate_over_binding() {
    let r = call_beside_value(file!(), ":user::string-predicate-over-binding");
    assert!(matches!(r, Ok(Value::bool(true))), "starts-with?(/admin/x, /admin) → true; got {r:?}");
}

/// 4 — a COMPUTED operand `(> (- ?hi ?lo) 10)` → true (the "any pure expr" proof, not just a 2-var cmp).
#[test]
fn computed_operand_true() {
    let r = call_beside_value(file!(), ":user::computed-operand-true");
    assert!(matches!(r, Ok(Value::bool(true))), "(20-5)>10 → true; got {r:?}");
}

/// 5 — the same computed operand, false branch.
#[test]
fn computed_operand_false() {
    let r = call_beside_value(file!(), ":user::computed-operand-false");
    assert!(matches!(r, Ok(Value::bool(false))), "(12-5)>10 → false; got {r:?}");
}

/// 6 — a USER-defined predicate over a binding (THE load-bearing case: filter with your own fn).
#[test]
fn user_fn_predicate() {
    let r = call_beside_value(file!(), ":user::user-fn-predicate");
    assert!(matches!(r, Ok(Value::bool(true))), "big?(150) → true; got {r:?}");
}

/// 7 — a non-bool result is a TypeMismatch (a `where` must be a predicate).
#[test]
fn non_bool_result_is_error() {
    let r = call_beside_value(file!(), ":user::non-bool-result-is-error");
    assert!(r.is_err(), "non-bool where expr must error; got {r:?}");
}

//! Arc 278 — Stone 6b-i: `eval-test`, the runtime evaluator for `where`/`:test` predicates.
//! RED at HEAD (`:wat::rete::eval-test` has no dispatch arm). GREEN when 6b-i lands.
//! Contract: DESIGN-STONE-6b-where-test.md.
//!
//! `(:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :bool`
//! evaluates a boolean expr against a token's merged bindings (`?var → value`). It reaches ANY pure ∧
//! deterministic op — comparisons, computed operands, string predicates, AND user fns (the whole point:
//! "use their own fn for filtering"). A non-bool result is a TypeMismatch (a `where` is a predicate).
//!
//! Run: cargo test --release -p wat --test probe_arc278_6b_eval_test

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::core::defn :test::big? [n <- :wat::core::i64] -> :wat::core::bool\n\
  (:wat::core::> n 100))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// Build `(:wat::rete::eval-test (quote <expr>) <bindings>)` and run it; bindings is a wat expr that
/// builds a PersistentMap (empty + /assoc), e.g. `{?a 5 ?b 3}`.
fn run(expr: &str, bindings: &str) -> Result<Value, String> {
    let compute = format!("(:wat::rete::eval-test (:wat::core::quote {expr}) {bindings})");
    let world = startup_from_source(WORLD, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&compute).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|t| t.value_owned())
}

/// Two-binding map `{?a a ?b b}`.
fn ab(a: i64, b: i64) -> String {
    format!(
        "(:wat::core::PersistentMap/assoc (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) \"?a\" {a}) \"?b\" {b})"
    )
}

/// 1 — a true comparison over bindings → true.
#[test]
fn comparison_true() {
    assert!(matches!(run("(:wat::core::> ?a ?b)", &ab(5, 3)), Ok(Value::bool(true))), "5>3 → true");
}

/// 2 — a false comparison over bindings → false.
#[test]
fn comparison_false() {
    assert!(matches!(run("(:wat::core::> ?a ?b)", &ab(3, 5)), Ok(Value::bool(false))), "3>5 → false");
}

/// 3 — a pure intrinsic predicate (string::starts-with?) over a string binding.
#[test]
fn string_predicate_over_binding() {
    let bindings = "(:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) \"?path\" \"/admin/x\")";
    let r = run("(:wat::core::string::starts-with? ?path \"/admin\")", bindings);
    assert!(matches!(r, Ok(Value::bool(true))), "starts-with?(/admin/x, /admin) → true; got {r:?}");
}

/// 4 — a COMPUTED operand `(> (- ?hi ?lo) 10)` → true (the "any pure expr" proof, not just a 2-var cmp).
#[test]
fn computed_operand_true() {
    let bindings = "(:wat::core::PersistentMap/assoc (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) \"?hi\" 20) \"?lo\" 5)";
    let r = run("(:wat::core::> (:wat::core::- ?hi ?lo) 10)", bindings);
    assert!(matches!(r, Ok(Value::bool(true))), "(20-5)>10 → true; got {r:?}");
}

/// 5 — the same computed operand, false branch.
#[test]
fn computed_operand_false() {
    let bindings = "(:wat::core::PersistentMap/assoc (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) \"?hi\" 12) \"?lo\" 5)";
    let r = run("(:wat::core::> (:wat::core::- ?hi ?lo) 10)", bindings);
    assert!(matches!(r, Ok(Value::bool(false))), "(12-5)>10 → false; got {r:?}");
}

/// 6 — a USER-defined predicate over a binding (THE load-bearing case: filter with your own fn).
#[test]
fn user_fn_predicate() {
    let bindings = "(:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) \"?x\" 150)";
    let r = run("(:test::big? ?x)", bindings);
    assert!(matches!(r, Ok(Value::bool(true))), "big?(150) → true; got {r:?}");
}

/// 7 — a non-bool result is a TypeMismatch (a `where` must be a predicate).
#[test]
fn non_bool_result_is_error() {
    let r = run("(:wat::core::+ ?a ?b)", &ab(1, 2));
    assert!(r.is_err(), "non-bool where expr must error; got {r:?}");
}

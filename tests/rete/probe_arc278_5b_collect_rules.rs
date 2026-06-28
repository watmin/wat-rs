//! Arc 278 stone 5b — disconfirming probe: `collect-rules` (namespace rule reflection). RED at HEAD.
//!
//! `(:wat::rete::collect-rules :ns) -> PersistentVector<Rule>` reflects the symbol table for zero-arg fns
//! returning `:wat::rete::Rule` (the `defrule` marker) whose name is in `:ns`, and invokes each. This is the
//! last piece before the north star: `(compile (collect-rules :weather))`.
//!
//! RED at HEAD: `collect-rules` is UnknownFunction.
//!
//! Run: cargo test --release -p wat --test probe_arc278_5b_collect_rules -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn ev(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

#[test]
fn collects_all_rules_in_a_namespace() {
    // :weather has 2 defrules (+ a non-rule defn `helper` that must NOT be counted).
    let got = ev("(:wat::core::length (:wat::rete::collect-rules :weather))");
    assert_eq!(got, Value::i64(2), "two :weather rules collected, helper excluded; got {got:?}");
}

#[test]
fn namespace_scoped() {
    // :other has exactly one rule; :weather's rules are NOT collected under :other.
    let got = ev("(:wat::core::length (:wat::rete::collect-rules :other))");
    assert_eq!(got, Value::i64(1), ":other has one rule; got {got:?}");
}

#[test]
fn empty_for_a_namespace_with_no_rules() {
    let got = ev("(:wat::core::length (:wat::rete::collect-rules :nonexistent))");
    assert_eq!(got, Value::i64(0), "no rules in :nonexistent → empty PV; got {got:?}");
}

#[test]
fn collected_values_are_rules_with_the_right_names() {
    // Sorted-by-name order: "cold-and-windy" < "cold-temp".
    let s = "(:wat::core::let [rs (:wat::rete::collect-rules :weather)]";
    assert_eq!(ev(&format!("{s} (:wat::rete::Rule/name (:wat::core::Option/expect (:wat::core::get rs 0) \"r0\")))")),
        Value::String(Arc::new("weather::cold-and-windy".to_string())), "first rule (sorted) is cold-and-windy");
    assert_eq!(ev(&format!("{s} (:wat::rete::Rule/name (:wat::core::Option/expect (:wat::core::get rs 1) \"r1\")))")),
        Value::String(Arc::new("weather::cold-temp".to_string())), "second rule (sorted) is cold-temp");
}

//! Arc 278 stone 5b — `collect-rules` (namespace rule reflection).
//!
//! `(:wat::rete::collect-rules :ns) -> PersistentVector<Rule>` reflects the symbol table for zero-arg fns
//! returning `:wat::rete::Rule` (the `defrule` marker) whose name is in `:ns`, and invokes each. Two
//! `:weather` rules (helper excluded); `:other` has one; `:nonexistent` is empty; names sort
//! cold-and-windy then cold-temp. Live mouth: `collect-rules`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_5b_collect_rules -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn collects_all_rules_in_a_namespace() {
    let got = call_beside_value(file!(), ":user::weather-rule-count").expect("eval");
    assert_eq!(got, Value::i64(2), "two :weather rules collected, helper excluded; got {got:?}");
}

#[test]
fn namespace_scoped() {
    let got = call_beside_value(file!(), ":user::other-rule-count").expect("eval");
    assert_eq!(got, Value::i64(1), ":other has one rule; got {got:?}");
}

#[test]
fn empty_for_a_namespace_with_no_rules() {
    let got = call_beside_value(file!(), ":user::nonexistent-rule-count").expect("eval");
    assert_eq!(got, Value::i64(0), "no rules in :nonexistent → empty PV; got {got:?}");
}

#[test]
fn collected_values_are_rules_with_the_right_names() {
    assert_eq!(call_beside_value(file!(), ":user::first-collected-rule-name").expect("eval"),
        Value::String(Arc::new("weather::cold-and-windy".to_string())), "first rule (sorted) is cold-and-windy");
    assert_eq!(call_beside_value(file!(), ":user::second-collected-rule-name").expect("eval"),
        Value::String(Arc::new("weather::cold-temp".to_string())), "second rule (sorted) is cold-temp");
}

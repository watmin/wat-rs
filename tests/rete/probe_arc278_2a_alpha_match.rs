//! Arc 278 stone 2a — disconfirming probe: `:wat::rete::alpha-match`. RED at HEAD.
//!
//! The rete single-fact matcher, purpose-built (NOT form::matches?): given a condition form (DATA) and a
//! fact (record), return `Some(bindings)` iff the fact's type == the condition head AND every clause holds,
//! else `None` (Clara no-error). Pure data-in/data-out — no Environment, no eval. Bindings = a PersistentMap
//! keyed by the logic-var name string ("?t").
//!
//! The DSL it interprets (its own classifier, NOT classify_clause):
//!   (?v <- :field)              bind ?v to the fact's :field
//!   (:wat::core::<op> a b)      FQDN constraint; operands ∈ {?var, :field, literal}, resolved purely
//!   (:wat::rete::and/or/not …)  clause combinators
//!   (:wat::rete::where …)       stone-6 escape (out of scope here)
//!
//! RED at HEAD: `:wat::rete::alpha-match` is unknown (Temp/quote/Option/PersistentMap all exist).
//!
//! Run: cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside;
use wat::runtime::Value;

fn is_none_option(v: &Value) -> bool {
    matches!(v, Value::Option(o) if o.is_none())
}

#[test]
fn alpha_match_binds_and_constrains() {
    let got = call_beside(file!(), ":user::match-binds-and-constrains").expect("eval");
    assert_eq!(
        got, Value::Option(Arc::new(Some(Value::i64(25)))),
        "alpha-match should bind ?t=25 and pass (> ?t 20); got {got:?}"
    );
}

#[test]
fn alpha_match_rejects_failed_constraint() {
    let got = call_beside(file!(), ":user::match-rejects-failed-constraint").expect("eval");
    assert!(is_none_option(&got), "failed constraint → None; got {got:?}");
}

#[test]
fn alpha_match_rejects_wrong_type() {
    let got = call_beside(file!(), ":user::match-rejects-wrong-type").expect("eval");
    assert!(is_none_option(&got), "wrong fact type → None; got {got:?}");
}

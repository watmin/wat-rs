//! Arc 278 stone 2b — disconfirming probe: `insert` (stage) + `fire-rules` (alpha slice). RED at HEAD.
//!
//! Model: `insert` stages a fact into Session.facts with ZERO activation; `fire-rules` runs the staged facts
//! through the network's AlphaNodes via `alpha-match` (2a) and populates alpha-memory. (v1 = alpha slice only;
//! beta join = stone 3, production/cascade = stone 4.)
//!
//! The network: one rule, one condition `(:user::Temp (?t <- :value) (:wat::core::> ?t 20))`. We stage a
//! matching fact (25) AND a non-matching one (15, fails > 20), fire, and inspect alpha-memory:
//!   (1) exactly one AlphaNode is populated,
//!   (2) it holds exactly one Element (15 was rejected — activation honors alpha-match's constraints),
//!   (3) that Element's bindings carry ?t == 25 (bindings flow alpha-match → Element).
//!
//! RED at HEAD: `:wat::rete::insert` / `fire-rules` unknown (compile/Session/Temp/alpha-match all exist).
//!
//! Run: cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn fire_populates_exactly_one_alpha() {
    // One condition → one AlphaNode; one of the two staged facts matches → exactly one populated alpha.
    let got = call_beside(file!(), ":user::alpha-populated-count").expect("eval");
    assert_eq!(got, Value::i64(1), "exactly one AlphaNode populated; got {got:?}");
}

#[test]
fn fire_stores_only_the_matching_element() {
    // The populated alpha holds ONE Element — 15 was rejected by (> ?t 20), proving activation honors the
    // full alpha-match (not just the type head).
    let got = call_beside(file!(), ":user::alpha-matching-element-count").expect("eval");
    assert_eq!(got, Value::i64(1), "only the matching fact (25) becomes an Element; got {got:?}");
}

#[test]
fn fire_element_carries_alpha_bindings() {
    // The stored Element's bindings carry ?t = 25 — bindings flow from alpha-match into the Element.
    let got = call_beside(file!(), ":user::alpha-element-t-binding").expect("eval");
    assert_eq!(got, Value::Option(Arc::new(Some(Value::i64(25)))), "Element binds ?t=25; got {got:?}");
}

//! Arc 278 stone 2b — `insert` (stage) + `fire-once` (alpha slice).
//!
//! Model: `insert` stages a fact into Session.facts with ZERO activation; `fire-once` runs the staged facts
//! through the network's AlphaNodes via compiled alpha exec and populates alpha-memory. (v1 = alpha slice only;
//! beta join = stone 3, production/cascade = stone 4.)
//!
//! The network: one rule, one condition `(:user::Temp (?t <- :value) (:wat::rete::core::i64::> ?t 20))`. We stage a
//! matching fact (25) AND a non-matching one (15, fails > 20), fire-once, and inspect alpha-memory:
//!   (1) exactly one AlphaNode is populated,
//!   (2) it holds exactly one Element (15 was rejected — activation honors alpha-match's constraints),
//!   (3) that Element's bindings carry ?t == 25 (bindings flow alpha-match → Element).
//!
//! arc 278 "alpha is fire-scoped" (v2): the fixture fires via native `fire-once` (single-pass),
//! not `fire-rules` (fixpoint). `fire-once` mirrors the oracle's `fire-once`, which genuinely
//! populates alpha — the fixpoint pair (`fire-rules` / `fire-rules$oracle`) now both return alpha
//! empty, so it stopped being a truthful place to observe alpha activation. See
//! `probe_arc278_alpha_is_fire_scoped.rs` for the differential covering the fixpoint verbs.
//!
//! Run: cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn compile_then_fire_empty_alpha() {
    let got = call_beside_value(file!(), ":user::compile-then-fire-empty-alpha").expect("eval");
    assert_eq!(got, Value::i64(0), "compile+fire-once with no facts populates no alpha; got {got:?}");
}

#[test]
fn seed_temps_stages_two_facts() {
    let got = call_beside_value(file!(), ":user::seed-temps-fact-count").expect("eval");
    assert_eq!(got, Value::i64(2), "seed-temps stages matching + non-matching Temp; got {got:?}");
}

#[test]
fn fire_populates_exactly_one_alpha() {
    // One condition → one AlphaNode; one of the two staged facts matches → exactly one populated alpha.
    let got = call_beside_value(file!(), ":user::alpha-populated-count").expect("eval");
    assert_eq!(got, Value::i64(1), "exactly one AlphaNode populated; got {got:?}");
}

#[test]
fn fire_stores_only_the_matching_element() {
    // The populated alpha holds ONE Element — 15 was rejected by (> ?t 20), proving activation honors the
    // full alpha-match (not just the type head).
    let got = call_beside_value(file!(), ":user::alpha-matching-element-count").expect("eval");
    assert_eq!(got, Value::i64(1), "only the matching fact (25) becomes an Element; got {got:?}");
}

#[test]
fn fire_element_carries_alpha_bindings() {
    // The stored Element's bindings carry ?t = 25 — bindings flow from alpha-match into the Element.
    let got = call_beside_value(file!(), ":user::alpha-element-t-binding").expect("eval");
    assert_eq!(got, Value::Option(Arc::new(Some(Value::i64(25)))), "Element binds ?t=25; got {got:?}");
}

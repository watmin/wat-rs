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
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::Record::def :user::Temp [value <- :wat::core::i64])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

// let-bindings shared by every assertion: build the fired session + grab alpha-memory.
const SETUP: &str = "\
   cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))\
   rule  (:wat::rete::Rule \"r\" (:wat::core::PersistentVector cond) (:wat::core::PersistentVector))\
   sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
   sess1 (:wat::rete::insert sess0 (:user::Temp 25))\
   sess2 (:wat::rete::insert sess1 (:user::Temp 15))\
   fired (:wat::rete::fire-rules sess2)\
   amem  (:wat::rete::Session/alpha-memory fired)";

fn ev(expr: &str) -> Value {
    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

#[test]
fn fire_populates_exactly_one_alpha() {
    // One condition → one AlphaNode; one of the two staged facts matches → exactly one populated alpha.
    let got = ev(&format!(
        "(:wat::core::let [{SETUP}] (:wat::core::length (:wat::core::PersistentMap/keys amem)))"
    ));
    assert_eq!(got, Value::i64(1), "exactly one AlphaNode populated; got {got:?}");
}

#[test]
fn fire_stores_only_the_matching_element() {
    // The populated alpha holds ONE Element — 15 was rejected by (> ?t 20), proving activation honors the
    // full alpha-match (not just the type head).
    let got = ev(&format!(
        "(:wat::core::let [{SETUP}\
           aid   (:wat::core::Option/expect (:wat::core::get (:wat::core::PersistentMap/keys amem) 0) \"aid\")\
           elems (:wat::core::Option/expect (:wat::core::PersistentMap/get amem aid) \"elems\")]\
          (:wat::core::length elems))"
    ));
    assert_eq!(got, Value::i64(1), "only the matching fact (25) becomes an Element; got {got:?}");
}

#[test]
fn fire_element_carries_alpha_bindings() {
    // The stored Element's bindings carry ?t = 25 — bindings flow from alpha-match into the Element.
    let got = ev(&format!(
        "(:wat::core::let [{SETUP}\
           aid   (:wat::core::Option/expect (:wat::core::get (:wat::core::PersistentMap/keys amem) 0) \"aid\")\
           elems (:wat::core::Option/expect (:wat::core::PersistentMap/get amem aid) \"elems\")\
           elem  (:wat::core::Option/expect (:wat::core::get elems 0) \"elem\")\
           binds (:wat::rete::Element/bindings elem)]\
          (:wat::core::PersistentMap/get binds \"?t\"))"
    ));
    assert_eq!(got, Value::Option(Arc::new(Some(Value::i64(25)))), "Element binds ?t=25; got {got:?}");
}

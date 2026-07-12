//! Arc 278 stone 3a — disconfirming probe: `RootJoinNode` seeding (first beta slice). RED at HEAD.
//!
//! After the alpha pass (2b), `fire-rules` grows a root-join pass: each first-condition Element is lifted into
//! a fresh Token and stored in beta-memory. (No hash-join yet = stone 3b; no production = stone 4.)
//!
//! One-condition rule `(:user::Temp (?t <- :value) (:wat::core::> ?t 20))` → 1 AlphaNode + 1 RootJoinNode.
//! Insert a matching fact (25), fire, inspect Session/beta-memory:
//!   (1) exactly one node populated (the RootJoinNode),
//!   (2) it holds exactly one Token,
//!   (3) that Token's bindings carry ?t == 25 (alpha-bindings carried into the seed),
//!   (4) that Token's matches chain has length 1 (the one supporting fact).
//!
//! RED at HEAD: `fire-rules` is alpha-only (2b) → beta-memory is empty → keys count is 0, not 1.
//!
//! Run: cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

const SETUP: &str = "\
   cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))\
   rule  (:wat::rete::Rule' \"r\" (:wat::core::PersistentVector cond) (:wat::core::PersistentVector))\
   sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
   sess1 (:wat::rete::insert sess0 (:user::Temp 25))\
   fired (:wat::rete::fire-rules sess1)\
   bmem  (:wat::rete::Session/beta-memory fired)";

// Reach the (single) seeded Token: first beta-memory key → its Token PV → element 0.
const TOK: &str = "\
   rjid (:wat::core::Option/expect -> :wat::core::i64 (:wat::core::get (:wat::core::PersistentMap/keys bmem) 0) \"rjid\")\
   toks (:wat::core::Option/expect -> :wat::core::PersistentVector (:wat::core::PersistentMap/get bmem rjid) \"toks\")\
   tok  (:wat::core::Option/expect -> :wat::rete::Token (:wat::core::get toks 0) \"tok\")";

fn ev(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::root_join_seeds_one_token_per_element
#[test]
#[ignore]
fn root_join_populates_one_beta_node() {
    let got = ev(&format!(
        "(:wat::core::let [{SETUP}] (:wat::core::length (:wat::core::PersistentMap/keys bmem)))"
    ));
    assert_eq!(got, Value::i64(1), "exactly one beta node (the RootJoinNode) seeded; got {got:?}");
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::root_join_seeds_one_token_per_element
#[test]
#[ignore]
fn root_join_seeds_one_token() {
    let got = ev(&format!(
        "(:wat::core::let [{SETUP}{TOK}] (:wat::core::length toks))"
    ));
    assert_eq!(got, Value::i64(1), "one Element → one seeded Token; got {got:?}");
}

// P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
// regenerates on re-fire. Join-correctness coverage relocated to:
//   src/rete/kernel.rs #[cfg(test)]::root_join_seeds_one_token_per_element
#[test]
#[ignore]
fn seeded_token_carries_bindings_and_support() {
    let binds = ev(&format!(
        "(:wat::core::let [{SETUP}{TOK} binds (:wat::rete::Token/bindings tok)] \
           (:wat::core::PersistentMap/get binds \"?t\"))"
    ));
    assert_eq!(binds, Value::Option(Arc::new(Some(Value::i64(25)))), "Token carries ?t=25; got {binds:?}");

    let support_len = ev(&format!(
        "(:wat::core::let [{SETUP}{TOK}] (:wat::core::length (:wat::rete::Token/matches tok)))"
    ));
    assert_eq!(support_len, Value::i64(1), "Token's support chain has one entry; got {support_len:?}");
}

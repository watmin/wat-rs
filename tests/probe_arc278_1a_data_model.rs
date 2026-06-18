//! Arc 278 stone 1a — disconfirming probe: the rete data model (RED at HEAD).
//!
//! Stone 1a mints `wat/rete.wat` with the engine's data records + the `Node` defenum + the `Session`
//! record (the whole caller-facing engine state), on the stone-0 persistent collections. This probe builds
//! a tiny network BY HAND — a RootJoinNode(id 0) → ProductionNode(id 1) — puts them in `Session.network`
//! (a PersistentMap id→Node), and asserts (a) the network holds both nodes, (b) `render-dag` produces a
//! non-empty inspectable string. No compile, no fire — just the data model standing as data.
//!
//! RED at HEAD: `:wat::rete::Session` / `RootJoinNode` / `render-dag` are unknown heads → eval error → the
//! `expect`s panic. Compiles at HEAD (public API + wat strings); fails at RUNTIME on exactly the gap.
//!
//! Run: cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A `let` that hand-builds a 2-node Session and exposes `s`; the caller appends the body expression.
const SESSION: &str = "\
(:wat::core::let \
  [n0 (:wat::rete::RootJoinNode 0 (:wat::core::PersistentVector 1) (:wat::core::PersistentVector)) \
   n1 (:wat::rete::ProductionNode 1 \"rule-1\") \
   net (:wat::core::PersistentMap 0 n0 1 n1) \
   em  (:wat::core::PersistentMap) \
   ev  (:wat::core::PersistentVector) \
   s   (:wat::rete::Session net ev em em em ev 2)] \
  ";

#[test]
#[ignore = "arc 278 stone 1a — un-ignore when wat/rete.wat ships"]
fn rete_data_model_constructs_and_renders() {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");

    let ev = |body: &str| -> Value {
        let expr = format!("{SESSION}{body})");
        let ast = wat::parse_one!(&expr).expect("parse");
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval raised: {e:?}\n  expr: {expr}"))
            .value_owned()
    };

    // (a) the network (id → Node) holds both nodes
    assert_eq!(
        ev("(:wat::core::PersistentMap/length (:wat::rete::Session/network s))"),
        Value::i64(2),
        "Session.network must hold both nodes"
    );

    // (b) render-dag produces a non-empty inspectable string
    match ev("(:wat::rete::render-dag s)") {
        Value::String(s) => assert!(!s.is_empty(), "render-dag must produce a non-empty graph string"),
        other => panic!("render-dag must return a String; got {other:?}"),
    }
}

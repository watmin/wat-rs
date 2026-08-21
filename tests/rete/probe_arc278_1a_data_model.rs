//! Arc 278 stone 1a — nine rete node records + `Session`. No `:wat::rete::Node` defenum.
//!
//! `wat/rete.wat` holds nine node records (Alpha / RootJoin / HashJoin / Production / Test /
//! Negation / Exists / Accumulate / Query) plus `Session`. This probe builds a tiny network BY
//! HAND — a `RootJoinNode`(id 0) → `ProductionNode`(id 1) — puts them in `Session.network`
//! (PersistentMap id→record), and asserts (a) the network holds both nodes, (b) `render-dag`
//! produces a non-empty inspectable string. No compile, no fire — the data model standing as data.
//! Live mouths: `Session`, `RootJoinNode`, `ProductionNode`, `render-dag`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn rete_data_model_constructs_and_renders() {
    // (a) the network (id → Node) holds both nodes
    assert_eq!(
        call_beside_value(file!(), ":user::network-length").expect("network-length eval"),
        Value::i64(2),
        "Session.network must hold both nodes"
    );

    // (b) render-dag produces a non-empty inspectable string
    match call_beside_value(file!(), ":user::render-dag-of-session").expect("render-dag eval") {
        Value::String(s) => assert!(!s.is_empty(), "render-dag must produce a non-empty graph string"),
        other => panic!("render-dag must return a String; got {other:?}"),
    }
}

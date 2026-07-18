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

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn rete_data_model_constructs_and_renders() {
    // (a) the network (id → Node) holds both nodes
    assert_eq!(
        call_beside(file!(), ":user::network-length").expect("network-length eval"),
        Value::i64(2),
        "Session.network must hold both nodes"
    );

    // (b) render-dag produces a non-empty inspectable string
    match call_beside(file!(), ":user::render-dag-of-session").expect("render-dag eval") {
        Value::String(s) => assert!(!s.is_empty(), "render-dag must produce a non-empty graph string"),
        other => panic!("render-dag must return a String; got {other:?}"),
    }
}

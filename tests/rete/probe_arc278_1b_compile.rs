//! Arc 278 stone 1b — disconfirming probe: `compile` (rule-set → shared, CONNECTED network). RED at HEAD.
//!
//! `(:wat::rete::compile rules)` walks each rule's conditions left-to-right and builds the network with NODE
//! SHARING (the non-redundancy DAG) AND wires the child edges (alpha→join, parent→join, join→production).
//! A network without edges is not a compiled DAG — so this probe proves BOTH:
//!   (1) SHARING — two rules with an identical FIRST condition share its AlphaNode + RootJoinNode
//!       (counts: 3 AlphaNode, 1 RootJoinNode, 2 HashJoinNode, 2 ProductionNode — not 4/2/2/2).
//!   (2) THE CHAIN — `render-dag` emits each node's child edges, and the shared RootJoinNode has TWO children
//!       (the divergence after the shared prefix). An edgeless node-set (the deferred-edges failure mode) shows
//!       the RootJoinNode with `[]` and fails (2).
//!
//! `render-dag` edge format (the contract this probe pins): one line per node —
//!     `  <id>  <kind> -> [<child-id> <child-id> ...]`
//! children space-separated inside brackets; leaves (ProductionNode/QueryNode) render `-> []`.
//!
//! RED at HEAD: `:wat::rete::compile` is unknown (Rule/PersistentVector/quote/render-dag exist from 0/1a, so
//! the failure isolates the missing `compile`). GREEN when 1b ships: sharing + wired edges.
//!
//! Run: cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored

use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

/// Children-count of the FIRST node line whose text contains `kind` — parses the `[...]` child list.
fn children_count(rendered: &str, kind: &str) -> usize {
    let line = rendered
        .lines()
        .find(|l| l.contains(kind))
        .unwrap_or_else(|| panic!("no `{kind}` line in render:\n{rendered}"));
    let open = line.find('[').unwrap_or_else(|| panic!("`{kind}` line has no `[`: {line}"));
    let close = line.find(']').unwrap_or_else(|| panic!("`{kind}` line has no `]`: {line}"));
    line[open + 1..close].split_whitespace().count()
}

fn render(prog: &str) -> String {
    let world = startup_bare().expect("startup");
    let ast = wat::parse_one!(prog).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile raised: {e:?}"))
        .value_owned()
    {
        Value::String(s) => s.to_string(),
        other => panic!("render-dag must return a String; got {other:?}"),
    }
}

#[test]
fn compile_shares_prefix_and_wires_the_chain() {
    // Two rules; FIRST condition identical (c1), second divergent (c2a vs c2b).
    let prog = "\
(:wat::core::let \
  [c1  (:wat::core::quote (:Temperature (= ?t :value))) \
   c2a (:wat::core::quote (:Humidity    (= ?h :value))) \
   c2b (:wat::core::quote (:Pressure    (= ?p :value))) \
   rA  (:wat::rete::Rule \"rA\" (:wat::core::PersistentVector c1 c2a) (:wat::core::PersistentVector)) \
   rB  (:wat::rete::Rule \"rB\" (:wat::core::PersistentVector c1 c2b) (:wat::core::PersistentVector)) \
   sess (:wat::rete::compile (:wat::core::PersistentVector rA rB))] \
  (:wat::rete::render-dag sess))";
    let r = render(prog);

    // (1) SHARING — shared first condition collapses to ONE alpha + ONE root-join.
    assert_eq!(r.matches("AlphaNode").count(), 3, "shared C1 → 3 alphas (c1, c2a, c2b), not 4. render:\n{r}");
    assert_eq!(r.matches("RootJoinNode").count(), 1, "shared C1 → 1 root-join, not 2. render:\n{r}");
    assert_eq!(r.matches("HashJoinNode").count(), 2, "two divergent C2 → 2 hash-joins. render:\n{r}");
    assert_eq!(r.matches("ProductionNode").count(), 2, "one production per rule (not shared). render:\n{r}");

    // (2) THE CHAIN — the shared root-join has TWO children (the divergence). An edgeless network shows []
    //     here and fails. This is what the first attempt deferred.
    assert_eq!(
        children_count(&r, "RootJoinNode"), 2,
        "the shared RootJoinNode must wire BOTH divergent hash-joins as children (edges wired + prefix shared). render:\n{r}"
    );
}

#[test]
fn compile_single_rule_wires_a_connected_chain() {
    // One single-condition rule → alpha → root-join → production, fully connected.
    let prog = "\
(:wat::core::let \
  [c1 (:wat::core::quote (:Temperature (= ?t :value))) \
   rC (:wat::rete::Rule \"rC\" (:wat::core::PersistentVector c1) (:wat::core::PersistentVector)) \
   sess (:wat::rete::compile (:wat::core::PersistentVector rC))] \
  (:wat::rete::render-dag sess))";
    let r = render(prog);

    assert_eq!(r.matches("AlphaNode").count(), 1, "one condition → one alpha. render:\n{r}");
    assert_eq!(r.matches("RootJoinNode").count(), 1, "first condition → one root-join. render:\n{r}");
    assert_eq!(r.matches("ProductionNode").count(), 1, "one rule → one production. render:\n{r}");
    // The chain is wired: alpha → root-join (1 child), root-join → production (1 child).
    assert_eq!(children_count(&r, "AlphaNode"), 1, "alpha must wire its join. render:\n{r}");
    assert_eq!(children_count(&r, "RootJoinNode"), 1, "root-join must wire the production. render:\n{r}");
}

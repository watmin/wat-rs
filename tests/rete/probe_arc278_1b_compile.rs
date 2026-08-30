//! Arc 278 stone 1b — `compile` (rule-set → shared, CONNECTED network).
//!
//! `(:wat::core::match (:wat::rete::compile rules) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))` walks each rule's conditions left-to-right and builds the network with NODE
//! SHARING (the non-redundancy DAG) AND wires the child edges (alpha→join, parent→join, join→production).
//! A network without edges is not a compiled DAG — so this probe proves BOTH:
//!   (1) SHARING — two rules with an identical FIRST condition share its AlphaNode + RootJoinNode
//!       (counts: 3 AlphaNode, 1 RootJoinNode, 2 HashJoinNode, 2 ProductionNode — not 4/2/2/2).
//!   (2) THE CHAIN — `render-dag` emits each node's child edges, and the shared RootJoinNode has TWO children
//!       (the divergence after the shared prefix). An edgeless node-set shows the RootJoinNode with `[]` and fails (2).
//!
//! `render-dag` edge format: one line per node —
//!     `  <id>  <kind> -> [<child-id> <child-id> ...]`
//! children space-separated inside brackets; leaves (ProductionNode/QueryNode) render `-> []`.
//! Live mouths: `compile`, `render-dag`, `Rule`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored

use wat::freeze::call_beside_value;

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

fn render(entry: &str) -> String {
    match call_beside_value(file!(), entry).unwrap_or_else(|e| panic!("compile raised: {e:?}")) {
        wat::runtime::Value::String(s) => s.to_string(),
        other => panic!("render-dag must return a String; got {other:?}"),
    }
}

#[test]
fn compile_shares_prefix_and_wires_the_chain() {
    let r = render(":user::compile-shared-prefix");

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
    let r = render(":user::compile-single-rule");

    assert_eq!(r.matches("AlphaNode").count(), 1, "one condition → one alpha. render:\n{r}");
    assert_eq!(r.matches("RootJoinNode").count(), 1, "first condition → one root-join. render:\n{r}");
    assert_eq!(r.matches("ProductionNode").count(), 1, "one rule → one production. render:\n{r}");
    // The chain is wired: alpha → root-join (1 child), root-join → production (1 child).
    assert_eq!(children_count(&r, "AlphaNode"), 1, "alpha must wire its join. render:\n{r}");
    assert_eq!(children_count(&r, "RootJoinNode"), 1, "root-join must wire the production. render:\n{r}");
}

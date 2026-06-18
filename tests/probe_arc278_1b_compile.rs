//! Arc 278 stone 1b — disconfirming probe: `compile` (rule-set → shared network). RED at HEAD.
//!
//! `(:wat::rete::compile rules)` walks each rule's conditions left-to-right and builds the network
//! (id→Node) with NODE SHARING — the non-redundancy DAG. The proof of sharing: two rules whose FIRST
//! condition is IDENTICAL must share that condition's AlphaNode — it appears ONCE in the network, not
//! twice. We compile two such rules (shared first condition, divergent second) and count AlphaNode lines in
//! `render-dag`: exactly 3 (the shared C1 alpha + C2a's + C2b's), not 4.
//!
//! RED at HEAD: `:wat::rete::compile` is an unknown head (the Rule/PersistentVector/quote construction all
//! exist from stones 0/1a, so the failure isolates exactly the missing `compile`). GREEN when 1b ships.
//!
//! Run: cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

#[test]
#[ignore = "arc 278 stone 1b — un-ignore when :wat::rete::compile ships"]
fn compile_shares_the_common_alpha_node() {
    let world = startup_from_source(
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");

    // Two rules; FIRST condition identical (c1), second divergent (c2a vs c2b).
    // Conditions are form::matches? clause-forms, quoted as data into the lhs vector.
    let prog = "\
(:wat::core::let \
  [c1  (:wat::core::quote (:Temperature (= ?t :value))) \
   c2a (:wat::core::quote (:Humidity    (= ?h :value))) \
   c2b (:wat::core::quote (:Pressure    (= ?p :value))) \
   rA  (:wat::rete::Rule \"rA\" (:wat::core::PersistentVector c1 c2a) (:wat::core::PersistentVector)) \
   rB  (:wat::rete::Rule \"rB\" (:wat::core::PersistentVector c1 c2b) (:wat::core::PersistentVector)) \
   sess (:wat::rete::compile (:wat::core::PersistentVector rA rB))] \
  (:wat::rete::render-dag sess))";

    let ast = wat::parse_one!(prog).expect("parse");
    let rendered = match eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile raised: {e:?}"))
        .value_owned()
    {
        Value::String(s) => s.to_string(),
        other => panic!("render-dag must return a String; got {other:?}"),
    };

    // SHARING PROOF: the shared first condition yields ONE AlphaNode (3 total: c1, c2a, c2b — not 4).
    let alpha_count = rendered.matches("AlphaNode").count();
    assert_eq!(
        alpha_count, 3,
        "two rules sharing their first condition must share its AlphaNode (expect 3 alphas, got {alpha_count}); render:\n{rendered}"
    );
}

//! Arc 278 — deep forward-chain cascade DIFFERENTIAL gate (the net under P4b's delta rewrite).
//!
//! A depth-N × width-M cascade where every level is a 2-way join on the prior level's DERIVED facts:
//! `Stage{k-1}(?id) ⋈ Tag{k-1}(?id) → Stage{k}(?id), Tag{k}(?id)`. Distinct record types per level (so the
//! type system itself proves each rule is unlocked only by the lower rule's output — complements the wat perf
//! script's single-Node-type shape). `Stage{N}` is reachable ONLY after N cascade rounds.
//!
//! The contract: native `fire-rules` and wat `fire-rules$oracle` derive the SAME deepest-level count (== width =
//! full closure). Native is semi-naive delta (`fire_fixpoint_delta`); `$oracle` is re-run-from-scratch.
//! This gate stays green as the proof that delta == oracle at depth, while the wat perf bench shows
//! the O(depth²)→linear bend.
//!
//! Run: cargo test --release -p wat --test probe_arc278_deep_cascade

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::loader::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Frozen world: `Stage{k}` + `Tag{k}` record defs for k in 0..=depth, plus main.
// rune:lint(no-inlined-wat) — world generated from runtime depth parameter (N record types) — cannot be pre-extracted to a static .wat file
fn gen_world(depth: usize) -> String {
    let mut s = String::new();
    for k in 0..=depth {
        s.push_str(&format!(
            "(:wat::core::defrecord :casc::Stage{k} [id <- :wat::core::i64])\n\
             (:wat::core::defrecord :casc::Tag{k}   [id <- :wat::core::i64])\n"
        ));
    }
    s.push_str(&format!(
        "(:wat::rete::defquery :casc::q-Stage{depth}\n\
           :params []\n\
           :when [(:casc::Stage{depth})])\n"
    ));
    s
}

/// Fire expression: depth join-rules (Stage{k-1}⋈Tag{k-1}→Stage{k},Tag{k}), seed width ids at level 0,
/// fire with `fire_verb`, return the count of the deepest type `Stage{depth}` (== width iff full closure).
fn gen_expr(depth: usize, width: usize, fire_verb: &str) -> String {
    let mut binds = String::new();
    for k in 1..=depth {
        let p = k - 1;
        binds.push_str(&format!(
            "  r{k}c1 (:wat::core::quote (:casc::Stage{p} (?id <- :id)))\
             \n  r{k}c2 (:wat::core::quote (:casc::Tag{p} (?id <- :id)))\
             \n  r{k}t1 (:wat::core::quote (:casc::Stage{k} ?id))\
             \n  r{k}t2 (:wat::core::quote (:casc::Tag{k} ?id))\
             \n  rule{k} (:wat::rete::Rule :name \"r{k}\" :lhs (:wat::core::PersistentVector r{k}c1 r{k}c2) :rhs (:wat::core::PersistentVector r{k}t1 r{k}t2))\n"
        ));
    }
    binds.push_str("  s0 (:wat::rete::compile-all (:wat::core::PersistentVector");
    for k in 1..=depth { binds.push_str(&format!(" rule{k}")); }
    binds.push_str(&format!(
        ") (:wat::core::PersistentVector (:casc::q-Stage{depth})))\n"
    ));
    let mut idx = 1usize;
    let mut prev = 0usize;
    for i in 0..width {
        binds.push_str(&format!("  s{idx} (:wat::rete::insert s{prev} (:casc::Stage0 :id {i}))\n"));
        prev = idx; idx += 1;
        binds.push_str(&format!("  s{idx} (:wat::rete::insert s{prev} (:casc::Tag0 :id {i}))\n"));
        prev = idx; idx += 1;
    }
    format!(
        "(:wat::core::let [{binds}\n fired ({fire_verb} s{prev})]\
           (:wat::core::length (:wat::rete::query fired (:casc::q-Stage{depth}))))"
    )
}

fn run(depth: usize, width: usize, fire_verb: &str) -> Value {
    let world = startup_from_source(&gen_world(depth), None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!(&gen_expr(depth, width, fire_verb)).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}")).value_owned()
}

/// depth 10: native fire-rules == fire-rules$oracle == full closure (width).
#[test]
fn deep_cascade_native_matches_wat_depth10() {
    let (depth, width) = (10, 3);
    let native = run(depth, width, ":wat::rete::fire-rules");
    let wat = run(depth, width, ":wat::rete::fire-rules$oracle");
    assert_eq!(native, wat, "native must equal wat at depth {depth}; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(width as i64), "full {depth}-deep closure → {width} Stage{depth}; got {native:?}");
}

/// depth 20: the distinction-biting depth — native fire-rules == fire-rules$oracle == full closure.
#[test]
fn deep_cascade_native_matches_wat_depth20() {
    let (depth, width) = (20, 2);
    let native = run(depth, width, ":wat::rete::fire-rules");
    let wat = run(depth, width, ":wat::rete::fire-rules$oracle");
    assert_eq!(native, wat, "native must equal wat at depth {depth}; {native:?} vs {wat:?}");
    assert_eq!(native, Value::i64(width as i64), "full {depth}-deep closure → {width} Stage{depth}; got {native:?}");
}

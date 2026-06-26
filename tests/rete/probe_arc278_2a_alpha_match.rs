//! Arc 278 stone 2a — disconfirming probe: `:wat::rete::alpha-match`. RED at HEAD.
//!
//! The rete single-fact matcher, purpose-built (NOT form::matches?): given a condition form (DATA) and a
//! fact (record), return `Some(bindings)` iff the fact's type == the condition head AND every clause holds,
//! else `None` (Clara no-error). Pure data-in/data-out — no Environment, no eval. Bindings = a PersistentMap
//! keyed by the logic-var name string ("?t").
//!
//! The DSL it interprets (its own classifier, NOT classify_clause):
//!   (?v <- :field)              bind ?v to the fact's :field
//!   (:wat::core::<op> a b)      FQDN constraint; operands ∈ {?var, :field, literal}, resolved purely
//!   (:wat::rete::and/or/not …)  clause combinators
//!   (:wat::rete::where …)       stone-6 escape (out of scope here)
//!
//! RED at HEAD: `:wat::rete::alpha-match` is unknown (Temp/quote/Option/PersistentMap all exist).
//!
//! Run: cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

// Condition: a Temp whose :value binds ?t and must be > 20.
const COND: &str =
    "(:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))";

fn world() -> wat::freeze::FrozenWorld {
    startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new())).expect("startup")
}

fn ev(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval `{expr}` raised: {e:?}"))
        .value_owned()
}

fn is_none_option(v: &Value) -> bool {
    matches!(v, Value::Option(o) if o.is_none())
}

#[test]
fn alpha_match_binds_and_constrains() {
    let w = world();
    // MATCH: 25 binds ?t and 25 > 20 holds → Some({"?t": 25}); PersistentMap/get "?t" → Some(25).
    let got = ev(&w, &format!(
        "(:wat::core::PersistentMap/get \
           (:wat::core::Option/expect \
             (:wat::rete::alpha-match {COND} (:user::Temp 25)) \"matched\") \
           \"?t\")"
    ));
    assert_eq!(
        got, Value::Option(Arc::new(Some(Value::i64(25)))),
        "alpha-match should bind ?t=25 and pass (> ?t 20); got {got:?}"
    );
}

#[test]
fn alpha_match_rejects_failed_constraint() {
    let w = world();
    // 15 binds ?t but 15 > 20 is false → None (no-error, not a raise).
    let got = ev(&w, &format!("(:wat::rete::alpha-match {COND} (:user::Temp 15))"));
    assert!(is_none_option(&got), "failed constraint → None; got {got:?}");
}

#[test]
fn alpha_match_rejects_wrong_type() {
    let w = world();
    // Condition head :user::Other ≠ fact type :user::Temp → None.
    let got = ev(&w, &format!(
        "(:wat::rete::alpha-match (:wat::core::quote (:user::Other (?t <- :value))) (:user::Temp 25))"
    ));
    assert!(is_none_option(&got), "wrong fact type → None; got {got:?}");
}

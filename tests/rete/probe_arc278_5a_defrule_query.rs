//! Arc 278 stone 5a — disconfirming probe: `defrule` (rule macro) + `query` (read derived facts). RED at HEAD.
//!
//! The wat half of the homoiconic surface. `query` reads derived facts of a type out of a fired session;
//! `defrule` expands the readable rule form into a zero-arg `defn` returning a `Rule`. The reflection that
//! auto-gathers rules (`collect-rules`) is 5b — here the one rule is collected manually by calling its fn.
//!
//! RED at HEAD: `query` is UnknownFunction; the `defrule` macro is undefined so a world containing it fails
//! to freeze.
//!
//! Run: cargo test --release -p wat --test probe_arc278_5a_defrule_query -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

// Paths to the co-located .wat fixtures (relative to the crate root).
const WORLD_PLAIN_PATH: &str = "tests/rete/probe_arc278_5a_defrule_query_plain.wat";
const WORLD_WITH_RULE_PATH: &str = "tests/rete/probe_arc278_5a_defrule_query_with_rule.wat";

fn ev(world_path: &str, expr: &str) -> Value {
    let world = startup_from_file(world_path).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

// A hand-built cold-and-windy rule + a fired session (no defrule needed) — for the query-only tests.
const HANDBUILT_FIRED: &str = "\
   c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))\
   c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))\
   rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\
   rule  (:wat::rete::Rule \"weather::cold-and-windy\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector rhs1))\
   sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
   s1    (:wat::rete::insert sess0 (:weather::Temperature 15 \"Oslo\"))\
   s2    (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))\
   fired (:wat::rete::fire-rules s2)";

// ── query ───────────────────────────────────────────────────────────────────

#[test]
fn query_reads_derived_facts_by_type() {
    let got = ev(WORLD_PLAIN_PATH, &format!(
        "(:wat::core::let [{HANDBUILT_FIRED}] \
           (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy)))"));
    assert_eq!(got, Value::i64(1), "query returns the one derived ColdAndWindy; got {got:?}");
}

#[test]
fn query_empty_for_absent_type() {
    let got = ev(WORLD_PLAIN_PATH, &format!(
        "(:wat::core::let [{HANDBUILT_FIRED}] \
           (:wat::core::length (:wat::rete::query fired :weather::WindSpeed)))"));
    assert_eq!(got, Value::i64(0), "no WindSpeed was derived → empty query; got {got:?}");
}

// ── defrule ───────────────────────────────────────────────────────────────────

#[test]
fn defrule_produces_a_rule_value() {
    // Calling the generated zero-arg fn yields a Rule with the expected name + lhs/rhs arity.
    let name = ev(WORLD_WITH_RULE_PATH,
        "(:wat::rete::Rule/name (:weather::cold-and-windy))");
    assert_eq!(name, Value::String(Arc::new("weather::cold-and-windy".to_string())),
        "defrule sets Rule.name to the fqdn without colon");
    let lhs = ev(WORLD_WITH_RULE_PATH,
        "(:wat::core::length (:wat::rete::Rule/lhs (:weather::cold-and-windy)))");
    assert_eq!(lhs, Value::i64(2), "two conditions in :when → lhs length 2");
    let rhs = ev(WORLD_WITH_RULE_PATH,
        "(:wat::core::length (:wat::rete::Rule/rhs (:weather::cold-and-windy)))");
    assert_eq!(rhs, Value::i64(1), "one insert in :then → rhs length 1");
}

#[test]
fn defrule_rule_fires_end_to_end() {
    // Collect the one rule MANUALLY (call its fn), compile, insert, fire, query → one ColdAndWindy.
    let got = ev(WORLD_WITH_RULE_PATH,
        "(:wat::core::let [\
           rules (:wat::core::PersistentVector (:weather::cold-and-windy))\
           sess0 (:wat::rete::compile rules)\
           s1    (:wat::rete::insert sess0 (:weather::Temperature 15 \"Oslo\"))\
           s2    (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))\
           fired (:wat::rete::fire-rules s2)]\
           (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy)))");
    assert_eq!(got, Value::i64(1), "a defrule'd rule compiles + fires + derives end to end; got {got:?}");
}

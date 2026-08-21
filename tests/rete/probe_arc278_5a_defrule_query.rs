//! Arc 278 stone 5a — `defrule` (rule macro) + `query` (harvest a Query).
//!
//! `query` harvests a Query out of a fired session; `defrule` expands the readable rule form
//! into a zero-arg `defn` returning a `Rule`. The reflection that auto-gathers rules (`collect-rules`)
//! is 5b — here the one rule is collected manually by calling its fn. Live mouths: `defrule`,
//! `query`, `compile-all`, `insert`, `fire-rules`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_5a_defrule_query -- --include-ignored

use std::sync::Arc;
use wat::freeze::{startup_from_file, FrozenWorld};
use wat::runtime::{apply_function, Value};

// Paths to the co-located .wat fixtures (relative to the crate root).
const WORLD_PLAIN_PATH: &str = "tests/rete/probe_arc278_5a_defrule_query_plain.wat";
const WORLD_WITH_RULE_PATH: &str = "tests/rete/probe_arc278_5a_defrule_query_with_rule.wat";

fn world(path: &str) -> FrozenWorld {
    startup_from_file(path).expect("startup")
}

fn call(w: &FrozenWorld, fn_name: &str) -> Value {
    let func = w.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    apply_function(func, vec![], w.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("eval `{fn_name}` raised: {e:?}"))
}

// ── query ───────────────────────────────────────────────────────────────────

#[test]
fn query_reads_derived_coldandwindy() {
    let got = call(&world(WORLD_PLAIN_PATH), ":user::query-coldandwindy-count");
    assert_eq!(got, Value::i64(1), "query harvests q-ColdAndWindy; one derived fact; got {got:?}");
}

#[test]
fn query_reads_inserted_windspeed() {
    let got = call(&world(WORLD_PLAIN_PATH), ":user::query-windspeed-count");
    assert_eq!(got, Value::i64(1), "query harvests q-WindSpeed; one inserted fact; got {got:?}");
}

// ── defrule ───────────────────────────────────────────────────────────────────

#[test]
fn defrule_produces_a_rule_value() {
    // Calling the generated zero-arg fn yields a Rule with the expected name + lhs/rhs arity.
    let w = world(WORLD_WITH_RULE_PATH);
    let name = call(&w, ":user::rule-name");
    assert_eq!(name, Value::String(Arc::new("weather::cold-and-windy".to_string())),
        "defrule sets Rule.name to the fqdn without colon");
    let lhs = call(&w, ":user::rule-lhs-length");
    assert_eq!(lhs, Value::i64(2), "two conditions in :when → lhs length 2");
    let rhs = call(&w, ":user::rule-rhs-length");
    assert_eq!(rhs, Value::i64(1), "one insert in :then → rhs length 1");
}

#[test]
fn defrule_rule_fires_end_to_end() {
    // Collect the one rule MANUALLY (call its fn), compile, insert, fire, query → one ColdAndWindy.
    let got = call(&world(WORLD_WITH_RULE_PATH), ":user::defrule-fires-end-to-end");
    assert_eq!(got, Value::i64(1), "a defrule'd rule compiles + fires + derives end to end; got {got:?}");
}

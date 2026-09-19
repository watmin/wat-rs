//! Drive native `rule_produces` against the oracle's `rule-produces` on a user-fn
//! `:then` head. Arm 1 of strike-produced-type-userfn-head.
//!
//! Native `produced_type` resolves the head through the SymbolTable and returns the
//! fn's declared return type when that is a non-`wat::core::` Path. The oracle takes
//! the first child and strips a colon — no resolution. The scratch `.wat` already
//! drove the oracle: ANCHOR `["pt::Rate"]`, MEASURE `["pt::first-rate"]`.
//!
//! Both halves run in this process. The scratch is `include_str!`'d so the two
//! cannot drift. Do not hardcode the oracle's answer.
//!
//! Views / RHS walks are the same extractors `fire/rules.rs` uses. No parallel
//! view-builder.

use super::*;

use crate::rete::kernel::stratify::rule_produces;

/// Same source as `wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat`.
const SRC: &str = include_str!(
    "../../../../wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat"
);

fn world() -> crate::freeze::FrozenWorld {
    startup_from_source(SRC, None, Arc::new(InMemoryLoader::new()))
        .expect("produced-type userfn world should freeze")
}

fn eval_form(world: &crate::freeze::FrozenWorld, src: &str) -> Value {
    let ast = crate::parse_one!(src).expect("parse form");
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval of {src} raised: {e:?}"))
        .value_owned()
}

fn pvec_strings(v: &Value, what: &str) -> Vec<String> {
    match v {
        Value::wat__core__PersistentVector(pv) => pv
            .iter()
            .map(|el| match el {
                Value::String(s) => (**s).clone(),
                other => panic!("{what}: element is not String: {other:?}"),
            })
            .collect(),
        other => panic!("{what}: expected PersistentVector of String, got {other:?}"),
    }
}

fn native_produced(world: &crate::freeze::FrozenWorld, rule_form: &str) -> Vec<String> {
    let rule = eval_form(world, rule_form);
    let rhs = rule_asts_field(&rule, "rhs");
    rule_produces(&rhs, world.symbols())
}

#[test]
fn native_rule_produces_diverges_on_a_userfn_then_head() {
    let world = world();

    let native_anchor = native_produced(&world, "(:pt::plain)");
    let native_measure = native_produced(&world, "(:pt::via-userfn)");
    let oracle_anchor = pvec_strings(
        &eval_form(&world, "(:wat::rete::rule-produces (:pt::plain))"),
        "oracle ANCHOR",
    );
    let oracle_measure = pvec_strings(
        &eval_form(
            &world,
            "(:wat::rete::rule-produces (:pt::via-userfn))",
        ),
        "oracle MEASURE",
    );

    println!(
        "NATIVE ANCHOR: {native_anchor:?}\n\
         ORACLE ANCHOR: {oracle_anchor:?}\n\
         NATIVE MEASURE: {native_measure:?}\n\
         ORACLE MEASURE: {oracle_measure:?}"
    );

    // ANCHOR — ordinary fact-type head. If native is wrong here the extractor is
    // broken, not divergent, and the MEASURE row cannot be read.
    assert_eq!(
        (&native_anchor[..], &oracle_anchor[..]),
        (&["pt::Rate".to_string()][..], &["pt::Rate".to_string()][..]),
        "ANCHOR: both engines must name pt::Rate for a fact-type :then head\n  \
         native ANCHOR = {native_anchor:?}\n  oracle ANCHOR = {oracle_anchor:?}"
    );

    // DIVERGENCE — both sides measured in this process. Native resolves the fn
    // to its return type; the oracle names the fn. If they agree, STOP-1: the
    // reading at produced_type is refuted.
    assert_eq!(
        (&native_measure[..], &oracle_measure[..]),
        (
            &["pt::Rate".to_string()][..],
            &["pt::first-rate".to_string()][..]
        ),
        "MEASURE: native pt::Rate, oracle pt::first-rate. If both Rate the \
         SymbolTable lookup did not fire. If both first-rate native does not \
         resolve. If the oracle half disagrees with the scratch .wat, STOP.\n  \
         native MEASURE = {native_measure:?}\n  oracle MEASURE = {oracle_measure:?}"
    );
}

/// Same source as `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat`.
const FACTS_SRC: &str = include_str!(
    "../../../../wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat"
);

fn facts_world() -> crate::freeze::FrozenWorld {
    startup_from_source(FACTS_SRC, None, Arc::new(InMemoryLoader::new()))
        .expect("produced-type userfn facts world should freeze")
}

fn pvec_i64(v: &Value, what: &str) -> Vec<i64> {
    match v {
        Value::wat__core__PersistentVector(pv) => pv
            .iter()
            .map(|el| match el {
                Value::i64(n) => *n,
                other => panic!("{what}: element is not i64: {other:?}"),
            })
            .collect(),
        other => panic!("{what}: expected PersistentVector of i64, got {other:?}"),
    }
}

/// Arm 2 — the names divergence DROPS a derived fact on the oracle.
/// Native [Bad Rate Out] = [0 1 1]; oracle = [0 1 0]. Clara 0.24.0 agrees
/// with native. Do not "fix" either stratifier in this file.
#[test]
fn userfn_then_head_drops_the_oracle_out() {
    let world = facts_world();
    let native = pvec_i64(
        &eval_form(&world, "(:user::native-facts)"),
        "native facts",
    );
    let oracle = pvec_i64(
        &eval_form(&world, "(:user::oracle-facts)"),
        "oracle facts",
    );
    println!("NATIVE facts [Bad Rate Out]: {native:?}\nORACLE facts [Bad Rate Out]: {oracle:?}");
    assert_eq!(
        native,
        vec![0, 1, 1],
        "native must derive Rate and Out from Src(1)"
    );
    assert_eq!(
        oracle,
        vec![0, 1, 0],
        "oracle derives Rate and DROPS Out — the consumer sat at stratum 0 \
         and was not re-fired. If both [0 1 1] the drop is gone. If both \
         [0 1 0] native now drops it too.\n  native = {native:?}\n  oracle = {oracle:?}"
    );
}

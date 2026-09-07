//! Conferre L2-3 — compare STRATUM NUMBERS, not derived facts.
//!
//! `stratify.rs` says it mirrors `stratify-sweep`. The native sweep has a term the
//! oracle does not: `exists_and_from_types` gets `+1` when the bagged type is
//! derived by this set. The oracle folds acc `:from` into `rule-consumes` and
//! `req-pos` is NOT +1. Every existing stratify differential (`probe_arc278_derived_exists_acc`,
//! `probe_arc278_7strat_native_differential`, grid `strat-neg`) compares query row counts, which
//! cannot see this.
//!
//! ⛔ WHY A GREEN OVER FACTS PROVES NOTHING HERE
//!
//! `probe_arc278_derived_exists_acc` is exactly this shape (`acc::count :from Ok`,
//! `Ok` derived) and is GREEN: native and oracle agree on FACTS. Agreement on
//! facts is reading 1 in the DESIGN — the engines can still assign different
//! strata. This file reads the maps.
//!
//! ⛔ AND A GREEN OVER TWO EMPTY MAPS PROVES NOTHING
//!
//! Both engines record only RAISED strata. If every type is stratum 0, both maps
//! are `{}` and the comparison is vacuous. The reach assertions here are two, in
//! ascending strength:
//!
//! 1. the native ANCHOR (negation over derived Ok) raises a stratum — the
//!    instrument discriminates;
//! 2. the MEASURE set is the bag set (`ok` + `tally`), not the anchor set, so a
//!    green on the bag map is not a copy of (1).
//!
//! Without (1) a green here would be the green-over-nothing this arc has found
//! five times. Both halves are driven in this test: native via `native_stratify`
//! on views from `fire/rules.rs`; oracle via `:wat::rete::stratify` in the same
//! frozen world. The scratch `.wat` is the same source, not a second measurement.
//!
//! Views are built the way `fire/rules.rs` builds them (the four extractors).
//! No parallel view-builder.

use super::*;

use crate::rete::kernel::stratify::{
    native_stratify, rule_bag_consumes, rule_consumes, rule_negates, rule_produces, StratifyView,
};
use std::collections::HashMap;

/// Same source as `wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat`.
const SRC: &str = include_str!(
    "../../../../wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat"
);

fn world() -> crate::freeze::FrozenWorld {
    startup_from_source(SRC, None, Arc::new(InMemoryLoader::new()))
        .expect("L2-3 stratify world should freeze")
}

fn eval_form(world: &crate::freeze::FrozenWorld, src: &str) -> Value {
    let ast = crate::parse_one!(src).expect("parse form");
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval of {src} raised: {e:?}"))
        .value_owned()
}

/// `:wat::rete::stratify` returns `(HashMap :- [String i64])`.
fn hashmap_i64(v: &Value, what: &str) -> HashMap<String, i64> {
    match v {
        Value::wat__std__HashMap(m) => m
            .iter()
            .map(|(k, val)| {
                let key = match k {
                    Value::String(s) => (**s).clone(),
                    other => panic!("{what}: oracle key is not String: {other:?}"),
                };
                let n = match val {
                    Value::i64(n) => *n,
                    other => panic!("{what}: oracle value is not i64: {other:?}"),
                };
                (key, n)
            })
            .collect(),
        other => panic!("{what}: `:wat::rete::stratify` returned {other:?}, not HashMap"),
    }
}

/// Byte-copy of `fire_rules_on_session`'s view construction (`fire/rules.rs`).
fn views_of(rules_value: &Value, sym: &crate::runtime::SymbolTable) -> Vec<StratifyView> {
    let pv = match rules_value {
        Value::wat__core__PersistentVector(pv) => pv,
        other => panic!("rules is a PersistentVector, got {other:?}"),
    };
    let mut out = Vec::with_capacity(pv.len());
    for r in pv.iter() {
        if rule_named_field(r, "name").is_none() {
            continue;
        }
        let lhs = rule_asts_field(r, "lhs");
        let rhs = rule_asts_field(r, "rhs");
        out.push(StratifyView {
            produced: rule_produces(&rhs, sym),
            negated: rule_negates(&lhs),
            consumed: rule_consumes(&lhs),
            exists_and_from_types: rule_bag_consumes(&lhs),
        });
    }
    out
}

fn fmt_map(m: &HashMap<String, i64>) -> String {
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    keys.iter()
        .map(|k| format!("\"{k}\" {}", m[*k]))
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn native_stratify_numbers_against_the_oracle_scratch() {
    let world = world();
    let sym = world.symbols();

    let anchor_rules = eval_form(
        &world,
        "(:wat::core::PersistentVector (:l23::ok) (:l23::neg))",
    );
    let bag_rules = eval_form(
        &world,
        "(:wat::core::PersistentVector (:l23::ok) (:l23::tally))",
    );

    let anchor_views = views_of(&anchor_rules, sym);
    let bag_views = views_of(&bag_rules, sym);

    let native_anchor = native_stratify(&anchor_views).expect("native stratify ANCHOR");
    let native_bag = native_stratify(&bag_views).expect("native stratify MEASURE");

    let oracle_anchor = hashmap_i64(
        &eval_form(
            &world,
            "(:wat::rete::stratify (:wat::core::PersistentVector (:l23::ok) (:l23::neg)))",
        ),
        "oracle ANCHOR",
    );
    let oracle_bag = hashmap_i64(
        &eval_form(
            &world,
            "(:wat::rete::stratify (:wat::core::PersistentVector (:l23::ok) (:l23::tally)))",
        ),
        "oracle MEASURE",
    );

    println!(
        "NATIVE ANCHOR keys (raw): [{}]\n\
         ORACLE ANCHOR keys (raw): [{}]\n\
         NATIVE MEASURE keys (raw): [{}]\n\
         ORACLE MEASURE keys (raw): [{}]\n\
         bag exists_and_from_types: {:?}",
        fmt_map(&native_anchor),
        fmt_map(&oracle_anchor),
        fmt_map(&native_bag),
        fmt_map(&oracle_bag),
        bag_views
            .iter()
            .map(|v| &v.exists_and_from_types)
            .collect::<Vec<_>>(),
    );

    // AGREEMENT — the term both engines share. Empty native here means the
    // instrument is inert; disagreement here means the two maps are not
    // comparable (STOP-2 spelling, or a different defect).
    assert_eq!(
        (native_anchor.get("l23::Ok2"), oracle_anchor.get("l23::Ok2")),
        (Some(&1), Some(&1)),
        "ANCHOR: both engines must raise Ok2 to 1 — the term they share\n  \
         native ANCHOR = {{{}}}\n  oracle ANCHOR = {{{}}}",
        fmt_map(&native_anchor),
        fmt_map(&oracle_anchor),
    );

    // DIVERGENCE — both sides measured in this process. Native Tally ⇒ 1;
    // oracle has no Tally (stratum 0, unrecorded). If oracle also raises
    // Tally, the scratch .wat and this in-test call disagree — STOP and
    // report; do not treat that as engine lockstep.
    assert_eq!(
        (native_bag.get("l23::Tally"), oracle_bag.get("l23::Tally")),
        (Some(&1), None),
        "MEASURE: native Tally ⇒ 1, oracle Tally absent. If both Some(1) the \
         in-test oracle disagrees with the scratch .wat. If both None the \
         native +1 is gone.\n  native MEASURE = {{{}}}\n  oracle MEASURE = {{{}}}",
        fmt_map(&native_bag),
        fmt_map(&oracle_bag),
    );
}

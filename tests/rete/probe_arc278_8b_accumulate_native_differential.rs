//! Arc 278 — Stone 8-b: the AccumulateNode in the NATIVE kernel + the DIFFERENTIAL (native==oracle).
//! RED at HEAD (8-a taught the ORACLE + compile the AccumulateNode, but the native delta engine
//! `fire_fixpoint_delta` has no accumulate-pass → the accumulate result-var is never bound → native
//! under-derives → native ≠ oracle). GREEN when 8-b lands. Contract: DESIGN-STONE-8-accumulators.md.
//!
//! `fire-rules` = native; `fire-rules-spec` = the wat oracle. For an `acc/` rule the two MUST agree.
//!
//! Run: cargo test --release -p wat --test probe_arc278_8b_accumulate_native_differential

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// rune:lint(no-inlined-wat) — world parameterized by runtime acc/gate strings — cartesian matrix of combinations cannot be pre-extracted
fn world(acc: &str, gate: &str) -> String {
    format!(
        "(:wat::core::defrecord :w::Station  [location <- :wat::core::String])\n\
         (:wat::core::defrecord :w::Reading  [location <- :wat::core::String  value <- :wat::core::i64])\n\
         (:wat::core::defrecord :w::Busy     [location <- :wat::core::String  n <- :wat::core::i64])\n\
         \n\
         (:wat::rete::defrule :w::busy\n\
           :when\n\
           [(:w::Station (?loc <- :location))\n\
            {acc}\n\
            (:wat::rete::where {gate})]\n\
           :then\n\
           [(:w::Busy :location ?loc :n ?n)])\n\
         \n\
         (:wat::rete::defquery :w::q-Busy\n\
           :params []\n\
           :when [(:w::Busy)])"
    )
}

fn busy_count(fire_fn: &str, acc: &str, gate: &str, readings: &[(&str, i64)]) -> Result<i64, String> {
    let reading_inserts: String = readings
        .iter()
        .map(|(loc, v)| format!("             session (:wat::rete::insert session (:w::Reading :location \"{loc}\" :value {v}))\n"))
        .collect();
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules :w)\n\
             session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Busy)))\n\
             session (:wat::rete::insert session (:w::Station :location \"Oslo\"))\n\
{reading_inserts}\
             fired   (:wat::rete::{fire_fn} session)]\n\
            (:wat::rete::query fired (:w::q-Busy))))"
    );
    let world_src = world(acc, gate);
    let w = startup_from_source(&world_src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    match eval_in_frozen(&ast, &w, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

const COUNT: &str = "(?n <- (:wat::rete::acc::count) :from (:w::Reading (?loc <- :location)))";
const SUM: &str = "(?n <- (:wat::rete::acc::sum ?v) :from (:w::Reading (?loc <- :location) (?v <- :value)))";

/// Assert native fire == oracle fire == `expect` for the given accumulate rule + facts.
fn diff(acc: &str, gate: &str, readings: &[(&str, i64)], expect: i64) {
    let native = busy_count("fire-rules", acc, gate, readings).expect("native");
    let oracle = busy_count("fire-rules-spec", acc, gate, readings).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (acc={acc} gate={gate}); native={native} oracle={oracle}");
    assert_eq!(native, expect, "value (native); got {native} want {expect}");
}

/// 1 — DIFFERENTIAL count exact: 3 Oslo Readings, gate `= 3` → both fire (1).
#[test]
fn differential_count_value() {
    diff(COUNT, "(:wat::rete::core::i64::= ?n 3)", &[("Oslo", 1), ("Oslo", 2), ("Oslo", 3)], 1);
}

/// 2 — DIFFERENTIAL the shared-var join: a Bergen Reading is not counted for Oslo (still 3 → fires).
#[test]
fn differential_join() {
    diff(COUNT, "(:wat::rete::core::i64::= ?n 3)", &[("Oslo", 1), ("Oslo", 2), ("Oslo", 3), ("Bergen", 9)], 1);
}

/// 3 — DIFFERENTIAL sum: 10+20+30 = 60, gate `= 60` → both fire.
#[test]
fn differential_sum() {
    diff(SUM, "(:wat::rete::core::i64::= ?n 60)", &[("Oslo", 10), ("Oslo", 20), ("Oslo", 30)], 1);
}

/// 4 — DIFFERENTIAL the minimum-finding-set composition: count >= 3 fires with 3, blocks with 2.
#[test]
fn differential_minimum_finding_set() {
    diff(COUNT, "(:wat::rete::core::i64::>= ?n 3)", &[("Oslo", 1), ("Oslo", 2), ("Oslo", 3)], 1);
    diff(COUNT, "(:wat::rete::core::i64::>= ?n 3)", &[("Oslo", 1), ("Oslo", 2)], 0);
}

/// 5 — DIFFERENTIAL count-on-empty: no readings → count 0, gate `= 0` → both fire (count emits on empty).
#[test]
fn differential_count_empty() {
    diff(COUNT, "(:wat::rete::core::i64::= ?n 0)", &[], 1);
}

//! Arc 278 stone 8-a — the AccumulateNode in the oracle (`fire-rules$oracle`).
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! An accumulate condition gathers the token-compatible `:from` facts (shared `?loc`), folds them (8-i),
//! binds the result `?var`, extends the token. The exact bound value is checked via a `(where (= ?n N))`
//! gate — count the survivors. Composition: the "minimum finding set to activate" =
//! `(acc/count) :from …` + `(where (>= ?n N))`. Live mouths: `collect-rules`, `compile-all`, `insert`,
//! `fire-rules$oracle`, `query`, `acc::count`, `acc::sum`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_8a_accumulate_oracle

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// A world whose rule counts the Readings at a Station's location and gates on `(where <gate>)`.
/// `acc` is the accumulate condition; `gate` the where-expr over `?n`.
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

/// Insert a Station(Oslo) + the given Readings, fire the oracle, count derived Busy facts.
fn busy_count(acc: &str, gate: &str, readings: &[(&str, i64)]) -> Result<i64, String> {
    let reading_inserts: String = readings
        .iter()
        .map(|(loc, v)| format!("             session (:wat::core::match (:wat::rete::insert session (:w::Reading :location \"{loc}\" :value {v})) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n"))
        .collect();
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules :w)\n\
             session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Busy)))\n\
             session (:wat::core::match (:wat::rete::insert session (:w::Station \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n\
{reading_inserts}\
             fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))]\n\
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

/// 1 — count binds the EXACT aggregate: 3 Oslo Readings → ?n = 3 (gate `= 3` fires; `= 5` does not).
#[test]
fn count_binds_exact_value() {
    let three = &[("Oslo", 10), ("Oslo", 20), ("Oslo", 30)];
    assert!(matches!(busy_count(COUNT, "(:wat::rete::core::i64::= ?n 3)", three), Ok(1)), "count = 3 → fires");
    assert!(matches!(busy_count(COUNT, "(:wat::rete::core::i64::= ?n 5)", three), Ok(0)), "count ≠ 5 → no fire");
}

/// 2 — the JOIN: a Reading at a DIFFERENT location is NOT counted (shared ?loc). Oslo count stays 3.
#[test]
fn accumulate_joins_on_shared_var() {
    let mixed = &[("Oslo", 10), ("Oslo", 20), ("Oslo", 30), ("Bergen", 99)];
    assert!(matches!(busy_count(COUNT, "(:wat::rete::core::i64::= ?n 3)", mixed), Ok(1)), "Bergen not counted → still 3");
}

/// 3 — sum folds the bound ?v: values 10+20+30 = 60 (gate `= 60` fires).
#[test]
fn sum_folds_the_value() {
    let three = &[("Oslo", 10), ("Oslo", 20), ("Oslo", 30)];
    assert!(matches!(busy_count(SUM, "(:wat::rete::core::i64::= ?n 60)", three), Ok(1)), "sum = 60 → fires");
}

/// 4 — the "minimum finding set to activate" (composition: acc + where ≥ N).
#[test]
fn minimum_finding_set_composition() {
    let three = &[("Oslo", 1), ("Oslo", 2), ("Oslo", 3)];
    assert!(matches!(busy_count(COUNT, "(:wat::rete::core::i64::>= ?n 3)", three), Ok(1)), "3 ≥ 3 → fires");
    assert!(matches!(busy_count(COUNT, "(:wat::rete::core::i64::>= ?n 4)", three), Ok(0)), "3 ≥ 4 → no fire");
}

/// 5 — EMPTY: a Station with zero Readings → count = 0 (count emits on empty; gate `= 0` fires).
#[test]
fn count_emits_on_empty() {
    assert!(matches!(busy_count(COUNT, "(:wat::rete::core::i64::= ?n 0)", &[]), Ok(1)), "no readings → count 0 → fires");
}

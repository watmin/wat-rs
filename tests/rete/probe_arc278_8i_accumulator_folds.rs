//! Arc 278 — Stone 8-i: the wat accumulator fold library (`:wat::rete::acc::*`), standalone.
//! RED at HEAD (the acc fold fns don't exist). GREEN when 8-i lands. Contract: DESIGN-STONE-8-accumulators.md.
//!
//! The accumulators are PURE WAT FOLDS over a `PV<Element>` (an Element = `(:wat::rete::Element fact bindings)`);
//! value-folds read a BOUND `?var` (a string key) from each element's bindings map. `mean` = `sum / count`
//! (composition). Each fold returns `Option<Value>`: `None` = no token on empty (min/max/mean of nothing);
//! `Some(v)` otherwise (count/sum emit `Some(0)` on empty; all/distinct → empty; group-by → empty map).
//!
//! Run: cargo test --release -p wat --test probe_arc278_8i_accumulator_folds

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::core::defrecord :net::Packet [src <- :wat::core::String])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// Wrap `body` in a let binding `els` = a PV of 3 Elements with bindings {?bytes, ?port} + Packet facts,
/// and `empty` = an empty PV. ?bytes = 100/200/300 (sum 600, min 100, max 300, mean 200);
/// ?port = 80/443/80 (distinct → 2; group-by → 2 keys).
fn run(body: &str) -> Result<Value, String> {
    let mk = |bytes: i64, port: i64, src: &str| {
        format!(
            "(:wat::rete::Element (:net::Packet \"{src}\") \
             (:wat::core::PersistentMap/assoc \
               (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) \"?bytes\" {bytes}) \
               \"?port\" {port}))"
        )
    };
    let e1 = mk(100, 80, "a");
    let e2 = mk(200, 443, "b");
    let e3 = mk(300, 80, "c");
    let compute = format!(
        "(:wat::core::let\n\
          [els (:wat::core::PersistentVector/conj\n\
                 (:wat::core::PersistentVector/conj\n\
                   (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) {e1})\n\
                   {e2})\n\
                 {e3})\n\
           empty (:wat::core::PersistentVector)]\n\
          {body})"
    );
    let world = startup_from_source(WORLD, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&compute).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|t| t.value_owned())
}

/// Some(i64) helper.
fn is_some_i64(v: &Value, n: i64) -> bool {
    matches!(v, Value::Option(o) if matches!(&**o, Some(Value::i64(m)) if *m == n))
}

/// count → BARE 3 (length is always concrete; never Option).
#[test]
fn count_folds() {
    assert!(matches!(run("(:wat::rete::acc::count els)").unwrap(), Value::i64(3)), "count = 3 (bare)");
}

/// sum ?bytes → BARE 600 (empty sum = 0; never Option).
#[test]
fn sum_folds() {
    assert!(matches!(run("(:wat::rete::acc::sum \"?bytes\" els)").unwrap(), Value::i64(600)), "sum = 600 (bare)");
}

/// min ?bytes → Some(100).
#[test]
fn min_folds() {
    assert!(is_some_i64(&run("(:wat::rete::acc::min \"?bytes\" els)").unwrap(), 100), "min = 100");
}

/// max ?bytes → Some(300).
#[test]
fn max_folds() {
    assert!(is_some_i64(&run("(:wat::rete::acc::max \"?bytes\" els)").unwrap(), 300), "max = 300");
}

/// mean ?bytes → Some(200) — THE composition: sum(600)/count(3).
#[test]
fn mean_is_sum_over_count() {
    assert!(is_some_i64(&run("(:wat::rete::acc::mean \"?bytes\" els)").unwrap(), 200), "mean = 600/3 = 200");
}

/// distinct ?port → BARE vec of length 2 (80, 443 — the duplicate 80 collapses).
#[test]
fn distinct_folds() {
    let v = run("(:wat::core::length (:wat::rete::acc::distinct \"?port\" els))").unwrap();
    assert!(matches!(v, Value::i64(2)), "distinct ports = 2; got {v:?}");
}

/// all → BARE vec of length 3 (the gathered facts).
#[test]
fn all_folds() {
    let v = run("(:wat::core::length (:wat::rete::acc::all els))").unwrap();
    assert!(matches!(v, Value::i64(3)), "all facts = 3; got {v:?}");
}

/// group-by ?port → BARE map with 2 keys (80 → [a,c], 443 → [b]).
#[test]
fn group_by_folds() {
    let v = run("(:wat::core::PersistentMap/length (:wat::rete::acc::group-by \"?port\" els))").unwrap();
    assert!(matches!(v, Value::i64(2)), "group-by → 2 keys; got {v:?}");
}

/// EMPTY: count over an empty set → BARE 0 (count always concrete — never None).
#[test]
fn count_empty_is_zero() {
    assert!(matches!(run("(:wat::rete::acc::count empty)").unwrap(), Value::i64(0)), "count [] = 0 (bare)");
}

/// EMPTY: min over an empty set → None (no token — there is no minimum of nothing).
#[test]
fn min_empty_is_none() {
    let v = run("(:wat::rete::acc::min \"?bytes\" empty)").unwrap();
    assert!(matches!(&v, Value::Option(o) if o.is_none()), "min [] = None; got {v:?}");
}

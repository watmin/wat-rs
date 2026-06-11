//! Forward-proof probe — the instinct-faithful ordering surface (intueri-ratified).
//!
//! wat today has `sort-by` taking a COMPARATOR (Clojure's `sort` mis-named) + `reverse`. An LLM,
//! untaught, reaches for `(sort xs)`, `(sort < xs)`, `(sort-by keyfn xs)` — and stumbles. This is
//! the corrected surface (Clojure-exact, fn-first):
//!   (sort coll)             — natural ascending
//!   (sort cmp coll)         — comparator cmp : (T,T)->bool   (Clojure's `(sort < xs)`)
//!   (sort-by keyfn coll)    — by key, natural                (keyfn : (T)->K)
//!   (sort-by keyfn cmp coll)— by key + comparator on keys
//!   (reverse coll)          — unchanged
//!
//! RED at HEAD: `sort` is UnknownFunction; `sort-by` rejects a key-fn (it wants a comparator).
//!
//! Run: `cargo test --release --test probe_arc251_ordering_surface`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Eval a `Vector<i64>`-returning body → Vec<i64> (or an error string).
fn eval_vec(body: &str) -> Result<Vec<i64>, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::i64> {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::Vec(v) => v.iter().map(|x| match x {
            Value::i64(n) => Ok(*n),
            other => Err(format!("non-i64 elem: {other:?}")),
        }).collect(),
        other => Err(format!("non-vec: {other:?}")),
    }
}

const V321: &str = "(:wat::core::Vector :wat::core::i64 3 1 2)";
const V123: &str = "(:wat::core::Vector :wat::core::i64 1 2 3)";

#[test]
fn c01_sort_natural() {
    assert_eq!(eval_vec(&format!("(:wat::core::sort {V321})")), Ok(vec![1, 2, 3]),
        "(sort xs) — natural ascending");
}

#[test]
fn c02_sort_comparator() {
    // (sort > xs) descending — Clojure's `(sort > xs)` idiom (boolean comparator, fn-first).
    assert_eq!(eval_vec(&format!(
        "(:wat::core::sort (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool (:wat::core::> a b)) {V123})")),
        Ok(vec![3, 2, 1]), "(sort cmp xs) — comparator descending");
}

#[test]
fn c03_sort_by_key() {
    // (sort-by keyfn xs) — key = negate; natural ascending ON THE KEYS => descending originals.
    assert_eq!(eval_vec(&format!(
        "(:wat::core::sort-by (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::- 0 x)) {V123})")),
        Ok(vec![3, 2, 1]), "(sort-by keyfn xs) — by key, natural");
}

#[test]
fn c04_sort_by_key_and_comparator() {
    // (sort-by keyfn cmp xs) — key = identity, cmp = > => descending.
    assert_eq!(eval_vec(&format!(
        "(:wat::core::sort-by \
           (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) \
           (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool (:wat::core::> a b)) \
           {V123})")),
        Ok(vec![3, 2, 1]), "(sort-by keyfn cmp xs)");
}

#[test]
fn c05_reverse_preserved() {
    assert_eq!(eval_vec(&format!("(:wat::core::reverse {V123})")), Ok(vec![3, 2, 1]),
        "reverse unchanged");
}

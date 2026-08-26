//! Arc 278 stone 8-custom — custom accumulators (any fenced user fold over the gather) + the differential.
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! The accumulator slot accepts a USER fn head (not just the 8 built-ins): `(?r <- (:my-fold ?v) :from (…))`
//! gathers the `?v` values into a `PV<T>` and applies `my-fold : (PV<T>) -> R`. Known head → built-in
//! fast-path; else eval the user fn over the gather. The compile fence rejects a fold that is not
//! (pure ∧ deterministic ∧ total ∧ primitive?). Native `fire-rules` == `fire-rules$oracle`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_8custom_native_differential

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::loader::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// World with a PURE custom fold (`sum-of-squares`) + a rule using it as an accumulator, gated by `gate`.
// rune:lint(no-inlined-wat) — world parameterized by runtime gate expression — many gate variants tested inline; also has a literal impure-src in fence_rejects_impure_fold
fn world(gate: &str) -> String {
    format!(
        "(:wat::core::defrecord :w::Station [location <- :wat::core::String])\n\
         (:wat::core::defrecord :w::Reading [location <- :wat::core::String  value <- :wat::core::i64])\n\
         (:wat::core::defrecord :w::Flagged [location <- :wat::core::String])\n\
         \n\
         ;; a PURE∧DET custom fold: sum of squares of the gathered values\n\
         (:wat::rete::core::defn :w::sum-of-squares [xs <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::i64\n\
           (:wat::rete::core::foldl\n\
             (:wat::rete::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64\n\
               (:wat::rete::i64::+ acc (:wat::rete::i64::* x x :undefined 0) :undefined 0))\n\
             0 xs))\n\
         \n\
         (:wat::rete::defrule :w::flag\n\
           :when\n\
           [(:w::Station (?loc <- :location))\n\
            (?s <- (:w::sum-of-squares ?v) :from (:w::Reading (?loc <- :location) (?v <- :value)))\n\
            (:wat::rete::where {gate})]\n\
           :then\n\
           [(:w::Flagged :location ?loc)])\n\
         \n\
         (:wat::rete::defquery :w::q-Flagged\n\
           :params []\n\
           :when [(:w::Flagged)])"
    )
}

fn flagged_count(fire_fn: &str, gate: &str, readings: &[i64]) -> Result<i64, String> {
    let reading_inserts: String = readings
        .iter()
        .map(|v| format!("             session (:wat::rete::insert session (:w::Reading :location \"Oslo\" :value {v}))\n"))
        .collect();
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules :w)\n\
             session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Flagged)))\n\
             session (:wat::rete::insert session (:w::Station :location \"Oslo\"))\n\
{reading_inserts}\
             fired   (:wat::rete::{fire_fn} session)]\n\
            (:wat::rete::query fired (:w::q-Flagged))))"
    );
    let w = startup_from_source(&world(gate), Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    match eval_in_frozen(&ast, &w, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

/// native == oracle == expect, for the given gate + readings.
fn diff(gate: &str, readings: &[i64], expect: i64) {
    let native = flagged_count("fire-rules", gate, readings).expect("native");
    let oracle = flagged_count("fire-rules$oracle", gate, readings).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (gate={gate}); native={native} oracle={oracle}");
    assert_eq!(native, expect, "value (native); got {native} want {expect}");
}

/// 1 — DIFFERENTIAL: sum-of-squares([1,2,3]) = 14; gate `= 14` → both fire (1).
#[test]
fn differential_custom_fold() {
    diff("(:wat::rete::i64::= ?s 14)", &[1, 2, 3], 1);
}

/// 2 — DIFFERENTIAL: the fold's value is EXACTLY 14, not something else; gate `= 99` → both 0.
#[test]
fn differential_custom_fold_value() {
    diff("(:wat::rete::i64::= ?s 99)", &[1, 2, 3], 0);
}

/// 3 — DIFFERENTIAL empty: sum-of-squares([]) = 0 (the fn handles empty); gate `= 0` → both fire (1).
///     v1 contract: a custom fold is `(PV<T>) -> R` and always emits (the fn handles the empty gather).
#[test]
fn differential_custom_empty() {
    diff("(:wat::rete::i64::= ?s 0)", &[], 1);
}

/// 4 — the compile FENCE rejects an IMPURE custom fold (calls println). The rule must fail to compile.
#[test]
fn fence_rejects_impure_fold() {
    let src = "(:wat::core::defrecord :w::Reading [location <- :wat::core::String  value <- :wat::core::i64])\n\
         (:wat::core::defrecord :w::Flagged [location <- :wat::core::String])\n\
         ;; an IMPURE fold — side-effects (println) → must be rejected by the pure∧det fence\n\
         (:wat::core::defn :w::bad-fold [xs <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::i64\n\
           (:wat::core::do\n\
             (:wat::kernel::println \"side effect\")\n\
             (:wat::core::length xs)))\n\
         (:wat::rete::defrule :w::bad\n\
           :when\n\
           [(:w::Reading (?loc <- :location) (?v <- :value))\n\
            (?s <- (:w::bad-fold ?v) :from (:w::Reading (?loc2 <- :location) (?v2 <- :value)))]\n\
           :then\n\
           [(:w::Flagged :location ?loc)])\n\
         ";
    let w = startup_from_source(src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("world should freeze (the impure fold is defined; the rule using it is the violation)");
    // Compiling the rule must be REJECTED (the accumulate-branch fences the user fold pure∧det).
    // The fence rejects by PANICKING (Option/expect → panic_any, same as raise!); catch it.
    // (Before the arc-296 None-fix an illegal `(:wat::core::None)` form threw a *catchable* error
    // here — that form was never legal and is now corrected; the fence's real reject is a panic.)
    let run = "(:wat::core::let [rules (:wat::rete::collect-rules :w)] (:wat::rete::compile rules))";
    let ast = wat::parse_one!(run).expect("parse");
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_in_frozen(&ast, &w, &Environment::new())
    }));
    let rejected = caught.map_or(true, |res| res.is_err());
    assert!(rejected, "impure custom fold must be rejected at compile; got Ok");
}

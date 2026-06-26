//! Arc 278 — Stone 6b-ii-a: `where`/TestNode in the ORACLE (`rete.wat` compile + fire) + the compile fence.
//! RED at HEAD (no TestNode: compile-condition treats `(:wat::rete::where …)` as an unsatisfiable alpha →
//! the rule never fires; and there is no pure∧det fence). GREEN when 6b-ii-a lands.
//! Contract: DESIGN-STONE-6b-where-test.md.
//!
//! Probed through the ORACLE (`fire-rules-spec`) — 6b-ii-a builds the oracle TestNode; the native kernel
//! port + differential are 6b-ii-b. A `where` is a left-only filter: it keeps a token iff `eval-test`
//! (6b-i) of its expr against the token's bindings is true. The compile fence rejects a `where` whose expr
//! is not (pure ∧ deterministic).
//!
//! Run: cargo test --release -p wat --test probe_arc278_6b_ii_a_where_oracle

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// A world whose single rule (namespace `:wg`) filters Temperature by `(where (> ?c 0))`.
const WORLD_CMP: &str = "\
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :wg::Gate            [celsius <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :wg::cold-gate\n\
  :when\n\
  [(:weather::Temperature (?c <- :celsius))\n\
   (:wat::rete::where (:wat::core::> ?c 0))]\n\
  :then\n\
  (:wat::rete::insert (:wg::Gate ?c)))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// A world whose rule filters by a USER-fn predicate `(where (:test::big? ?c))`.
const WORLD_USERFN: &str = "\
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :wb::Gate            [celsius <- :wat::core::i64])\n\
\n\
(:wat::core::defn :test::big? [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 100))\n\
\n\
(:wat::rete::defrule :wb::big-gate\n\
  :when\n\
  [(:weather::Temperature (?c <- :celsius))\n\
   (:wat::rete::where (:test::big? ?c))]\n\
  :then\n\
  (:wat::rete::insert (:wb::Gate ?c)))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// A world whose rule's `where` is IMPURE (io) — the fence must reject it at compile.
const WORLD_IMPURE: &str = "\
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :wf::bad-gate\n\
  :when\n\
  [(:weather::Temperature (?c <- :celsius))\n\
   (:wat::rete::where (:wat::core::record? (:wat::io::IOReader/open-file \"x\")))]\n\
  :then\n\
  (:wat::rete::insert (:wf::Gate ?c)))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// Count derived facts of `gate_type` after firing the oracle on a single inserted Temperature.
fn run_count(world_src: &str, ns: &str, gate_type: &str, celsius: i64) -> Result<Value, String> {
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules {ns})\n\
             session (:wat::rete::compile rules)\n\
             session (:wat::rete::insert session (:weather::Temperature {celsius} \"Oslo\"))\n\
             fired   (:wat::rete::fire-rules-spec session)]\n\
            (:wat::rete::query fired {gate_type})))"
    );
    let world = startup_from_source(world_src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|t| t.value_owned())
}

/// 1 — the where PASSES: Temp(5), (> 5 0) true → exactly one Gate derived.
#[test]
fn where_passes_when_predicate_true() {
    let r = run_count(WORLD_CMP, ":wg", ":wg::Gate", 5);
    assert!(matches!(r, Ok(Value::i64(1))), "where (> 5 0) true → 1 Gate; got {r:?}");
}

/// 2 — the where BLOCKS: Temp(-5), (> -5 0) false → zero Gates (the filter actually filters).
#[test]
fn where_blocks_when_predicate_false() {
    let r = run_count(WORLD_CMP, ":wg", ":wg::Gate", -5);
    assert!(matches!(r, Ok(Value::i64(0))), "where (> -5 0) false → 0 Gates; got {r:?}");
}

/// 3 — a USER-fn predicate in the where works through the network: big?(150) → one Gate.
#[test]
fn where_with_user_fn_predicate_passes() {
    let r = run_count(WORLD_USERFN, ":wb", ":wb::Gate", 150);
    assert!(matches!(r, Ok(Value::i64(1))), "where (big? 150) true → 1 Gate; got {r:?}");
}

/// 3b — the same user-fn predicate blocks below threshold: big?(50) → zero.
#[test]
fn where_with_user_fn_predicate_blocks() {
    let r = run_count(WORLD_USERFN, ":wb", ":wb::Gate", 50);
    assert!(matches!(r, Ok(Value::i64(0))), "where (big? 50) false → 0 Gates; got {r:?}");
}

/// 4 — the compile FENCE rejects an impure `where` (io): compiling the rule raises. (At HEAD there is no
/// fence → compile succeeds → this fails RED.)
#[test]
fn fence_rejects_impure_where_at_compile() {
    let r = run_count(WORLD_IMPURE, ":wf", ":wf::Gate", 5);
    assert!(r.is_err(), "an impure (io) where must fail to compile; got {r:?}");
}

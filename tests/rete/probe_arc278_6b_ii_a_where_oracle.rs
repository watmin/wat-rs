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

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

// Paths to the co-located .wat fixtures (relative to the crate root).
const WORLD_CMP_PATH: &str    = "tests/rete/probe_arc278_6b_ii_a_where_oracle_cmp.wat";
const WORLD_USERFN_PATH: &str = "tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat";
const WORLD_IMPURE_PATH: &str = "tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat";

/// Count derived facts of `gate_type` after firing the oracle on a single inserted Temperature.
fn run_count(world_path: &str, ns: &str, gate_type: &str, celsius: i64) -> Result<Value, String> {
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules {ns})\n\
             session (:wat::rete::compile rules)\n\
             session (:wat::rete::insert session (:weather::Temperature {celsius} \"Oslo\"))\n\
             fired   (:wat::rete::fire-rules-spec session)]\n\
            (:wat::rete::query fired {gate_type})))"
    );
    let world = startup_from_file(world_path)
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    // The rete compile fence rejects an impure/non-deterministic condition by PANICKING
    // (Option/expect → panic_any — the engine's compile-rejection mechanism, same as raise!).
    // Catch it so a rejection surfaces as Err, not an uncaught test panic. (Before the arc-296
    // None-fix an illegal `(:wat::core::None)` form threw a *catchable* UnknownFunction here
    // instead — that form was never legal and is now corrected; the fence's real reject is a panic.)
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
    })) {
        Err(_) => Err("compile/eval rejected (fence panic)".to_string()),
        Ok(res) => res.map_err(|e| format!("eval: {e:?}")).map(|t| t.value_owned()),
    }
}

/// 1 — the where PASSES: Temp(5), (> 5 0) true → exactly one Gate derived.
#[test]
fn where_passes_when_predicate_true() {
    let r = run_count(WORLD_CMP_PATH, ":wg", ":wg::Gate", 5);
    assert!(matches!(r, Ok(Value::i64(1))), "where (> 5 0) true → 1 Gate; got {r:?}");
}

/// 2 — the where BLOCKS: Temp(-5), (> -5 0) false → zero Gates (the filter actually filters).
#[test]
fn where_blocks_when_predicate_false() {
    let r = run_count(WORLD_CMP_PATH, ":wg", ":wg::Gate", -5);
    assert!(matches!(r, Ok(Value::i64(0))), "where (> -5 0) false → 0 Gates; got {r:?}");
}

/// 3 — a USER-fn predicate in the where works through the network: big?(150) → one Gate.
#[test]
fn where_with_user_fn_predicate_passes() {
    let r = run_count(WORLD_USERFN_PATH, ":wb", ":wb::Gate", 150);
    assert!(matches!(r, Ok(Value::i64(1))), "where (big? 150) true → 1 Gate; got {r:?}");
}

/// 3b — the same user-fn predicate blocks below threshold: big?(50) → zero.
#[test]
fn where_with_user_fn_predicate_blocks() {
    let r = run_count(WORLD_USERFN_PATH, ":wb", ":wb::Gate", 50);
    assert!(matches!(r, Ok(Value::i64(0))), "where (big? 50) false → 0 Gates; got {r:?}");
}

/// 4 — the compile FENCE rejects an impure `where` (io): compiling the rule raises. (At HEAD there is no
/// fence → compile succeeds → this fails RED.)
#[test]
fn fence_rejects_impure_where_at_compile() {
    let r = run_count(WORLD_IMPURE_PATH, ":wf", ":wf::Gate", 5);
    assert!(r.is_err(), "an impure (io) where must fail to compile; got {r:?}");
}

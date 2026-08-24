//! Arc 278 — THE NORTH STAR: the cold-and-windy rule, end to end. The engine's acceptance test AND the
//! executable spec of the rete DSL surface.
//!
//! Live mouths: `defrule`, `defquery`, `collect-rules`, `compile-all`, `insert`, `fire-rules`, `query`.
//! Temp 15<20 AND Wind 45>30 at the same location "Oslo" (equality join on `?loc`) derives exactly one
//! ColdAndWindy. The living contract for the DSL — if a surface detail is refined, update this test
//! (curare), never let it drift.
//!
//! The DSL it pins:
//!   - facts are plain typed records.
//!   - (:wat::rete::defrule :ns::name :when [conditions] :then <pure-rhs>) — namespaced rule macro.
//!   - condition = (:FactType <clause>...):
//!     (?var <- :field)            bind/join  (fresh binds; bound ?var ⇒ cross-fact equality join on the field)
//!     (:wat::rete::core::<ty>::<op> a b)  per-type constraint (operands ∈ {?var, :field, literal})
//!   - :then = a vector of bare fact-forms, nothing else. Each `(:Type …)` declares a logical derived fact
//!     (support = the firing token; auto-retracted if support vanishes); fact args may be pure exprs over the
//!     bound ?vars. The engine COLLECTS those forms at fire — pure: no IO, no retract, no insert-unconditional!,
//!     no bang. A deliberate cut from Clara's general RHS: ours only ever inserts logical facts. Session-mouth
//!     `insert` is the fact-assert verb, not the RHS spelling.
//!   - lifecycle: collect-rules → compile-all → insert (value-threaded) → fire-rules (PURE, new frozen session) → query.
//!
//! Run: cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): the lifecycle (collect → compile → insert → insert → fire → query, wrapped in
// `length`) lives in the co-located fixture as `:user::compute`, driven via `call_beside_value`.

#[test]
fn cold_and_windy_fires_and_derives_the_fact() {
    // The rule fires (Temp 15<20 AND Wind 45>30 at the SAME location "Oslo" — the equality join on ?loc),
    // logically inserting ONE ColdAndWindy fact; `query` reads the derived facts back out.
    let count = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("rete lifecycle raised: {e:?}"));
    assert_eq!(count, Value::i64(1), "exactly one ColdAndWindy derived (the Oslo equality join)");
}

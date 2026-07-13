//! Arc 278 — THE NORTH STAR: the cold-and-windy rule, end to end. The engine's acceptance test AND the
//! executable spec of the rete DSL surface. RED at HEAD; GREEN at the stone-5 milestone (a working
//! equality-join forward-chaining engine with truth maintenance — DESIGN.md decomposition step 5).
//!
//! Every per-stone strike builds toward THIS. It is the living contract for the DSL — if a surface detail
//! is refined during a stone, update this test (curare), never let it drift.
//!
//! The DSL it pins:
//!   - facts are plain typed records.
//!   - (:wat::rete::defrule :ns::name :when [conditions] :then <pure-rhs>) — namespaced rule macro.
//!   - condition = (:FactType <clause>...):
//!       (?var <- :field)            bind/join  (fresh binds; bound ?var ⇒ cross-fact equality join on the field)
//!       (:wat::core::<op> a b)      constraint (FQDN value op; operands ∈ {?var, :field, literal}, resolved purely)
//!   - :then = N inserts, nothing else. Each (:wat::rete::insert <fact>) declares a logical derived fact
//!     (support = the firing token; auto-retracted if support vanishes); fact args may be pure exprs over the
//!     bound ?vars. The engine COLLECTS the inserts at fire — pure: no IO, no retract, no insert-unconditional!,
//!     no bang. A deliberate cut from Clara's general RHS: ours only ever inserts logical facts.
//!   - lifecycle: collect-rules → compile → insert (value-threaded) → fire-rules (PURE, new frozen session) → query.
//!
//! Run: cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy -- --include-ignored

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// The lifecycle, value-threaded: collect → compile → insert → insert → fire → query, then COUNT the derived
// facts (wrapped in `length` so the test is a single eval to a scalar — no env gymnastics).
const RUN: &str = "\
(:wat::core::length\n\
  (:wat::core::let\n\
    [rules    (:wat::rete::collect-rules :weather)\n\
     session  (:wat::rete::compile rules)\n\
     session  (:wat::rete::insert session (:weather::Temperature :celsius 15 :location \"Oslo\"))\n\
     session  (:wat::rete::insert session (:weather::WindSpeed    :kph 45 :location \"Oslo\"))\n\
     fired    (:wat::rete::fire-rules session)]\n\
    (:wat::rete::query fired :weather::ColdAndWindy)))";

#[test]
fn cold_and_windy_fires_and_derives_the_fact() {
    let world = startup_beside(file!())
        .expect("world (records + defrule) should freeze once the rete surface exists");

    // The rule fires (Temp 15<20 AND Wind 45>30 at the SAME location "Oslo" — the equality join on ?loc),
    // logically inserting ONE ColdAndWindy fact; `query` reads the derived facts back out.
    let count = eval_in_frozen(&wat::parse_one!(RUN).expect("parse lifecycle"), &world, &Environment::new())
        .unwrap_or_else(|e| panic!("rete lifecycle raised: {e:?}"))
        .value_owned();
    assert_eq!(count, Value::i64(1), "exactly one ColdAndWindy derived (the Oslo equality join)");
}

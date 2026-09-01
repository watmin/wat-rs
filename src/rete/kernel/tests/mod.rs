//! `kernel::tests` — split from a single 10,189-line `tests.rs` (2026-08-30).
//!
//! The split follows `partire`'s recorded five-module verdict; both ward reports and the
//! corrections weighed against them live in
//! `docs/arc/2026/06/278-rules-engine/NOTE-tests-rs-two-casts.md`. **Read that before
//! re-proposing a boundary** — two cuts are affirmatively REFUSED there, with reasons.
//!
//! This file holds the prelude and the fixtures used by MORE THAN ONE child. A helper used by
//! exactly one child lives in that child; that is the whole placement rule, and it is
//! mechanical rather than a matter of taste. A child needs only `use super::*;` — it reaches
//! kernel's items through this module's own glob.


use super::*;

use crate::freeze::{eval_in_frozen, startup_from_source};

use crate::load::InMemoryLoader;

use crate::rete::matcher::Bindings;

use crate::runtime::{Environment, Value};

use crate::types::Nature;

use crate::value::value::AggregateValue;

use rustc_hash::FxHashMap;

use std::sync::Arc;

/// The cold-and-windy world: Temperature + WindSpeed + ColdAndWindy records + the rule.
const WORLD: &str = "\
(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])\n\
\n\
(:wat::rete::defrule :weather::cold-and-windy\n\
  :when\n\
  [(:weather::Temperature\n\
     (?loc <- :location)\n\
     (?c   <- :celsius)\n\
     (:wat::rete::core::i64::< ?c 20))\n\
   (:weather::WindSpeed\n\
     (?loc <- :location)\n\
     (?k   <- :kph)\n\
     (:wat::rete::core::i64::> ?k 30))]\n\
  :then\n\
  [(:weather::ColdAndWindy ?loc)])\n\
\n\
";

fn freeze_src(src: &str) -> crate::freeze::FrozenWorld {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("world should freeze")
}

fn eval_in(world: &crate::freeze::FrozenWorld, src: &str) -> Value {
    let ast = crate::parse_one!(src).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

// ── The keyed-gather gate (DESIGN-STONE-keyed-gather.md) ──────────────────────────────────
//
// Two AccumulateNodes and one ExistsNode over `Reading`, joined to `Group` on `?g` — the
// `accum` grid axis's shape, reduced to the two node kinds whose gather is under test.


// ── Where does the accum fire actually spend its time? ────────────────────────────────────
//
// The `accum` grid axis is ~1.5× behind a WARMED Clara (2.19 vs 4.66 µs/fact at 40,200 facts;
// Clara's own per-fact cost falls 4.6× across the ladder as its JIT warms, ours is flat). The
// keyed gather is under 10% of our fire, so the cost is elsewhere — and there is no `perf` on
// this box. Rather than narrate a plausible root, the loop reports its own split.
//
// The world mirrors `wat-scripts/perf/grid/accum.wat` — FIVE rules (count/sum/min/max + exists)
// over Group ⋈ Reading — byte-for-byte modulo the namespace, so this apportions the AXIS's time
// and not a lookalike's. (`ACCUM_GATHER_WORLD` above is deliberately smaller: it exists to gate
// the gather's SHAPE, where two accumulators are enough.)

const ACCUM_AXIS_WORLD: &str = "\
(:wat::core::defrecord :apx::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::MinF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::MaxF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::ExistsF [g <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :apx::count-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::count) :from (:apx::Reading (?g <- :g)))]\n\
  :then [(:apx::CountF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::sum-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::sum ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::SumF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::min-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::min ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::MinF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::max-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::max ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::MaxF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::exists-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (:wat::rete::exists (:apx::Reading (?g <- :g)))]\n\
  :then [(:apx::ExistsF ?g)])\n\
\n\
(:wat::core::defn :apx::val [g <- :wat::core::i64  j <- :wat::core::i64] -> :wat::core::i64\n\
  (:wat::core::let [x (:wat::core::i64::+ (:wat::core::i64::* g 31) (:wat::core::i64::* j 17))]\n\
    (:wat::core::i64::- x (:wat::core::i64::* (:wat::core::i64::/ x 1000) 1000))))\n\
\n\
(:wat::core::defn :apx::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert s (:apx::Reading :g g :v (:apx::val g j))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
\n\
(:wat::core::defn :apx::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:apx::seed-readings (:wat::core::match (:wat::rete::insert s (:apx::Group g)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n\
";

/// Fire the axis world and return the operation counts (see `census_count`).
fn accum_count_census(g: i64, w: i64) -> Vec<(&'static str, u64)> {
    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let src = format!(
            "(:wat::core::match (:wat::rete::fire-rules (:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {g} {w})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_count_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
            .value_owned()
    });
    rows
}

// ── A0 depth-cost split (arc 278, 2026-07-31) ─────────────────────────────────────────────
//
// The grid's deep-cascade axis reads `:winner :clara` at [50 100] (all five runs), and holding
// the derived-fact count CONSTANT while varying depth showed the cost tracks DEPTH, not size:
// 6000 derived facts cost us 34.7ms at depth 10 and 119.5ms at depth 60, where Clara paid
// 76.7 → 114.2. Grounded, the round body runs FOUR full-network scans per round
// (root-join :2070, hash-join :2127, accumulate :2327, filter :2423) and a depth-D cascade
// needs D rounds — so we visit O(D) nodes D times while exactly one level can do work.
//
// This probe measures the SPLIT that decides the fix: at EQUAL work, how much of the extra
// cost at depth is per-round scaffolding over idle nodes, versus real per-fact work? If the
// idle scan dominates, a dirty-node agenda captures it; if it does not, only per-element
// incremental propagation (T3) helps. It asserts nothing about which — it prints the rows.

const DEPTH_SPLIT_WORLD: &str = "\
(:wat::core::defrecord :cascade::Node [level <- :wat::core::i64  id <- :wat::core::i64])\n\
(:wat::core::defrecord :cascade::Tag  [level <- :wat::core::i64  id <- :wat::core::i64])\n\
\n\
(:wat::core::defn :dc::build-rule [k <- :wat::core::i64] -> :wat::rete::Rule\n\
  (:wat::core::let [prev (:wat::core::i64::- k 1)\n\
                    c1 (:wat::core::quasiquote (:cascade::Node (?id <- :id) (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))\n\
                    c2 (:wat::core::quasiquote (:cascade::Tag  (?id <- :id) (?m <- :level) (:wat::rete::core::i64::= ?m (:wat::core::unquote prev))))\n\
                    t1 (:wat::core::quasiquote (:cascade::Node (:wat::core::unquote k) ?id))\n\
                    t2 (:wat::core::quasiquote (:cascade::Tag  (:wat::core::unquote k) ?id))]\n\
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)\n\
      :lhs (:wat::core::PersistentVector c1 c2)\n\
      :rhs (:wat::core::PersistentVector t1 t2))))\n\
\n\
(:wat::core::defn :dc::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
      (:wat::core::PersistentVector/conj acc (:dc::build-rule k)))\n\
    (:wat::core::PersistentVector (:dc::build-rule 1))\n\
    (:wat::core::range 2 (:wat::core::i64::+ depth 1))))\n\
\n\
(:wat::core::defn :dc::seed-level-0 [session <- :wat::rete::Session  width <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert s (:cascade::Node :level 0 :id i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) (:cascade::Tag :level 0 :id i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    session\n\
    (:wat::core::range 0 width)))\n";

// ── AlphaTree (DESIGN-STONE-alpha-discrimination-tree.md) ────────────────────────────────

use super::{build_alpha_index, class_field_names, session_facts, sorted_node_ids};

use crate::ast::WatAST;

use crate::rete::alpha_tree::AlphaTree;

use std::collections::HashMap;

/// Like `depth_split_phases`, but returns the fired session (seed + every derived fact) and
/// the frozen world (for `.symbols()`) instead of the phase census — the alpha-tree tests
/// below inspect the ACTUAL network and fact set the fire pass produced, rather than firing
/// a second time or hand-building a fixture.
fn fire_cascade(depth: i64, width: i64) -> (crate::freeze::FrozenWorld, Value) {
    let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("depth-split world should freeze");
    let src = format!(
            "(:wat::core::match (:wat::rete::fire-rules (:dc::seed-level-0 (:wat::core::match (:wat::rete::compile (:dc::build-rules {depth})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {width})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let fired = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("fire raised at depth={depth} width={width}: {e:?}"))
        .value_owned();
    (world, fired)
}

mod pass_semantics;
mod arm_lease;
mod alpha_discrimination;
mod binding_repr_bench;

/// Assert a phase census contains every phase the workload must exercise.
///
/// ⛔ ONE DEFINITION, MANY CALLERS — deliberately. Five census tests make this same claim, and
/// `probare` found 26 tests in this suite whose only assertion was a liveness guard that had
/// been COPY-PASTED between them (`"compile+seed produced 0 facts"` appeared verbatim six
/// times). A guard duplicated is a guard whose blindness is duplicated, which is the same defect
/// `complectens` found in the instrument itself — 37 hand-rolled `ms` closures. Fixing hollow
/// assertions by writing a new one per test would rebuild the thing being fixed.
///
/// The claim is ENGINE-facing: a phase's absence means marks were lost or the fixture changed
/// shape. It does not mean the phase became free — a phase that costs nothing still emits a row.
pub(super) fn assert_phases_present<'a>(
    have: impl IntoIterator<Item = &'a str>,
    want: &[&str],
    ctx: &str,
) {
    let have: Vec<&str> = have.into_iter().collect();
    assert!(
        !have.is_empty(),
        "the census recorded no phases at all — nothing below describes a fire{ctx}"
    );
    for w in want {
        assert!(
            have.iter().any(|h| h == w),
            "phase `{w}` is absent from the census. A fire of this shape MUST exercise it, so its \
             absence means marks were lost or the fixture changed shape — not that the phase \
             became free. present: {have:?}{ctx}"
        );
    }
}

// ── Timing primitives — ONE name per contract ─────────────────────────────────
//
// These replace 22 nested copies of a single name, `time_ns`, that carried TWO contracts: a
// 2-ary form returning elapsed/n (7 copies) and a 1-ary form returning elapsed (15 copies).
// They differ by a factor of n — 20k to 300k at the call sites that used it. The arity told you
// which, if you looked; the PRINTED NUMBER did not, so a reader comparing a figure from one
// test against a figure from another had nothing to warn them.
//
// ⛔ The cure is not this comment. It is that NEITHER NAME CAN BE READ AS THE OTHER, and that
// there is now one definition of each instead of twenty-two — WITHIN `kernel::tests`. Two 2-ary
// `time_ns` definitions with the same ambiguous name and the same elapsed/n contract still live
// at `src/rete/compiled_rhs.rs:540` and `:720`; the sweep did not reach them.

// ── THE ESTIMATOR IS THE MINIMUM, NOT THE MEAN ───────────────────────────────
//
// ⛔ EVERY COST SPLIT IN THIS SUITE USED `mean of RUNS` AND IT WAS WRONG. Measured 2026-08-30 on
// `accum_alpha_push_split`: the FIRST arm of each round paid a cold start of 287.4 ms against
// 11.5 and 11.4 ms for the identical work — 25x — and the mean carried that one round into the
// reported figure. `M` was never slow; `M` goes first.
//
// The damage was not subtle. `H−M` read −93.33 ms: arm H, which is arm M PLUS a HashMap entry,
// measured 9x FASTER than M. Three tests printed subtractions with impossible signs; the rest
// were silently inflating whichever arm ran first, so "X dominates" conclusions in those tables
// could be artefacts of ORDERING. With the minimum, `A−M` went −90.76 → +1.6..+2.3 ms across
// runs, and every impossible sign resolved.
//
// ⛔ WHY THE MINIMUM IS HONEST HERE, AND NOT JUST CONVENIENT — because discarding an
// inconvenient round IS what taking a minimum does, this had to be earned rather than assumed.
// Two measurements settle it (2026-08-30):
//
//   1. THE COST IS ONE-TIME, NOT PER-ROUND. Running the SAME work once, UNTIMED, before the
//      first timed pass drops round 0 from 286.5 ms to 12.1 ms. It is a first-execution cost.
//      (It is NOT capacity growth — pre-reserving 300k pool entries changed nothing, 298.7 ms.)
//   2. IT DOES NOT TRANSFER TO PRODUCTION. Across six rounds the isolated arm warms up 2500%
//      (286.5 -> 11.6 ms) while `alpha_activate_fact`, THE PRODUCTION PATH DOING MORE WORK,
//      warms up 20% (16.4 -> 13.7 ms). A real fire does not pay this.
//
// So a per-FACT cost estimate must not carry a ONE-TIME cost, and the minimum is the estimator
// that excludes it. ⚠ What is NOT established is the exact first-execution cost — CPU clock
// ramp, first-touch page faults, and lazy init are all candidates and none was isolated. The
// estimator choice does not depend on which, but do not read this note as a diagnosis of it.
//
// ⚠ Minimum also discards the production path's genuine ~2.7 ms warm-up. That is deliberate for
// a per-fact split — you want the marginal cost, not the one-time allocation — but it IS a
// choice, and a fire on a fresh session does pay something like it.
//
// ⛔ THE ARGUMENT WAS ALREADY IN THIS FILE, ON `calibrate_mark_ns`: "TAKE THE MINIMUM OF SEVERAL
// BATCHES, not one … the true cost cannot be lower, and everything above it is interference."
// The calibration constant used the minimum. The splits it feeds used the mean. One instrument,
// two estimators, and the wrong one on the larger measurement — which is exactly what
// `render_phase_table`'s own doc warns about: "two copies is how one of them silently stops
// subtracting."
//
// ⚠ AND THE 90 ms ARTEFACT WAS HIDING A RESOLUTION FLOOR. With it gone, `H−M` measures −0.48,
// +0.19, +0.24 ms across three runs — IT CHANGES SIGN. A per-fact HashMap entry is below what a
// 12 ms arm at RUNS=3 can resolve. Sub-millisecond rows in these tables are noise wearing a
// number; do not read one as a finding without re-measuring at higher RUNS or larger N.

/// Nanoseconds PER ITERATION — runs `body` `n` times and divides by `n`.
pub(super) fn ns_per_iter(n: usize, mut body: impl FnMut()) -> f64 {
    let t0 = std::time::Instant::now();
    for _ in 0..n {
        body();
    }
    t0.elapsed().as_nanos() as f64 / n as f64
}

/// TOTAL nanoseconds for ONE run of `body`. Divided by nothing.
pub(super) fn elapsed_ns(mut body: impl FnMut()) -> f64 {
    let t0 = std::time::Instant::now();
    body();
    t0.elapsed().as_nanos() as f64
}

/// Nanoseconds → milliseconds. Was written out as a local closure 37 times across eight
/// modules. `render_phase_table`'s own doc says why one copy matters: "two copies is how one of
/// them silently stops subtracting."
pub(super) fn ms(ns: f64) -> f64 {
    ns / 1e6
}

// ── The shared fire-cost instrument ───────────────────────────────────────────
//
// Hoisted out of `fire_cost_census.rs` when that file was split by subject (2026-08-30). These
// are the helpers used by MORE THAN ONE cost module — the same placement rule as everything
// else in this file, applied one level down.
//
// ⛔ `calibrate_mark_ns` LIVED 9,089 LINES BELOW ITS FIRST CALLER — `complectens`' top finding.
//
// The first repair moved it here and claimed the class was "UNREPRESENTABLE" because a parent
// module is above every child. `intueri` falsified that the same day: the argument is about
// CROSS-file ordering, and the original defect was INTRA-file — the function sat 193 lines below
// `render_phase_table`, still forward, in this very file. A claim of impossibility that leaves
// its own instance standing is worse than no claim.
//
// It is now defined immediately above its first caller, which is a real fix and a smaller one.
// Nothing structural prevents the next helper from landing below its callers; if you add one,
// put it above them.

// ── Arc 278 A8 — the node-share fire-path census ─────────────────────────────
//
// A8 (node-share) is the one grid cell Clara wins, and the compiler was cleared on 2026-07-30:
// `wat-scripts/scratch-pad/probe-node-share-dedup.wat` counts the compiled network at `4 + 2N`
// (Alpha flat at 2, HashJoin flat at 1) for N = 1..32, so the shared prefix collapses exactly
// as `find-or-mint-hash-join` intends. The blow-up — >4 GiB to join 500 facts against 20 rules
// — therefore lives in the FIRE path, and this is the instrument that reads it.
//
// It measures, it does not guess. Every native structure the loop grows is counted per round
// (see `RoundCensus`), so the growth term names itself instead of confirming a hypothesis about
// which one it is. The world below is copied from `wat-scripts/perf/grid/node-share.wat` —
// same `build-rule`, same `seed` — so this measures the AXIS and not a lookalike.

/// The node-share world: A/B/Out plus the axis's own rule-builder and seeder.
///
/// `build-rule i n` is byte-identical to the axis's: the leading `[A (?k)] ⋈ [B (?k)]` carries
/// no `i`, so it is the shared prefix under test; only the trailing `where` holds the per-rule
/// literal. `mod` is spelled as the truncating-division idiom (wat has no native i64 mod).
const NODE_SHARE_WORLD: &str = "\
(:wat::core::defrecord :nsh::A   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :nsh::B   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :nsh::Out [k <- :wat::core::i64])\n\
\n\
(:wat::core::defn :nsh::build-rule [i <- :wat::core::i64  n <- :wat::core::i64] -> :wat::rete::Rule\n\
  (:wat::core::let [a-c     (:wat::core::quasiquote (:nsh::A (?k <- :k)))\n\
                    b-c     (:wat::core::quasiquote (:nsh::B (?k <- :k)))\n\
                    where-c (:wat::core::quasiquote\n\
                              (:wat::rete::where\n\
                                (:wat::rete::core::i64::= (:wat::core::unquote i)\n\
                                  (:wat::rete::core::i64::- ?k\n\
                                    (:wat::rete::core::i64::* (:wat::rete::core::i64::/ ?k (:wat::core::unquote n) :undefined 0) (:wat::core::unquote n) :undefined 0)\n\
                                    :undefined 0))))\n\
                    ins     (:wat::core::quasiquote (:nsh::Out ?k))]\n\
    (:wat::rete::Rule :name (:wat::core::i64::to-string i)\n\
      :lhs (:wat::core::PersistentVector a-c b-c where-c)\n\
      :rhs (:wat::core::PersistentVector ins))))\n\
\n\
(:wat::core::defn :nsh::build-rules [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  i <- :wat::core::i64]\n\
      -> (:wat::core::PersistentVector :- [:wat::rete::Rule])\n\
      (:wat::core::PersistentVector/conj acc (:nsh::build-rule i n)))\n\
    (:wat::core::PersistentVector)\n\
    (:wat::core::range 0 n)))\n\
\n\
(:wat::core::defn :nsh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert s (:nsh::A i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) (:nsh::B i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    session\n\
    (:wat::core::range 0 items)))\n\
";
/// Fire the axis world at `g` groups × `w` readings; return the per-phase nanosecond split.
///
/// Only `fire-rules` is inside the armed window — compile and seed run first, un-timed, exactly
/// as the grid harness does it, so this apportions the same span the grid's `:native-ns` covers.
fn accum_phase_census(g: i64, w: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged =
        format!("(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {g} {w})");
    let src = format!("(:wat::core::match (:wat::rete::fire-rules {staged}) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))");
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let t0 = std::time::Instant::now();
    let (_fired, mut rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
            .value_owned()
    });
    // ⚠ The WHOLE fire, so the census can declare its own COVERAGE. The six phases live inside
    // `fire_fixpoint_delta`'s round loop; everything outside it — network extraction,
    // alpha_by_type, parents_of, per-round setup, the terminate step, merge_facts,
    // to_persistent — is NOT covered by any mark. Apportioning the phases as if they were the
    // whole fire is precisely the instrument-boundary error this file keeps warning about.
    // ⚠ This wraps the WHOLE driver expression — `(fire-rules (seed (compile ...)))` — so it
    // includes compile and SEED, not just the fire. Named accordingly: an earlier version
    // called it "WHOLE FIRE" and the ~205ms of seeding read as unaccounted fire, which is the
    // third instrument-boundary error in this file today and all three were mine. The four
    // outer marks (IN/SETUP/ROUND LOOP/OUT) are what partition the fire, and their sum matches
    // the grid's own :wat-ns to ~1% — that agreement is the cross-check.
    rows.push((
        "WHOLE EVAL (compile+seed+fire)",
        t0.elapsed().as_nanos() as u64,
        1,
    ));
    rows
}
/// Cost of one `phase_start`/`phase_end` pair, in ns — the constant every
/// census harness subtracts.
///
/// TAKE THE MINIMUM OF SEVERAL BATCHES, not one. A single 200k batch reads
/// anywhere from ~105 to ~155 ns depending on what else the box is doing, and a
/// row with 40 000 pairs multiplies that spread into a **±2 ms swing** — enough
/// that `prod:compiled-rhs` was seen at 2.541 and 4.826 ms for identical code.
/// The minimum is the right estimator for a tight loop: the true cost cannot be
/// lower, and everything above it is interference.
fn calibrate_mark_ns() -> f64 {
    const BATCHES: usize = 5;
    const PER_BATCH: u64 = 200_000;
    let mut best = f64::INFINITY;
    for _ in 0..BATCHES {
        let t0 = std::time::Instant::now();
        super::with_phase_census(|| {
            for _ in 0..PER_BATCH {
                let m = super::phase_start();
                super::phase_end("cal", m);
            }
        });
        let ns = t0.elapsed().as_nanos() as f64 / PER_BATCH as f64;
        if ns < best {
            best = ns;
        }
    }
    best
}

/// Render an instrument-subtracted phase table for ANY axis.
///
/// Extracted 2026-08-01 when node-share needed the same table accum already had. Copying it
/// would have put the instrument-subtraction arithmetic in two places, and the whole reason
/// that arithmetic exists is that a table which misreports its own instrument is worse than no
/// table — two copies is how one of them silently stops subtracting.
///
/// `census(a, b)` fires the axis at that size and returns (phase, ns, mark-pairs-fired).
/// `facts(a, b)` is the fact count for the header. `top` partitions the fire (summing a parent
/// with its children is what made an earlier version of this table report 124% coverage).
fn render_phase_table(
    label: &str,
    sizes: &[(i64, i64)],
    top: &[&'static str],
    required: &[&'static str],
    facts: impl Fn(i64, i64) -> i64,
    census: impl Fn(i64, i64) -> Vec<(&'static str, u64, u64)>,
) -> String {
    let cal_ns_per_pair = calibrate_mark_ns();
    const RUNS: usize = 3;

    let mut table = format!(
        "\n{label} — per-phase split (native fire-rules only), MINIMUM of {RUNS} runs\n\
             instrument: ~{cal_ns_per_pair:.1} ns per mark pair; `net` = raw MINUS this row's own \
             pairs. PARENT rows still contain their children's share.\n"
    );
    for &(a, b) in sizes {
        // rune:perspicere(read-once) — census sample bags; alias would be a one-site mumble.
        let mut samples: std::collections::HashMap<&'static str, Vec<u64>> =
            std::collections::HashMap::new();
        let mut pairs: std::collections::HashMap<&'static str, u64> =
            std::collections::HashMap::new();
        let mut order: Vec<&'static str> = Vec::new();
        for _ in 0..RUNS {
            let rows = census(a, b);
            assert!(
                !rows.is_empty(),
                "{label}: census recorded NOTHING at {a}/{b}"
            );
            for (name, ns, k) in rows {
                if !samples.contains_key(name) {
                    order.push(name);
                }
                samples.entry(name).or_default().push(ns);
                pairs.insert(name, k);
            }
        }
        let missing: Vec<&str> = required
            .iter()
            .copied()
            .filter(|p| !samples.contains_key(p))
            .collect();
        assert!(
            missing.is_empty(),
            "{label}: phase(s) {missing:?} never recorded at {a}/{b}"
        );

        // THE REPORTED FIGURE IS THE MINIMUM, which is what this table's header has always
        // claimed and what `calibrate_mark_ns` twenty lines up has always computed. It returned
        // `sum / xs.len()` until arc 278 C1 — a mean under a MINIMUM header, one instrument with
        // two estimators, and the wrong one on the larger measurement.
        //
        // TWO columns, not three. While the reported value was a mean, `lo`/`hi` were a genuine
        // spread around it; now that the reported value IS `lo`, printing both would be the same
        // number twice. The second column is therefore the WORST run — the reported value is the
        // floor, `hi` is how much interference the box added on its noisiest pass, and the gap
        // between them is the only thing a reader can still learn from the spread.
        let stat = |xs: &[u64]| -> (f64, u64) {
            (
                *xs.iter().min().expect("non-empty") as f64,
                *xs.iter().max().expect("non-empty"),
            )
        };
        let net_of = |k: &str, xs: &[u64]| -> f64 {
            stat(xs).0 - *pairs.get(k).unwrap_or(&0) as f64 * cal_ns_per_pair
        };
        let total_min: f64 = top
            .iter()
            .filter_map(|k| samples.get(k).map(|xs| stat(xs).0))
            .sum();
        assert!(total_min > 0.0, "{label}: phase total is zero at {a}/{b}");
        let total_net: f64 = top
            .iter()
            .filter_map(|k| samples.get(k).map(|xs| net_of(k, xs)))
            .sum();
        let instrument: f64 = pairs.values().map(|k| *k as f64 * cal_ns_per_pair).sum();

        table.push_str(&format!(
            "\n  {a}/{b}  ({} facts)   FIRE {:.2} ms raw / {:.2} net   \
                 instrument {:.2} ms across {} pairs\n",
            facts(a, b),
            total_min / 1e6,
            total_net / 1e6,
            instrument / 1e6,
            pairs.values().sum::<u64>(),
        ));
        for phase in &order {
            if *phase == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            let xs = samples.get(phase).expect("discovered, so present");
            let (best, worst) = stat(xs);
            let net = net_of(phase, xs);
            let flag = if net <= 0.0 {
                "  ⚠ BELOW ITS OWN INSTRUMENT"
            } else {
                ""
            };
            table.push_str(&format!(
                "    {:<20} {:>8.2} ms raw  {:>8.2} net  {:>5.1}%  [worst {:.2}]  {}x{}\n",
                phase,
                best / 1e6,
                net / 1e6,
                100.0 * net / total_net,
                worst as f64 / 1e6,
                *pairs.get(phase).unwrap_or(&0),
                flag,
            ));
        }
    }
    table
}
/// Fire the node-share world at `n` rules x `m` items; per-phase split with pair counts.
///
/// node-share is the grid's WEAKEST engine cell (:ratio 1.56 at [50 200]) and had no phase
/// census at all — only a COUNT census at M=50. Ranking its sinks off accum's or fanout's
/// table would be the R61 error: alpha is 4.7% of fanout and ~40% of accum.
fn node_share_phase_census(n: i64, m: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("node-share world should freeze");
    let staged = format!("(:nsh::seed (:wat::core::match (:wat::rete::compile (:nsh::build-rules {n})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {m})");
    let src = format!("(:wat::core::match (:wat::rete::fire-rules {staged}) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))");
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
            .value_owned()
    });
    rows
}
/// Fire a depth×width cascade through the native path; per-phase split with pair counts.
fn cascade_phase_census(depth: i64, width: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("depth-split world should freeze");
    let src = format!(
            "(:wat::core::match (:wat::rete::fire-rules (:dc::seed-level-0 (:wat::core::match (:wat::rete::compile (:dc::build-rules {depth})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {width})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("cascade fire raised at depth={depth} width={width}: {e:?}"))
            .value_owned()
    });
    rows
}
// ── The fact-heavy, rule-LIGHT census (arc 278, 2026-08-01) ───────────────────────────────
//
// The depth-split probe answers "what does DEPTH cost" on a rule-heavy cascade. This one
// answers the complementary question the compiled-conditions stone needs: what does a match
// cost PER FACT when the discrimination tree buys nothing?
//
// Fanout is that shape — ONE rule, two conditions, two fact types, so D=1 per type and the
// tree has nothing to prune. Every millisecond in `alpha:match` here is per-CALL cost, not
// per-candidate: the redundant head compare, `classify_rete_clause` on a static AST, the
// linear field-name scan, and the two heap allocations that rebuild a constant binding key.
//
// Sizing the stone off the CASCADE's per-fact rate would be extrapolating across workload
// shapes, which is the error that has cost this arc twice today. Measure the shape you mean.

const FANOUT_CENSUS_WORLD: &str = "\
(:wat::core::defrecord :fan::Left  [key <- :wat::core::i64  lid <- :wat::core::i64])\n\
(:wat::core::defrecord :fan::Right [key <- :wat::core::i64  rid <- :wat::core::i64])\n\
(:wat::core::defrecord :fan::Pair  [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64])\n\
\n\
(:wat::core::defn :fan::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert acc (:fan::Left :key k :lid f)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) (:fan::Right :key k :rid f)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    s\n\
    (:wat::core::range 0 fanout)))\n\
\n\
(:wat::core::defn :fan::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:fan::seed-key acc k fanout))\n\
    s\n\
    (:wat::core::range 0 keys)))\n\
\n\
(:wat::rete::defrule :fan::fan-rule\n\
  :when\n\
  [(:fan::Left  (?k <- :key) (?l <- :lid))\n\
   (:fan::Right (?k <- :key) (?r <- :rid))]\n\
  :then\n\
  [(:fan::Pair ?k ?l ?r)])\n";
fn fanout_phase_census(keys: i64, fanout: i64) -> Vec<(&'static str, u64, u64)> {
    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout census world should freeze");
    let staged = format!(
        "(:fan::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :fan)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {keys} {fanout})"
    );
    let src = format!("(:wat::core::match (:wat::rete::fire-rules {staged}) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))");
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census_counted(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fanout fire raised at keys={keys} fanout={fanout}: {e:?}"))
            .value_owned()
    });
    rows
}

mod accum_alpha_cost;
mod accum_cost;
mod cascade_cost;
mod fanout_cost;
mod gather_probe_cost;
mod harvest_cost;
mod node_share_cost;
mod rank_and_instrument;
mod strat_cost;
mod termination_verdict;

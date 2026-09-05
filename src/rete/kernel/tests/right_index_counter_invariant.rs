//! ★ D2 — READ THE COUNTER. `indexed_n[J]` against the bucket population it describes.
//!
//! ## What has three writers and one maintainer
//!
//! `right_idx[J]` (key → `Vec<Element>`, the P6 persistent right index of HashJoinNode `J`) is
//! appended to from THREE places. Exactly one of them maintains `indexed_n[J]`, the high-water
//! mark that same place reads back as `already`:
//!
//! | writer | appends `right_idx[J]` | maintains `indexed_n[J]` |
//! |---|---|---|
//! | `keyed_join_persistent` (`fire/mod.rs`) | yes | **yes** — reads `already`, appends the tail, writes back |
//! | `hash_join_delta` first-keying catch-up (`fire/pass/hash_join.rs`) | yes | **no** |
//! | `hash_join_delta` step-2 Δright (`fire/pass/hash_join.rs`) | yes | **no** |
//!
//! `keyed_join_persistent`'s guard is `right_elements[already..]` — `already` is the ONLY thing
//! stopping a second visit from re-pushing every element the bucket already holds. So a join
//! written by the maintainer AND by a bypass is the hazard: the mark is stale-low against the
//! population, and the next visit doubles the bucket.
//!
//! ## ⛔ WHY AN END-TO-END DIFFERENTIAL CANNOT SEE THIS
//!
//! Two prior drives declared this latent. Both compared FACTS — native vs oracle counts, and a
//! query whose rows mirror the join chain. `seen_insert` dedups the derived fact set, so a
//! doubled bucket is invisible to either by construction (D7's shape, and C16's). This file does
//! not build a third one. It reads the counter.
//!
//! ## ⛔ AND A GREEN INVARIANT OVER UNREACHED CODE PROVES NOTHING
//!
//! If the bypass sites never fire, `indexed_n[J] == Σ|right_idx[J]|` holds TRIVIALLY — every
//! append came from the maintainer, which by definition keeps them equal. Worse: if the two
//! writer populations never OVERLAP (each join written by one writer only), the invariant is
//! still vacuous with respect to the defect, because the hazard needs both writers on the SAME
//! join. So the reach assertions here are three, in ascending strength:
//!
//! 1. both bypass sites executed and appended (`census::RIGHT_IDX_SITE_*` rows, elements > 0);
//! 2. the shape compiled a HashJoin chain — a join whose PARENT is a HashJoin;
//! 3. ★ at least one join id carries a maintainer mark AND bypass appends — the two writers
//!    actually met on one index, which is the only configuration in which the asymmetry can bite.
//!
//! Without (3) a green here would be the green-over-nothing this arc has found five times.

use super::*;

use crate::rete::kernel::census::{
    with_fire_census, with_right_idx_appends, RIGHT_IDX_SITE_CATCHUP, RIGHT_IDX_SITE_STEP2,
};

/// ★ THE SHAPE: `filter → HashJoin(a) → HashJoin(b)`, driven in TWO WAVES.
///
/// `chain` compiles to `α(A) → RootJoin → Test → HashJoin(a){α B} → HashJoin(b){α C} → Prod` —
/// the `R W R R` form (`tests/rete/probe_arc278_where_is_positionally_free.wat:48`). The mid-chain
/// `where` is what puts a Test between the root join and the first HashJoin, and that is what
/// splits the two writers apart:
///
/// * `HashJoin(a)`'s parent is the **Test**, which is not a join parent, so `hash_join_delta`
///   never visits it. It is reached only through `join_after_filter` (pass 3.6) →
///   `keyed_join_persistent`. **`a` is the maintainer-only control.**
/// * `HashJoin(b)`'s parent is `HashJoin(a)` — a join parent — so `hash_join_delta` (pass 3)
///   visits it AND `filter_after_join` (pass 3.7) left-activates it through
///   `keyed_join_persistent`. **`b` is the index both writers touch.**
///
/// ⛔ ONE WAVE IS NOT ENOUGH, AND THE FIRST DRIVE OF THIS FILE PROVED IT. Seeding A, B, C once
/// and deriving C a round later put `indexed_n` on `a` and left `b` with no mark at all: the
/// maintainer's early-return (`right_elements.is_empty()`) fires before it caches join keys, so
/// `b` was first keyed by `hash_join_delta`'s catch-up and the maintainer never came back. Each
/// index had exactly ONE writer and the invariant held by partition — the vacuous green this
/// file's reach assertions exist to refuse.
///
/// TWO WAVES fix it. Wave 1 (`k < n`) seeds A/B/C, so round 0's 3.6 and 3.7 key BOTH joins
/// through the maintainer and both carry a mark. Wave 2 arrives as DERIVED facts: `M(k)` for
/// `k ∈ [n, 2n)` is seeded, and `derive-a`/`derive-b`/`derive-c` turn each into a second A, B and
/// C. Round 1 therefore hands `hash_join_delta` a non-empty Δright on an ALREADY-KEYED `b` —
/// `first_keying` is false, step 2 runs, and it appends without touching the mark the maintainer
/// will read back as `already` in the same round's pass 3.7.
const D2_WORLD: &str = "\
(:wat::core::defrecord :d2::A [k <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::B [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::C [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::M [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::D [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::Hit [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::Hit2 [k <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :d2::derive-a\n\
  :when [(:d2::M (?k <- :k))]\n\
  :then [(:d2::A ?k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])\n\
\n\
(:wat::rete::defrule :d2::derive-b\n\
  :when [(:d2::M (?k <- :k))]\n\
  :then [(:d2::B ?k)])\n\
\n\
(:wat::rete::defrule :d2::derive-c\n\
  :when [(:d2::M (?k <- :k))]\n\
  :then [(:d2::C ?k)])\n\
\n\
(:wat::rete::defrule :d2::chain\n\
  :when [(:d2::A (?k <- :k) (?v <- :v))\n\
         (:wat::rete::where (:wat::rete::core::i64::> ?v 0))\n\
         (:d2::B (?k <- :k))\n\
         (:d2::C (?k <- :k))]\n\
  :then [(:d2::Hit ?k)])\n\
\n\
(:wat::rete::defrule :d2::derive-d\n\
  :when [(:d2::M (?k <- :k))]\n\
  :then [(:d2::D ?k)])\n\
\n\
(:wat::rete::defrule :d2::chain2\n\
  :when [(:d2::A (?k <- :k) (?v <- :v))\n\
         (:wat::rete::where (:wat::rete::core::i64::> ?v 0))\n\
         (:d2::B (?k <- :k))\n\
         (:d2::D (?k <- :k))]\n\
  :then [(:d2::Hit2 ?k)])\n\
\n\
(:wat::core::defn :d2::ins-a [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::A :k k :v (:wat::core::i64::+ k 1))) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert a: ceiling\" :wat::core::None :wat::core::None))))\n\
(:wat::core::defn :d2::ins-b [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::B k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert b: ceiling\" :wat::core::None :wat::core::None))))\n\
(:wat::core::defn :d2::ins-c [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::C k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert c: ceiling\" :wat::core::None :wat::core::None))))\n\
(:wat::core::defn :d2::ins-m [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::M k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert m: ceiling\" :wat::core::None :wat::core::None))))\n\
\n\
(:wat::core::defn :d2::wave1 [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:d2::ins-c (:d2::ins-b (:d2::ins-a acc k) k) k))\n\
    s\n\
    (:wat::core::range 0 n)))\n\
\n\
(:wat::core::defn :d2::wave2 [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:d2::ins-m acc k))\n\
    s\n\
    (:wat::core::range n (:wat::core::i64::* n 2))))\n\
\n\
(:wat::core::defn :d2::seed [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:d2::wave2 (:d2::wave1 s n) n))\n\
";

/// A single-HashJoin control: the SAME driver, a rule with one join and no chain.
///
/// Mutation 3's target. `HashJoin(b)` does not exist, so there is no index two writers can meet
/// on, and the applicability guard below must REFUSE this shape rather than pass green over it.
const D2_SINGLE_JOIN_WORLD: &str = "\
(:wat::core::defrecord :d2::A [k <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::B [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::C [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::M [k <- :wat::core::i64])\n\
(:wat::core::defrecord :d2::Hit [k <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :d2::derive-a\n\
  :when [(:d2::M (?k <- :k))]\n\
  :then [(:d2::A ?k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])\n\
\n\
(:wat::rete::defrule :d2::derive-b\n\
  :when [(:d2::M (?k <- :k))]\n\
  :then [(:d2::B ?k)])\n\
\n\
(:wat::rete::defrule :d2::derive-c\n\
  :when [(:d2::M (?k <- :k))]\n\
  :then [(:d2::C ?k)])\n\
\n\
(:wat::rete::defrule :d2::chain\n\
  :when [(:d2::A (?k <- :k) (?v <- :v))\n\
         (:wat::rete::where (:wat::rete::core::i64::> ?v 0))\n\
         (:d2::B (?k <- :k))]\n\
  :then [(:d2::Hit ?k)])\n\
\n\
(:wat::core::defn :d2::ins-a [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::A :k k :v (:wat::core::i64::+ k 1))) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert a: ceiling\" :wat::core::None :wat::core::None))))\n\
(:wat::core::defn :d2::ins-b [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::B k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert b: ceiling\" :wat::core::None :wat::core::None))))\n\
(:wat::core::defn :d2::ins-c [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::C k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert c: ceiling\" :wat::core::None :wat::core::None))))\n\
(:wat::core::defn :d2::ins-m [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::match (:wat::rete::insert s (:d2::M k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! \"insert m: ceiling\" :wat::core::None :wat::core::None))))\n\
\n\
(:wat::core::defn :d2::wave1 [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:d2::ins-c (:d2::ins-b (:d2::ins-a acc k) k) k))\n\
    s\n\
    (:wat::core::range 0 n)))\n\
\n\
(:wat::core::defn :d2::wave2 [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:d2::ins-m acc k))\n\
    s\n\
    (:wat::core::range n (:wat::core::i64::* n 2))))\n\
\n\
(:wat::core::defn :d2::seed [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:d2::wave2 (:d2::wave1 s n) n))\n\
";

/// One join's reading in one round: `(join id, indexed_n[J] — `None` when the maintainer has
/// never visited it, Σ|right_idx[J]|)`.
type JoinMark = (i64, Option<usize>, usize);

/// One round's readings: `(round index, every join's mark)`.
type RoundMarks = (usize, Vec<JoinMark>);

/// What one fire of the D2 shape recorded: the per-round census, and the append rows.
struct D2Reading {
    /// Per round: `(round, [(join id, indexed_n or None, Σ|right_idx[J]|)])`.
    rounds: Vec<RoundMarks>,
    /// `(join id, site, elements appended)` — one row per site that RAN, `0` if it appended none.
    appends: Vec<(i64, &'static str, usize)>,
}

/// Compile, seed and fire `world`, with both instruments armed.
fn fire_d2(world_src: &str, n: i64, what: &str) -> D2Reading {
    let world = startup_from_source(world_src, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("{what}: world should freeze: {e:?}"));
    let src = format!(
        "(:wat::core::match (:wat::rete::fire-rules (:d2::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :d2)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {n})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the D2 fire driver");
    let ((_fired, census), appends) = with_right_idx_appends(|| {
        with_fire_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("{what}: fire raised at n={n}: {e:?}"))
                .value_owned()
        })
    });
    D2Reading {
        rounds: census
            .into_iter()
            .map(|r| (r.round, r.right_idx_by_join))
            .collect(),
        appends,
    }
}

impl D2Reading {
    /// Elements a bypass site appended, per join.
    fn bypass_appends(&self, join: i64) -> usize {
        self.appends
            .iter()
            .filter(|(j, _, _)| *j == join)
            .map(|(_, _, n)| *n)
            .sum()
    }

    /// Total elements a given site appended across every join.
    fn site_total(&self, site: &str) -> usize {
        self.appends
            .iter()
            .filter(|(_, s, _)| *s == site)
            .map(|(_, _, n)| *n)
            .sum()
    }

    /// Did the site RUN at all (a row exists, whatever its count)?
    fn site_ran(&self, site: &str) -> bool {
        self.appends.iter().any(|(_, s, _)| *s == site)
    }

    /// Join ids carrying a maintainer mark in the LAST round that has one.
    fn maintained_joins(&self) -> Vec<i64> {
        let mut out: Vec<i64> = Vec::new();
        for (_, rows) in &self.rounds {
            for (j, mark, _) in rows {
                if mark.is_some() && !out.contains(j) {
                    out.push(*j);
                }
            }
        }
        out.sort_unstable();
        out
    }

    /// The full reading, rendered — every round, every join, both columns. Printed into any
    /// failure so the arm that fired is legible without a re-run (there is no re-running a
    /// counter: the state is gone).
    fn render(&self) -> String {
        let mut s = String::new();
        s.push_str("  per-round  (join, indexed_n, Σ|right_idx[J]|):\n");
        for (round, rows) in &self.rounds {
            s.push_str(&format!("    round {round}: "));
            if rows.is_empty() {
                s.push_str("(no join index)\n");
                continue;
            }
            for (j, mark, els) in rows {
                let m = match mark {
                    Some(n) => n.to_string(),
                    None => "ABSENT".to_string(),
                };
                s.push_str(&format!("[J{j} n={m} els={els}] "));
            }
            s.push('\n');
        }
        s.push_str("  right_idx appends by site  (join, site, elements):\n");
        if self.appends.is_empty() {
            s.push_str("    (no bypass site ran)\n");
        }
        for (j, site, n) in &self.appends {
            s.push_str(&format!("    J{j}  {site}  {n}\n"));
        }
        s
    }
}

/// The applicability + non-vacuity guard, as ONE verb.
///
/// Extracted so the control test below can DRIVE it rather than describe it: a guard asserted
/// only in the test it protects is a guard whose refusal has never been observed. Panics with a
/// message naming which condition failed; the full reading is printed either way.
fn assert_applicable(r: &D2Reading) {
    assert!(
        !r.rounds.is_empty(),
        "the census recorded no rounds — the fire did not run through the instrumented loop\n{}",
        r.render()
    );
    let joins: std::collections::BTreeSet<i64> = r
        .rounds
        .iter()
        .flat_map(|(_, rows)| rows.iter().map(|(j, _, _)| *j))
        .collect();
    assert!(
        joins.len() >= 2,
        "INAPPLICABLE SHAPE: this probe needs a HashJoin CHAIN — two joins with persistent right \
         indexes — and this fire produced {} ({joins:?}). A single-HashJoin shape cannot put two \
         writers on one index, so a green here would measure nothing.\n{}",
        joins.len(),
        r.render()
    );

    // ── Non-vacuity 1: both bypass sites executed, and appended. ─────────────────────────────
    assert!(
        r.site_ran(RIGHT_IDX_SITE_CATCHUP),
        "INAPPLICABLE SHAPE: hash_join_delta's first-keying catch-up NEVER RAN — the invariant \
         would hold trivially over code this workload does not reach\n{}",
        r.render()
    );
    assert!(
        r.site_ran(RIGHT_IDX_SITE_STEP2),
        "INAPPLICABLE SHAPE: hash_join_delta's step-2 Δright append NEVER RAN — same vacuity\n{}",
        r.render()
    );
    assert!(
        r.site_total(RIGHT_IDX_SITE_CATCHUP) > 0,
        "INAPPLICABLE SHAPE: the catch-up block ran but appended ZERO elements — it wrote nothing \
         to bypass the counter with\n{}",
        r.render()
    );
    assert!(
        r.site_total(RIGHT_IDX_SITE_STEP2) > 0,
        "INAPPLICABLE SHAPE: step 2 ran but appended ZERO elements — it wrote nothing to bypass \
         the counter with\n{}",
        r.render()
    );

    // ── Non-vacuity 2 (★): the two writers met on ONE index. ─────────────────────────────────
    //
    // The hazard is not "a bypass appended somewhere" — it is a bypass appending to an index the
    // MAINTAINER also owns, so that `already` goes stale against the bucket. A workload where
    // each join has exactly one writer satisfies the invariant BY PARTITION and says nothing.
    // The first drive of this file landed in exactly that state and the guard caught it.
    let maintained = r.maintained_joins();
    assert!(
        !maintained.is_empty(),
        "INAPPLICABLE SHAPE: no join carries a maintainer mark — `keyed_join_persistent` never \
         ran, so `indexed_n` describes nothing and the invariant is empty\n{}",
        r.render()
    );
    let overlap: Vec<i64> = maintained
        .iter()
        .copied()
        .filter(|j| r.bypass_appends(*j) > 0)
        .collect();
    assert!(
        !overlap.is_empty(),
        "INAPPLICABLE SHAPE — ⛔ VACUOUS: no index was written by BOTH the maintainer and a bypass \
         site. Maintained joins {maintained:?}; bypass appends land on {:?}. The asymmetry cannot \
         bite where the writers do not meet, so this fire cannot decide D2 either way.\n{}",
        r.appends
            .iter()
            .map(|(j, _, _)| *j)
            .collect::<std::collections::BTreeSet<_>>(),
        r.render()
    );
}

/// ★ THE STRIKE. Read `indexed_n[J]` against `Σ|right_idx[J]|` after every round, on a shape where
/// the maintainer and the bypass sites provably write the SAME index.
///
/// ⛔ THIS TEST IS RED AT `72b894ccb`, AND THAT IS THE FINDING. D2 stood as a bounded negative —
/// "the code asymmetry is REAL; no constructed input reaches it. LATENT, not live." It is live.
/// Do not silence this by weakening the assertion; the cure is a separate strike on the engine.
///
/// ⛔ BANKED `#[ignore]`, NOT WEAKENED — the repo's RED-at-HEAD idiom
/// (`probe_undefined_builtin_resolves.rs:17`, `probe_arc255_reflection_parity.rs:70`). The
/// assertion is INTACT and the reading below is what it produces today. Un-ignore this the
/// moment the cure lands: it is the cure's acceptance test, and it is self-clearing — a green
/// run under `--ignored` means D2 is dead.
///
///     cargo nextest run --release -E 'test(right_index_counter_tracks_its_bucket_population)' --run-ignored all
///
/// The reading at `72b894ccb`: J6 `indexed_n=12` vs 18 elements (step-2 Δright appended 6
/// without advancing the mark, then the maintainer re-pushed `[6..12]`); J11 `indexed_n=6` vs
/// 12 (first-keying catch-up indexed all 6, then the maintainer re-pushed all 6 from mark 0).
/// J4 and J9 are the maintainer-only controls and hold.
#[test]
#[ignore = "RED-at-HEAD: D2 is LIVE — right_idx bypass appends do not advance indexed_n (arc 278). \
Un-ignore when the newtype cure lands; this is its acceptance test."]
fn right_index_counter_tracks_its_bucket_population() {
    let r = fire_d2(D2_WORLD, 6, "d2-two-hashjoin");
    assert_applicable(&r);

    // ── ★ THE INVARIANT, per round, per join. ────────────────────────────────────────────────
    //
    // `indexed_n[J]` is the count of right elements the maintainer believes it has indexed into
    // `right_idx[J]`; `Σ|right_idx[J]|` is what the buckets actually hold. The maintainer pushes
    // exactly one element per element it counts, so on any index only IT writes the two agree by
    // construction. A disagreement means an append bypassed the mark.
    //
    // ⛔ EVERY violation is collected before panicking, never the first one found. Two joins here
    // diverge by different bypass sites and a fail-fast assert would have named only one, making
    // the second look clean.
    let mut violations: Vec<(usize, i64, usize, usize, usize)> = Vec::new();
    for (round, rows) in &r.rounds {
        for (join, mark, elements) in rows {
            // No mark: the maintainer has never visited this index. Not a violation — but note
            // that it is the configuration in which the maintainer's NEXT visit re-pushes
            // everything the buckets already hold.
            let Some(mark) = mark else { continue };
            if mark != elements {
                violations.push((*round, *join, *mark, *elements, r.bypass_appends(*join)));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n⛔ D2 IS LIVE — the right-index high-water mark disagrees with its bucket population.\n\
         \n\
         {} (round, join, indexed_n[J], Σ|right_idx[J]|, elements a bypass site appended to J):\n\
         {:#?}\n\
         \n\
         `keyed_join_persistent` (`fire/mod.rs`) reads `indexed_n[J]` as `already` and indexes\n\
         `right_elements[already..]`. A mark BELOW the population means a bypass site in\n\
         `hash_join_delta` (`fire/pass/hash_join.rs`) appended without advancing it, so the\n\
         maintainer's next visit re-pushes elements the buckets already hold — doubled buckets,\n\
         duplicated join output. `seen_insert` dedups the derived FACTS, which is why every\n\
         end-to-end drive of this question came back clean.\n\
         \n{}",
        violations.len(),
        violations,
        r.render()
    );
}

/// ★ MUTATION 3, STANDING: the guard must REFUSE a single-HashJoin shape, not pass green over it.
///
/// A probe that quietly measures nothing on the wrong input is the defect this file exists to
/// disprove. Asserting the control's SHAPE would not prove that — it would describe the fixture.
/// This drives `assert_applicable` on the control and requires it to panic, and requires the
/// panic to be the INAPPLICABLE one rather than any other failure.
#[test]
fn a_single_hashjoin_shape_is_refused_as_inapplicable() {
    let r = fire_d2(D2_SINGLE_JOIN_WORLD, 6, "d2-single-hashjoin");
    let rendered = r.render();
    let hushed = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| assert_applicable(&r)));
    std::panic::set_hook(hushed);

    let Err(payload) = outcome else {
        panic!(
            "the applicability guard ACCEPTED a single-HashJoin shape. The strike test's verdict \
             is therefore unprotected: it would report `invariant holds` on a fire where the two \
             writers cannot meet.\n{rendered}"
        );
    };
    let msg = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic payload>")
        .to_string();
    // rune:lint(loose-assert) — the payload is a rendered per-round census whose join ids, counts
    // and round numbers all move with the fixture; the only stable thing worth asserting is WHICH
    // guard refused. An exact compare would pin the census and go red on any workload change,
    // which is the pinned-count failure this arc has already paid for twice.
    assert!(
        msg.contains("INAPPLICABLE SHAPE"),
        "the guard refused the control, but NOT for inapplicability — so this proves the control \
         is broken rather than that the guard discriminates. Panic was:\n{msg}"
    );
}

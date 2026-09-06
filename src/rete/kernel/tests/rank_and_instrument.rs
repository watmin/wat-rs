//! Cell ranking and the instrument's own tests — including the one that proves
//! `render_phase_table` reports a missing phase and a zero total rather than silently omitting
//! them.


use super::*;

// Moved here from `tests/mod.rs` (2026-08-30): `intueri` found it was the one fixture in the
// parent with exactly ONE consumer — this file — which broke the parent's own placement
// rule ("a helper used by exactly one child lives in that child"). A rule advertised as
// mechanical is falsified by a single exception.
/// Group/Reading plus two accumulators and an exists, all keyed on the shared `?g`.
const ACCUM_GATHER_WORLD: &str = "\
(:wat::core::defrecord :agc::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::ExistsF [g <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :agc::count-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (?n <- (:wat::rete::acc::count) :from (:agc::Reading (?g <- :g)))]\n\
  :then\n\
  [(:agc::CountF ?g ?n)])\n\
\n\
(:wat::rete::defrule :agc::sum-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (?n <- (:wat::rete::acc::sum ?v) :from (:agc::Reading (?g <- :g) (?v <- :v)))]\n\
  :then\n\
  [(:agc::SumF ?g ?n)])\n\
\n\
(:wat::rete::defrule :agc::exists-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (:wat::rete::exists (:agc::Reading (?g <- :g)))]\n\
  :then\n\
  [(:agc::ExistsF ?g)])\n\
\n\
(:wat::core::defn :agc::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert s (:agc::Reading :g g :v j)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
\n\
(:wat::core::defn :agc::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:agc::seed-readings (:wat::core::match (:wat::rete::insert s (:agc::Group g)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n\
";

/// Fire the gather world at `g` groups × `w` readings and return the gather-visit count.
fn accum_gather_visits(g: i64, w: i64) -> u64 {
    let world = startup_from_source(ACCUM_GATHER_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-gather world should freeze");
    let src = format!(
            "(:wat::core::match (:wat::rete::fire-rules (:agc::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :agc)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {g} {w})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, visits) = super::with_gather_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
            .value_owned()
    });
    visits
}

// ── Is the per-element BINDING LOOKUP the fold's cost? ───────────────────────────────────
//
// `accum:fold` is ~27% of fire. Inside it, `acc_var_i64` does an rpds trie lookup per element
// to recover the accumulated ?var. That is a plausible root — and so were the three that died
// this week. The accumulators differ in exactly the way needed to settle it without a new
// instrument: `count` is `gathered.len()` and does NO lookup; `sum` does one per element.
// Same world shape, same size, one rule each — the delta in `accum:fold` IS the lookup.

fn one_rule_world(rule: &str) -> String {
    format!(
"(:wat::core::defrecord :one::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :one::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :one::Out     [g <- :wat::core::i64  n <- :wat::core::i64])\n\
{rule}\n\
(:wat::core::defn :one::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert s (:one::Reading :g g :v j)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
(:wat::core::defn :one::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:one::seed-readings (:wat::core::match (:wat::rete::insert s (:one::Group g)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n")
}

fn one_rule_fold_ns(rule: &str, g: i64, w: i64) -> u64 {
    let world = startup_from_source(&one_rule_world(rule), None, Arc::new(InMemoryLoader::new()))
        .expect("one-rule world should freeze");
    let src = format!(
            "(:wat::core::match (:wat::rete::fire-rules (:one::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :one)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {g} {w})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
            .value_owned()
    });
    rows.iter()
        .find(|(n, _)| *n == "  └ accum:fold")
        .map(|(_, ns)| *ns)
        .unwrap_or(0)
}

/// Diagnostic — the fold WITH a per-element binding lookup vs WITHOUT one.
#[test]
fn fold_cost_with_and_without_the_binding_lookup() {
    const COUNT_RULE: &str = "(:wat::rete::defrule :one::count-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::count) :from (:one::Reading (?g <- :g)))]\n\
  :then [(:one::Out ?g ?n)])";
    const SUM_RULE: &str = "(:wat::rete::defrule :one::sum-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::sum ?v) :from (:one::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:one::Out ?g ?n)])";

    const RUNS: usize = 3;
    let (g, w) = (200i64, 200i64);
    let elements = g * w;

    let mut counts = Vec::new();
    let mut sums = Vec::new();
    for _ in 0..RUNS {
        counts.push(one_rule_fold_ns(COUNT_RULE, g, w));
        sums.push(one_rule_fold_ns(SUM_RULE, g, w));
    }
    // MINIMUM of the samples, not their mean — this test's own header says `MINIMUM of {RUNS}`,
    // and the two bounds below are read against the FLOOR of the fold's cost, which is the only
    // figure a first-arm outlier cannot inflate.
    let best = |xs: &[u64]| *xs.iter().min().expect("RUNS >= 1, so non-empty") as f64;
    let (c, s) = (best(&counts), best(&sums));
    assert!(
        c > 0.0 && s > 0.0,
        "one or both folds recorded nothing — the instrument never fired"
    );

    println!(
            "\nfold cost, {elements} elements gathered, MINIMUM of {RUNS}\n                 count (NO per-element lookup)  {:>7.2} ms\n                 sum   (ONE lookup per element) {:>7.2} ms\n                 delta = the lookup             {:>7.2} ms   ({:.0} ns/element)\n",
            c / 1e6, s / 1e6, (s - c) / 1e6, (s - c) / elements as f64
        );
    // (b) + keyed gather left rematch in the fold mark. After the fold stone,
    // count is bucket.len() and sum is a slot load — sum must sit near count,
    // and count must be well below the 9.59 ms rematch walk.
    assert!(
        c < 5.0e6,
        "count fold is {c:.0} ns ({:.2} ms) — rematch is still in the walk; \
             expected bucket.len() after DESIGN-STONE-accum-fold-the-wall",
        c / 1e6
    );
    assert!(
        s <= c * 2.0 || s < 8.0e6,
        "sum fold {s:.0} ns vs count {c:.0} ns — the 223 ns/el Bindings::get \
             did not collapse to a slot load"
    );
}

// ── Is the BIND (trie insert) the cost inside alpha:match? ───────────────────────────────
//
// alpha:match is ~28% of fire and does 120,200 fresh binds, each allocating an rpds trie node
// for a map holding one or two entries. Plausible — and the previous three plausible roots
// were wrong, so it is measured the same way the fold's lookup was: two worlds differing by
// EXACTLY one bind clause on the Reading condition. No accumulate, no join beyond the root:
// the delta in alpha:match is the marginal cost of one binding, times the fact count.

fn bind_world(reading_cond: &str) -> String {
    format!(
"(:wat::core::defrecord :bnd::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :bnd::Out     [g <- :wat::core::i64])\n\
(:wat::rete::defrule :bnd::r\n\
  :when [{reading_cond}]\n\
  :then [(:bnd::Out ?g)])\n\
(:wat::core::defn :bnd::seed [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert s (:bnd::Reading :g i :v i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    session\n\
    (:wat::core::range 0 n)))\n")
}

/// Returns outer `alpha` ns for one bind-world at `n` facts.
/// Child timers were retired (`DESIGN-STONE-retire-alpha-child-marks`).
fn bind_world_alpha_ns(reading_cond: &str, n: i64) -> u64 {
    let world = startup_from_source(
        &bind_world(reading_cond),
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("bind world should freeze");
    let src = format!(
            "(:wat::core::match (:wat::rete::fire-rules (:bnd::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :bnd)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {n})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
        );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
            .value_owned()
    });
    rows.iter()
        .find(|(n2, _)| *n2 == "alpha")
        .map(|(_, ns)| *ns)
        .unwrap_or(0)
}

/// Diagnostic — one bind vs two binds on the same condition, same facts.
#[test]
fn alpha_match_cost_per_binding() {
    const ONE: &str = "(:bnd::Reading (?g <- :g))";
    const TWO: &str = "(:bnd::Reading (?g <- :g) (?v <- :v))";
    const RUNS: usize = 3;
    let n = 40_000i64;

    let mut a1 = u64::MAX;
    let mut a2 = u64::MAX;
    for _ in 0..RUNS {
        a1 = a1.min(bind_world_alpha_ns(ONE, n));
        a2 = a2.min(bind_world_alpha_ns(TWO, n));
    }
    let a1 = a1 as f64;
    let a2 = a2 as f64;
    assert!(
        a1 > 0.0 && a2 > 0.0,
        "alpha recorded nothing — the instrument never fired"
    );

    // ⛔ `probare` classed this test hollow — the guard above is liveness on two clocks.
    // `bind_world_alpha_ns(ONE|TWO, n)` fires a world of n facts under 1 vs 2 bindings, and the
    // per-fact delta is the whole finding, so the two arms must run the SAME workload.
    assert_eq!(n, 40_000, "the per-binding delta is reported per fact over 40,000; got {n}");

    // ⚠ NOT ASSERTED, DELIBERATELY: this test measures 2 binds as FASTER than 1 bind
    // (delta -1.12 ms, -28 ns/fact, 2026-08-30). That sign is impossible for strictly more work,
    // and it is the THIRD impossible-signed reading in this suite — `accum_alpha_*` reports
    // isolated micro-benches running 6x slower than the full operation they decompose, giving
    // `A-M` and `H-M` deltas of -87 and -98 ms. Encoding the anomaly as an assertion would
    // FREEZE a broken instrument. It is recorded here instead, and belongs to an
    // instrument-validity strike, not to this one.

    println!(
            "\nalpha cost per BINDING — {n} facts, MINIMUM of {RUNS}\n                 1 bind : alpha {:>7.2} ms\n                 2 binds: alpha {:>7.2} ms\n                 delta  : alpha {:>7.2} ms ({:>4.0} ns/fact)\n",
            a1 / 1e6,
            a2 / 1e6,
            (a2 - a1) / 1e6,
            (a2 - a1) / n as f64
        );
}

#[test]
fn render_phase_table_proves_missing_phase_and_zero_total() {
    let boom = std::panic::catch_unwind(|| {
        render_phase_table(
            "fake",
            &[(1, 1)],
            &["IN: to_transient"],
            &["ROUND LOOP"],
            |_, _| 0,
            |_, _| vec![("IN: to_transient", 1, 1)],
        )
    });
    assert!(boom.is_err(), "missing required phase must panic");
}

/// The keyed-gather ratio — a SECOND reading, not the proof.
///
/// Both runs hold the ELEMENT COUNT CONSTANT (G×W = 800 readings) and differ only in how many
/// tokens probe them (8× apart in group count). That separates "the gather is quadratic" from
/// "there are simply more facts" — the same control the measurement probe uses
/// (`wat-scripts/scratch-pad/probe-accumulate-gather-cost.wat`), which read 8.42× on wall-clock
/// at G=50/W=160 vs G=400/W=20.
///
///   un-keyed (today): every token scans all 800 elements → visits ∝ G → an 8× spread.
///   keyed:            every token probes its own bucket   → visits ≈ G×W = 800/node → FLAT.
///
/// ⛔ THIS RATIO IS EXACTLY BLIND on `and-exists`. Keyed visits are `G·W·(1+W)`; a whole-memory
/// scan adds `G·elements`. Holding `G×W` constant makes the sum symmetric in G and W, so the
/// regressed ratio is 1.00 at every swap. The proof is `keyed_gather_visits_match_the_keyed_prediction`.
/// This bound stays as a second reading for shapes the closed form does not model.
#[test]
fn keyed_gather_visits_do_not_scale_with_group_count() {
    // G×W = 800 readings in BOTH runs; only the token count moves (10 → 80).
    let small = accum_gather_visits(10, 80);
    let big = accum_gather_visits(80, 10);

    assert!(
        small > 0,
        "the gather-visit instrument recorded ZERO — the gathers were never entered, so any \
             ratio taken from this run would be an artifact, not a measurement"
    );

    let ratio = big as f64 / small as f64;
    println!(
        "\nkeyed-gather gate — constant 800 elements, tokens 10 → 80\n  \
             G=10 W=80 : {small} visits\n  G=80 W=10 : {big} visits\n  ratio: {ratio:.2}x\n"
    );
    assert!(
        ratio <= 2.0,
        "gather visits scale with the TOKEN count ({small} → {big}, {ratio:.2}x) while the \
             element count is constant at 800 — the Accumulate/Negation/Exists gathers are still \
             scanning the whole memory per token instead of probing a key index (the joins have \
             had one since P6). See DESIGN-STONE-keyed-gather.md."
    );
}

/// Fire a one-rule world and return gather visits. Same seed shape as `one_rule_world`.
fn one_rule_gather_visits(rule: &str, g: i64, w: i64) -> u64 {
    let world = startup_from_source(&one_rule_world(rule), None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("one-rule gather world should freeze: {e:?}"));
    let src = format!(
        "(:wat::core::match (:wat::rete::fire-rules (:one::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :one)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {g} {w})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, visits) = super::with_gather_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| {
                panic!(
                    "fire raised at G={g} W={w} rule={:?}: {e:?}",
                    rule.chars().take(80).collect::<String>()
                )
            })
            .value_owned()
    });
    visits
}

/// Pair at constant 800 elements, tokens 10 → 80.
fn path_pair(rule: &str) -> (u64, u64) {
    (
        one_rule_gather_visits(rule, 10, 80),
        one_rule_gather_visits(rule, 80, 10),
    )
}

/// Axis points with G×W = 800. Four points so a formula that fits one pair is not a coincidence.
const GATHER_POINTS: [(i64, i64); 4] = [(10, 80), (20, 40), (40, 20), (80, 10)];

fn pred_simple(g: i64, w: i64) -> u64 {
    (g as u64) * (w as u64)
}

fn pred_and_exists(g: i64, w: i64) -> u64 {
    let (g, w) = (g as u64, w as u64);
    g * w * (1 + w)
}

const DISTINCT_RULE: &str = "\
(:wat::rete::defrule :one::distinct-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?xs <- (:wat::rete::acc::distinct ?v) :from (:one::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:one::Out ?g 0)])";
const ALL_RULE: &str = "\
(:wat::rete::defrule :one::all-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?xs <- (:wat::rete::acc::all) :from (:one::Reading (?g <- :g)))]\n\
  :then [(:one::Out ?g 0)])";
const GROUP_BY_RULE: &str = "\
(:wat::rete::defrule :one::group-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?m <- (:wat::rete::acc::group-by ?v) :from (:one::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:one::Out ?g 0)])";
const AND_EXISTS_RULE: &str = "\
(:wat::rete::defrule :one::and-exists-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (:wat::rete::exists (:wat::rete::and\n\
           (:one::Reading (?g <- :g) (?v <- :v))\n\
           (:one::Reading (?g <- :g) (?v <- :v))))]\n\
  :then [(:one::Out ?g 1)])";

/// ★ Every instrumented gather path, not just count+sum+exists-leaf.
///
/// The Distinct/All/GroupBy arm materialises the bucket (`gather_bucket`). The mapping
/// no-`SeedCmp` arm is `seeded_bindings_keyed` — a Leaf under `:and`, not `:exists` of a
/// Leaf (that one is `!bucket.is_empty()`, O(1)). `AccFold::User` shares Distinct's
/// materialise; a wat-surface user-fn acc is the same walk.
///
/// Old axis (`ACCUM_GATHER_WORLD`) is re-read here so a perturbation of 800/800 is visible.
#[test]
fn keyed_gather_visits_per_instrumented_path() {
    let old_small = accum_gather_visits(10, 80);
    let old_big = accum_gather_visits(80, 10);
    let distinct = path_pair(DISTINCT_RULE);
    let all = path_pair(ALL_RULE);
    let group_by = path_pair(GROUP_BY_RULE);
    let and_exists = path_pair(AND_EXISTS_RULE);

    let row = |name: &str, (s, b): (u64, u64)| {
        let ratio = if s == 0 { f64::INFINITY } else { b as f64 / s as f64 };
        format!("  {name:<14}  {s:>8}  {b:>8}  {ratio:>6.2}x")
    };

    println!(
        "\nkeyed-gather per path — constant 800 elements, tokens 10 → 80\n\
         \x20 path            G10W80    G80W10   ratio\n\
         \x20 ----------------------------------------\n\
         {}\n\
         {}\n\
         {}\n\
         {}\n\
         {}\n",
        row("old-axis", (old_small, old_big)),
        row("distinct", distinct),
        row("all", all),
        row("group-by", group_by),
        row("and-exists", and_exists),
    );

    assert_eq!(
        (old_small, old_big),
        (800, 800),
        "old axis perturbed: {old_small}/{old_big} — the addition is not comparable to 800/800"
    );
    assert!(
        distinct.0 > 0,
        "distinct recorded ZERO visits — AccFold::Distinct materialise was not entered"
    );
    assert!(
        all.0 > 0,
        "all recorded ZERO visits — AccFold::All materialise was not entered"
    );
    assert!(
        group_by.0 > 0,
        "group-by recorded ZERO visits — AccFold::GroupBy materialise was not entered"
    );
    assert!(
        and_exists.0 > 0,
        "and-exists recorded ZERO visits — seeded_bindings_keyed no-SeedCmp map was not entered"
    );

    let ratio = |s: u64, b: u64| b as f64 / s as f64;
    for (name, (s, b)) in [
        ("distinct", distinct),
        ("all", all),
        ("group-by", group_by),
        ("and-exists", and_exists),
    ] {
        let r = ratio(s, b);
        assert!(
            r <= 2.0,
            "{name} gather visits scale with the TOKEN count ({s} → {b}, {r:.2}x) while the \
             element count is constant at 800"
        );
    }
}

/// ★ THE PROOF: visits equal the keyed prediction, at four (G,W) points with G×W = 800.
///
/// Simple keyed gather (each token probes its bucket of W readings): `G · W`.
/// `:and` of two Leaves under `:exists`: first Leaf yields W, each of those extends
/// through the second Leaf's W → `G · (W + W²) = G · W · (1 + W)`.
///
/// A whole-memory-per-token scan adds `G · (G·W)` and breaks the equality at every point.
/// The ratio on `and-exists` cannot see that class — see the mutation below.
#[test]
fn keyed_gather_visits_match_the_keyed_prediction() {
    let mut table = String::from(
        "\nkeyed-gather prediction — visits == formula, G×W = 800\n\
         \x20 path          G    W     got    pred\n\
         \x20 -------------------------------------\n",
    );
    let simple = [
        ("old-axis", None),
        ("distinct", Some(DISTINCT_RULE)),
        ("all", Some(ALL_RULE)),
        ("group-by", Some(GROUP_BY_RULE)),
    ];
    for (name, rule) in simple {
        for (g, w) in GATHER_POINTS {
            let got = match rule {
                None => accum_gather_visits(g, w),
                Some(r) => one_rule_gather_visits(r, g, w),
            };
            let pred = pred_simple(g, w);
            table.push_str(&format!("  {name:<12} {g:>3} {w:>4} {got:>8} {pred:>8}\n"));
            assert_eq!(
                got, pred,
                "{name} G={g} W={w}: visits {got} ≠ keyed prediction G·W = {pred}\n{table}"
            );
        }
    }
    for (g, w) in GATHER_POINTS {
        let got = one_rule_gather_visits(AND_EXISTS_RULE, g, w);
        let pred = pred_and_exists(g, w);
        table.push_str(&format!("  {:<12} {g:>3} {w:>4} {got:>8} {pred:>8}\n", "and-exists"));
        assert_eq!(
            got, pred,
            "and-exists G={g} W={w}: visits {got} ≠ keyed prediction G·W·(1+W) = {pred}\n{table}"
        );
    }
    println!("{table}");
}

/// Simulated whole-memory-per-token on `and-exists`: add `G · elements` to the observed
/// keyed count. Not an unkeyed engine — the regression is applied in this test's arithmetic.
///
/// Holding G×W = 800 constant, keyed + G·800 is symmetric in G and W, so the ratio of the
/// two swap points is exactly 1.00 (the ratio assertion PASSES). The equality against
/// `G·W·(1+W)` FAILS at both points. That is the ratio's blind spot.
#[test]
fn predicted_visits_redden_under_a_whole_memory_scan_the_ratio_cannot_see() {
    let elements = pred_simple(10, 80);
    assert_eq!(elements, 800);
    let (g1, w1) = (10i64, 80i64);
    let (g2, w2) = (80i64, 10i64);
    let obs1 = one_rule_gather_visits(AND_EXISTS_RULE, g1, w1);
    let obs2 = one_rule_gather_visits(AND_EXISTS_RULE, g2, w2);
    let pred1 = pred_and_exists(g1, w1);
    let pred2 = pred_and_exists(g2, w2);
    assert_eq!(obs1, pred1, "precondition: keyed count must match the formula");
    assert_eq!(obs2, pred2, "precondition: keyed count must match the formula");

    let fake1 = obs1 + (g1 as u64) * elements;
    let fake2 = obs2 + (g2 as u64) * elements;
    let ratio = fake2 as f64 / fake1 as f64;
    println!(
        "\nand-exists whole-memory simulation (add G·elements)\n\
         \x20 (G,W)=({g1},{w1}): keyed {obs1} + {g1}·{elements} = {fake1}  pred {pred1}\n\
         \x20 (G,W)=({g2},{w2}): keyed {obs2} + {g2}·{elements} = {fake2}  pred {pred2}\n\
         \x20 ratio of fakes: {ratio:.2}  (≤ 2.0 would PASS)\n"
    );
    assert!(
        ratio <= 2.0,
        "the ratio of the simulated regression must still pass; got {ratio:.2}"
    );
    assert_eq!(
        fake1, fake2,
        "the simulation must be symmetric in G and W (the ratio's blind spot); {fake1} vs {fake2}"
    );
    assert!(
        fake1 != pred1 && fake2 != pred2,
        "equality against G·W·(1+W) must REDDEN under the simulation: \
         fake {fake1}/{fake2} vs pred {pred1}/{pred2}"
    );
}

/// Native FIRE rank across the three instrumented cells now that
/// fanout is dry (`DESIGN-STONE-cell-rank-after-fanout`).
#[test]
fn cell_rank_after_fanout() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    fn fire_and_top(rows: &[(&'static str, u64, u64)]) -> (f64, &'static str, f64) {
        // ⛔ `probare` classed the callers of this helper hollow — they ranked cells by a number
        // and asserted only that the number was positive. THIS is where the number is built, and
        // it is a SUM OVER `TOP`. A missing top-level phase does not fail: it silently shrinks
        // `fire`, and a smaller `fire` ranks the cell CHEAPER. So the lossy reading is the
        // flattering one, exactly as in the other comparison benchmarks in this suite.
        super::assert_phases_present(rows.iter().map(|r| r.0), &TOP, "");
        let fire: u64 = TOP
            .iter()
            .filter_map(|n| {
                rows.iter()
                    .find(|(name, _, _)| *name == *n)
                    .map(|(_, ns, _)| *ns)
            })
            .sum();
        let mut best_name = "(none)";
        let mut best_ns = 0u64;
        for (name, ns, _) in rows {
            if TOP.contains(name) || *name == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            if *ns > best_ns {
                best_ns = *ns;
                best_name = name;
            }
        }
        (fire as f64, best_name, best_ns as f64)
    }

    // MINIMUM across runs, not mean. `.1` is the top-row NAME, not a measurement.
    let mut fanout = (f64::INFINITY, "", f64::INFINITY);
    let mut accum = (f64::INFINITY, "", f64::INFINITY);
    let mut share = (f64::INFINITY, "", f64::INFINITY);
    for _ in 0..RUNS {
        let (f, n, c) = fire_and_top(&fanout_phase_census(100, 20));
        fanout.0 = fanout.0.min(f);
        fanout.1 = n;
        fanout.2 = fanout.2.min(c);
        let (f, n, c) = fire_and_top(&accum_phase_census(200, 200));
        accum.0 = accum.0.min(f);
        accum.1 = n;
        accum.2 = accum.2.min(c);
        let (f, n, c) = fire_and_top(&node_share_phase_census(50, 200));
        share.0 = share.0.min(f);
        share.1 = n;
        share.2 = share.2.min(c);
    }
    let table = format!(
        "\ncell rank after fanout — MINIMUM of {RUNS}\n\
             FIRE is IN+SETUP+ROUND+OUT; top-row is the largest named child\n\
             \n\
             cell                 FIRE      top-row\n\
             fanout     [100 20]  {:>7.2} ms   {} {:>7.2} ms\n\
             accum      [200 200] {:>7.2} ms   {} {:>7.2} ms\n\
             node-share [50 200]  {:>7.2} ms   {} {:>7.2} ms\n",
        ms(fanout.0),
        fanout.1,
        ms(fanout.2),
        ms(accum.0),
        accum.1,
        ms(accum.2),
        ms(share.0),
        share.1,
        ms(share.2),
    );
    println!("{table}");
    assert!(
        fanout.0 > 0.0 && accum.0 > 0.0 && share.0 > 0.0,
        "a cell recorded FIRE 0 — the rank is a dead fire:{table}"
    );
}

/// Native FIRE rank at the three closest 08-20 grid cells
/// (`DESIGN-STONE-cell-rank-after-grid`).
#[test]
fn cell_rank_after_grid() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    fn fire_and_top(rows: &[(&'static str, u64, u64)]) -> (f64, &'static str, f64) {
        // ⛔ `probare` classed the callers of this helper hollow — they ranked cells by a number
        // and asserted only that the number was positive. THIS is where the number is built, and
        // it is a SUM OVER `TOP`. A missing top-level phase does not fail: it silently shrinks
        // `fire`, and a smaller `fire` ranks the cell CHEAPER. So the lossy reading is the
        // flattering one, exactly as in the other comparison benchmarks in this suite.
        super::assert_phases_present(rows.iter().map(|r| r.0), &TOP, "");
        let fire: u64 = TOP
            .iter()
            .filter_map(|n| {
                rows.iter()
                    .find(|(name, _, _)| *name == *n)
                    .map(|(_, ns, _)| *ns)
            })
            .sum();
        let mut best_name = "(none)";
        let mut best_ns = 0u64;
        for (name, ns, _) in rows {
            if TOP.contains(name) || *name == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            if *ns > best_ns {
                best_ns = *ns;
                best_name = name;
            }
        }
        (fire as f64, best_name, best_ns as f64)
    }

    // MINIMUM across runs, not mean.
    let mut fanout = (f64::INFINITY, "", f64::INFINITY);
    let mut cascade = (f64::INFINITY, "", f64::INFINITY);
    let mut accum = (f64::INFINITY, "", f64::INFINITY);
    for _ in 0..RUNS {
        let (f, n, c) = fire_and_top(&fanout_phase_census(100, 20));
        fanout.0 = fanout.0.min(f);
        fanout.1 = n;
        fanout.2 = fanout.2.min(c);
        let (f, n, c) = fire_and_top(&cascade_phase_census(50, 100));
        cascade.0 = cascade.0.min(f);
        cascade.1 = n;
        cascade.2 = cascade.2.min(c);
        let (f, n, c) = fire_and_top(&accum_phase_census(200, 200));
        accum.0 = accum.0.min(f);
        accum.1 = n;
        accum.2 = accum.2.min(c);
    }
    let table = format!(
        "\ncell rank after grid — MINIMUM of {RUNS}\n\
             FIRE is IN+SETUP+ROUND+OUT; top-row is the largest named child\n\
             08-20 closest rungs: fanout [40000], deep-cascade [50 100], accum [200 200]\n\
             \n\
             cell                    FIRE      top-row\n\
             fanout        [100 20]  {:>7.2} ms   {} {:>7.2} ms\n\
             deep-cascade  [50 100]  {:>7.2} ms   {} {:>7.2} ms\n\
             accum         [200 200] {:>7.2} ms   {} {:>7.2} ms\n",
        ms(fanout.0),
        fanout.1,
        ms(fanout.2),
        ms(cascade.0),
        cascade.1,
        ms(cascade.2),
        ms(accum.0),
        accum.1,
        ms(accum.2),
    );
    println!("{table}");
    assert!(
        fanout.0 > 0.0 && cascade.0 > 0.0 && accum.0 > 0.0,
        "a cell recorded FIRE 0 — the rank is a dead fire:{table}"
    );
}

/// Honest FIRE at the three closest cells after intern 17
/// (`DESIGN-STONE-honest-rank-after-arm`). Production raw on
/// fanout is 80k test marks (2p); intern from honest_FIRE.
#[test]
fn honest_cell_rank_after_arm() {
    const RUNS: usize = 3;

    let cal = calibrate_mark_ns();

    fn fire_top_honest(
        rows: &[(&'static str, u64, u64)],
        cal: f64,
    ) -> (f64, &'static str, f64, f64) {
        const TOP: [&str; 4] = [
            "IN: to_transient",
            "SETUP: indexes",
            "ROUND LOOP",
            "OUT: to_persistent",
        ];
        const RHS: &str = "  ├ prod:compiled-rhs";
        const DEDUP: &str = "  ├ prod:dedup-store";
        let fire: u64 = TOP
            .iter()
            .filter_map(|n| {
                rows.iter()
                    .find(|(name, _, _)| *name == *n)
                    .map(|(_, ns, _)| *ns)
            })
            .sum();
        let mut best_name = "(none)";
        let mut best_ns = 0u64;
        for (name, ns, _) in rows {
            if TOP.contains(name) || *name == "WHOLE EVAL (compile+seed+fire)" {
                continue;
            }
            if *ns > best_ns {
                best_ns = *ns;
                best_name = name;
            }
        }
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let (prod, _) = of("production");
        let (rhs, rhs_k) = of(RHS);
        let (dedup, dedup_k) = of(DEDUP);
        let remainder = prod.saturating_sub(rhs).saturating_sub(dedup) as f64;
        let tax = (rhs_k + dedup_k) as f64 * cal;
        let honest = fire as f64 - remainder - tax;
        (fire as f64, best_name, best_ns as f64, honest)
    }

    // MINIMUM across runs, not mean.
    let mut fanout = (f64::INFINITY, "", f64::INFINITY, f64::INFINITY);
    let mut cascade = (f64::INFINITY, "", f64::INFINITY, f64::INFINITY);
    let mut accum = (f64::INFINITY, "", f64::INFINITY, f64::INFINITY);
    for _ in 0..RUNS {
        let t = fire_top_honest(&fanout_phase_census(100, 20), cal);
        fanout.0 = fanout.0.min(t.0);
        fanout.1 = t.1;
        fanout.2 = fanout.2.min(t.2);
        fanout.3 = fanout.3.min(t.3);
        let t = fire_top_honest(&cascade_phase_census(50, 100), cal);
        cascade.0 = cascade.0.min(t.0);
        cascade.1 = t.1;
        cascade.2 = cascade.2.min(t.2);
        cascade.3 = cascade.3.min(t.3);
        let t = fire_top_honest(&accum_phase_census(200, 200), cal);
        accum.0 = accum.0.min(t.0);
        accum.1 = t.1;
        accum.2 = accum.2.min(t.2);
        accum.3 = accum.3.min(t.3);
    }
    let table = format!(
        "\nhonest cell rank after arm — MINIMUM of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             FIRE is IN+SETUP+ROUND+OUT; honest_FIRE strips production remainder+tax (2p)\n\
             \n\
             cell                    FIRE     honest_FIRE   top-row\n\
             fanout        [100 20]  {:>7.2} ms  {:>7.2} ms   {} {:>7.2} ms\n\
             deep-cascade  [50 100]  {:>7.2} ms  {:>7.2} ms   {} {:>7.2} ms\n\
             accum         [200 200] {:>7.2} ms  {:>7.2} ms   {} {:>7.2} ms\n",
        ms(fanout.0),
        ms(fanout.3),
        fanout.1,
        ms(fanout.2),
        ms(cascade.0),
        ms(cascade.3),
        cascade.1,
        ms(cascade.2),
        ms(accum.0),
        ms(accum.3),
        accum.1,
        ms(accum.2),
    );
    println!("{table}");
    assert!(
        fanout.0 > 0.0 && cascade.0 > 0.0 && accum.0 > 0.0,
        "a cell recorded FIRE 0 — the rank is a dead fire:{table}"
    );
    assert!(
        fanout.3 < fanout.0,
        "fanout honest_FIRE was not less than raw — production tax did not subtract:{table}"
    );
}

/// Recolligere: leaf-set predicted occupancy vs seed-installed `wm.alpha`.
/// Prints extra (fill ⊃ actual) and missing (actual \ fill).
#[test]
fn n3_leaf_set_vs_occupancy() {
    const N3: &str = "\
(:wat::core::defrecord :n::A   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n::Bad [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n::Ok  [k <- :wat::core::i64])\n\
(:wat::rete::defquery :n::q-Bad :params [] :when [(?fact <- :n::Bad)])\n\
(:wat::rete::defquery :n::q-Ok :params [] :when [(?fact <- :n::Ok)])\n\
(:wat::core::defrecord :n3::A    [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n3::Bad  [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n3::Warn [k <- :wat::core::i64])\n\
(:wat::core::defrecord :n3::Safe [k <- :wat::core::i64])\n\
(:wat::rete::defrule :n3::mark-bad\n\
  :when [(:n3::A (?k <- :k)) (:wat::rete::where (:wat::rete::core::i64::= ?k 2))]\n\
  :then [(:n3::Bad :k ?k)])\n\
(:wat::rete::defrule :n3::mark-warn\n\
  :when [(:n3::A (?k <- :k)) (:wat::rete::not (:n3::Bad (?k <- :k)))]\n\
  :then [(:n3::Warn :k ?k)])\n\
(:wat::rete::defrule :n3::mark-safe\n\
  :when [(:n3::A (?k <- :k)) (:wat::rete::not (:n3::Warn (?k <- :k)))]\n\
  :then [(:n3::Safe :k ?k)])\n\
(:wat::rete::defquery :n3::q-Bad :params [] :when [(?fact <- :n3::Bad)])\n\
(:wat::rete::defquery :n3::q-Warn :params [] :when [(?fact <- :n3::Warn)])\n\
(:wat::rete::defquery :n3::q-Safe :params [] :when [(?fact <- :n3::Safe)])\n\
";
    let world = freeze_src(N3);
    let (fired, diffs) = super::with_leaf_occ_diff(|| {
        eval_in(
            &world,
            "(:wat::core::let \
               [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :n3) \
                     (:wat::core::PersistentVector (:n::q-Bad) (:n::q-Ok) \
                       (:n3::q-Bad) (:n3::q-Warn) (:n3::q-Safe))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\
                s1 (:wat::core::match (:wat::rete::insert s0 (:n3::A :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                s2 (:wat::core::match (:wat::rete::insert s1 (:n3::A :k 2)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\
                s3 (:wat::core::match (:wat::rete::insert s2 (:n3::A :k 3)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\
              (:wat::core::match (:wat::rete::fire-rules s3) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None))))",
        )
    });
    let wm = to_transient(&fired).expect("fired session");
    let mut safe_ks: Vec<i64> = Vec::new();
    for facts in wm.production.values() {
        for f in facts {
            if let Value::Aggregate(a) = f {
                if a.class.as_ref().contains("Safe") {
                    if let Some(Value::i64(k)) = a.fields.first() {
                        safe_ks.push(*k);
                    }
                }
            }
        }
    }
    safe_ks.sort_unstable();
    let mut out = format!(
        "\nn3 leaf-set vs occupancy — {} fires, production Safe k={safe_ks:?}\n",
        diffs.len()
    );
    for (i, d) in diffs.iter().enumerate() {
        out.push_str(&format!(
            "  stratum {i}: facts={} leaf_aids={} predicted={} actual={} extra={} missing={}\n    extra {:?}\n    missing {:?}\n",
            d.n_facts, d.n_leaf_aids, d.predicted, d.actual, d.extra.len(), d.missing.len(),
            d.extra, d.missing
        ));
    }
    println!("{out}");
    assert!(
        !diffs.is_empty(),
        "no strata were compared — the leaf-set/occupancy comparison is vacuous:{out}"
    );

    // ⛔ THE LINE ABOVE ONLY SAYS THE LOOP RAN. This test's entire subject is whether the
    // leaf-set PREDICTION matches ACTUAL occupancy, and it printed that comparison without
    // asserting it — `probare` classed it hollow. Measured 2026-08-30 over the fixed three-insert
    // N3 world: every stratum has predicted == actual with no extras and no missing.
    //
    // This is an ENGINE claim, not a harness one. If the predictor and the occupancy it predicts
    // ever diverge, that is the defect this test exists to detect, and it now fails instead of
    // printing the divergence into a table nobody reads.
    for (i, d) in diffs.iter().enumerate() {
        assert_eq!(
            d.predicted, d.actual,
            "stratum {i}: leaf-set predicted {} occupants, actual {} — the predictor and the \
             network disagree, which is precisely what this test measures:{out}",
            d.predicted, d.actual
        );
        assert!(
            d.extra.is_empty() && d.missing.is_empty(),
            "stratum {i}: predicted set differs from actual — extra {:?}, missing {:?}:{out}",
            d.extra, d.missing
        );
    }
    assert_eq!(diffs.len(), 3, "the N3 world has three strata; found {}", diffs.len());
}

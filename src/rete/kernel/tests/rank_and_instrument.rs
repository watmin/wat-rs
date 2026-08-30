//! Cell ranking and the instrument's own tests — including the one that proves
//! `render_phase_table` reports a missing phase and a zero total rather than silently omitting
//! them.


use super::*;

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
    let mean = |xs: &[u64]| xs.iter().sum::<u64>() as f64 / xs.len() as f64;
    let (c, s) = (mean(&counts), mean(&sums));
    assert!(
        c > 0.0 && s > 0.0,
        "one or both folds recorded nothing — the instrument never fired"
    );

    println!(
            "\nfold cost, {elements} elements gathered, mean of {RUNS}\n                 count (NO per-element lookup)  {:>7.2} ms\n                 sum   (ONE lookup per element) {:>7.2} ms\n                 delta = the lookup             {:>7.2} ms   ({:.0} ns/element)\n",
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

    let mut a1 = 0u64;
    let mut a2 = 0u64;
    for _ in 0..RUNS {
        a1 += bind_world_alpha_ns(ONE, n);
        a2 += bind_world_alpha_ns(TWO, n);
    }
    let r = RUNS as f64;
    let a1 = a1 as f64 / r;
    let a2 = a2 as f64 / r;
    assert!(
        a1 > 0.0 && a2 > 0.0,
        "alpha recorded nothing — the instrument never fired"
    );

    println!(
            "\nalpha cost per BINDING — {n} facts, mean of {RUNS}\n                 1 bind : alpha {:>7.2} ms\n                 2 binds: alpha {:>7.2} ms\n                 delta  : alpha {:>7.2} ms ({:>4.0} ns/fact)\n",
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

/// The keyed-gather gate — RED until the Accumulate/Negation/Exists gathers are keyed.
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
/// What would turn this red — the R59 question, answered before the assertion was written:
///   (a) the instrument recording nothing (`small == 0`) — asserted separately, because a
///       silent zero would make the ratio 0/0 and "pass" while measuring nothing at all;
///   (b) a gather that still walks the whole element memory per token — the defect under test;
///   (c) a keyed gather whose buckets are wrong in a way that re-scans (e.g. an empty key
///       tuple degenerating every element into one bucket for a workload that DOES share vars).
///
/// It cannot pass by luck or by machine speed: it counts examinations, not nanoseconds.
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

    let mut fanout = (0.0, "", 0.0);
    let mut accum = (0.0, "", 0.0);
    let mut share = (0.0, "", 0.0);
    for _ in 0..RUNS {
        let (f, n, c) = fire_and_top(&fanout_phase_census(100, 20));
        fanout.0 += f;
        fanout.1 = n;
        fanout.2 += c;
        let (f, n, c) = fire_and_top(&accum_phase_census(200, 200));
        accum.0 += f;
        accum.1 = n;
        accum.2 += c;
        let (f, n, c) = fire_and_top(&node_share_phase_census(50, 200));
        share.0 += f;
        share.1 = n;
        share.2 += c;
    }
    let r = RUNS as f64;
    fanout.0 /= r;
    fanout.2 /= r;
    accum.0 /= r;
    accum.2 /= r;
    share.0 /= r;
    share.2 /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\ncell rank after fanout — mean of {RUNS}\n\
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

    let mut fanout = (0.0, "", 0.0);
    let mut cascade = (0.0, "", 0.0);
    let mut accum = (0.0, "", 0.0);
    for _ in 0..RUNS {
        let (f, n, c) = fire_and_top(&fanout_phase_census(100, 20));
        fanout.0 += f;
        fanout.1 = n;
        fanout.2 += c;
        let (f, n, c) = fire_and_top(&cascade_phase_census(50, 100));
        cascade.0 += f;
        cascade.1 = n;
        cascade.2 += c;
        let (f, n, c) = fire_and_top(&accum_phase_census(200, 200));
        accum.0 += f;
        accum.1 = n;
        accum.2 += c;
    }
    let r = RUNS as f64;
    fanout.0 /= r;
    fanout.2 /= r;
    cascade.0 /= r;
    cascade.2 /= r;
    accum.0 /= r;
    accum.2 /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\ncell rank after grid — mean of {RUNS}\n\
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

    let mut fanout = (0.0, "", 0.0, 0.0);
    let mut cascade = (0.0, "", 0.0, 0.0);
    let mut accum = (0.0, "", 0.0, 0.0);
    for _ in 0..RUNS {
        let t = fire_top_honest(&fanout_phase_census(100, 20), cal);
        fanout.0 += t.0;
        fanout.1 = t.1;
        fanout.2 += t.2;
        fanout.3 += t.3;
        let t = fire_top_honest(&cascade_phase_census(50, 100), cal);
        cascade.0 += t.0;
        cascade.1 = t.1;
        cascade.2 += t.2;
        cascade.3 += t.3;
        let t = fire_top_honest(&accum_phase_census(200, 200), cal);
        accum.0 += t.0;
        accum.1 = t.1;
        accum.2 += t.2;
        accum.3 += t.3;
    }
    let r = RUNS as f64;
    fanout.0 /= r;
    fanout.2 /= r;
    fanout.3 /= r;
    cascade.0 /= r;
    cascade.2 /= r;
    cascade.3 /= r;
    accum.0 /= r;
    accum.2 /= r;
    accum.3 /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\nhonest cell rank after arm — mean of {RUNS}\n\
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
        "leaf occ census recorded 0 fires:{out}"
    );
}

//! Fan-out cost — one fact reaching many productions, plus the beta write/read traffic it
//! generates.


use super::*;

/// THREE conditions — the shape neither the fanout nor the cascade produces.
///
/// Two conditions give `root-join -> J` where `J` is terminal, so every hash-join in those
/// worlds is a leaf. Three give `root-join -> J1 -> J2`, and **J1 is a MIDDLE join**: its beta
/// is the left input of J2's catch-up, so it must be READ. Without this world the beta-traffic
/// probe can only observe leaves, and "a hash-join's beta is never read" would be an
/// over-generalisation from a corpus that contains no counter-example — the exact shape of
/// claim this arc keeps having to retract.
///
/// `keys=10 x fanout=5`: 50 of each record, A⋈B = 250 pairs, A⋈B⋈C = 1250 triples.
const TRI_CENSUS_WORLD: &str = "\
(:wat::core::defrecord :tri::A [key <- :wat::core::i64  a <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::B [key <- :wat::core::i64  b <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::C [key <- :wat::core::i64  c <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::Trip [key <- :wat::core::i64  a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])\n\
\n\
(:wat::core::defn :tri::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert acc (:tri::A :key k :a f)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) (:tri::B :key k :b f)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))) (:tri::C :key k :c f)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None))))\n\
    s\n\
    (:wat::core::range 0 fanout)))\n\
\n\
(:wat::core::defn :tri::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:tri::seed-key acc k fanout))\n\
    s\n\
    (:wat::core::range 0 keys)))\n\
\n\
(:wat::rete::defrule :tri::tri-rule\n\
  :when\n\
  [(:tri::A (?k <- :key) (?a <- :a))\n\
   (:tri::B (?k <- :key) (?b <- :b))\n\
   (:tri::C (?k <- :key) (?c <- :c))]\n\
  :then\n\
  [(:tri::Trip ?k ?a ?b ?c)])\n";

/// ★ Does the fire ever READ the beta memory it writes?
///
/// `wm.beta` takes a Token CLONE per join result and is `clear()`ed before freeze, so nothing
/// downstream can see it. Inside the fire it is read at two places only, both against the
/// PARENT of a hash-join being keyed for the first time. That makes "a terminal join's beta is
/// written and never read" a HYPOTHESIS — and the identical shape ("surely this store is
/// redundant") was proposed for production-memory one session ago and was FALSE. So it gets
/// measured, not reasoned.
///
/// Two shapes, because one of them is the control: the CASCADE chains joins (level N feeds
/// level N+1), so its middle betas MUST show reads. If every node in both shapes read zero,
/// the instrument is broken, not the engine.
#[test]
fn beta_write_read_traffic() {
    /// Returns the human table AND the structured rows. The controls below assert on the
    /// ROWS, never on the table text: the rows are what was measured, and a `contains` over a
    /// formatted table would pass on a reordered column, a renamed verdict, or a substring
    /// appearing by accident — the exact laundering `no_loose_string_assert` exists to stop.
    fn traffic(label: &str, world_src: &str, driver: &str) -> (String, Vec<(i64, u64, u64)>) {
        let world = startup_from_source(world_src, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let ast = crate::parse_one!(driver).expect("parse the fire driver");
        let (_fired, rows) = super::with_beta_traffic(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("{label} fire raised: {e:?}"))
                .value_owned()
        });

        let mut out = format!("\n  BETA TRAFFIC — {label}\n\n    node    written      read   verdict\n    ------------------------------------------------\n");
        let (mut tot_w, mut tot_r, mut dead_w, mut dead_n) = (0u64, 0u64, 0u64, 0usize);
        for (id, w, r) in &rows {
            tot_w += w;
            tot_r += r;
            let verdict = if *w > 0 && *r == 0 {
                dead_w += w;
                dead_n += 1;
                "WRITTEN, NEVER READ"
            } else if *r > 0 {
                "read"
            } else {
                "-"
            };
            out.push_str(&format!("    {id:>4}  {w:>9}  {r:>8}   {verdict}\n"));
        }
        out.push_str(&format!(
            "\n    total written {tot_w}, total read {tot_r}\n    \
                 write-only nodes: {dead_n}  —  tokens cloned into them and never read: {dead_w} \
                 ({:.1}% of all beta writes)\n",
            if tot_w > 0 {
                dead_w as f64 * 100.0 / tot_w as f64
            } else {
                0.0
            },
        ));
        // The instrument must have seen traffic at all, or its zeros mean nothing.
        assert!(
            tot_w > 0,
            "{label}: recorded no beta writes — the instrument is not armed.{out}"
        );

        // ★ THE GUARD'S INVARIANT — and this is the DANGEROUS direction.
        //
        // `beta_readers` writes a node's beta iff that node parents a HashJoinNode, and the
        // two readers only ever read such a parent, so the sets coincide by construction.
        // Should a THIRD reader ever be added that reads some other node, `wm.beta.get()`
        // returns `None`, `all_left` comes back EMPTY, and the join silently drops tokens —
        // no panic, no error, just wrong answers that a differential would have to catch
        // downstream. A node with reads and zero writes is that bug, caught here at its
        // source.
        let starved: Vec<&(i64, u64, u64)> =
            rows.iter().filter(|&&(_, w, r)| r > 0 && w == 0).collect();
        assert!(
            starved.is_empty(),
            "{label}: {} node(s) READ a beta that was never WRITTEN — {starved:?}.\n\
                 The beta_readers guard (a node is written iff it parents a HashJoinNode) no \
                 longer covers every reader, so `wm.beta.get()` hands back None and the join \
                 silently loses tokens. Widen the guard to include the new reader; do NOT relax \
                 this assertion.{out}",
            starved.len(),
        );
        (out, rows)
    }

    let (fanout, _fanout_rows) = traffic(
        "fanout [100 x 20] — one rule, two conditions (the join is TERMINAL)",
        FANOUT_CENSUS_WORLD,
        "(:wat::core::match (:wat::rete::fire-rules (:fan::seed (:wat::core::match (:wat::rete::compile \
             (:wat::rete::collect-rules :fan)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 100 20)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))",
    );
    let (cascade, cascade_rows) = traffic(
            "deep-cascade [10 x 100] — CHAINED joins (the CONTROL: middle betas must be read)",
            DEPTH_SPLIT_WORLD,
            "(:wat::core::match (:wat::rete::fire-rules (:dc::seed-level-0 (:wat::core::match (:wat::rete::compile (:dc::build-rules 10)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 100)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))",
        );
    // THE case neither shape above produces: a MIDDLE hash-join, whose beta feeds the next
    // join's catch-up. Both worlds above are two-condition rules, so every hash-join in them
    // is a leaf; a rule about "hash-join betas" drawn from those alone would be generalising
    // from a corpus with no counter-example in it.
    let (tri, tri_rows) = traffic(
        "tri [10 x 5] — THREE conditions: root-join -> J1 -> J2, so J1 is a MIDDLE join",
        TRI_CENSUS_WORLD,
        "(:wat::core::match (:wat::rete::fire-rules (:tri::seed (:wat::core::match (:wat::rete::compile \
             (:wat::rete::collect-rules :tri)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 10 5)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))",
    );
    println!("{fanout}{cascade}{tri}");

    // Both controls assert on the ROWS — the measured (node, written, read) triples — not on
    // the table text. A `contains` over a rendered table would survive a renamed verdict, a
    // reordered column, or a chance substring, and would be asserting the FORMATTER rather
    // than the measurement.
    let readers =
        |rows: &[(i64, u64, u64)]| -> usize { rows.iter().filter(|&&(_, _, r)| r > 0).count() };

    // Control 1: SOMETHING must read a beta, or a zero elsewhere proves nothing rather than
    // proving the store is dead (a green that cannot go red is a claim with nothing behind it).
    assert!(
        readers(&cascade_rows) > 0,
        "the CONTROL failed — the cascade read no beta at all, so the instrument is measuring \
             nothing and the fanout zeros are meaningless.{cascade}"
    );

    // Control 2, the sharper one. The guard this probe justifies is "a node needs its beta iff
    // it parents a HashJoinNode". In `tri`, J1 parents J2 — so if J1 read ZERO the rule is
    // wrong and the guard would delete a live store on every 3+-condition rule. TWO nodes must
    // read here (the root-join AND J1); one alone means only the root-join was observed and
    // the middle-join case is still untested.
    let tri_readers = readers(&tri_rows);
    assert!(
        tri_readers >= 2,
        "a three-condition rule showed only {tri_readers} node(s) reading beta. Either the \
             middle join J1 is NOT read — which kills the parent-of-a-HashJoinNode guard — or the \
             network is not the shape this world intends. Do not draw the stone on this.{tri}"
    );
}

/// Diagnostic — DESIGN-STONE-compiled-rhs.md's zero-allocation gate, not a positive count.
///
/// `match:key-alloc` is armed inside `matcher.rs`'s two `Value::String(Arc::new(...))` sites
/// (alpha's `?v <- :field` and the RHS's `resolve_operand`). Alpha is compiled (arc 278
/// compiled-conditions), and as of this stone the RHS is too: `exec_compiled_rhs` walks a
/// pre-built `CompiledRhs` program and never re-allocates a `?var` key, so on a fire with BOTH
/// compiled paths live, `match:key-alloc` is expected to be EXACTLY ZERO — a fire that still
/// counted here would mean a form fell through to the `build_insert_fact` fallback. (This
/// mirrors `a8_node_share_fire_census`'s HOLD → PRODUCE re-point earlier the same day: the
/// property this test proves changed, so the assertion had to be re-pointed rather than left
/// to keep passing on a claim it no longer supports.)
#[test]
fn fanout_rhs_key_alloc_census() {
    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout census world should freeze");
    let src = "(:wat::core::match (:wat::rete::fire-rules (:fan::seed (:wat::core::match (:wat::rete::compile \
                   (:wat::rete::collect-rules :fan)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 100 20)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))";
    let ast = crate::parse_one!(src).expect("parse the fire driver");
    let (_fired, rows) = super::with_count_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fanout count census fire raised: {e:?}"))
            .value_owned()
    });
    let get = |n: &str| {
        rows.iter()
            .find(|(k, _)| *k == n)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    let table = format!(
        "\n  FANOUT RHS ALLOCATION CENSUS — keys=100 x fanout=20, 40,000 derived Pairs\n\
             \n  match:key-alloc (RHS + alpha, both compiled — expect 0)  {:>10}\n\
             \x20 per derived fact                                       {:>10.2}\n\
             \x20 match:calls (interpreter entries — expect 0)           {:>10}\n\
             \x20 prod:derivations (non-vacuity guard — expect 40,000)   {:>10}\n",
        get("match:key-alloc"),
        get("match:key-alloc") as f64 / 40_000.0,
        get("match:calls"),
        get("prod:derivations"),
    );
    println!("{table}");
    // Arc 278 DESIGN-STONE-compiled-rhs.md — this stone makes ZERO the correct answer (the
    // compiled RHS rebuilds no `?var` key), so the pre-stone ">0" assertion INVERTS rather
    // than simply strengthens. Re-pointed, not weakened: exactly 0 proves no form fell
    // through to the `build_insert_fact` fallback, AND `prod:derivations == 40_000` is kept
    // as a non-vacuity guard — a fire that never ran would also read 0 key allocations, and
    // without this second assertion that dead-fire zero would be indistinguishable from the
    // proof this test exists to make.
    assert_eq!(
        get("match:key-alloc"),
        0,
        "expected ZERO key allocations — the compiled RHS pre-builds every ?var key at rule \
             setup and never reallocates one per fact; a nonzero count means some :then form fell \
             through to the build_insert_fact fallback.{table}"
    );
    assert_eq!(
        get("prod:derivations"),
        40_000,
        "non-vacuity guard: expected exactly 40,000 derivations (the fanout cell's documented \
             size) — a count other than this means the key-alloc==0 reading above cannot be \
             trusted as proof of the compiled path (it could equally be an artifact of a fire that \
             never ran).{table}"
    );
}

/// Diagnostic — per-CALL alpha cost on a rule-light, fact-heavy workload (`D=1`).
///
/// `keys=100, fanout=20` is R4's exact 40,000-derived-pair cell. Prints the phase split so the
/// compiled-conditions stone can size its scorecard from a measurement of the shape it targets
/// instead of from the cascade's per-fact rate.
#[test]
fn fanout_per_call_alpha_census() {
    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout census world should freeze");
    let src = "(:wat::core::match (:wat::rete::fire-rules (:fan::seed (:wat::core::match (:wat::rete::compile \
                   (:wat::rete::collect-rules :fan)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 100 20)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))";
    let ast = crate::parse_one!(src).expect("parse the fire driver");
    let (_fired, rows) = super::with_phase_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fanout census fire raised: {e:?}"))
            .value_owned()
    });

    // The denominator is THE FIRE — and it is NAMED, not inferred, because inferring it from
    // the row text has now been wrong twice. Draft 1 summed the INDENTED rows and printed
    // shares totalling 209.3% (a nested row is a component of its parent, so that double-counts
    // upward). Draft 2 summed the UN-indented rows — which looks right and is not, because
    // `production` / `hash-join` / `alpha` / `root-join` / `accumulate` / `filter` carry
    // unindented NAMES while living INSIDE `ROUND LOOP`; that inflated the divisor ~60% and
    // quietly understated every share. A wrong number that looks plausible is worse than one
    // that reads 209%. These four are the actual brackets around a fire; everything else is a
    // component of one of them.
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    let fire: u64 = rows
        .iter()
        .filter(|(n, _)| FIRE_PHASES.contains(n))
        .map(|(_, ns)| *ns)
        .sum();
    let mut table = String::from(
        "\n  FANOUT PER-CALL CENSUS — keys=100 x fanout=20 (R4's 40,000-pair cell), D=1\n\
             \n  phase                                 ms   % of fire\n\
             \x20 ------------------------------------------------------\n",
    );
    for (n, ns) in &rows {
        table.push_str(&format!(
            "  {n:<32} {:>8.3} {:>10.1}%\n",
            *ns as f64 / 1e6,
            if fire > 0 {
                *ns as f64 * 100.0 / fire as f64
            } else {
                0.0
            }
        ));
    }
    table.push_str(&format!(
        "  {:<32} {:>8.3}     100.0%\n",
        "THE FIRE (top-level phases)",
        fire as f64 / 1e6
    ));
    let total = fire;
    println!("{table}");
    // ⛔ WAS ONLY `assert!(total > 0)` — a liveness check. The sibling above
    // (`fanout_rhs_key_alloc_census`) already shows the right shape: a NON-VACUITY GUARD on a
    // deterministic count, so a reading cannot be mistaken for proof when the fire never ran.
    // The same discipline, applied to what a phase census actually holds — its structure.
    assert!(total > 0, "the phase census recorded nothing.{table}");

    let phase = |want: &str| rows.iter().any(|(n, _)| *n == want);
    for want in ["production", "hash-join", "alpha", "root-join"] {
        assert!(
            phase(want),
            "phase `{want}` is absent from the census — a 100x20 fan-out MUST exercise it, so \
             its absence means the workload changed shape, not that it got faster.{table}"
        );
    }

    // Structural claim this workload exists to demonstrate: on a rule-light, fact-heavy fan-out
    // the PRODUCTION phase dominates. Measured 2026-08-30: production 79.5% vs hash-join 8.7%,
    // a 9x gap — so a 2x floor cannot flake on a loaded runner and still fires if the cost
    // centre moves somewhere else, which is exactly the regression this census would be read to
    // detect.
    let ns_of = |want: &str| rows.iter().find(|(n, _)| *n == want).map_or(0, |(_, ns)| *ns);
    let (prod, hj) = (ns_of("production"), ns_of("hash-join"));
    assert!(
        prod > hj * 2,
        "production ({prod} ns) no longer dominates hash-join ({hj} ns) by 2x on a fan-out \
         workload — measured at 9x. The cost centre has moved and this census's reading of \
         `where the time goes` is describing a different engine.{table}"
    );
}

/// Fanout phase table at the GRID ladder. `(keys, fanout)` 25/50/100 × 20
/// is items 10000/20000/40000. Prints; does not gate FIRE
/// (`DESIGN-STONE-fanout-phase-census`).
#[test]
fn fanout_fire_phase_census() {
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const REQUIRED: [&str; 6] = [
        "SETUP: indexes",
        "ROUND LOOP",
        "alpha",
        "root-join",
        "hash-join",
        "production",
    ];
    let table = render_phase_table(
        "fanout fire",
        &[(25, 20), (50, 20), (100, 20)],
        &TOP,
        &REQUIRED,
        |keys, fanout| keys * fanout * 2,
        fanout_phase_census,
    );
    println!("{table}");
    let rows = fanout_phase_census(100, 20);
    let ns_of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, ns, _)| *ns)
            .unwrap_or(0)
    };
    let round_loop = ns_of("ROUND LOOP");
    let hash_join = ns_of("hash-join");
    assert!(
        round_loop > 0,
        "ROUND LOOP recorded 0ns at keys=100 fanout=20 — the fire never ran:\n{table}"
    );
    assert!(
        hash_join > 0,
        "hash-join recorded 0ns at the 40k-pair cell — this axis is a join:\n{table}"
    );
    // ⛔ The two guards above are LIVENESS — `probare` classed this test hollow.
    super::assert_phases_present(
        rows.iter().map(|r| r.0),
        &["alpha", "root-join", "hash-join", "production"],
        &table,
    );
}

/// Leftover production: remainder_raw vs children's instrument left in the parent.
/// Subtracting child *nets* from production *net* double-counts those clock reads
/// as unmarked work (`DESIGN-STONE-prod-leftover-split`).
#[test]
fn fanout_production_leftover_split() {
    const RUNS: usize = 3;
    const RHS: &str = "  ├ prod:compiled-rhs";
    const DEDUP: &str = "  ├ prod:dedup-store";

    let cal = calibrate_mark_ns();

    // MINIMUM across runs, not mean — the header this test prints has always said so.
    let mut prod_raw = f64::INFINITY;
    let mut rhs_raw = f64::INFINITY;
    let mut dedup_raw = f64::INFINITY;
    let mut rhs_pairs = 0u64;
    let mut dedup_pairs = 0u64;
    let mut prod_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = fanout_phase_census(100, 20);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let (p_ns, p_k) = of("production");
        let (r_ns, r_k) = of(RHS);
        let (d_ns, d_k) = of(DEDUP);
        prod_raw = prod_raw.min(p_ns as f64);
        rhs_raw = rhs_raw.min(r_ns as f64);
        dedup_raw = dedup_raw.min(d_ns as f64);
        prod_pairs = p_k;
        rhs_pairs = r_k;
        dedup_pairs = d_k;
    }
    let prod_net = prod_raw - prod_pairs as f64 * cal;
    let rhs_net = rhs_raw - rhs_pairs as f64 * cal;
    let dedup_net = dedup_raw - dedup_pairs as f64 * cal;
    let remainder_raw = prod_raw - rhs_raw - dedup_raw;
    let tax_in_parent = (rhs_pairs + dedup_pairs) as f64 * cal;
    let naive = prod_net - rhs_net - dedup_net;
    let table = format!(
        "\nproduction leftover split — fanout [100 20], MINIMUM of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             production            {:>7.2} ms raw  {:>7.2} net  {:>6}x\n\
             prod:compiled-rhs     {:>7.2} ms raw  {:>7.2} net  {:>6}x\n\
             prod:dedup-store      {:>7.2} ms raw  {:>7.2} net  {:>6}x\n\
             \n\
             remainder_raw         {:>7.2} ms   (prod_raw − rhs_raw − dedup_raw)\n\
             tax_in_parent         {:>7.2} ms   ((rhs + dedup) pairs × cal)\n\
             naive_unmarked        {:>7.2} ms   (prod_net − rhs_net − dedup_net)\n\
             = remainder_raw + tax {:>7.2} ms\n",
        ms(prod_raw),
        ms(prod_net),
        prod_pairs,
        ms(rhs_raw),
        ms(rhs_net),
        rhs_pairs,
        ms(dedup_raw),
        ms(dedup_net),
        dedup_pairs,
        ms(remainder_raw),
        ms(tax_in_parent),
        ms(naive),
        ms(remainder_raw + tax_in_parent),
    );
    println!("{table}");
    assert!(
        prod_raw > 0.0,
        "production recorded 0 — the fire never ran:{table}"
    );
    assert_eq!(
        rhs_pairs, 40_000,
        "compiled-rhs pairs must be the 40k cell, not a dead fire:{table}"
    );
}

/// Rank harvest / compiled-rhs / OUT freeze at fanout [100 20].
/// Grid compile-alls `:fan::q-Pair`; `FANOUT_CENSUS_WORLD` does not
/// (`DESIGN-STONE-fanout-three-leftover`).
#[test]
fn fanout_three_leftover_split() {
    use std::time::Instant;

    const KEYS: i64 = 100;
    const FANOUT: i64 = 20;
    const RUNS: usize = 3;
    const RHS: &str = "  ├ prod:compiled-rhs";
    const HARVEST: &str = "  ├ harvest:query";
    const OUT_PROD: &str = "  ├ out:production";
    const OUT_QUERY: &str = "  └ out:query";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const QUERY_TAIL: &str = "\n\
(:wat::rete::defquery :fan::q-Pair\n\
  :params []\n\
  :when [(?fact <- :fan::Pair)])\n";

    let cal = calibrate_mark_ns();

    struct Shot {
        wall: f64,
        fire: f64,
        harvest: f64,
        out_query: f64,
        rhs_raw: f64,
        rhs_net: f64,
        rhs_pairs: u64,
        out_prod: f64,
        query_maps: usize,
    }

    let query_map_count = |fired: &Value| -> usize {
        match session_named_field(fired, "query-memory") {
            Some(Value::wat__core__PersistentMap(pm)) => pm
                .iter()
                .map(|(_, v)| match v {
                    Value::wat__core__PersistentVector(pv) => pv.len(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        }
    };

    let shot = |with_query: bool| -> Shot {
        let world_src = if with_query {
            format!("{FANOUT_CENSUS_WORLD}{QUERY_TAIL}")
        } else {
            FANOUT_CENSUS_WORLD.to_string()
        };
        let world = startup_from_source(&world_src, None, Arc::new(InMemoryLoader::new()))
            .expect("fanout three-leftover world should freeze");
        let compile = if with_query {
            "(:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :fan) \
              (:wat::core::PersistentVector (:fan::q-Pair))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))"
        } else {
            "(:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :fan)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))"
        };
        let seed_src = format!("(:fan::seed {compile} {KEYS} {FANOUT})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, &crate::rust_caller_span!(), world.symbols(), None).unwrap_or_else(|e| {
                panic!("fire-rules raised with_query={with_query}: {e:?}")
            })
        });
        let wall = t0.elapsed().as_nanos() as f64;
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let fire: u64 = FIRE_PHASES.iter().map(|n| of(n).0).sum();
        let (rhs_raw, rhs_pairs) = of(RHS);
        let rhs_net = rhs_raw as f64 - rhs_pairs as f64 * cal;
        Shot {
            wall,
            fire: fire as f64,
            harvest: of(HARVEST).0 as f64,
            out_query: of(OUT_QUERY).0 as f64,
            rhs_raw: rhs_raw as f64,
            rhs_net,
            rhs_pairs,
            out_prod: of(OUT_PROD).0 as f64,
            query_maps: query_map_count(&fired),
        }
    };

    // MINIMUM across runs, not mean. `rhs_pairs`/`query_maps` are COUNTS, not measurements, and
    // are assigned rather than reduced.
    let mut without = Shot {
        wall: f64::INFINITY,
        fire: f64::INFINITY,
        harvest: f64::INFINITY,
        out_query: f64::INFINITY,
        rhs_raw: f64::INFINITY,
        rhs_net: f64::INFINITY,
        rhs_pairs: 0,
        out_prod: f64::INFINITY,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: f64::INFINITY,
        fire: f64::INFINITY,
        harvest: f64::INFINITY,
        out_query: f64::INFINITY,
        rhs_raw: f64::INFINITY,
        rhs_net: f64::INFINITY,
        rhs_pairs: 0,
        out_prod: f64::INFINITY,
        query_maps: 0,
    };
    for _ in 0..RUNS {
        let a = shot(false);
        let b = shot(true);
        without.wall = without.wall.min(a.wall);
        without.fire = without.fire.min(a.fire);
        without.harvest = without.harvest.min(a.harvest);
        without.out_query = without.out_query.min(a.out_query);
        without.rhs_raw = without.rhs_raw.min(a.rhs_raw);
        without.rhs_net = without.rhs_net.min(a.rhs_net);
        without.rhs_pairs = a.rhs_pairs;
        without.out_prod = without.out_prod.min(a.out_prod);
        without.query_maps = a.query_maps;
        with.wall = with.wall.min(b.wall);
        with.fire = with.fire.min(b.fire);
        with.harvest = with.harvest.min(b.harvest);
        with.out_query = with.out_query.min(b.out_query);
        with.rhs_raw = with.rhs_raw.min(b.rhs_raw);
        with.rhs_net = with.rhs_net.min(b.rhs_net);
        with.rhs_pairs = b.rhs_pairs;
        with.out_prod = with.out_prod.min(b.out_prod);
        with.query_maps = b.query_maps;
    }

    let a_harvest = with.harvest + with.out_query;
    let delta = with.wall - without.wall;
    // `\x20` below is a LOAD-BEARING escape, not decoration. A `\`-newline string continuation
    // strips the continued line's LEADING whitespace, so a row indented in source printed
    // flush-left and its numbers landed left of the parent's. The escape stops the strip: it
    // restores the child indent and leaves every number and column position where it was.
    let table = format!(
        "\nfanout three leftover — [100 20], MINIMUM of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             without query          wall {:>7.2}  FIRE {:>7.2}  query-maps {}\n\
             with    q-Pair         wall {:>7.2}  FIRE {:>7.2}  query-maps {}\n\
             delta (A candidate)           {:>7.2} ms\n\
             \n\
             A  harvest:query              {:>7.2} ms\n\
          \x20  out:query                  {:>7.2} ms\n\
          \x20  A sum                      {:>7.2} ms\n\
             B  compiled-rhs net           {:>7.2} ms   {:>6}x  (with-query)\n\
             C  out:production             {:>7.2} ms   (with-query)\n",
        ms(without.wall),
        ms(without.fire),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        with.query_maps,
        ms(delta),
        ms(with.harvest),
        ms(with.out_query),
        ms(a_harvest),
        ms(with.rhs_net),
        with.rhs_pairs,
        ms(with.out_prod),
    );
    println!("{table}");
    assert_eq!(
        without.rhs_pairs, 40_000,
        "without-query compiled-rhs pairs must be 40k:{table}"
    );
    assert_eq!(
        with.rhs_pairs, 40_000,
        "with-query compiled-rhs pairs must be 40k:{table}"
    );
    assert_eq!(
        without.query_maps, 0,
        "census world has no query — query-memory must be empty:{table}"
    );
    assert_eq!(
        with.query_maps, 40_000,
        "grid q-Pair must harvest 40k maps:{table}"
    );
    assert!(
        with.fire > 0.0,
        "with-query FIRE recorded 0 — the fire never ran:{table}"
    );
}

/// Honest FIRE after 2s: strip the 80k test marks 2p named
/// (`DESIGN-STONE-honest-fire-rank`).
#[test]
fn fanout_honest_fire_rank() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const RHS: &str = "  ├ prod:compiled-rhs";
    const DEDUP: &str = "  ├ prod:dedup-store";
    const PROBE: &str = "  ├ hj:catchup:probe";

    let cal = calibrate_mark_ns();

    // MINIMUM across runs, not mean.
    let mut fire = f64::INFINITY;
    let mut prod_raw = f64::INFINITY;
    let mut rhs_raw = f64::INFINITY;
    let mut dedup_raw = f64::INFINITY;
    let mut probe_raw = f64::INFINITY;
    let mut hash_raw = f64::INFINITY;
    let mut alpha_raw = f64::INFINITY;
    let mut out_raw = f64::INFINITY;
    let mut rhs_pairs = 0u64;
    let mut dedup_pairs = 0u64;
    let mut prod_pairs = 0u64;
    let mut probe_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = fanout_phase_census(100, 20);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire = fire.min(TOP.iter().map(|n| of(n).0 as f64).sum::<f64>());
        let (p_ns, p_k) = of("production");
        let (r_ns, r_k) = of(RHS);
        let (d_ns, d_k) = of(DEDUP);
        let (pr_ns, pr_k) = of(PROBE);
        prod_raw = prod_raw.min(p_ns as f64);
        rhs_raw = rhs_raw.min(r_ns as f64);
        dedup_raw = dedup_raw.min(d_ns as f64);
        probe_raw = probe_raw.min(pr_ns as f64);
        hash_raw = hash_raw.min(of("hash-join").0 as f64);
        alpha_raw = alpha_raw.min(of("alpha").0 as f64);
        out_raw = out_raw.min(of("OUT: to_persistent").0 as f64);
        prod_pairs = p_k;
        rhs_pairs = r_k;
        dedup_pairs = d_k;
        probe_pairs = pr_k;
    }
    let rhs_net = rhs_raw - rhs_pairs as f64 * cal;
    let dedup_net = dedup_raw - dedup_pairs as f64 * cal;
    let probe_net = probe_raw - probe_pairs as f64 * cal;
    let remainder_raw = prod_raw - rhs_raw - dedup_raw;
    let tax_in_parent = (rhs_pairs + dedup_pairs) as f64 * cal;
    let honest_prod = rhs_net + dedup_net;
    let honest_fire = fire - remainder_raw - tax_in_parent;
    let table = format!(
        "\nhonest FIRE rank — fanout [100 20], MINIMUM of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             FIRE                    {:>7.2} ms\n\
             production              {:>7.2} ms raw   {:>6}x\n\
             compiled-rhs            {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             dedup-store             {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             probe                   {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             hash-join               {:>7.2} ms\n\
             alpha                   {:>7.2} ms\n\
             OUT                     {:>7.2} ms\n\
             \n\
             remainder_raw           {:>7.2} ms\n\
             tax_in_parent           {:>7.2} ms\n\
             honest_prod             {:>7.2} ms   (rhs_net + dedup_net)\n\
             honest_FIRE             {:>7.2} ms   (FIRE − remainder − tax)\n",
        ms(fire),
        ms(prod_raw),
        prod_pairs,
        ms(rhs_raw),
        ms(rhs_net),
        rhs_pairs,
        ms(dedup_raw),
        ms(dedup_net),
        dedup_pairs,
        ms(probe_raw),
        ms(probe_net),
        probe_pairs,
        ms(hash_raw),
        ms(alpha_raw),
        ms(out_raw),
        ms(remainder_raw),
        ms(tax_in_parent),
        ms(honest_prod),
        ms(honest_fire),
    );
    println!("{table}");
    assert!(fire > 0.0, "FIRE recorded 0 — the fire never ran:{table}");
    assert_eq!(
        rhs_pairs, 40_000,
        "compiled-rhs pairs must be the 40k cell:{table}"
    );
}

/// Complete phase apportionment of the fanout census world — every mark the
/// fire path emits, calibrated and ranked. The theater hunt kept asking "what
/// is left?" from a list; this answers it from the instrument.
#[test]
fn fanout_phase_dump() {
    use std::time::Instant;

    const RUNS: usize = 3;
    const KEYS: i64 = 100;
    const FANOUT: i64 = 20;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    let cal = calibrate_mark_ns();

    let world = startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("fanout world should freeze");

    // MINIMUM across runs, not mean — each map slot is seeded to +∞ on first sight of its phase.
    let mut acc: FxHashMap<String, (f64, u64)> = FxHashMap::default();
    let mut wall = f64::INFINITY;

    for _ in 0..RUNS {
        let seed_src = format!(
            "(:fan::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :fan)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {KEYS} {FANOUT})"
        );
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (_fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, &crate::rust_caller_span!(), world.symbols(), None)
                .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
        });
        wall = wall.min(t0.elapsed().as_nanos() as f64);
        for (name, ns, k) in rows {
            let e = acc.entry(name.to_string()).or_insert((f64::INFINITY, 0));
            e.0 = e.0.min(ns as f64 - k as f64 * cal);
            e.1 = k;
        }
    }


    // NESTING TAX. A parent's span CONTAINS its children's mark pairs, and the
    // per-row calibration only removes a row's OWN pairs. `prod:compiled-rhs`
    // and `prod:dedup-store` each fire once per derivation — 40k pairs apiece on
    // this cell — so `production` was reading ~7.5 ms of pure instrument.
    // Measured directly by deleting the two marks and re-running: production
    // 18.992 -> 11.524 ms, wall 24.491 -> 16.690. Subtract the children's tax
    // from the parent, or the biggest number in the table is the instrument.
    #[allow(unused_variables)]
    let child_tax: f64 = ["  ├ prod:compiled-rhs", "  ├ prod:dedup-store"]
        .iter()
        .map(|n| acc.get(*n).map(|e| e.1).unwrap_or(0) as f64 * cal)
        .sum();
    // NOT applied to the parent. `cal` is measured in a tight loop and
    // OVERSTATES the in-context cost of a mark: it estimates this tax at
    // ~11-12 ms, while deleting the two marks and re-running measured 7.5 ms
    // (production 18.992 -> 11.524, wall 24.491 -> 16.690). Subtracting the
    // estimate would trade an inflated number for a deflated one. The raw rows
    // stand; the direct experiment is the reference.
    let top_sum: f64 = TOP.iter().map(|n| acc.get(*n).map(|e| e.0).unwrap_or(0.0)).sum::<f64>();

    let mut sub: Vec<(String, f64, u64)> = acc
        .iter()
        .filter(|(n, _)| !TOP.contains(&n.as_str()) && n.as_str() != "cal")
        .map(|(n, (ns, k))| (n.clone(), *ns, *k))
        .collect();
    sub.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut body = String::new();
    for (name, msv, k) in &sub {
        body.push_str(&format!("{name:<26} {:>8.3} ms   {k:>7} pairs\n", ms(*msv)));
    }
    let named: f64 = sub.iter().map(|(_, v, _)| *v).sum();

    println!(
        "\nfanout phase dump — [{KEYS} {FANOUT}], no query, MINIMUM of {RUNS}\n\
         instrument: {cal:.1} ns per mark pair\n\
         \n\
         wall                       {:>8.3} ms\n\
         FIRE (4 top phases)        {:>8.3} ms\n\
         \n{body}\
         \n\
         sub-phase sum (parents+children, DOUBLE COUNTS) {:>8.3} ms\n\
         \n\
         ⚠ READ THE PARENTS WITH CARE. `production` brackets two marks that fire\n\
         ONCE PER DERIVATION (40k pairs here), and per-row calibration only\n\
         removes a row's OWN pairs, so the parent still carries its children's\n\
         tax. Deleting those two marks and re-running gave production\n\
         18.992 -> 11.524 ms and wall 24.491 -> 16.690 — about 7.5 ms of the\n\
         parent is instrument, i.e. ~94 ns per pair IN SITU.\n\
         \n\
         `cal` is now the MINIMUM of five 200k batches and reads ~66 ns, stable\n\
         to under a nanosecond. It used to be one batch and read anywhere from\n\
         105 to 155 ns, which at 40k pairs swung a child row by ±2 ms — this\n\
         very row was recorded at both 2.541 and 4.826 ms for identical code.\n\
         Every net figure in this arc taken with the old calibration is\n\
         therefore UNDER-reported.\n\
         \n\
         The min-of-5 tight loop (~66 ns) still sits below the in-situ cost\n\
         (~94 ns), so a 40k-pair net row remains over-reported by roughly\n\
         {:>5.2} ms. That is now a STABLE bias of known sign and size rather\n\
         than run-to-run noise. Before/after deltas from one session are sound;\n\
         absolute parent times from this table are still not.\n",
        ms(wall),
        ms(top_sum),
        ms(named),
        (94.0 - cal) * 40_000.0 / 1e6,
    );

    // ⛔ WAS ONLY `assert!(wall > 0.0)` — liveness on the harness clock, which says nothing about
    // whether the DUMP found anything. This test's whole purpose is completeness: its doc says
    // "the theater hunt kept asking 'what is left?' from a list; this answers it from the
    // instrument". A dump that enumerated nothing would answer "nothing is left" and pass.
    assert!(wall > 0.0, "harness recorded no time");
    assert!(
        !sub.is_empty(),
        "the phase dump enumerated ZERO named marks — it would report `nothing is left` from an \
         empty instrument rather than from a complete one, which is the opposite of what this \
         test exists to establish"
    );
    for want in TOP {
        assert!(
            acc.contains_key(want),
            "top-level phase `{want}` is missing from the dump — a fan-out fire must emit it, so \
             its absence means marks were lost, not that the phase became free"
        );
    }
    assert!(
        top_sum > 0.0 && named > 0.0,
        "top-level phases ({top_sum:.0}) or named sub-marks ({named:.0}) summed to zero — the \
         apportionment printed above is describing an instrument that recorded nothing"
    );
}

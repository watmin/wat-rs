//! Accumulator ALPHA cost — the alpha-side half of an accumulate: memory shape, tree walk,
//! class lookup, push, and the seed-after-fold ordering.


use super::*;

/// How many DISTINCT alpha memories do the accumulate nodes actually read?
///
/// `accum:index-builds 4` over `index-elements 160,000` is consistent with ONE shared alpha
/// (4 builds of 40,000) or TWO (1 + 3 builds of 40,000) — the counts alone cannot tell them
/// apart, and the size of the cache win differs (3 of 4 builds saved vs 2 of 4). The round
/// census already counts alpha nodes and their elements, so it settles it.
#[test]
fn accum_alpha_memory_shape() {
    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let src = "(:wat::core::match (:wat::rete::fire-rules (:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200 200)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))";
    let ast = crate::parse_one!(src).expect("parse the fire driver");
    let (_fired, census) = super::with_fire_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
            .value_owned()
    });
    // ⛔ WAS `assert!(!census.is_empty())` — a LIVENESS check, which fails only if the fire
    // never ran and says nothing about what it computed. The workload here is FIXED (G=200,
    // W=200), so every count below is deterministic: measured byte-identical across runs
    // 2026-08-30. They are the memory SHAPE this test is named for, and a change in any of them
    // is a real regression or a deliberate change that must update this number.
    assert!(!census.is_empty(), "round census recorded nothing");
    assert_eq!(census.len(), 2, "fixpoint should close in 2 rounds for this workload");
    let last = census.last().expect("non-empty");
    assert_eq!(
        census[0].delta_facts_in, 40_200,
        "round 0 sees every input fact: 200 Groups + 40,000 Readings"
    );
    assert_eq!(
        last.alpha_nodes, 3,
        "three alpha nodes: the rule's two fact patterns plus the accumulate's source"
    );
    assert_eq!(
        last.alpha_elements, 80_200,
        "alpha memory holds one element per (fact, matching alpha) pair"
    );
    assert_eq!(
        last.production_facts, 1_000,
        "200 groups x 5 productions each"
    );
    assert_eq!(
        last.seen_facts, 41_200,
        "40,200 input + 1,000 derived, all seen exactly once"
    );
    println!(
        "\naccum alpha memories — G=200 W=200 (200 Groups + 40,000 Readings)\n\
             rounds {}\n                 last alpha_nodes {}   last alpha_elements {}\n",
        census.len(),
        last.alpha_nodes,
        last.alpha_elements
    );
    for row in &census {
        println!(
            "  r{}  dIn {}  aNodes {}  aEls {}  prod {}  seen {}",
            row.round,
            row.delta_facts_in,
            row.alpha_nodes,
            row.alpha_elements,
            row.production_facts,
            row.seen_facts
        );
    }
}

/// Honest alpha 18 ms: seed vs delta, then stacked isolated lumps
/// (`DESIGN-STONE-alpha-leftover-split`). No per-fact timers.
#[test]
fn accum_alpha_leftover_split() {
    use std::hint::black_box;

    const RUNS: usize = 3;


    let mut fire = 0.0;
    let mut alpha = 0.0;
    let mut seed = 0.0;
    let mut delta = 0.0;
    let mut seed_pairs = 0u64;
    let mut delta_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = accum_phase_census(200, 200);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire += [
            "IN: to_transient",
            "SETUP: indexes",
            "ROUND LOOP",
            "OUT: to_persistent",
        ]
        .iter()
        .map(|n| of(n).0 as f64)
        .sum::<f64>();
        alpha += of("alpha").0 as f64;
        let (s_ns, s_k) = of("  ├ alpha:seed");
        seed += s_ns as f64;
        seed_pairs = s_k;
        let (d_ns, d_k) = of("  └ alpha:delta");
        delta += d_ns as f64;
        delta_pairs = d_k;
    }
    let r = RUNS as f64;
    fire /= r;
    alpha /= r;
    seed /= r;
    delta /= r;

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let input_pv: crate::value::pvec::PVec = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    let facts: Vec<Value> = input_pv.iter().cloned().collect();
    // ⛔ WAS ONLY `!facts.is_empty()` — a liveness guard. `probare` classed the tests using it
    // hollow: fixed workload, timed arms, printed table, and nothing asserting WHAT was measured.
    // `[200 200]` yields exactly 40,200 facts (200 Groups + 40,000 Readings). Pin it: a drift
    // makes every millisecond below a figure for a different workload, and the split this test is
    // named for is then not the split it reports.
    assert_eq!(
        facts.len(),
        40_200,
        "workload drifted to {} facts, not the 40,200 the [200 200] accum axis produces — every \
         timing below is measuring something other than this test's subject",
        facts.len()
    );

    let reset = |wm: &mut super::FireSession, d_alpha: &mut AlphaDelta| {
        wm.alpha.clear();
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
        d_alpha.clear();
    };

    let mut wp = 0.0;
    let mut w = 0.0;
    let mut c = 0.0;
    let mut t = 0.0;
    let mut m = 0.0;
    let mut a = 0.0;
    for _ in 0..RUNS {
        wp += elapsed_ns(|| {
            for f in input_pv.iter() {
                black_box(f);
            }
        });
        w += elapsed_ns(|| {
            for f in &facts {
                black_box(f);
            }
        });
        c += elapsed_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((ag.class.as_ref(), ag.fields.as_slice()));
                    }
                    _ => {}
                }
            }
        });
        t += elapsed_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box(
                            arm.alpha_tree
                                .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                        );
                    }
                    _ => {}
                }
            }
        });
        m += elapsed_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm, &mut FxHashMap::default());
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    black_box(crate::rete::compiled_cond::exec_compiled(
                        crate::rete::compiled_cond::test_sym(),
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ));
                }
            }
        });
        a += elapsed_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            let mut cond_key_ids: CondKeyIds = HashMap::new();
            for (&id, c) in &arm.compiled_conds {
                cond_key_ids.insert(
                    id,
                    crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
                );
            }
            let bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
            for (i, fact) in facts.iter().enumerate() {
                super::alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut super::AlphaActivateCx {
            sym: crate::rete::compiled_cond::test_sym(),
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree: &arm.alpha_tree,
                        compiled_conds: &arm.compiled_conds,
                        match_scratch: &mut scratch,
                        cand_scratch: &mut cand,
                        cond_key_ids: &cond_key_ids,
                        bind_only: &bind_only,
                    },
                )
                .expect("isolated activate");
                black_box(d_alpha.len());
            }
        });
    }
    wp /= r;
    w /= r;
    c /= r;
    t /= r;
    m /= r;
    a /= r;

    let table = format!(
        "\naccum alpha leftover split — [200 200], mean of {RUNS}\n\
             in-fire (2 pairs, not per fact)\n\
             FIRE                       {:>7.2} ms\n\
             alpha                      {:>7.2} ms\n\
               seed                     {:>7.2} ms  {:>6}x\n\
               delta                    {:>7.2} ms  {:>6}x\n\
               seed+delta               {:>7.2} ms\n\
             \n\
             isolated (cold intern each run, {} facts)\n\
             Wp  PV iter                {:>7.2} ms\n\
             W   Vec iter               {:>7.2} ms\n\
             C   class extract          {:>7.2} ms\n\
             T   + candidates           {:>7.2} ms\n\
             M   + exec_compiled        {:>7.2} ms\n\
             A   alpha_activate_fact    {:>7.2} ms\n\
             \n\
             C−W   extract              {:>7.2} ms\n\
             T−C   tree                 {:>7.2} ms\n\
             M−T   exec_compiled+intern {:>7.2} ms\n\
             A−M   push                 {:>7.2} ms\n\
             A vs seed                  {:>7.2} ms isolated vs {:>7.2} in-fire\n",
        ms(fire),
        ms(alpha),
        ms(seed),
        seed_pairs,
        ms(delta),
        delta_pairs,
        ms(seed + delta),
        facts.len(),
        ms(wp),
        ms(w),
        ms(c),
        ms(t),
        ms(m),
        ms(a),
        ms(c - w),
        ms(t - c),
        ms(m - t),
        ms(a - m),
        ms(a),
        ms(seed),
    );
    println!("{table}");
    assert!(
        seed > 0.0,
        "alpha:seed recorded 0 — the mark never fired:{table}"
    );
    assert!(
        a > 0.0,
        "isolated activate recorded 0 — the loop never ran:{table}"
    );
    assert!(
        seed_pairs > 0,
        "alpha:seed pairs 0 — leftover split is a dead fire:{table}"
    );
}

/// Split in-fire `alpha:seed` after seen folded in
/// (`DESIGN-STONE-alpha-seed-after-fold`). Isolated loops
/// walk the facts PV, `candidates_into`, `seen_insert`.
#[test]
fn accum_alpha_seed_after_fold_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;

    const RUNS: usize = 3;


    let mut fire = 0.0;
    let mut seed = 0.0;
    for _ in 0..RUNS {
        let rows = accum_phase_census(200, 200);
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        fire += [
            "IN: to_transient",
            "SETUP: indexes",
            "ROUND LOOP",
            "OUT: to_persistent",
        ]
        .iter()
        .map(|n| of(n) as f64)
        .sum::<f64>();
        seed += of("  ├ alpha:seed") as f64;
    }
    let r = RUNS as f64;
    fire /= r;
    seed /= r;

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let input_pv: crate::value::pvec::PVec = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    assert!(
        !input_pv.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );
    let n_facts = input_pv.len();
    // ⛔ `probare` classed this test hollow — the guard above is liveness and `n_facts` was
    // computed, printed, and never checked. Same fixed `[200 200]` axis as its siblings in this
    // file: 40,200 facts (200 Groups + 40,000 Readings). A drift makes every seed/fold timing
    // below a figure for a different workload.
    assert_eq!(
        n_facts, 40_200,
        "workload drifted to {n_facts} facts, not the 40,200 the [200 200] accum axis produces"
    );

    let reset = |wm: &mut super::FireSession, d_alpha: &mut AlphaDelta| {
        wm.alpha.clear();
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
        d_alpha.clear();
    };
    let intern_keys = |wm: &mut super::FireSession| -> CondKeyIds {
        let mut cond_key_ids: CondKeyIds = HashMap::new();
        for (&id, c) in &arm.compiled_conds {
            cond_key_ids.insert(
                id,
                crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
            );
        }
        cond_key_ids
    };

    let mut p = 0.0;
    let mut s = 0.0;
    let mut x = 0.0;
    let mut k = 0.0;
    let mut e = 0.0;
    let mut n = 0.0;
    let mut a = 0.0;
    for _ in 0..RUNS {
        p += elapsed_ns(|| {
            for f in input_pv.iter() {
                black_box(f);
            }
        });
        s += elapsed_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                black_box(f);
            }
            black_box(seen_ids.len() + seen_rest.len());
        });
        x += elapsed_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((ag.class.as_ref(), ag.fields.as_slice()));
                    }
                    _ => {}
                }
            }
            black_box(seen_ids.len());
        });
        k += elapsed_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut cand = Vec::new();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        arm.alpha_tree.candidates_into(
                            ag.class.as_ref(),
                            ag.fields.as_slice(),
                            &mut cand,
                        );
                        black_box(cand.len());
                    }
                    _ => {}
                }
            }
        });
        let mut d_alpha_e: AlphaDelta = FxHashMap::default();
        reset(&mut wm, &mut d_alpha_e);
        let _cond_key_ids_e = intern_keys(&mut wm);
        e += elapsed_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            for f in input_pv.iter() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                arm.alpha_tree
                    .candidates_into(ag.class.as_ref(), ag.fields.as_slice(), &mut cand);
                for aid in cand.iter().copied() {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, aid).expect("compiled cond");
                    black_box(crate::rete::compiled_cond::exec_compiled_with_key_ids(
                        crate::rete::compiled_cond::test_sym(),
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                        _cond_key_ids_e.get(&aid).map(|v| v.as_slice()),
                    ));
                }
            }
        });
        let mut d_alpha_n: AlphaDelta = FxHashMap::default();
        reset(&mut wm, &mut d_alpha_n);
        let cond_key_ids_n = intern_keys(&mut wm);
        n += elapsed_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            for (i, f) in input_pv.iter().enumerate() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, f);
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                arm.alpha_tree
                    .candidates_into(ag.class.as_ref(), ag.fields.as_slice(), &mut cand);
                for aid in cand.iter().copied() {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, aid).expect("compiled cond");
                    if let Some((off, len)) = crate::rete::compiled_cond::exec_compiled_with_key_ids(
                        crate::rete::compiled_cond::test_sym(),
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                        cond_key_ids_n.get(&aid).map(|v| v.as_slice()),
                    ) {
                        let el = super::make_element(i as u32, off, len);
                        let slot = {
                            let v = Arc::make_mut(wm.alpha.entry(aid).or_default());
                            v.push(el);
                            v.len() - 1
                        };
                        d_alpha_n.entry(aid).or_default().push(slot);
                    }
                }
            }
            black_box(d_alpha_n.len());
        });
        let mut d_alpha_a: AlphaDelta = FxHashMap::default();
        reset(&mut wm, &mut d_alpha_a);
        let cond_key_ids_a = intern_keys(&mut wm);
        let mut bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
        for (&id, c) in &arm.compiled_conds {
            if let Some(fields) = crate::rete::compiled_cond::bind_only_fields(c) {
                bind_only.insert(id, fields);
            }
        }
        a += elapsed_ns(|| {
            let mut seen_ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n_facts, Default::default());
            let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            for (i, fact) in input_pv.iter().enumerate() {
                super::seen_insert(&mut seen_ids, &mut seen_rest, fact);
                super::alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut super::AlphaActivateCx {
            sym: crate::rete::compiled_cond::test_sym(),
                        wm: &mut wm,
                        d_alpha: &mut d_alpha_a,
                        alpha_tree: &arm.alpha_tree,
                        compiled_conds: &arm.compiled_conds,
                        match_scratch: &mut scratch,
                        cand_scratch: &mut cand,
                        cond_key_ids: &cond_key_ids_a,
                        bind_only: &bind_only,
                    },
                )
                .expect("isolated activate");
            }
            black_box(d_alpha_a.len());
        });
    }
    p /= r;
    s /= r;
    x /= r;
    k /= r;
    e /= r;
    n /= r;
    a /= r;

    let table = format!(
        "\naccum alpha:seed after fold — [200 200], mean of {RUNS}, {n_facts} facts\n\
             in-fire\n\
             FIRE                       {:>7.2} ms\n\
             alpha:seed                 {:>7.2} ms\n\
             \n\
             isolated (PV walk, candidates_into, seen_insert, cold bind_vals)\n\
             P   PV iter                {:>7.2} ms\n\
             S   + seen_insert          {:>7.2} ms\n\
             X   + class extract        {:>7.2} ms\n\
             K   + candidates_into      {:>7.2} ms\n\
             E   + exec_compiled        {:>7.2} ms\n\
             N   + push                 {:>7.2} ms\n\
             A   seen+activate          {:>7.2} ms\n\
             \n\
             S−P   seen                 {:>7.2} ms\n\
             X−S   extract              {:>7.2} ms\n\
             K−X   tree                 {:>7.2} ms\n\
             E−K   exec+intern          {:>7.2} ms\n\
             N−E   push                 {:>7.2} ms\n\
             A−N   wrapper              {:>7.2} ms\n\
             A vs seed                  {:>7.2} ms isolated vs {:>7.2} in-fire\n",
        ms(fire),
        ms(seed),
        ms(p),
        ms(s),
        ms(x),
        ms(k),
        ms(e),
        ms(n),
        ms(a),
        ms(s - p),
        ms(x - s),
        ms(k - x),
        ms(e - k),
        ms(n - e),
        ms(a - n),
        ms(a),
        ms(seed),
    );
    println!("{table}");
    assert!(
        seed > 0.0,
        "alpha:seed recorded 0 — the mark never fired:{table}"
    );
    assert!(
        a > 0.0,
        "isolated seen+activate recorded 0 — the loop never ran:{table}"
    );
    assert!(n_facts > 0, "compile+seed produced 0 facts:{table}");
}

/// Tree 4.46 ms: class HashMap vs walk vs Vec alloc
/// (`DESIGN-STONE-alpha-tree-walk-split`).
#[test]
fn accum_alpha_tree_walk_split() {
    use std::hint::black_box;

    const RUNS: usize = 3;


    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    // ⛔ WAS ONLY `!facts.is_empty()` — a liveness guard. `probare` classed the tests using it
    // hollow: fixed workload, timed arms, printed table, and nothing asserting WHAT was measured.
    // `[200 200]` yields exactly 40,200 facts (200 Groups + 40,000 Readings). Pin it: a drift
    // makes every millisecond below a figure for a different workload, and the split this test is
    // named for is then not the split it reports.
    assert_eq!(
        facts.len(),
        40_200,
        "workload drifted to {} facts, not the 40,200 the [200 200] accum axis produces — every \
         timing below is measuring something other than this test's subject",
        facts.len()
    );

    let mut e = 0.0;
    let mut g = 0.0;
    let mut i = 0.0;
    let mut t = 0.0;
    for _ in 0..RUNS {
        e += elapsed_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((ag.class.as_ref(), ag.fields.as_slice()));
                    }
                    _ => {}
                }
            }
        });
        g += elapsed_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box((
                            ag.fields.as_slice(),
                            arm.alpha_tree.has_class(ag.class.as_ref()),
                        ));
                    }
                    _ => {}
                }
            }
        });
        i += elapsed_ns(|| {
            let mut buf = Vec::new();
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        arm.alpha_tree.candidates_into(
                            ag.class.as_ref(),
                            ag.fields.as_slice(),
                            &mut buf,
                        );
                        black_box(buf.len());
                    }
                    _ => {}
                }
            }
        });
        t += elapsed_ns(|| {
            for f in &facts {
                match f {
                    Value::Aggregate(ag) if ag.nature != Nature::Struct => {
                        black_box(
                            arm.alpha_tree
                                .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                        );
                    }
                    _ => {}
                }
            }
        });
    }
    let r = RUNS as f64;
    e /= r;
    g /= r;
    i /= r;
    t /= r;

    let table = format!(
        "\naccum alpha-tree walk split — 40,200 facts, mean of {RUNS}\n\
             \n\
             E   class extract              {:>7.2} ms\n\
             G   + has_class                {:>7.2} ms\n\
             I   + walk into reused Vec     {:>7.2} ms\n\
             T   candidates() new Vec       {:>7.2} ms\n\
             \n\
             G−E   class HashMap            {:>7.2} ms\n\
             I−G   walk                     {:>7.2} ms\n\
             T−I   Vec alloc                {:>7.2} ms\n",
        ms(e),
        ms(g),
        ms(i),
        ms(t),
        ms(g - e),
        ms(i - g),
        ms(t - i),
    );
    println!("{table}");
    assert!(i > 0.0, "walk recorded 0 — the loop never ran:{table}");
}

/// Class lookup 3.26 ms: std HashMap vs FxHash vs linear
/// (`DESIGN-STONE-alpha-class-lookup`).
#[test]
fn accum_alpha_class_lookup_split() {
    use rustc_hash::FxHashMap;
    use std::collections::HashMap;
    use std::hint::black_box;

    const RUNS: usize = 3;


    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of seeded session");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    // rune:perspicere(read-once) — one census collect; alias would be a mumble.
    let classes: Vec<Arc<str>> = facts
        .iter()
        .filter_map(|f| match f {
            Value::Aggregate(ag) if ag.nature != Nature::Struct => Some(ag.class.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !classes.is_empty(),
        "compile+seed produced 0 classed facts — lookup split is vacuous"
    );
    let mut unique: Vec<String> = Vec::new();
    for c in &classes {
        let s = c.as_ref();
        if !unique.iter().any(|u| u == s) {
            unique.push(s.to_string());
        }
    }
    let n_types = unique.len();

    // ⛔ THE LIVENESS CHECK ABOVE IS NOT THE CLAIM. `probare` classed this test hollow: it built a
    // 40,200-fact workload, timed three map implementations, printed a table, and asserted only
    // that the clock moved. Everything below is fixed by the workload and was already computed.
    assert_eq!(
        n_types, 2,
        "the accum axis declares exactly two fact types; a change here means the fixture drifted \
         and the class-lookup cost this test reports is for a different shape of network"
    );
    assert_eq!(
        unique,
        vec!["apx::Group".to_string(), "apx::Reading".to_string()],
        "the two types are the axis's own, in first-seen order — if these are not them, the \
         lookup being measured is not the one the accum axis performs"
    );

    let mut std_map: HashMap<String, u8> = HashMap::with_capacity(n_types);
    let mut fx_map: FxHashMap<String, u8> = FxHashMap::default();
    fx_map.reserve(n_types);
    let mut lin: Vec<(String, u8)> = Vec::with_capacity(n_types);
    for (i, u) in unique.iter().enumerate() {
        std_map.insert(u.clone(), i as u8);
        fx_map.insert(u.clone(), i as u8);
        lin.push((u.clone(), i as u8));
    }

    let mut s = 0.0;
    let mut f = 0.0;
    let mut l = 0.0;
    for _ in 0..RUNS {
        s += elapsed_ns(|| {
            for c in &classes {
                black_box(std_map.get(c.as_ref()));
            }
        });
        f += elapsed_ns(|| {
            for c in &classes {
                black_box(fx_map.get(c.as_ref()));
            }
        });
        l += elapsed_ns(|| {
            for c in &classes {
                let cs = c.as_ref();
                black_box(lin.iter().find(|(k, _)| k == cs).map(|(_, v)| v));
            }
        });
    }
    let r = RUNS as f64;
    s /= r;
    f /= r;
    l /= r;
    let best = f.min(l);
    let winner = if l <= f { "L" } else { "F" };

    let table = format!(
        "\naccum alpha class-lookup split — {} facts, {n_types} types, mean of {RUNS}\n\
             types: {unique:?}\n\
             \n\
             S  std HashMap (engine)        {:>7.2} ms\n\
             F  FxHashMap                   {:>7.2} ms\n\
             L  linear Vec                  {:>7.2} ms\n\
             \n\
             S−F                            {:>7.2} ms\n\
             S−L                            {:>7.2} ms\n\
             winner {winner}  cut               {:>7.2} ms\n",
        classes.len(),
        ms(s),
        ms(f),
        ms(l),
        ms(s - f),
        ms(s - l),
        ms(s - best),
    );
    println!("{table}");
    assert!(
        s > 0.0,
        "std HashMap lookup recorded 0 — the loop never ran:{table}"
    );
    assert!(n_types > 0, "zero types — split is vacuous:{table}");
}

/// A−M 3.45 ms: HashMap entry vs Vec push vs d_alpha
/// (`DESIGN-STONE-alpha-push-split`).
#[test]
fn accum_alpha_push_split() {
    use crate::rete::compiled_cond::exec_compiled;
    use std::hint::black_box;

    const RUNS: usize = 3;


    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let staged = "(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200 200)";
    let ast = crate::parse_one!(staged).expect("parse compile+seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised: {e:?}"))
        .value_owned();
    let mut wm = super::to_transient(&session).expect("to_transient of seeded session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for accum network");
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => panic!("seeded session facts are a PersistentVector"),
    };
    // ⛔ WAS ONLY `!facts.is_empty()` — a liveness guard. `probare` classed the tests using it
    // hollow: fixed workload, timed arms, printed table, and nothing asserting WHAT was measured.
    // `[200 200]` yields exactly 40,200 facts (200 Groups + 40,000 Readings). Pin it: a drift
    // makes every millisecond below a figure for a different workload, and the split this test is
    // named for is then not the split it reports.
    assert_eq!(
        facts.len(),
        40_200,
        "workload drifted to {} facts, not the 40,200 the [200 200] accum axis produces — every \
         timing below is measuring something other than this test's subject",
        facts.len()
    );

    let reset = |wm: &mut super::FireSession, d_alpha: &mut AlphaDelta| {
        wm.alpha.clear();
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
        d_alpha.clear();
    };

    let mut m = 0.0;
    let mut h = 0.0;
    let mut v = 0.0;
    let mut d = 0.0;
    let mut a = 0.0;
    for _ in 0..RUNS {
        m += elapsed_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm, &mut FxHashMap::default());
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    black_box(exec_compiled(
                        crate::rete::compiled_cond::test_sym(),
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ));
                }
            }
        });
        h += elapsed_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    if exec_compiled(
                        crate::rete::compiled_cond::test_sym(),
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    )
                    .is_some()
                    {
                        black_box(wm.alpha.entry(*aid).or_default().len());
                    }
                }
            }
        });
        v += elapsed_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            for (i, f) in facts.iter().enumerate() {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    if let Some((off, len)) = exec_compiled(
                        crate::rete::compiled_cond::test_sym(),
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ) {
                        let el = super::make_element(i as u32, off, len);
                        Arc::make_mut(wm.alpha.entry(*aid).or_default()).push(el);
                    }
                }
            }
        });
        d += elapsed_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            for (i, f) in facts.iter().enumerate() {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    if let Some((off, len)) = exec_compiled(
                        crate::rete::compiled_cond::test_sym(),
                        compiled,
                        ag.fields.as_slice(),
                        &mut scratch,
                        &mut wm.bind_intern(),
                        f,
                    ) {
                        let el = super::make_element(i as u32, off, len);
                        let slot = {
                            let mem = Arc::make_mut(wm.alpha.entry(*aid).or_default());
                            mem.push(el);
                            mem.len() - 1
                        };
                        d_alpha.entry(*aid).or_default().push(slot);
                    }
                }
            }
        });
        a += elapsed_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            let mut cand = Vec::new();
            let mut d_alpha: AlphaDelta = FxHashMap::default();
            reset(&mut wm, &mut d_alpha);
            let mut cond_key_ids: CondKeyIds = HashMap::new();
            for (&id, c) in &arm.compiled_conds {
                cond_key_ids.insert(
                    id,
                    crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
                );
            }
            let bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
            for (i, fact) in facts.iter().enumerate() {
                super::alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut super::AlphaActivateCx {
            sym: crate::rete::compiled_cond::test_sym(),
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree: &arm.alpha_tree,
                        compiled_conds: &arm.compiled_conds,
                        match_scratch: &mut scratch,
                        cand_scratch: &mut cand,
                        cond_key_ids: &cond_key_ids,
                        bind_only: &bind_only,
                    },
                )
                .expect("isolated activate");
                black_box(d_alpha.len());
            }
        });
    }
    let r = RUNS as f64;
    m /= r;
    h /= r;
    v /= r;
    d /= r;
    a /= r;

    let table = format!(
        "\naccum alpha push split — 40,200 facts, mean of {RUNS}\n\
             \n\
             M   exec_compiled              {:>7.2} ms\n\
             H   + alpha.entry              {:>7.2} ms\n\
             V   + Vec::push                {:>7.2} ms\n\
             D   + d_alpha.entry            {:>7.2} ms\n\
             A   alpha_activate_fact        {:>7.2} ms\n\
             \n\
             H−M   HashMap entry            {:>7.2} ms\n\
             V−H   Vec push                 {:>7.2} ms\n\
             D−V   d_alpha                  {:>7.2} ms\n\
             A−D   leftover                 {:>7.2} ms\n\
             A−M   push lump                {:>7.2} ms\n",
        ms(m),
        ms(h),
        ms(v),
        ms(d),
        ms(a),
        ms(h - m),
        ms(v - h),
        ms(d - v),
        ms(a - d),
        ms(a - m),
    );
    println!("{table}");
    assert!(
        d > 0.0,
        "d_alpha path recorded 0 — the loop never ran:{table}"
    );
}

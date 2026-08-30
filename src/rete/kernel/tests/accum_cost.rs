//! Accumulator cost — where an `accumulate` node's time goes.


use super::*;

/// Diagnostic — how MANY matcher operations, at the size the phase census apportions.
///
/// Counted rather than timed: one level below `alpha` the operations cost ~100-300ns while a
/// mark pair costs ~52ns, so a timer would tax them 20-50% and — worse — unevenly, in
/// proportion to call count rather than cost. A `Cell` increment is ~1-2ns.
///
/// Arc 278 DESIGN-STONE-compiled-conditions.md — a real fire's step 1 no longer
/// runs `alpha_match_inner`: `match:calls` (and its `match:clause`/`match:bind-insert`
/// siblings) are armed INSIDE `alpha_match_inner`'s own body, so they read zero here
/// now by construction, not by regression. `compiled:calls` is what actually fires
/// on this path — occupancy leaf-fill, skip-span, and `exec_compiled_with_key_ids`
/// all increment it (`DESIGN-STONE-occupancy-leaf-column`). [`exec_compiled`] is
/// the `#[cfg(test)]` door (no interned keys).
///
/// `match:key-alloc` is printed but NOT asserted at zero here: this world's RHS insert forms
/// (`build_insert_fact`, the production pass) resolve `?var` args through the SAME
/// `resolve_operand` alpha-match uses. RHS is compiled (`DESIGN-STONE-compiled-rhs.md`);
/// leftover `match:key-alloc` on this world is the oracle `build_insert_fact` path the
/// differential still runs. So a real fire's `match:key-alloc` can be non-zero even with
/// the compiled path in place; the actual row-2 gate that isolates ALPHA-MATCH's failure path
/// is `compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`, which never
/// touches RHS resolution.
#[test]
fn accum_matcher_op_census() {
    let rows = accum_count_census(200, 200);
    assert!(
        !rows.is_empty(),
        "the operation census counted NOTHING — the counters were never reached, so any \
             rate derived from them would be an artifact"
    );
    let calls = rows
        .iter()
        .find(|(n, _)| *n == "compiled:calls")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    assert!(
        calls > 0,
        "compiled:calls is zero — occupancy fill / skip-span / exec_compiled never counted"
    );

    let mut out = String::from("\naccum matcher ops — G=200 W=200 (40,200 facts)\n");
    for (name, n) in &rows {
        out.push_str(&format!("    {name:<20} {n:>10}\n"));
    }
    println!("{out}");
}

/// Diagnostic — print where the accum fire's time goes, per phase, at two sizes.
///
/// This APPORTIONS; it does not gate. The assertions exist only so it cannot report an artifact:
///   (a) the instrument must have recorded something (an unarmed or never-entered loop would
///       give an empty table that reads as "no time anywhere"),
///   (b) every one of the six phases must appear — a phase missing from the map means its marks
///       were never reached, and its share would silently land on the others.
/// Read the table with `--no-capture`.
#[test]
fn accum_fire_phase_census() {
    // The four OUTER marks partition the whole fire; everything else nests inside one of
    // them. The total is the outer four ONLY — summing a parent with its children is what
    // made the first version of this table report 124% coverage.
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    // ★ 2026-08-01 — this list is now a FLOOR, not the table's contents.
    //
    // It used to BE the table: `for phase in PHASES`. So when `alpha:candidates` was added to
    // mark the discrimination-tree walk — the largest unmarked computation inside the phase
    // that dominates our weakest grid axis — the mark fired and the row simply did not appear.
    // A census that lists its rows cannot report a sink nobody thought to list, which is the
    // whole job of a census. (`feedback_a_gate_that_discovers_beats_one_that_lists`.)
    //
    // Now: the table DISCOVERS every phase the run actually recorded, in first-fired order,
    // and this array is asserted to be a SUBSET of what was discovered — so a mark that is
    // deleted or stops firing still fails loudly, while a mark that is ADDED shows up for
    // free. Both directions covered; neither can go quiet.
    const REQUIRED_PHASES: [&str; 25] = [
        "IN: to_transient",
        "SETUP: indexes",
        "  ├ setup:seen",
        "  │  setup:seen:alloc",
        "  ├ setup:arm",
        "ROUND LOOP",
        "alpha",
        "  ├ alpha:seed",
        "  └ alpha:delta",
        "root-join",
        "hash-join",
        "accumulate",
        "  ├ accum:snapshot",
        "  ├ accum:index",
        "  └ accum:fold",
        "filter",
        "production",
        "OUT: to_persistent",
        "  ├ out:alpha",
        "  ├ out:beta",
        "  ├ out:production",
        "  └ out:query",
        "  ├ round:preamble",
        "  └ round:epilogue",
        "  └ round:drop-memories",
    ];

    let table = render_phase_table(
        "accum fire",
        &[(25, 50), (50, 100), (100, 200), (200, 200)],
        &TOP,
        &REQUIRED_PHASES,
        |g, w| g * (w + 1),
        accum_phase_census,
    );
    println!("{table}");
    let rows = accum_phase_census(200, 200);
    let ns_of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, ns, _)| *ns)
            .unwrap_or(0)
    };
    let fold_ms = ns_of("  └ accum:fold") as f64 / 1e6;
    assert!(
        fold_ms > 0.0,
        "accum:fold at [200 200] recorded zero — the mark moved"
    );
    assert!(
        fold_ms < 25.0,
        "accum:fold at [200 200] is {fold_ms:.2} ms — DESIGN-STONE-accum-fold-the-wall \
             requires < 25 ms (was 68.49).{table}"
    );
    let snap_ms = ns_of("  ├ accum:snapshot") as f64 / 1e6;
    assert!(
        snap_ms > 0.0,
        "accum:snapshot at [200 200] recorded zero — the mark was deleted"
    );
    assert!(
        snap_ms < 1.0,
        "accum:snapshot at [200 200] is {snap_ms:.2} ms — \
             DESIGN-STONE-gather-no-snapshot requires < 1 ms (was 5.56).{table}"
    );
}

const ACCUM_QUERY_TAIL: &str = "\n\
(:wat::rete::defquery :apx::q-CountF\n\
  :params []\n\
  :when [(?fact <- :apx::CountF)])\n\
(:wat::rete::defquery :apx::q-SumF\n\
  :params []\n\
  :when [(?fact <- :apx::SumF)])\n\
(:wat::rete::defquery :apx::q-MinF\n\
  :params []\n\
  :when [(?fact <- :apx::MinF)])\n\
(:wat::rete::defquery :apx::q-MaxF\n\
  :params []\n\
  :when [(?fact <- :apx::MaxF)])\n\
(:wat::rete::defquery :apx::q-ExistsF\n\
  :params []\n\
  :when [(?fact <- :apx::ExistsF)])\n";

/// Rank accum `[200 200]` FIRE with vs without the five grid queries
/// (`DESIGN-STONE-accum-class-index`). Census compiles without queries.
#[test]
fn accum_query_harvest_split() {
    use std::time::Instant;

    const G: i64 = 200;
    const W: i64 = 200;
    const RUNS: usize = 3;
    const HARVEST: &str = "  ├ harvest:query";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];

    struct Shot {
        wall: f64,
        fire: f64,
        setup: f64,
        round: f64,
        alpha: f64,
        harvest: f64,
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
            format!("{ACCUM_AXIS_WORLD}{ACCUM_QUERY_TAIL}")
        } else {
            ACCUM_AXIS_WORLD.to_string()
        };
        let world = startup_from_source(&world_src, None, Arc::new(InMemoryLoader::new()))
            .expect("accum query-harvest world should freeze");
        let compile = if with_query {
            "(:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :apx) \
              (:wat::core::PersistentVector \
                (:apx::q-CountF) (:apx::q-SumF) (:apx::q-MinF) \
                (:apx::q-MaxF) (:apx::q-ExistsF))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))"
        } else {
            "(:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))"
        };
        let seed_src = format!("(:apx::seed {compile} {G} {W})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None).unwrap_or_else(|e| {
                panic!("fire-rules raised with_query={with_query}: {e:?}")
            })
        });
        let wall = t0.elapsed().as_nanos() as f64;
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        let fire: u64 = FIRE_PHASES.iter().map(|n| of(n)).sum();
        Shot {
            wall,
            fire: fire as f64,
            setup: of("SETUP: indexes") as f64,
            round: of("ROUND LOOP") as f64,
            alpha: of("alpha") as f64,
            harvest: of(HARVEST) as f64,
            query_maps: query_map_count(&fired),
        }
    };

    let mut without = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
        alpha: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
        alpha: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    for _ in 0..RUNS {
        let a = shot(false);
        let b = shot(true);
        without.wall += a.wall;
        without.fire += a.fire;
        without.setup += a.setup;
        without.round += a.round;
        without.alpha += a.alpha;
        without.harvest += a.harvest;
        without.query_maps = a.query_maps;
        with.wall += b.wall;
        with.fire += b.fire;
        with.setup += b.setup;
        with.round += b.round;
        with.alpha += b.alpha;
        with.harvest += b.harvest;
        with.query_maps = b.query_maps;
    }
    let r = RUNS as f64;
    without.wall /= r;
    without.fire /= r;
    without.setup /= r;
    without.round /= r;
    without.alpha /= r;
    without.harvest /= r;
    with.wall /= r;
    with.fire /= r;
    with.setup /= r;
    with.round /= r;
    with.alpha /= r;
    with.harvest /= r;

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\naccum query harvest split — [200 200], mean of {RUNS}\n\
             \n\
             without queries       wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  alpha {:>7.2}  harvest {:>7.2}  maps {}\n\
             with    five q-*      wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  alpha {:>7.2}  harvest {:>7.2}  maps {}\n\
             delta (query tax)            {:>7.2} ms   harvest {:>7.2}  ROUND-harvest {:>7.2}\n",
        ms(without.wall),
        ms(without.fire),
        ms(without.setup),
        ms(without.round),
        ms(without.alpha),
        ms(without.harvest),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        ms(with.setup),
        ms(with.round),
        ms(with.alpha),
        ms(with.harvest),
        with.query_maps,
        ms(with.wall - without.wall),
        ms(with.harvest - without.harvest),
        ms((with.round - without.round) - (with.harvest - without.harvest)),
    );
    assert_eq!(without.query_maps, 0, "compile without queries has empty query-memory");
    assert_eq!(
        with.query_maps, 1000,
        "five types × 200 groups = 1000 query maps"
    );
}

/// Apportion accum harvest:query (6.23 ms / 1k maps) into all-class
/// index vs wanted-only vs derived-only vs wrap
/// (`DESIGN-STONE-accum-wanted-harvest`). No fire-path change.
#[test]
fn accum_harvest_index_parts() {
    use std::collections::{HashMap, HashSet};
    use std::hint::black_box;
    use std::time::Instant;

    const G: usize = 200;
    const W: usize = 200;
    const RUNS: usize = 3;
    const GROUP: &str = "apx::Group";
    const READING: &str = "apx::Reading";
    const WANTED: [&str; 5] = [
        "apx::CountF",
        "apx::SumF",
        "apx::MinF",
        "apx::MaxF",
        "apx::ExistsF",
    ];

    let rec = |class: &str, names: &Arc<Vec<String>>, fields: Vec<Value>| {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            Arc::clone(names),
            Arc::new(fields),
        )))
    };
    let g_names = Arc::new(vec!["g".into()]);
    let r_names = Arc::new(vec!["g".into(), "v".into()]);
    let n_names = Arc::new(vec!["g".into(), "n".into()]);

    let mut input: Vec<Value> = Vec::with_capacity(G + G * W);
    for g in 0..G {
        input.push(rec(GROUP, &g_names, vec![Value::i64(g as i64)]));
        for j in 0..W {
            input.push(rec(
                READING,
                &r_names,
                vec![Value::i64(g as i64), Value::i64(j as i64)],
            ));
        }
    }
    let mut derived: Vec<Value> = Vec::with_capacity(G * WANTED.len());
    for class in WANTED {
        for g in 0..G {
            if class == "apx::ExistsF" {
                derived.push(rec(class, &g_names, vec![Value::i64(g as i64)]));
            } else {
                derived.push(rec(
                    class,
                    &n_names,
                    vec![Value::i64(g as i64), Value::i64(W as i64)],
                ));
            }
        }
    }
    let input_pv = crate::value::pvec::PVec::from_vec(input);
    let wanted: HashSet<&str> = WANTED.iter().copied().collect();
    let var = Value::String(Arc::new("?fact".to_string()));

    let index_all = || -> HashMap<&str, Vec<&Value>> {
        let mut idx: HashMap<&str, Vec<&Value>> = HashMap::new();
        for f in input_pv.iter() {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        for f in &derived {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        idx
    };
    let index_wanted_both = || -> HashMap<&str, Vec<&Value>> {
        let mut idx: HashMap<&str, Vec<&Value>> = HashMap::new();
        for f in input_pv.iter() {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct && wanted.contains(a.class.as_ref()) {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        for f in &derived {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct && wanted.contains(a.class.as_ref()) {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        idx
    };
    let index_wanted_derived = || -> HashMap<&str, Vec<&Value>> {
        let mut idx: HashMap<&str, Vec<&Value>> = HashMap::new();
        for f in &derived {
            if let Value::Aggregate(a) = f {
                if a.nature != Nature::Struct && wanted.contains(a.class.as_ref()) {
                    idx.entry(a.class.as_ref()).or_default().push(f);
                }
            }
        }
        idx
    };

    fn time_ns(n: usize, mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        for _ in 0..n {
            body();
        }
        t0.elapsed().as_nanos() as f64 / n as f64
    }

    {
        black_box(index_all());
        black_box(index_wanted_both());
        black_box(index_wanted_derived());
        let facts: Vec<&Value> = derived.iter().collect();
        let maps: Vec<crate::value::pmap::PMap> = facts
            .iter()
            .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
            .collect();
        black_box(maps);
    }

    let mut i = 0.0;
    let mut w = 0.0;
    let mut d = 0.0;
    let mut m = 0.0;
    let mut maps_n = 0usize;
    for _ in 0..RUNS {
        i += time_ns(1, || {
            black_box(index_all());
        });
        w += time_ns(1, || {
            black_box(index_wanted_both());
        });
        d += time_ns(1, || {
            black_box(index_wanted_derived());
        });
        let facts: Vec<&Value> = derived.iter().collect();
        maps_n = facts.len();
        m += time_ns(1, || {
            let maps: Vec<crate::value::pmap::PMap> = facts
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        });
    }
    let runs = RUNS as f64;
    i /= runs;
    w /= runs;
    d /= runs;
    m /= runs;
    assert!(i > 0.0, "all-class index recorded 0 ns — the loop never ran");
    assert_eq!(maps_n, 1000, "five types × 200 groups = 1000 maps");

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\naccum harvest index parts — [200 200], mean of {RUNS}\n\
             input 200 Group + 40,000 Reading; derived 1,000\n\
             \n\
             I  both bags, every class             {:>7.2} ms\n\
             W  both bags, wanted only             {:>7.2} ms\n\
             D  derived only, wanted only          {:>7.2} ms\n\
             M  wrap 1,000 maps                    {:>7.2} ms\n\
             I−W  Reading vec                      {:>7.2} ms\n\
             W−D  input walk                       {:>7.2} ms\n",
        ms(i),
        ms(w),
        ms(d),
        ms(m),
        ms(i - w),
        ms(w - d),
    );
}

/// Leftover accum `[200 200]`: alpha remainder vs tax, then the
/// named engine rows (`DESIGN-STONE-accum-leftover-split`).
#[test]
fn accum_leftover_split() {
    const RUNS: usize = 3;
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const ALPHA_KIDS: [&str; 4] = [
        "  ├ alpha:candidates",
        "  ├ alpha:match",
        "  ├ alpha:element",
        "  └ alpha:push",
    ];
    const PROD_KIDS: [&str; 2] = ["  ├ prod:compiled-rhs", "  ├ prod:dedup-store"];

    let cal = calibrate_mark_ns();

    let mut fire = 0.0;
    let mut alpha_raw = 0.0;
    let mut kid_raw = [0.0; 4];
    let mut kid_pairs = [0u64; 4];
    let mut prod_raw = 0.0;
    let mut pk_raw = [0.0; 2];
    let mut pk_pairs = [0u64; 2];
    let mut seen_raw = 0.0;
    let mut drop_raw = 0.0;
    let mut fold_raw = 0.0;
    let mut index_raw = 0.0;
    let mut snap_raw = 0.0;
    let mut accum_raw = 0.0;
    let mut filter_raw = 0.0;
    let mut hash_raw = 0.0;
    let mut out_raw = 0.0;
    let mut alpha_pairs = 0u64;
    let mut prod_pairs = 0u64;
    let mut seen_pairs = 0u64;
    let mut drop_pairs = 0u64;
    let mut fold_pairs = 0u64;
    for _ in 0..RUNS {
        let rows = accum_phase_census(200, 200);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire += TOP.iter().map(|n| of(n).0 as f64).sum::<f64>();
        let (a_ns, a_k) = of("alpha");
        alpha_raw += a_ns as f64;
        alpha_pairs = a_k;
        for (i, name) in ALPHA_KIDS.iter().enumerate() {
            let (ns, k) = of(name);
            kid_raw[i] += ns as f64;
            kid_pairs[i] = k;
        }
        let (p_ns, p_k) = of("production");
        prod_raw += p_ns as f64;
        prod_pairs = p_k;
        for (i, name) in PROD_KIDS.iter().enumerate() {
            let (ns, k) = of(name);
            pk_raw[i] += ns as f64;
            pk_pairs[i] = k;
        }
        let (s_ns, s_k) = of("  ├ setup:seen");
        seen_raw += s_ns as f64;
        seen_pairs = s_k;
        let (d_ns, d_k) = of("  └ round:drop-memories");
        drop_raw += d_ns as f64;
        drop_pairs = d_k;
        let (f_ns, f_k) = of("  └ accum:fold");
        fold_raw += f_ns as f64;
        fold_pairs = f_k;
        index_raw += of("  ├ accum:index").0 as f64;
        snap_raw += of("  ├ accum:snapshot").0 as f64;
        accum_raw += of("accumulate").0 as f64;
        filter_raw += of("filter").0 as f64;
        hash_raw += of("hash-join").0 as f64;
        out_raw += of("OUT: to_persistent").0 as f64;
    }
    let r = RUNS as f64;
    fire /= r;
    alpha_raw /= r;
    prod_raw /= r;
    seen_raw /= r;
    drop_raw /= r;
    fold_raw /= r;
    index_raw /= r;
    snap_raw /= r;
    accum_raw /= r;
    filter_raw /= r;
    hash_raw /= r;
    out_raw /= r;
    for x in &mut kid_raw {
        *x /= r;
    }
    for x in &mut pk_raw {
        *x /= r;
    }
    let net = |raw: f64, pairs: u64| raw - pairs as f64 * cal;
    let kid_net: [f64; 4] = std::array::from_fn(|i| net(kid_raw[i], kid_pairs[i]));
    let pk_net: [f64; 2] = std::array::from_fn(|i| net(pk_raw[i], pk_pairs[i]));
    // Child timers retired: remainder/tax of those pairs is 0, and
    // the outer `alpha` row *is* honest_alpha (`DESIGN-STONE-retire-alpha-child-marks`).
    let kids_retired = kid_pairs.iter().all(|k| *k == 0);
    let remainder_alpha = if kids_retired {
        0.0
    } else {
        alpha_raw - kid_raw.iter().sum::<f64>()
    };
    let tax_alpha: f64 = if kids_retired {
        0.0
    } else {
        kid_pairs.iter().map(|k| *k as f64 * cal).sum()
    };
    let honest_alpha: f64 = if kids_retired {
        net(alpha_raw, alpha_pairs).max(0.0)
    } else {
        kid_net.iter().map(|n| n.max(0.0)).sum()
    };
    let remainder_prod = prod_raw - pk_raw.iter().sum::<f64>();
    let tax_prod: f64 = pk_pairs.iter().map(|k| *k as f64 * cal).sum();
    let honest_prod: f64 = pk_net.iter().map(|n| n.max(0.0)).sum();
    let honest_fire = fire - remainder_alpha - tax_alpha - remainder_prod - tax_prod;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum leftover split — [200 200], mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             FIRE                      {:>7.2} ms\n\
             alpha                     {:>7.2} ms raw  {:>6}x\n\
               candidates              {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               match                   {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               element                 {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               push                    {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             remainder_alpha           {:>7.2} ms\n\
             tax_in_alpha              {:>7.2} ms\n\
             honest_alpha              {:>7.2} ms\n\
             \n\
             setup:seen                {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             drop-memories             {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             accumulate                {:>7.2} ms\n\
               snapshot                {:>7.2} ms\n\
               index                   {:>7.2} ms\n\
               fold                    {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             production                {:>7.2} ms raw  {:>6}x\n\
               compiled-rhs            {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               dedup-store             {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             remainder_prod            {:>7.2} ms\n\
             tax_in_prod               {:>7.2} ms\n\
             honest_prod               {:>7.2} ms\n\
             filter                    {:>7.2} ms\n\
             hash-join                 {:>7.2} ms\n\
             OUT                       {:>7.2} ms\n\
             \n\
             honest_FIRE               {:>7.2} ms\n",
        ms(fire),
        ms(alpha_raw),
        alpha_pairs,
        ms(kid_raw[0]),
        ms(kid_net[0]),
        kid_pairs[0],
        ms(kid_raw[1]),
        ms(kid_net[1]),
        kid_pairs[1],
        ms(kid_raw[2]),
        ms(kid_net[2]),
        kid_pairs[2],
        ms(kid_raw[3]),
        ms(kid_net[3]),
        kid_pairs[3],
        ms(remainder_alpha),
        ms(tax_alpha),
        ms(honest_alpha),
        ms(seen_raw),
        ms(net(seen_raw, seen_pairs)),
        seen_pairs,
        ms(drop_raw),
        ms(net(drop_raw, drop_pairs)),
        drop_pairs,
        ms(accum_raw),
        ms(snap_raw),
        ms(index_raw),
        ms(fold_raw),
        ms(net(fold_raw, fold_pairs)),
        fold_pairs,
        ms(prod_raw),
        prod_pairs,
        ms(pk_raw[0]),
        ms(pk_net[0]),
        pk_pairs[0],
        ms(pk_raw[1]),
        ms(pk_net[1]),
        pk_pairs[1],
        ms(remainder_prod),
        ms(tax_prod),
        ms(honest_prod),
        ms(filter_raw),
        ms(hash_raw),
        ms(out_raw),
        ms(honest_fire),
    );
    println!("{table}");
    assert!(fire > 0.0, "FIRE recorded 0 — the fire never ran:{table}");
    assert!(
        alpha_pairs > 0,
        "alpha recorded 0 pairs — leftover split is a dead fire:{table}"
    );
    assert!(
        kids_retired,
        "alpha child timers still fire — pairs {kid_pairs:?}:{table}"
    );
    assert!(
        honest_fire > 0.0,
        "honest_FIRE must be > 0 after retiring child tax:{table}"
    );
}

/// `M−T` 7.65 ms: ops vs intern (`DESIGN-STONE-compiled-match-split`).
#[test]
fn accum_compiled_match_split() {
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

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
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let n_fact_bind = arm
        .compiled_conds
        .values()
        .filter(|c| c.fact_bind().is_some())
        .count();
    let mut n_cands = 0u64;
    let mut n_ops_true = 0u64;
    {
        let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
        for f in &facts {
            let Value::Aggregate(ag) = f else { continue };
            if ag.nature == Nature::Struct {
                continue;
            }
            let alphas = arm
                .alpha_tree
                .candidates(ag.class.as_ref(), ag.fields.as_slice());
            n_cands += alphas.len() as u64;
            for aid in &alphas {
                let compiled =
                    super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                scratch.clear();
                scratch.resize(compiled.n_slots(), None);
                if crate::rete::compiled_cond::exec_ops(
                    compiled.ops(),
                    &mut scratch,
                    ag.fields.as_slice(),
                    true,
                    crate::rete::compiled_cond::test_exec_cx(),
                ) {
                    n_ops_true += 1;
                }
            }
        }
    }

    let reset_pools = |wm: &mut super::FireSession| {
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
    };

    let mut t = 0.0;
    let mut o = 0.0;
    let mut mc = 0.0;
    let mut mw = 0.0;
    for _ in 0..RUNS {
        t += time_ns(|| {
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                black_box(
                    arm.alpha_tree
                        .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                );
            }
        });
        o += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(crate::rete::compiled_cond::exec_ops(
                        compiled.ops(),
                        &mut scratch,
                        ag.fields.as_slice(),
                        true,
                        crate::rete::compiled_cond::test_exec_cx(),
                    ));
                }
            }
        });
        mc += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset_pools(&mut wm);
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
    }
    {
        let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
        reset_pools(&mut wm);
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
                let _ = crate::rete::compiled_cond::exec_compiled(
                    crate::rete::compiled_cond::test_sym(),
                    compiled,
                    ag.fields.as_slice(),
                    &mut scratch,
                    &mut wm.bind_intern(),
                    f,
                );
            }
        }
        for _ in 0..RUNS {
            mw += time_ns(|| {
                let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
                wm.bind_pool.clear();
                for f in &facts {
                    let Value::Aggregate(ag) = f else { continue };
                    if ag.nature == Nature::Struct {
                        continue;
                    }
                    let alphas = arm
                        .alpha_tree
                        .candidates(ag.class.as_ref(), ag.fields.as_slice());
                    for aid in &alphas {
                        let compiled = super::rematch_compiled(&arm.compiled_conds, *aid)
                            .expect("compiled cond");
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
        }
    }
    let r = RUNS as f64;
    t /= r;
    o /= r;
    mc /= r;
    mw /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum compiled-match split — 40,200 facts, mean of {RUNS}\n\
             fact_bind conds {n_fact_bind}   candidates {n_cands}   ops-true {n_ops_true}\n\
             \n\
             T   candidates                 {:>7.2} ms\n\
             O   + exec_ops                 {:>7.2} ms\n\
             Mc  + exec_compiled (cold)     {:>7.2} ms\n\
             Mw  + exec_compiled (warm)     {:>7.2} ms\n\
             \n\
             O−T   ops                      {:>7.2} ms\n\
             Mc−O  intern/materialize cold  {:>7.2} ms\n\
             Mc−Mw intern cold tax          {:>7.2} ms\n",
        ms(t),
        ms(o),
        ms(mc),
        ms(mw),
        ms(o - t),
        ms(mc - o),
        ms(mc - mw),
    );
    println!("{table}");
    assert!(o > 0.0, "exec_ops recorded 0 — the loop never ran:{table}");
    assert!(n_cands > 0, "zero candidates — split is vacuous:{table}");
}

/// intern/materialize 6.18 ms: clone vs intern_key vs intern_val vs push
/// (`DESIGN-STONE-materialize-split`).
#[test]
fn accum_materialize_split() {
    use crate::rete::compiled_cond::{exec_ops, intern_key, intern_val, materialize_into};
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

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
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let reset = |wm: &mut super::FireSession| {
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.i64_by_fact.clear();
    };

    let mut o = 0.0;
    let mut c = 0.0;
    let mut k = 0.0;
    let mut v = 0.0;
    let mut p = 0.0;
    let mut m = 0.0;
    for _ in 0..RUNS {
        o += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(exec_ops(
                        compiled.ops(),
                        &mut scratch,
                        ag.fields.as_slice(),
                        true,
                        crate::rete::compiled_cond::test_exec_cx(),
                    ));
                }
            }
        });
        c += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true, crate::rete::compiled_cond::test_exec_cx()) {
                        continue;
                    }
                    for &slot in compiled.output_slots() {
                        black_box(scratch.get(slot).and_then(|x| x.clone()));
                    }
                }
            }
        });
        k += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true, crate::rete::compiled_cond::test_exec_cx()) {
                        continue;
                    }
                    for &slot in compiled.output_slots() {
                        black_box(scratch.get(slot).and_then(|x| x.clone()));
                    }
                    for key in compiled.slot_keys() {
                        black_box(intern_key(&mut wm.bind_keys, key));
                    }
                }
            }
        });
        v += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true, crate::rete::compiled_cond::test_exec_cx()) {
                        continue;
                    }
                    for (i, &slot) in compiled.output_slots().iter().enumerate() {
                        let Some(val) = scratch.get(slot).and_then(|x| x.clone()) else {
                            continue;
                        };
                        black_box(intern_key(&mut wm.bind_keys, &compiled.slot_keys()[i]));
                        black_box(intern_val(&mut wm.bind_vals, &mut wm.bind_val_ids, val));
                    }
                }
            }
        });
        p += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true, crate::rete::compiled_cond::test_exec_cx()) {
                        continue;
                    }
                    for (i, &slot) in compiled.output_slots().iter().enumerate() {
                        let Some(val) = scratch.get(slot).and_then(|x| x.clone()) else {
                            continue;
                        };
                        let kid = intern_key(&mut wm.bind_keys, &compiled.slot_keys()[i]);
                        let vid = intern_val(&mut wm.bind_vals, &mut wm.bind_val_ids, val);
                        wm.bind_pool.push((kid, vid));
                    }
                }
            }
        });
        m += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
            reset(&mut wm);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true, crate::rete::compiled_cond::test_exec_cx()) {
                        continue;
                    }
                    black_box(materialize_into(
                        compiled,
                        &scratch,
                        &mut crate::rete::compiled_cond::BindIntern {
                            keys: &mut wm.bind_keys,
                            vals: &mut wm.bind_vals,
                            ids: &mut wm.bind_val_ids,
                            pool: &mut wm.bind_pool,
                        },
                        f,
                        None,
                    ));
                }
            }
        });
    }
    let r = RUNS as f64;
    o /= r;
    c /= r;
    k /= r;
    v /= r;
    p /= r;
    m /= r;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum materialize split — 40,200 facts, mean of {RUNS}\n\
             \n\
             O   exec_ops                   {:>7.2} ms\n\
             C   + clone slots              {:>7.2} ms\n\
             K   + intern_key               {:>7.2} ms\n\
             V   + intern_val               {:>7.2} ms\n\
             P   + pool.push                {:>7.2} ms\n\
             M   + materialize_into         {:>7.2} ms\n\
             \n\
             C−O   clone                    {:>7.2} ms\n\
             K−C   intern_key               {:>7.2} ms\n\
             V−K   intern_val               {:>7.2} ms\n\
             P−V   pool.push                {:>7.2} ms\n\
             M−P   materialize leftover     {:>7.2} ms\n",
        ms(o),
        ms(c),
        ms(k),
        ms(v),
        ms(p),
        ms(m),
        ms(c - o),
        ms(k - c),
        ms(v - k),
        ms(p - v),
        ms(m - p),
    );
    println!("{table}");
    assert!(
        v > 0.0,
        "intern_val recorded 0 — the loop never ran:{table}"
    );
}

/// intern_val 2.77 ms: Value map vs i64 map vs small-int table
/// (`DESIGN-STONE-intern-val-i64`).
#[test]
fn accum_intern_val_i64_split() {
    use crate::rete::compiled_cond::exec_ops;
    use rustc_hash::FxHashMap;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

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

    let mut payloads: Vec<Value> = Vec::new();
    {
        let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
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
                scratch.clear();
                scratch.resize(compiled.n_slots(), None);
                if !exec_ops(compiled.ops(), &mut scratch, ag.fields.as_slice(), true, crate::rete::compiled_cond::test_exec_cx()) {
                    continue;
                }
                for &slot in compiled.output_slots() {
                    if let Some(v) = scratch.get(slot).and_then(|x| x.clone()) {
                        payloads.push(v);
                    }
                }
            }
        }
    }
    assert!(
        !payloads.is_empty(),
        "no interned fillers — intern_val split is vacuous"
    );
    let mut n_i64 = 0u64;
    let mut n_other = 0u64;
    let mut min_i = i64::MAX;
    let mut max_i = i64::MIN;
    for v in &payloads {
        match v {
            Value::i64(n) => {
                n_i64 += 1;
                min_i = min_i.min(*n);
                max_i = max_i.max(*n);
            }
            _ => n_other += 1,
        }
    }
    let table_ok = n_other == 0 && min_i >= 0 && max_i < 4096;

    let mut vns = 0.0;
    let mut ins = 0.0;
    let mut ans = 0.0;
    for _ in 0..RUNS {
        vns += time_ns(|| {
            let mut vals = Vec::new();
            let mut ids = crate::rete::compiled_cond::ValIntern::default();
            for v in &payloads {
                black_box(crate::rete::compiled_cond::intern_val(
                    &mut vals,
                    &mut ids,
                    v.clone(),
                ));
            }
        });
        ins += time_ns(|| {
            let mut vals = Vec::new();
            let mut ids: FxHashMap<i64, u32> = FxHashMap::default();
            for v in &payloads {
                let Value::i64(n) = *v else {
                    panic!("non-i64 in I arm");
                };
                let id = if let Some(&id) = ids.get(&n) {
                    id
                } else {
                    let id = vals.len() as u32;
                    ids.insert(n, id);
                    vals.push(Value::i64(n));
                    id
                };
                black_box(id);
            }
        });
        if table_ok {
            ans += time_ns(|| {
                let mut vals = Vec::new();
                let mut slot = vec![u32::MAX; (max_i as usize) + 1];
                for v in &payloads {
                    let Value::i64(n) = *v else { panic!() };
                    let i = n as usize;
                    let id = if slot[i] != u32::MAX {
                        slot[i]
                    } else {
                        let id = vals.len() as u32;
                        slot[i] = id;
                        vals.push(Value::i64(n));
                        id
                    };
                    black_box(id);
                }
            });
        }
    }
    let r = RUNS as f64;
    vns /= r;
    ins /= r;
    if table_ok {
        ans /= r;
    }
    let best = if table_ok { ins.min(ans) } else { ins };
    let winner = if table_ok && ans <= ins { "A" } else { "I" };

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum intern_val i64 split — {} fillers, mean of {RUNS}\n\
             i64 {n_i64}  other {n_other}  min {min_i}  max {max_i}  table_ok {table_ok}\n\
             \n\
             V  FxHashMap<Value> (engine)   {:>7.2} ms\n\
             I  FxHashMap<i64>              {:>7.2} ms\n\
             A  slot table                  {:>7.2} ms\n\
             \n\
             V−I                            {:>7.2} ms\n\
             V−A                            {:>7.2} ms\n\
             winner {winner}  cut               {:>7.2} ms\n",
        payloads.len(),
        ms(vns),
        ms(ins),
        if table_ok { ms(ans) } else { f64::NAN },
        ms(vns - ins),
        if table_ok { ms(vns - ans) } else { f64::NAN },
        ms(vns - best),
    );
    println!("{table}");
    assert!(
        vns > 0.0,
        "intern_val recorded 0 — the loop never ran:{table}"
    );
    assert!(n_i64 > 0, "zero i64 fillers — split is vacuous:{table}");
}

/// O−T 1.90 ms: scratch reset vs exec_ops body
/// (`DESIGN-STONE-exec-ops-split`).
#[test]
fn accum_exec_ops_split() {
    use crate::rete::compiled_cond::exec_ops;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

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
    assert!(
        !facts.is_empty(),
        "compile+seed produced 0 facts — isolated loops would be vacuous"
    );

    let mut t = 0.0;
    let mut rset = 0.0;
    let mut f = 0.0;
    let mut o = 0.0;
    for _ in 0..RUNS {
        t += time_ns(|| {
            for f in &facts {
                let Value::Aggregate(ag) = f else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                black_box(
                    arm.alpha_tree
                        .candidates(ag.class.as_ref(), ag.fields.as_slice()),
                );
            }
        });
        rset += time_ns(|| {
            let mut scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(scratch.len());
                }
            }
        });
        f += time_ns(|| {
            let mut scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
            for fct in &facts {
                let Value::Aggregate(ag) = fct else { continue };
                if ag.nature == Nature::Struct {
                    continue;
                }
                let alphas = arm
                    .alpha_tree
                    .candidates(ag.class.as_ref(), ag.fields.as_slice());
                for aid in &alphas {
                    let compiled =
                        super::rematch_compiled(&arm.compiled_conds, *aid).expect("compiled cond");
                    let n = compiled.n_slots();
                    if scratch.len() != n {
                        scratch.resize(n, None);
                    }
                    scratch.fill(None);
                    black_box(scratch.len());
                }
            }
        });
        o += time_ns(|| {
            let mut scratch = Vec::with_capacity(arm.compiled_max_slots);
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
                    scratch.clear();
                    scratch.resize(compiled.n_slots(), None);
                    black_box(exec_ops(
                        compiled.ops(),
                        &mut scratch,
                        ag.fields.as_slice(),
                        true,
                        crate::rete::compiled_cond::test_exec_cx(),
                    ));
                }
            }
        });
    }
    let runs = RUNS as f64;
    t /= runs;
    rset /= runs;
    f /= runs;
    o /= runs;

    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\naccum exec_ops split — 40,200 facts, mean of {RUNS}\n\
             \n\
             T   candidates                 {:>7.2} ms\n\
             R   + clear/resize             {:>7.2} ms\n\
             F   + fill(None)               {:>7.2} ms\n\
             O   + exec_ops                 {:>7.2} ms\n\
             \n\
             R−T   scratch clear/resize     {:>7.2} ms\n\
             F−T   scratch fill             {:>7.2} ms\n\
             O−R   exec_ops body            {:>7.2} ms\n\
             O−T   ops lump                 {:>7.2} ms\n",
        ms(t),
        ms(rset),
        ms(f),
        ms(o),
        ms(rset - t),
        ms(f - t),
        ms(o - rset),
        ms(o - t),
    );
    println!("{table}");
    assert!(o > 0.0, "exec_ops recorded 0 — the loop never ran:{table}");
}

/// In-fire `setup:seen` alloc vs insert, plus isolated on real seeded facts
/// (`DESIGN-STONE-seen-fire-context`).
#[test]
fn accum_seen_fire_context_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const RUNS: usize = 3;
    const G: i64 = 200;
    const W: i64 = 200;

    let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("accum-axis world should freeze");
    let src =
        format!("(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {G} {W})");
    let ast = crate::parse_one!(src.as_str()).expect("parse seed");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient");
    let pv: crate::value::pvec::PVec = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => panic!("seeded facts is a PersistentVector"),
    };
    let n = pv.len();
    assert!(
        n > 40_000,
        "seeded [200 200] must hold ~40,200 facts, got {n}"
    );

    fn time_ns(mut body: impl FnMut()) -> f64 {
        let t0 = Instant::now();
        body();
        t0.elapsed().as_nanos() as f64
    }

    let mut a = 0.0;
    let mut x = 0.0;
    let mut s = 0.0;
    for _ in 0..RUNS {
        a += time_ns(|| {
            let ids: FxHashSet<u64> = FxHashSet::with_capacity_and_hasher(n, Default::default());
            let rest: FxHashSet<Value> = FxHashSet::default();
            black_box(ids.len() + rest.len());
        });
        x += time_ns(|| {
            let mut sum = 0u64;
            for f in pv.iter() {
                if let Value::Aggregate(ag) = f {
                    sum = sum.wrapping_add(ag.identity());
                }
            }
            black_box(sum);
        });
        s += time_ns(|| {
            let mut ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(n, Default::default());
            let mut rest: FxHashSet<Value> = FxHashSet::default();
            for f in pv.iter() {
                super::seen_insert(&mut ids, &mut rest, f);
            }
            black_box(ids.len() + rest.len());
        });
    }

    let mut fire_seen = 0.0;
    let mut fire_alloc = 0.0;
    let mut fire_ins = 0.0;
    for _ in 0..RUNS {
        let rows = accum_phase_census(G, W);
        let of = |name: &str| -> u64 {
            rows.iter()
                .find(|(nm, _, _)| *nm == name)
                .map(|(_, ns, _)| *ns)
                .unwrap_or(0)
        };
        fire_seen += of("  ├ setup:seen") as f64;
        fire_alloc += of("  │  setup:seen:alloc") as f64;
        fire_ins += of("  │  setup:seen:insert") as f64;
    }
    let r = RUNS as f64;
    a /= r;
    x /= r;
    s /= r;
    fire_seen /= r;
    fire_alloc /= r;
    fire_ins /= r;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\nseen fire-context split — accum [{G} {W}], {n} facts, mean of {RUNS}\n\
             \n\
             in-fire\n\
             setup:seen                    {:>7.2} ms\n\
               alloc                       {:>7.2} ms\n\
               insert                      {:>7.2} ms\n\
             \n\
             isolated (same seeded PV)\n\
             A  HashSet with_capacity      {:>7.2} ms\n\
             X  identity() walk            {:>7.2} ms\n\
             S  seen_insert loop           {:>7.2} ms\n\
             \n\
             S−A  insert beyond alloc      {:>7.2} ms\n\
             in-fire insert − S            {:>7.2} ms\n\
             in-fire seen − S              {:>7.2} ms\n",
        ms(fire_seen),
        ms(fire_alloc),
        ms(fire_ins),
        ms(a),
        ms(x),
        ms(s),
        ms(s - a),
        ms(fire_ins - s),
        ms(fire_seen - s),
    );
    println!("{table}");
    assert!(
        s > 0.0,
        "isolated seen_insert recorded 0 — the loop never ran:{table}"
    );
}

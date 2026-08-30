//! Stratification cost — merge, clone and stratum-split accounting for a stratified fire.


use super::*;

/// Rank strat-neg `[6 2000]` FIRE with vs without the ten grid queries
/// (`DESIGN-STONE-strat-neg-harvest-split`). Grid compile-alls q-S0..q-S9.
#[test]
fn strat_neg_query_harvest_split() {
    use std::time::Instant;

    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const RUNS: usize = 3;
    const HARVEST: &str = "  ├ harvest:query";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const WORLD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/wat-scripts/perf/grid/strat-neg.wat"));
    const QUERIES: &str = "(:wat::core::PersistentVector \
        (:strat::q-S0) (:strat::q-S1) (:strat::q-S2) (:strat::q-S3) (:strat::q-S4) \
        (:strat::q-S5) (:strat::q-S6) (:strat::q-S7) (:strat::q-S8) (:strat::q-S9))";

    struct Shot {
        wall: f64,
        fire: f64,
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

    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("strat-neg query-harvest world should freeze");

    let shot = |with_query: bool| -> Shot {
        let compile = if with_query {
            format!("(:wat::core::match (:wat::rete::compile-all (:strat::build-rules {STRATA}) {QUERIES}) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))")
        } else {
            format!("(:wat::core::match (:wat::rete::compile (:strat::build-rules {STRATA})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))")
        };
        let seed_src = format!("(:strat::seed-items {compile} {ITEMS})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse strat-neg seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("strat-neg seed raised: {e:?}"))
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
            harvest: of(HARVEST) as f64,
            query_maps: query_map_count(&fired),
        }
    };

    let mut without = Shot {
        wall: 0.0,
        fire: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: 0.0,
        fire: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    for _ in 0..RUNS {
        let a = shot(false);
        let b = shot(true);
        without.wall += a.wall;
        without.fire += a.fire;
        without.harvest += a.harvest;
        without.query_maps = a.query_maps;
        with.wall += b.wall;
        with.fire += b.fire;
        with.harvest += b.harvest;
        with.query_maps = b.query_maps;
    }
    let r = RUNS as f64;
    without.wall /= r;
    without.fire /= r;
    without.harvest /= r;
    with.wall /= r;
    with.fire /= r;
    with.harvest /= r;

    println!(
        "\nstrat-neg query harvest split — [6 2000], mean of {RUNS}\n\
             \n\
             without queries       wall {:>7.2}  FIRE {:>7.2}  harvest {:>7.2}  maps {}\n\
             with    ten q-S*      wall {:>7.2}  FIRE {:>7.2}  harvest {:>7.2}  maps {}\n\
             delta (query tax)            {:>7.2} ms\n",
        ms(without.wall),
        ms(without.fire),
        ms(without.harvest),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        ms(with.harvest),
        with.query_maps,
        ms(with.wall - without.wall),
    );
    assert_eq!(
        without.query_maps, 0,
        "compile without queries has empty query-memory"
    );
    assert_eq!(
        with.query_maps, 6000,
        "6 strata × 1000 (even/odd) = 6000 query maps"
    );
}

/// Apportion the stratified membership set: REBUILD-per-stratum (today,
/// `fire/mod.rs` `merge_facts`) vs CARRIED-across-strata, at the strat-neg
/// `[6 2000]` ladder (`NEXT-STRIKES-theater-hunt.md` T1).
///
/// DISCONFIRMING PROBE — prints the parts only, no fire-path change. If the
/// delta is under the 0.5 ms gate, the strike STOPS and the rebuild is
/// recorded as physics.
#[test]
fn strat_merge_present_parts() {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const STRATA: usize = 6;
    const ITEMS: i64 = 2000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["k".to_string()]);
    let mk = |class: &str, k: i64| -> Value {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            names.clone(),
            Arc::new(vec![Value::i64(k)]),
        )))
    };

    // Mirrors wat-scripts/perf/grid/strat-neg.wat: Item(k) seeds; each stratum
    // derives S_i over half the items (`k mod 2` / `NOT S(i-1)` alternation).
    let seed: Vec<Value> = (0..ITEMS).map(|k| mk("sn::Item", k)).collect();
    let derived: Vec<Vec<Value>> = (0..STRATA)
        .map(|s| {
            let class = format!("sn::S{s}");
            (0..ITEMS)
                .filter(|k| k % 2 == (s as i64 % 2))
                .map(|k| mk(class.as_str(), k))
                .collect()
        })
        .collect();

    let total_derived: usize = derived.iter().map(|d| d.len()).sum();

    // Hash counts, exactly: REBUILD re-hashes the whole closure each stratum.
    let mut rebuild_hashes = 0usize;
    let mut acc = seed.len();
    for d in &derived {
        rebuild_hashes += acc;
        acc += d.len();
    }
    let carried_hashes = seed.len() + total_derived;

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: merge_facts rebuilds `present` from the whole closure per stratum.
        let t0 = Instant::now();
        let mut pv = crate::value::pvec::PVec::from_vec(seed.clone());
        for ds in &derived {
            let mut present: HashSet<Value> = pv.iter().cloned().collect();
            for f in ds {
                if present.insert(f.clone()) {
                    pv.push_back_mut(f.clone());
                }
            }
        }
        a += t0.elapsed().as_nanos() as f64;
        len_a = pv.len();
        black_box(&pv);

        // B — CARRIED: the set is built once and threaded across strata.
        let t0 = Instant::now();
        let mut pv = crate::value::pvec::PVec::from_vec(seed.clone());
        let mut present: HashSet<Value> = pv.iter().cloned().collect();
        for ds in &derived {
            for f in ds {
                if present.insert(f.clone()) {
                    pv.push_back_mut(f.clone());
                }
            }
        }
        b += t0.elapsed().as_nanos() as f64;
        len_b = pv.len();
        black_box(&pv);
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);

    let table = format!(
        "\nstrat merge present — [{STRATA} {ITEMS}], mean of {RUNS}\n\
         \n\
         seed {} facts, derived {} across {STRATA} strata, closure {}\n\
         hashes  REBUILD {rebuild_hashes}   CARRIED {carried_hashes}   wasted {}\n\
         \n\
         A  REBUILD per stratum (today)   {:>7.2} ms\n\
         B  CARRIED across strata         {:>7.2} ms\n\
         A-B  the theater                 {:>7.2} ms\n",
        seed.len(),
        total_derived,
        len_a,
        rebuild_hashes - carried_hashes,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, len_b, "both paths must build the same closure:{table}");
    assert_eq!(
        len_a,
        seed.len() + total_derived,
        "closure must be seed + every derived fact:{table}"
    );
    assert!(a > 0.0 && b > 0.0, "probe recorded no time:{table}");
}

/// Apportion the stratified `acc_derived` union: TWO CLONES per derived fact
/// (today, `fire/rules.rs:206`) vs ONE — the vec push can MOVE, because
/// `new_derived` is dead after the loop (`NEXT-STRIKES-theater-hunt.md` T5).
///
/// DISCONFIRMING PROBE — prints the parts only. A wash STOPS the strike and
/// the second clone is recorded as physics.
#[test]
fn strat_acc_derived_clone_parts() {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    const STRATA: usize = 6;
    const PER_STRATUM: i64 = 1000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["k".to_string()]);
    let mk = |class: &str, k: i64| -> Value {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            names.clone(),
            Arc::new(vec![Value::i64(k)]),
        )))
    };
    let strata: Vec<Vec<Value>> = (0..STRATA)
        .map(|s| {
            let class = format!("sn::S{s}");
            (0..PER_STRATUM).map(|k| mk(class.as_str(), k)).collect()
        })
        .collect();

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: clone for the set, clone again for the vec.
        let t0 = Instant::now();
        let mut set: HashSet<Value> = HashSet::new();
        let mut out: Vec<Value> = Vec::new();
        for ds in &strata {
            let new_derived: Vec<Value> = ds.clone();
            for d in &new_derived {
                if set.insert(d.clone()) {
                    out.push(d.clone());
                }
            }
        }
        a += t0.elapsed().as_nanos() as f64;
        len_a = out.len();
        black_box((&set, &out));

        // B — CUT: consume new_derived, so the push is a move.
        let t0 = Instant::now();
        let mut set: HashSet<Value> = HashSet::new();
        let mut out: Vec<Value> = Vec::new();
        for ds in &strata {
            let new_derived: Vec<Value> = ds.clone();
            for d in new_derived {
                if set.insert(d.clone()) {
                    out.push(d);
                }
            }
        }
        b += t0.elapsed().as_nanos() as f64;
        len_b = out.len();
        black_box((&set, &out));
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);
    let total = STRATA * PER_STRATUM as usize;

    let table = format!(
        "\nstrat acc_derived clones — {STRATA} strata x {PER_STRATUM}, mean of {RUNS}\n\
         \n\
         {total} derived facts; A does {} clones, B does {}\n\
         \n\
         A  TWO CLONES (today)            {:>7.3} ms\n\
         B  ONE CLONE + move              {:>7.3} ms\n\
         A-B  the theater                 {:>7.3} ms\n",
        total * 2,
        total,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, len_b, "both paths must union the same set:{table}");
    assert_eq!(len_a, total, "all facts distinct in this fixture:{table}");
}

/// Apportion strat-neg `[6 2000]` FIRE across the per-stratum phases the
/// stratified loop was never instrumented for: slice+arm / sub-session /
/// collect / merge / acc-union, against the fire itself
/// (`NEXT-STRIKES-theater-hunt.md` — the loop was UNMEASURED).
///
/// No query: this isolates `fire_rules_stratified` from harvest.
#[test]
fn strat_neg_stratum_split() {
    use std::time::Instant;

    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const RUNS: usize = 3;
    const WORLD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/wat-scripts/perf/grid/strat-neg.wat"));
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const STRAT_PHASES: [&str; 5] = [
        "  ├ strat:slice",
        "  ├ strat:session",
        "  ├ strat:collect",
        "  ├ strat:merge",
        "  └ strat:acc",
    ];

    let cal = calibrate_mark_ns();

    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("strat-neg stratum-split world should freeze");

    let seed_src = format!(
        "(:strat::seed-items (:wat::core::match (:wat::rete::compile (:strat::build-rules {STRATA})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {ITEMS})"
    );

    let mut wall = 0.0f64;
    let mut fire = 0.0f64;
    let mut net = [0.0f64; 5];
    let mut pairs = [0u64; 5];

    for _ in 0..RUNS {
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse strat-neg seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("strat-neg seed raised: {e:?}"))
        .value_owned();

        let t0 = Instant::now();
        let (_fired, rows) = super::with_phase_census_counted(|| {
            fire_rules_on_session(&staged, world.symbols(), None)
                .unwrap_or_else(|e| panic!("fire-rules raised: {e:?}"))
        });
        wall += t0.elapsed().as_nanos() as f64;

        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        fire += FIRE_PHASES.iter().map(|n| of(n).0).sum::<u64>() as f64;
        for (i, name) in STRAT_PHASES.iter().enumerate() {
            let (ns, k) = of(name);
            net[i] += ns as f64 - k as f64 * cal;
            pairs[i] = k;
        }
    }

    let r = RUNS as f64;
    let strat_sum: f64 = net.iter().sum::<f64>() / r;

    let mut body = String::new();
    for (i, name) in STRAT_PHASES.iter().enumerate() {
        body.push_str(&format!(
            "{name:<22} {:>8.3} ms   {} pairs\n",
            ms(net[i] / r),
            pairs[i]
        ));
    }

    let table = format!(
        "\nstrat-neg stratum split — [{STRATA} {ITEMS}], no query, mean of {RUNS}\n\
         instrument: {cal:.1} ns per mark pair\n\
         \n\
         wall                   {:>8.3} ms\n\
         FIRE (4 phases)        {:>8.3} ms\n\
         \n{body}\
         strat sum              {:>8.3} ms\n",
        ms(wall / r),
        ms(fire / r),
        ms(strat_sum),
    );
    println!("{table}");

    assert!(wall > 0.0, "harness recorded no time:{table}");
    assert!(
        pairs[0] as i64 >= STRATA,
        "slice must fire once per stratum:{table}"
    );
}

/// Is the closure PVec SHARED when `merge_facts` appends to it? `1` owner means
/// `push_back_mut` grows in place; `>1` means `Arc::make_mut` deep-copies the
/// whole `Vec` on the first push of every stratum.
#[test]
fn strat_merge_pv_owner_count() {
    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const WORLD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/wat-scripts/perf/grid/strat-neg.wat"));

    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("world should freeze");
    let seed_src = format!(
        "(:strat::seed-items (:wat::core::match (:wat::rete::compile (:strat::build-rules {STRATA})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {ITEMS})"
    );
    let staged = eval_in_frozen(
        &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
        &world,
        &Environment::new(),
    )
    .unwrap_or_else(|e| panic!("seed raised: {e:?}"))
    .value_owned();

    // Owner counts land in the COUNT census, not PHASE_NANOS. `merge:pv-owners`
    // sums the owner count; `merge:pv-calls` counts the calls, so the mean is
    // the two divided — one counter cannot carry both.
    let (_fired, counts) = super::with_count_census(|| {
        fire_rules_on_session(&staged, world.symbols(), None)
            .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
    });
    let get = |name: &str| -> u64 {
        counts
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    };
    let (total, calls) = (get("merge:pv-owners"), get("merge:pv-calls"));

    let mean = if calls > 0 { total as f64 / calls as f64 } else { 0.0 };
    let arm = if total == 0 { "Tree (rpds VectorSync)" } else { "Array" };
    let out = format!(
        "\nmerge_facts receiver — strat-neg [{STRATA} {ITEMS}]\n\
         calls {calls}   summed owners {total}   mean {mean:.2}   arm {arm}\n\
         \n\
         `array_owners` reports 0 for the Tree arm, so a summed 0 means the\n\
         closure PVec is rpds `VectorSync`: `push_back_mut` path-copies\n\
         O(log n) and there is NO `Arc::make_mut` whole-Vec deep copy here.\n\
         This is the measurement that FALSIFIED the copy-on-write hypothesis\n\
         in `strat_merge_cow_parts`, which built its fixture with\n\
         `PVec::from_vec` (the Array arm) and found 2.019 ms of theater the\n\
         fire does not actually pay.\n\
         \n\
         LATENT CLIFF: that 8x IS real for the Array arm. If the closure ever\n\
         reaches `merge_facts` as Array while the caller's copy is alive, the\n\
         first push of every stratum deep-copies the whole Vec. This test is\n\
         the tripwire: if `arm` ever reads Array, go read\n\
         `strat_merge_cow_parts` before anything else.\n"
    );
    println!("{out}");
    assert!(calls > 0, "merge_facts never ran:{out}");
    assert_eq!(
        calls as i64, STRATA,
        "merge_facts must run once per stratum:{out}"
    );
}

/// Apportion the closure append: SHARED receiver (today — `merge_facts` clones
/// the PVec while the caller's copy is still alive, so the first
/// `push_back_mut` hits `Arc::make_mut` and deep-copies the whole `Vec`) vs
/// UNIQUE receiver, at the strat-neg `[6 2000]` ladder.
///
/// DISCONFIRMING PROBE — prints the parts only. A wash STOPS the strike.
#[test]
fn strat_merge_cow_parts() {
    use std::hint::black_box;
    use std::time::Instant;

    const STRATA: usize = 6;
    const SEED: i64 = 2000;
    const PER_STRATUM: usize = 1000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["k".to_string()]);
    let mk = |class: &str, k: i64| -> Value {
        Value::Aggregate(Arc::new(AggregateValue::record(
            class.into(),
            names.clone(),
            Arc::new(vec![Value::i64(k)]),
        )))
    };
    let seed: Vec<Value> = (0..SEED).map(|k| mk("sn::Item", k)).collect();
    let strata: Vec<Vec<Value>> = (0..STRATA)
        .map(|s| {
            let class = format!("sn::S{s}");
            (0..PER_STRATUM as i64).map(|k| mk(class.as_str(), k)).collect()
        })
        .collect();

    // What the shared path actually copies: the closure length at each stratum.
    let mut copied = 0usize;
    let mut n = seed.len();
    for _ in 0..STRATA {
        copied += n;
        n += PER_STRATUM;
    }

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: the caller's copy stays alive across the append, so the
        // receiver is shared and Arc::make_mut deep-copies the whole Vec once
        // per stratum.
        let t0 = Instant::now();
        let mut acc = crate::value::pvec::PVec::from_vec(seed.clone());
        for ds in &strata {
            let mut pv = acc.clone(); // <- the clone merge_facts makes today
            for f in ds {
                pv.push_back_mut(f.clone());
            }
            acc = pv; // caller's copy replaced only AFTER the appends
        }
        a += t0.elapsed().as_nanos() as f64;
        len_a = acc.len();
        black_box(&acc);

        // B — CUT: the receiver is moved in, so it is unique and grows in place.
        let t0 = Instant::now();
        let mut acc = crate::value::pvec::PVec::from_vec(seed.clone());
        for ds in &strata {
            let mut pv = acc; // moved, not cloned
            for f in ds {
                pv.push_back_mut(f.clone());
            }
            acc = pv;
        }
        b += t0.elapsed().as_nanos() as f64;
        len_b = acc.len();
        black_box(&acc);
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);

    let table = format!(
        "\nstrat merge copy-on-write — [{STRATA} {SEED}], mean of {RUNS}\n\
         \n\
         closure {} -> {}; the SHARED path deep-copies {copied} Values total\n\
         \n\
         A  SHARED receiver (today)       {:>8.3} ms\n\
         B  UNIQUE receiver               {:>8.3} ms\n\
         A-B  the theater                 {:>8.3} ms\n",
        seed.len(),
        len_a,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, len_b, "both paths must build the same closure:{table}");
    assert_eq!(len_a, SEED as usize + STRATA * PER_STRATUM, "closure size:{table}");
}

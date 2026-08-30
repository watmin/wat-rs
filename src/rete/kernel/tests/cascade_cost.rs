//! Cascade cost — a derived fact re-entering the network, and depth's price at equal work.


use super::*;

/// Fire a depth×width cascade through the native path; return the per-phase nanosecond rows.
fn depth_split_phases(depth: i64, width: i64) -> Vec<(&'static str, u64)> {
    cascade_phase_census(depth, width)
        .into_iter()
        .map(|(n, ns, _)| (n, ns))
        .collect()
}

/// Diagnostic — where the depth cost lands, at CONSTANT work (10,000 derived facts).
///
/// Shallow-and-wide vs deep-and-narrow derive exactly the same number of facts, so any
/// difference between the two columns is depth, and the per-phase breakdown says which
/// phase is paying for it.
#[test]
fn a0_depth_cost_split_at_equal_work() {
    // 2*depth*width derived facts: both columns derive 10,000.
    let shallow = depth_split_phases(10, 500); // 10 rounds  · 500 ids per level
    let deep = depth_split_phases(50, 100); // 50 rounds · 100 ids per level  (the :clara cell)

    let names: std::collections::BTreeSet<&'static str> =
        shallow.iter().chain(deep.iter()).map(|(n, _)| *n).collect();

    let sum = |rows: &[(&'static str, u64)]| -> u64 {
        rows.iter()
            .filter(|(n, _)| n.starts_with("  "))
            .map(|(_, ns)| *ns)
            .sum()
    };
    let (s_tot, d_tot) = (sum(&shallow), sum(&deep));

    let get = |rows: &[(&'static str, u64)], name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, ns)| *ns)
            .unwrap_or(0)
    };

    let mut table = String::from(
        "\n  A0 DEPTH-COST SPLIT — 10,000 derived facts in BOTH columns\n\
             \n  phase                          depth10×w500      depth50×w100         delta\n\
             \x20 ---------------------------------------------------------------------------\n",
    );
    for n in &names {
        let (a, b) = (get(&shallow, n), get(&deep, n));
        table.push_str(&format!(
            "  {n:<28} {:>10.3} ms {:>13.3} ms {:>+11.3} ms\n",
            a as f64 / 1e6,
            b as f64 / 1e6,
            (b as f64 - a as f64) / 1e6
        ));
    }
    table.push_str(&format!(
        "  {:<28} {:>10.3} ms {:>13.3} ms {:>+11.3} ms   ({:.2}x)\n",
        "TOTAL (nested phases)",
        s_tot as f64 / 1e6,
        d_tot as f64 / 1e6,
        (d_tot as f64 - s_tot as f64) / 1e6,
        if s_tot > 0 {
            d_tot as f64 / s_tot as f64
        } else {
            0.0
        }
    ));

    println!("{table}");
    assert!(
        s_tot > 0 && d_tot > 0,
        "the phase census recorded nothing — the probe measured its own scaffolding, not the \
             fire. A zero here means `with_phase_census` never saw a round.{table}"
    );
}

/// Kind lists interned on the arm (`DESIGN-STONE-arm-kind-lists`).
/// Prints sizes at cascade depth 50. Lists are disjoint subsequences
/// of `node_ids`. Does not wall-gate FIRE.
#[test]
fn cascade_kind_list_split() {
    let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("depth-split world should freeze");
    let src = "(:wat::core::match (:wat::rete::compile (:dc::build-rules 50)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))";
    let ast = crate::parse_one!(src).expect("parse compile");
    let session = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile raised: {e:?}"))
        .value_owned();
    let wm = super::to_transient(&session).expect("to_transient of compiled session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for cascade network");
    let k = &arm.kind_ids;
    let n = arm.node_ids.len();
    let table = format!(
        "\ncascade kind lists — depth 50 compile\n\
             node_ids          {:>6}\n\
             alpha             {:>6}\n\
             join_parent       {:>6}\n\
             acc               {:>6}\n\
             filter            {:>6}\n\
             prod              {:>6}\n\
             filter_or_acc     {:>6}\n",
        n,
        k.alpha.len(),
        k.join_parent.len(),
        k.acc.len(),
        k.filter.len(),
        k.prod.len(),
        k.filter_or_acc.len(),
    );
    println!("{table}");

    let in_nodes = |ids: &[i64]| ids.iter().all(|id| arm.node_ids.contains(id));
    assert!(
        in_nodes(&k.alpha)
            && in_nodes(&k.join_parent)
            && in_nodes(&k.acc)
            && in_nodes(&k.filter)
            && in_nodes(&k.prod),
        "a kind list id is not in node_ids:{table}"
    );

    let mut seen = std::collections::HashSet::new();
    for id in k
        .alpha
        .iter()
        .chain(&k.join_parent)
        .chain(&k.acc)
        .chain(&k.filter)
        .chain(&k.prod)
    {
        assert!(seen.insert(*id), "kind lists overlap on {id}:{table}");
    }

    let sorted = |v: &[i64]| v.windows(2).all(|w| w[0] < w[1]);
    assert!(
        sorted(&k.alpha)
            && sorted(&k.join_parent)
            && sorted(&k.acc)
            && sorted(&k.filter)
            && sorted(&k.prod)
            && sorted(&k.filter_or_acc),
        "a kind list is not strictly increasing:{table}"
    );
    assert_eq!(
        k.filter_or_acc.len(),
        k.filter.len() + k.acc.len(),
        "filter_or_acc is not the merge of filter+acc:{table}"
    );
    assert!(
        n > 0 && k.alpha.len() + k.join_parent.len() + k.prod.len() > 0,
        "compile produced an empty network:{table}"
    );
}

/// Rank deep-cascade `[50 100]` FIRE with vs without q-Node / q-Tag
/// (`DESIGN-STONE-cascade-harvest-split`). Census compiles without queries.
#[test]
fn cascade_query_harvest_split() {
    use std::time::Instant;

    const DEPTH: i64 = 50;
    const WIDTH: i64 = 100;
    const RUNS: usize = 3;
    const HARVEST: &str = "  ├ harvest:query";
    const SETUP: &str = "SETUP: indexes";
    const ROUND: &str = "ROUND LOOP";
    const FIRE_PHASES: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    const WORLD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/wat-scripts/perf/grid/deep-cascade.wat"));
    const QUERIES: &str =
        "(:wat::core::PersistentVector (:cascade::q-Node) (:cascade::q-Tag))";

    struct Shot {
        wall: f64,
        fire: f64,
        setup: f64,
        round: f64,
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
        .expect("cascade query-harvest world should freeze");

    let shot = |with_query: bool| -> Shot {
        let compile = if with_query {
            format!("(:wat::core::match (:wat::rete::compile-all (:dc::build-rules {DEPTH}) {QUERIES}) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))")
        } else {
            format!("(:wat::core::match (:wat::rete::compile (:dc::build-rules {DEPTH})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))")
        };
        let seed_src = format!("(:dc::seed-level-0 {compile} {WIDTH})");
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse cascade seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("cascade seed raised: {e:?}"))
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
            setup: of(SETUP) as f64,
            round: of(ROUND) as f64,
            harvest: of(HARVEST) as f64,
            query_maps: query_map_count(&fired),
        }
    };

    let mut without = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
        harvest: 0.0,
        query_maps: 0,
    };
    let mut with = Shot {
        wall: 0.0,
        fire: 0.0,
        setup: 0.0,
        round: 0.0,
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
        without.harvest += a.harvest;
        without.query_maps = a.query_maps;
        with.wall += b.wall;
        with.fire += b.fire;
        with.setup += b.setup;
        with.round += b.round;
        with.harvest += b.harvest;
        with.query_maps = b.query_maps;
    }
    let r = RUNS as f64;
    without.wall /= r;
    without.fire /= r;
    without.setup /= r;
    without.round /= r;
    without.harvest /= r;
    with.wall /= r;
    with.fire /= r;
    with.setup /= r;
    with.round /= r;
    with.harvest /= r;

    let ms = |ns: f64| ns / 1e6;
    println!(
        "\ncascade query harvest split — [50 100], mean of {RUNS}\n\
             \n\
             without queries       wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  harvest {:>7.2}  maps {}\n\
             with    q-Node q-Tag  wall {:>7.2}  FIRE {:>7.2}  SETUP {:>7.2}  ROUND {:>7.2}  harvest {:>7.2}  maps {}\n\
             delta (query tax)            {:>7.2} ms\n",
        ms(without.wall),
        ms(without.fire),
        ms(without.setup),
        ms(without.round),
        ms(without.harvest),
        without.query_maps,
        ms(with.wall),
        ms(with.fire),
        ms(with.setup),
        ms(with.round),
        ms(with.harvest),
        with.query_maps,
        ms(with.wall - without.wall),
    );
    assert_eq!(
        without.query_maps, 0,
        "compile without queries has empty query-memory"
    );
    assert_eq!(
        with.query_maps, 10_200,
        "5100 Node + 5100 Tag (level 0 input ∪ 50 derived levels × 100) = 10200 maps"
    );
}

/// Leftover cascade SETUP: arm vs remainder (`DESIGN-STONE-cascade-setup-split`).
#[test]
fn cascade_setup_leftover_split() {
    const RUNS: usize = 3;

    let cal = calibrate_mark_ns();

    let mut setup_raw = 0.0;
    let mut seen_raw = 0.0;
    let mut arm_raw = 0.0;
    let mut setup_pairs = 0u64;
    let mut seen_pairs = 0u64;
    let mut arm_pairs = 0u64;
    let mut builds = 0usize;
    for _ in 0..RUNS {
        let before = super::ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
        let rows = cascade_phase_census(50, 100);
        let after = super::ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
        builds += after.saturating_sub(before);
        let of = |name: &str| -> (u64, u64) {
            rows.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, ns, k)| (*ns, *k))
                .unwrap_or((0, 0))
        };
        let (s_ns, s_k) = of("SETUP: indexes");
        let (seen_ns, seen_k) = of("  ├ setup:seen");
        let (arm_ns, arm_k) = of("  ├ setup:arm");
        setup_raw += s_ns as f64;
        seen_raw += seen_ns as f64;
        arm_raw += arm_ns as f64;
        setup_pairs = s_k;
        seen_pairs = seen_k;
        arm_pairs = arm_k;
    }
    let r = RUNS as f64;
    setup_raw /= r;
    seen_raw /= r;
    arm_raw /= r;
    let remainder_raw = setup_raw - seen_raw - arm_raw;
    let ms = |ns: f64| ns / 1e6;
    let table = format!(
        "\ncascade SETUP leftover split — [50 100], mean of {RUNS}\n\
             instrument: {cal:.1} ns per mark pair\n\
             \n\
             SETUP                     {:>7.2} ms raw  {:>6}x\n\
               setup:seen              {:>7.2} raw  {:>7.2} net  {:>6}x\n\
               setup:arm               {:>7.2} raw  {:>7.2} net  {:>6}x\n\
             remainder (SETUP−seen−arm){:>7.2} ms\n\
             ARM_BUILDS                {:>7}  ({:.2} per run)\n",
        ms(setup_raw),
        setup_pairs,
        ms(seen_raw),
        ms(seen_raw - seen_pairs as f64 * cal),
        seen_pairs,
        ms(arm_raw),
        ms(arm_raw - arm_pairs as f64 * cal),
        arm_pairs,
        ms(remainder_raw),
        builds,
        builds as f64 / r,
    );
    println!("{table}");
    assert!(
        setup_raw > 0.0,
        "SETUP recorded 0 — the fire never ran:{table}"
    );
    assert!(
        arm_pairs > 0,
        "setup:arm recorded 0 pairs — the mark never fired:{table}"
    );
}

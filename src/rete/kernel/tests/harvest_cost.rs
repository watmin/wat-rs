//! Harvest cost — reading answers OUT: production memory, query memory, and the closed bag.


use super::*;

/// Apportion `out:production` (3.26 ms / 40k) without a Session rewrite
/// (`DESIGN-STONE-out-production-split`).
#[test]
fn out_production_cost_split() {
    use std::hint::black_box;

    const N: usize = 40_000;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "fan::Pair".into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();


    black_box(facts.clone());
    {
        let mut pv = rpds::VectorSync::new_sync();
        for v in facts.clone() {
            pv.push_back_mut(v);
        }
        black_box(pv);
        let mut map: ProductionMemory = HashMap::new();
        map.insert(1, facts.clone());
        black_box(super::production_to_pm(map));
        let collected: rpds::VectorSync<Value> = facts.clone().into_iter().collect();
        black_box(collected);
    }

    let mut c = f64::INFINITY;
    let mut v = f64::INFINITY;
    let mut h = f64::INFINITY;
    let mut i = f64::INFINITY;
    for _ in 0..RUNS {
        c = c.min(ns_per_iter(1, || {
            black_box(facts.clone());
        }));
        v = v.min(ns_per_iter(1, || {
            let mut pv = rpds::VectorSync::new_sync();
            for val in facts.clone() {
                pv.push_back_mut(val);
            }
            black_box(pv);
        }));
        h = h.min(ns_per_iter(1, || {
            let mut map: ProductionMemory = HashMap::new();
            map.insert(1, facts.clone());
            black_box(super::production_to_pm(map));
        }));
        i = i.min(ns_per_iter(1, || {
            let collected: rpds::VectorSync<Value> = facts.clone().into_iter().collect();
            black_box(collected);
        }));
    }
    assert!(
        h > 0.0,
        "production_to_pm recorded 0 ns — the loop never ran"
    );

    // ⛔ THE GUARD ABOVE IS NOT THE CLAIM — `probare` classed this test hollow. The function under
    // measurement, `production_to_pm`, is a real conversion with a real contract: node id → its
    // production vector. Nothing checked that it PRESERVES anything, so a conversion that dropped
    // facts would time faster and read as an improvement.
    //
    // Deliberately NOT asserted on `facts.len()` — `facts` is `(0..N)` by construction, so its
    // length is a tautology. This asserts what the ENGINE produced from it.
    let mut probe: ProductionMemory = HashMap::new();
    probe.insert(1, facts.clone());
    let pm = super::production_to_pm(probe);
    match &pm {
        Value::wat__core__PersistentMap(m) => {
            assert_eq!(m.len(), 1, "one production node in, one key out; got {}", m.len());
            match m.get(&Value::i64(1)) {
                Some(Value::wat__core__PersistentVector(pv)) => assert_eq!(
                    pv.len(),
                    N,
                    "production_to_pm returned {} facts for node 1, not the {N} it was given — \
                     the conversion is dropping facts, and a lossy conversion times FASTER, so \
                     this test's millisecond figures would read as an improvement",
                    pv.len()
                ),
                other => panic!("node 1 should map to a PersistentVector; got {other:?}"),
            }
        }
        other => panic!("production_to_pm must return a PersistentMap; got {other:?}"),
    }

    // ⛔ `H−V` IS NOT A DECOMPOSITION AND ITS OLD LABEL ("wrap") CLAIMED IT WAS. Measured stable
    // at −2.8 to −3.4 ms across runs (2026-08-30) — not noise, and not the cold-start artefact
    // the minimum estimator cured. The arms are ALTERNATIVE ALGORITHMS, not superset and subset:
    // `V` pushes 40,000 times into an RRB vector; `H` calls `production_to_pm`, which takes the
    // BULK path — `PVec::from_vec` (`session.rs`, "Bulk Array arm — not N RRB push_back",
    // `DESIGN-STONE-promoting-vector`). So H does not contain V, and subtracting them yields a
    // COMPARISON, not a component cost. The number was always right; the label was wrong.
    println!(
        "\nout:production split — {N} Pair records, MINIMUM of {RUNS}\n\
             unscaled (the cell is 40k); C is the Arc-bump clone fire does not pay\n\
             \n\
             C  clone 40k Vec                      {:>7.2} ms\n\
             V  clone + push_back_mut              {:>7.2} ms\n\
             H  clone + production_to_pm (authority)  {:>7.2} ms\n\
             I  clone + VectorSync::from_iter      {:>7.2} ms\n\
             \n\
             V−C  node-per-fact                    {:>7.2} ms\n\
             H−V  bulk from_vec MINUS N push_back  {:>7.2} ms  (negative = bulk wins)\n\
             V−I  from_iter drop-in                {:>7.2} ms\n",
        ms(c),
        ms(v),
        ms(h),
        ms(i),
        ms(v - c),
        ms(h - v),
        ms(v - i),
    );
}

/// Apportion `out:query` (3.08 ms / 40k) without a Session rewrite
/// (`DESIGN-STONE-out-query-split`). Same 40k VectorSync as 2u.
#[test]
fn out_query_cost_split() {
    use std::collections::HashMap;
    use std::hint::black_box;

    const N: usize = 40_000;
    const RUNS: usize = 3;

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let maps: Vec<crate::value::pmap::PMap> = (0..N)
        .map(|i| {
            let fact = Value::Aggregate(Arc::new(AggregateValue::record(
                "fan::Pair".into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )));
            crate::value::pmap::PMap::from_pairs([(var.clone(), fact)])
        })
        .collect();


    black_box(maps.clone());
    {
        let mut pv = rpds::VectorSync::new_sync();
        for m in maps.clone() {
            pv.push_back_mut(Value::wat__core__PersistentMap(m));
        }
        black_box(pv);
        let mut q: QueryMemory = HashMap::new();
        q.insert("q-Pair".to_string(), maps.clone());
        black_box(super::query_memory_to_pm(q));
        let collected: rpds::VectorSync<Value> = maps
            .clone()
            .into_iter()
            .map(Value::wat__core__PersistentMap)
            .collect();
        black_box(collected);
    }

    let mut c = f64::INFINITY;
    let mut v = f64::INFINITY;
    let mut h = f64::INFINITY;
    let mut i = f64::INFINITY;
    for _ in 0..RUNS {
        c = c.min(ns_per_iter(1, || {
            black_box(maps.clone());
        }));
        v = v.min(ns_per_iter(1, || {
            let mut pv = rpds::VectorSync::new_sync();
            for m in maps.clone() {
                pv.push_back_mut(Value::wat__core__PersistentMap(m));
            }
            black_box(pv);
        }));
        h = h.min(ns_per_iter(1, || {
            let mut q: QueryMemory = HashMap::new();
            q.insert("q-Pair".to_string(), maps.clone());
            black_box(super::query_memory_to_pm(q));
        }));
        i = i.min(ns_per_iter(1, || {
            let collected: rpds::VectorSync<Value> = maps
                .clone()
                .into_iter()
                .map(Value::wat__core__PersistentMap)
                .collect();
            black_box(collected);
        }));
    }
    assert!(
        h > 0.0,
        "query_memory_to_pm recorded 0 ns — the loop never ran"
    );

    // ⛔ THE GUARD ABOVE IS NOT THE CLAIM — `probare` classed this test hollow. It times
    // `query_memory_to_pm`, a real conversion with a real contract, and nothing checked it
    // PRESERVES anything. Same reasoning as `out_production_cost_split`: a conversion that drops
    // maps converts less, times FASTER, and prints as an improvement.
    //
    // Deliberately not asserted on `maps.len()` — `maps` is `(0..N)` by construction, so its
    // length is a tautology. This asserts what the ENGINE produced from it.
    let mut probe: QueryMemory = HashMap::new();
    probe.insert("q-Pair".to_string(), maps.clone());
    match &super::query_memory_to_pm(probe) {
        Value::wat__core__PersistentMap(m) => {
            assert_eq!(m.len(), 1, "one query name in, one key out; got {}", m.len());
            match m.get(&Value::String(Arc::new("q-Pair".to_string()))) {
                Some(Value::wat__core__PersistentVector(pv)) => assert_eq!(
                    pv.len(),
                    N,
                    "query_memory_to_pm returned {} binding maps for `q-Pair`, not the {N} it was \
                     given — a lossy conversion times faster, so these figures would read as a win",
                    pv.len()
                ),
                other => panic!("`q-Pair` should map to a PersistentVector; got {other:?}"),
            }
        }
        other => panic!("query_memory_to_pm must return a PersistentMap; got {other:?}"),
    }

    println!(
        "\nout:query split — {N} one-entry PMaps, MINIMUM of {RUNS}\n\
             unscaled (the cell is 40k); C is the Arc-bump clone fire does not pay\n\
             \n\
             C  clone 40k Vec<PMap>                {:>7.2} ms\n\
             V  clone + wrap + push_back_mut       {:>7.2} ms\n\
             H  clone + query_memory_to_pm         {:>7.2} ms\n\
             I  clone + VectorSync::from_iter      {:>7.2} ms\n\
             \n\
             V−C  node-per-fact                    {:>7.2} ms\n\
             H−V  bulk from_vec MINUS N push_back  {:>7.2} ms  (negative = bulk wins)\n\
             V−I  from_iter drop-in                {:>7.2} ms\n",
        ms(c),
        ms(v),
        ms(h),
        ms(i),
        ms(v - c),
        ms(h - v),
        ms(v - i),
    );
}

/// Apportion harvest:query (7.69 ms / 40k) into scan vs wrap
/// (`DESIGN-STONE-harvest-wrap-split`). No fire-path change.
#[test]
fn harvest_wrap_split() {
    use std::hint::black_box;

    const N: usize = 40_000;
    const RUNS: usize = 3;
    const CLASS: &str = "fan::Pair";

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                CLASS.into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();
    let pv = crate::value::pvec::PVec::from_vec(facts);

    let matches_class = |f: &Value| match f {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.class.as_ref() == CLASS,
        _ => false,
    };


    // Warm the same shapes the timed loops will run.
    {
        let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
        black_box(&collected);
        let maps: Vec<crate::value::pmap::PMap> = collected
            .iter()
            .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
            .collect();
        black_box(maps);
    }

    let mut s = f64::INFINITY;
    let mut w = f64::INFINITY;
    let mut h = f64::INFINITY;
    for _ in 0..RUNS {
        s = s.min(ns_per_iter(1, || {
            let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
            black_box(collected);
        }));
        let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
        w = w.min(ns_per_iter(1, || {
            let maps: Vec<crate::value::pmap::PMap> = collected
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        }));
        h = h.min(ns_per_iter(1, || {
            let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
            let maps: Vec<crate::value::pmap::PMap> = collected
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        }));
    }
    // ⛔ WAS ONLY `assert!(h > 0.0)` — liveness. The split this test exists to APPORTION is
    // scan (s) vs wrap (w) vs both (h), so the assertable claim is the apportionment itself.
    assert!(h > 0.0, "harvest wrap recorded 0 ns — the loop never ran");

    // NON-VACUITY: the timed closures must have walked the whole 40k bag. If the class filter
    // stopped matching, every timing above would fall toward zero and still pass a liveness
    // check — the reading would look like a speedup and be a broken filter.
    let collected: Vec<&Value> = pv.iter().filter(|f| matches_class(f)).collect();
    assert_eq!(
        collected.len(),
        N,
        "the class filter matched {} of {N} `{CLASS}` facts — every timing above is measuring a \
         different, smaller workload than the one this split is named for",
        collected.len()
    );

    // APPORTIONMENT: scan and wrap are the two halves of the combined pass, so they must
    // roughly account for it. Bounds are deliberately loose (0.5x–2x) because these are wall
    // clocks on a shared runner; what they catch is a phase silently dropping out of the
    // combined measurement, which is what would make the "split" stop being a split.
    assert!(
        h >= (s + w) * 0.5 && h <= (s + w) * 2.0,
        "combined harvest ({:.2} ms) is not accounted for by scan ({:.2} ms) + wrap ({:.2} ms) \
         — the apportionment this test reports no longer adds up, so one of the three closures \
         is measuring something other than what its name says",
        ms(h), ms(s), ms(w)
    );

    println!(
        "\nharvest wrap split — {N} one-entry maps, MINIMUM of {RUNS}\n\
             unscaled (the cell is 40k)\n\
             \n\
             S  scan (filter PVec by class)        {:>7.2} ms\n\
             W  wrap (from_pairs × 40k)            {:>7.2} ms\n\
             H  harvest (scan then wrap)           {:>7.2} ms\n\
             S+W                                   {:>7.2} ms\n",
        ms(s),
        ms(w),
        ms(h),
        ms(s + w),
    );
}

/// Apportion wrap (8.78 ms / 40k) into clones / Arc / intern-id
/// (`DESIGN-STONE-harvest-wrap-parts`). No fire-path change.
#[test]
fn harvest_wrap_parts() {
    use std::hint::black_box;
    use std::sync::atomic::{AtomicU64, Ordering};

    const N: usize = 40_000;
    const RUNS: usize = 3;
    const CLASS: &str = "fan::Pair";

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                CLASS.into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();
    let pv = crate::value::pvec::PVec::from_vec(facts);
    let collected: Vec<&Value> = pv
        .iter()
        .filter(|f| match f {
            Value::Aggregate(a) if a.nature != Nature::Struct => a.class.as_ref() == CLASS,
            _ => false,
        })
        .collect();
    assert_eq!(collected.len(), N, "setup: 40k Pair facts");
    let pairs: Vec<(Value, Value)> = collected
        .iter()
        .map(|f| (var.clone(), (*f).clone()))
        .collect();


    {
        black_box(pairs.clone());
        let intern = AtomicU64::new(1);
        for p in &pairs {
            let a: Arc<[(Value, Value)]> = Arc::from([p.clone()]);
            black_box(a);
            intern.fetch_add(1, Ordering::Relaxed);
        }
        let maps: Vec<crate::value::pmap::PMap> = collected
            .iter()
            .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
            .collect();
        black_box(maps);
    }

    let mut c = f64::INFINITY;
    let mut r = f64::INFINITY;
    let mut i = f64::INFINITY;
    let mut w = f64::INFINITY;
    for _ in 0..RUNS {
        c = c.min(ns_per_iter(1, || {
            let cloned: Vec<(Value, Value)> = collected
                .iter()
                .map(|f| (var.clone(), (*f).clone()))
                .collect();
            black_box(cloned);
        }));
        r = r.min(ns_per_iter(1, || {
            for p in &pairs {
                let a: Arc<[(Value, Value)]> = Arc::from([p.clone()]);
                black_box(a);
            }
        }));
        i = i.min(ns_per_iter(1, || {
            let intern = AtomicU64::new(1);
            for _ in 0..N {
                black_box(intern.fetch_add(1, Ordering::Relaxed));
            }
        }));
        w = w.min(ns_per_iter(1, || {
            let maps: Vec<crate::value::pmap::PMap> = collected
                .iter()
                .map(|f| crate::value::pmap::PMap::from_pairs([(var.clone(), (*f).clone())]))
                .collect();
            black_box(maps);
        }));
    }
    assert!(w > 0.0, "from_pairs wrap recorded 0 ns — the loop never ran");

    println!(
        "\nharvest wrap parts — {N} one-entry maps, MINIMUM of {RUNS}\n\
             scan paid outside; pairs pre-cloned for R\n\
             \n\
             C  clone (var, fact) × 40k            {:>7.2} ms\n\
             R  Arc::from([pair]) × 40k            {:>7.2} ms\n\
             I  fetch_add × 40k                    {:>7.2} ms\n\
             W  from_pairs × 40k                   {:>7.2} ms\n\
             W−C  wrap minus clones                {:>7.2} ms\n",
        ms(c),
        ms(r),
        ms(i),
        ms(w),
        ms(w - c),
    );
}

/// Class-scan harvest is input ∪ derived. Skip-input must not drop
/// an inserted fact of a queried class (`DESIGN-STONE-accum-wanted-harvest`).
#[test]
fn class_scan_harvest_includes_input() {
    const WORLD: &str = "\
(:wat::core::defrecord :hs::T [x <- :wat::core::i64])\n\
(:wat::core::defrecord :hs::U [x <- :wat::core::i64])\n\
(:wat::rete::defrule :hs::never\n\
  :when [(:hs::T (?x <- :x) (:wat::rete::core::i64::< ?x 0))]\n\
  :then [(:hs::U ?x)])\n\
(:wat::rete::defquery :hs::q-T\n\
  :params []\n\
  :when [(?fact <- :hs::T)])\n\
(:wat::rete::defquery :hs::q-U\n\
  :params []\n\
  :when [(?fact <- :hs::U)])\n";
    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("input-scan world should freeze");
    let src = "(:wat::core::match (:wat::rete::fire-rules\n\
        (:wat::core::match (:wat::rete::insert\n\
          (:wat::core::match (:wat::rete::insert\n\
            (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :hs)\n\
              (:wat::core::PersistentVector (:hs::q-T) (:hs::q-U))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\n\
            (:hs::T 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n\
          (:hs::T 2)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))";
    let fired = eval_in_frozen(
        &crate::parse_one!(src).expect("parse input-scan fire"),
        &world,
        &Environment::new(),
    )
    .unwrap_or_else(|e| panic!("input-scan fire raised: {e:?}"))
    .value_owned();
    let maps = match session_named_field(&fired, "query-memory") {
        Some(Value::wat__core__PersistentMap(pm)) => pm
            .iter()
            .map(|(k, v)| {
                let name = match k {
                    Value::String(s) => s.as_ref().clone(),
                    _ => String::new(),
                };
                let n = match v {
                    Value::wat__core__PersistentVector(pv) => pv.len(),
                    _ => 0,
                };
                (name, n)
            })
            .collect::<Vec<_>>(),
        other => panic!("query-memory missing: {other:?}"),
    };
    let t = maps
        .iter()
        .find(|(n, _)| n == "hs::q-T")
        .map(|(_, n)| *n)
        .unwrap_or(0);
    let u = maps
        .iter()
        .find(|(n, _)| n == "hs::q-U")
        .map(|(_, n)| *n)
        .unwrap_or(0);
    assert_eq!(
        (t, u),
        (2, 0),
        "q-T must harvest the two inserted T; q-U empty (rule never fires): {maps:?}"
    );
}

/// Apportion the class-scan harvest bag: BUILD-THEN-EXTEND (today,
/// `harvest_class_scan_filter`) vs WRITE-IN-PLACE, at the fanout 40k ladder
/// (`NEXT-STRIKES-theater-hunt.md` T3).
///
/// DISCONFIRMING PROBE — prints the parts only, no fire-path change. A wash
/// STOPS the strike and the intermediate bag is recorded as physics.
#[test]
fn harvest_bag_copy_parts() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 40_000;
    const RUNS: usize = 3;
    const CLASS: &str = "fan::Pair";

    let var = Value::String(Arc::new("?fact".to_string()));
    let names = Arc::new(vec!["key".into(), "lid".into(), "rid".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                CLASS.into(),
                names.clone(),
                Arc::new(vec![Value::i64(i as i64), Value::i64(1), Value::i64(2)]),
            )))
        })
        .collect();

    // The callee as it stands today: allocates its own bag and returns it.
    fn scan_returning(facts: &[Value], cap: usize, var: &Value) -> Vec<crate::value::pmap::PMap> {
        let mut maps = Vec::with_capacity(cap);
        for f in facts {
            maps.push(crate::value::pmap::PMap::from_one(var.clone(), f.clone()));
        }
        maps
    }

    // The cut: write into the caller's vec, no intermediate bag.
    fn scan_into(out: &mut Vec<crate::value::pmap::PMap>, facts: &[Value], var: &Value) {
        out.reserve(facts.len());
        for f in facts {
            out.push(crate::value::pmap::PMap::from_one(var.clone(), f.clone()));
        }
    }

    let mut a = 0.0f64;
    let mut b = 0.0f64;
    let mut len_a = 0usize;
    let mut len_b = 0usize;

    for _ in 0..RUNS {
        // A — TODAY: capacity-less caller vec, extend from a freshly built bag.
        let t0 = Instant::now();
        let mut maps: Vec<crate::value::pmap::PMap> = Vec::new();
        maps.extend(scan_returning(&facts, facts.len(), &var));
        a += t0.elapsed().as_nanos() as f64;
        len_a = maps.len();
        black_box(&maps);
        drop(maps);

        // B — CUT: the callee writes straight into the caller's vec.
        let t0 = Instant::now();
        let mut maps: Vec<crate::value::pmap::PMap> = Vec::new();
        scan_into(&mut maps, &facts, &var);
        b += t0.elapsed().as_nanos() as f64;
        len_b = maps.len();
        black_box(&maps);
        drop(maps);
    }

    let r = RUNS as f64;
    let (a, b) = (a / r, b / r);
    let pmap_bytes = std::mem::size_of::<crate::value::pmap::PMap>();

    let table = format!(
        "\nharvest bag copy — {N} one-entry maps, MINIMUM of {RUNS}\n\
         PMap is {pmap_bytes} B, so the intermediate bag is {:.2} MB\n\
         \n\
         A  BUILD-THEN-EXTEND (today)     {:>7.2} ms\n\
         B  WRITE-IN-PLACE                {:>7.2} ms\n\
         A-B  the theater                 {:>7.2} ms\n",
        (pmap_bytes * N) as f64 / 1e6,
        ms(a),
        ms(b),
        ms(a - b),
    );
    println!("{table}");

    assert_eq!(len_a, N, "A must build 40k maps:{table}");
    assert_eq!(len_b, N, "B must build 40k maps:{table}");
    assert!(a > 0.0 && b > 0.0, "probe recorded no time:{table}");
}

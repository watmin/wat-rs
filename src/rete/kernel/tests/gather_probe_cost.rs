//! Gather / probe / seen-set cost — the join-key index and the per-token probes over it.


use super::*;

/// The gather-index-cache gate — RED until the index is cached per (alpha, join-keys).
///
/// `gather_index` is a pure function of (alpha memory, join keys), yet it is rebuilt per NODE
/// per round. At G=200 W=200 the accum world has THREE alpha nodes (200 Groups + two Reading
/// alphas of 40,000 — one binding ?g, one binding ?g,?v) and FIVE readers of them:
///   count   -> Reading-?g     (accumulate pass)
///   exists  -> Reading-?g     (filter pass)
///   sum/min/max -> Reading-?g?v (accumulate pass)
/// Five builds over TWO distinct (alpha_id, join_keys) pairs; three are pure repetition, each
/// dragging a full 40,000-element clone with it.
///
/// What would turn this red once green — the R59 question, answered before the assertion:
///   (a) the instrument counting nothing (`builds == 0`) — asserted separately, since a silent
///       zero would satisfy `<= 2` while measuring nothing at all;
///   (b) a cache keyed on `alpha_id` ALONE — it would read 2 here (every reader keys on ?g) and
///       be WRONG the moment two readers of one alpha have parents binding different variable
///       sets. This gate cannot catch that; the DESIGN's contract clause and the differentials
///       are what stand between it and a silent empty gather.
///   (c) the cache outliving a round — `wm.alpha` grows in step 1, so a stale index under-reads
///       and `count`/`sum` emit identities for groups that do have elements.
/// Landed: the fire-scoped `gather_cache` keyed on `(alpha_id, join_keys)`
/// (`DESIGN-STONE-gather-index-cache.md`, persist
/// `DESIGN-STONE-persist-gather-across-rounds.md`) makes this GREEN at 2
/// builds / 80,000 elements on a cold accum fire.
#[test]
fn gather_index_is_built_once_per_alpha_and_keyset() {
    let rows = accum_count_census(200, 200);
    let builds = rows
        .iter()
        .find(|(n, _)| *n == "accum:index-builds")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let elements = rows
        .iter()
        .find(|(n, _)| *n == "accum:index-elements")
        .map(|(_, c)| *c)
        .unwrap_or(0);

    assert!(
        builds > 0,
        "the index-build counter recorded ZERO — the counters were never reached, so `builds \
             <= 2` would pass while measuring nothing"
    );
    println!("\ngather index — builds {builds}, elements indexed {elements}\n");

    assert!(
        builds <= 2,
        "gather_index ran {builds} times over only TWO distinct (alpha_id, join_keys) pairs — \
             the index is being rebuilt per NODE instead of cached per (alpha, key-set). See \
             DESIGN-STONE-gather-index-cache.md."
    );
    assert!(
        elements <= 80_000,
        "indexed {elements} elements where 80,000 (the two distinct alpha memories, once each) \
             suffices — each redundant build drags a full-memory clone with it"
    );
}

/// `FxHashSet<Value>` vs fingerprint set (`DESIGN-STONE-seen-identity-set`).
#[test]
fn seen_identity_set_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["g".into(), "w".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "acc::Fact".into(),
                names.clone(),
                Arc::new(vec![
                    Value::i64((i / 200) as i64),
                    Value::i64((i % 200) as i64),
                ]),
            )))
        })
        .collect();
    for f in &facts {
        match f {
            Value::Aggregate(a) => assert!(a.identity() != 0, "fixture facts must be stamped"),
            other => panic!("fixture is a Record, got {other:?}"),
        }
    }


    black_box(facts.clone());
    {
        let mut s: FxHashSet<Value> = FxHashSet::with_capacity_and_hasher(N, Default::default());
        for f in &facts {
            s.insert(f.clone());
        }
        black_box(s.len());
        let mut ids: FxHashSet<u64> = FxHashSet::with_capacity_and_hasher(N, Default::default());
        for f in &facts {
            let Value::Aggregate(a) = f else { panic!() };
            ids.insert(a.identity());
        }
        black_box(ids.len());
    }

    let mut c = 0.0;
    let mut s = 0.0;
    let mut i = 0.0;
    for _ in 0..RUNS {
        c += elapsed_ns(|| {
            black_box(facts.clone());
        });
        s += elapsed_ns(|| {
            let mut set: FxHashSet<Value> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            for f in &facts {
                set.insert(f.clone());
            }
            black_box(set.len());
        });
        i += elapsed_ns(|| {
            let mut ids: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            for f in &facts {
                let Value::Aggregate(a) = f else { panic!() };
                ids.insert(a.identity());
            }
            black_box(ids.len());
        });
    }
    let r = RUNS as f64;
    c /= r;
    s /= r;
    i /= r;
    assert!(
        s > 0.0,
        "Value-set insert recorded 0 ns — the loop never ran"
    );

    println!(
        "\nseen identity-set split — {N} stamped Records, mean of {RUNS}\n\
             unscaled (accum [200 200] input count)\n\
             \n\
             C  clone 40,200 Values                {:>7.2} ms\n\
             S  FxHashSet<Value> insert (engine)   {:>7.2} ms\n\
             I  FxHashSet<u64> insert (identity)   {:>7.2} ms\n\
             \n\
             S−C  HashSet beyond clone             {:>7.2} ms\n\
             S−I  predicted cut                    {:>7.2} ms\n",
        ms(c),
        ms(s),
        ms(i),
        ms(s - c),
        ms(s - i),
    );
}

/// PersistentVector walk vs Vec walk on leftover `setup:seen`
/// (`DESIGN-STONE-seen-pv-walk`).
#[test]
fn seen_pv_walk_split() {
    use rustc_hash::FxHashSet;
    use std::hint::black_box;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let names = Arc::new(vec!["g".into(), "w".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "acc::Fact".into(),
                names.clone(),
                Arc::new(vec![
                    Value::i64((i / 200) as i64),
                    Value::i64((i % 200) as i64),
                ]),
            )))
        })
        .collect();
    let pv: rpds::VectorSync<Value> = facts.iter().cloned().collect();
    let ids: Vec<u64> = facts
        .iter()
        .map(|f| match f {
            Value::Aggregate(a) => a.identity(),
            _ => panic!("fixture is a Record"),
        })
        .collect();
    assert!(
        ids.iter().all(|&id| id != 0),
        "fixture facts must be stamped"
    );


    {
        let mut n = 0usize;
        for f in pv.iter() {
            n += 1;
            black_box(f);
        }
        black_box(n);
    }

    let mut w = 0.0;
    let mut i = 0.0;
    let mut v = 0.0;
    let mut p = 0.0;
    let mut d = 0.0;
    for _ in 0..RUNS {
        w += elapsed_ns(|| {
            let mut n = 0usize;
            for f in pv.iter() {
                n += 1;
                black_box(f);
            }
            black_box(n);
        });
        i += elapsed_ns(|| {
            let mut set: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            for id in &ids {
                set.insert(*id);
            }
            black_box(set.len());
        });
        v += elapsed_ns(|| {
            let mut ids_set: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            let mut rest: FxHashSet<Value> = FxHashSet::default();
            for f in &facts {
                super::seen_insert(&mut ids_set, &mut rest, f);
            }
            black_box(ids_set.len() + rest.len());
        });
        p += elapsed_ns(|| {
            let mut ids_set: FxHashSet<u64> =
                FxHashSet::with_capacity_and_hasher(N, Default::default());
            let mut rest: FxHashSet<Value> = FxHashSet::default();
            for f in pv.iter() {
                super::seen_insert(&mut ids_set, &mut rest, f);
            }
            black_box(ids_set.len() + rest.len());
        });
        d += elapsed_ns(|| {
            let collected: Vec<Value> = pv.iter().cloned().collect();
            black_box(collected.len());
        });
    }
    let r = RUNS as f64;
    w /= r;
    i /= r;
    v /= r;
    p /= r;
    d /= r;
    assert!(p > 0.0, "PV+insert recorded 0 ns — the loop never ran");

    println!(
        "\nseen PV-walk split — {N} stamped Records, mean of {RUNS}\n\
             unscaled (accum [200 200] input count)\n\
             \n\
             W  PersistentVector iter only         {:>7.2} ms\n\
             I  FxHashSet<u64> from Vec<u64>       {:>7.2} ms\n\
             V  Vec<Value> iter + seen_insert      {:>7.2} ms\n\
             P  PV iter + seen_insert (engine)     {:>7.2} ms\n\
             D  PV collect into Vec                {:>7.2} ms\n\
             \n\
             P−V  walk                             {:>7.2} ms\n\
             D+V  decode-then-Vec                  {:>7.2} ms\n\
             (D+V)−P                               {:>7.2} ms\n",
        ms(w),
        ms(i),
        ms(v),
        ms(p),
        ms(d),
        ms(p - v),
        ms(d + v),
        ms(d + v - p),
    );
}

/// Four clears of leftover `drop-memories` (`DESIGN-STONE-drop-memories-split`).
#[test]
fn drop_memories_cost_split() {
    use std::hint::black_box;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let gkey = Value::String(Arc::new("?g".into()));
    let vkey = Value::String(Arc::new("?v".into()));
    let names = Arc::new(vec!["g".into(), "v".into()]);
    let facts: Vec<Value> = (0..N)
        .map(|i| {
            Value::Aggregate(Arc::new(AggregateValue::record(
                "acc::Reading".into(),
                names.clone(),
                Arc::new(vec![
                    Value::i64((i / 200) as i64),
                    Value::i64((i % 200) as i64),
                ]),
            )))
        })
        .collect();

    fn build_alpha(
        facts: &[Value],
        gkey: &Value,
        vkey: &Value,
    ) -> (Vec<super::Element>, Vec<(u32, u32)>) {
        let mut keys = Vec::new();
        let mut vals = Vec::new();
        let mut ids = crate::rete::compiled_cond::ValIntern::default();
        let mut pool = Vec::new();
        let mut alpha = Vec::with_capacity(facts.len());
        for i in 0..facts.len() {
            let g = Value::i64((i / 200) as i64);
            let v = Value::i64((i % 200) as i64);
            let binds = super::span_from_pairs(
                &mut crate::rete::compiled_cond::BindIntern {
                    keys: &mut keys,
                    vals: &mut vals,
                    ids: &mut ids,
                    pool: &mut pool,
                },
                [(gkey.clone(), g), (vkey.clone(), v)],
            );
            alpha.push(super::Element {
                fact: i as u32,
                binds,
            });
        }
        (alpha, pool)
    }


    let mut a = 0.0;
    let mut b = 0.0;
    let mut m = 0.0;
    let mut t = 0.0;
    let mut d = 0.0;
    for _ in 0..RUNS {
        let (mut alpha, _) = build_alpha(&facts, &gkey, &vkey);
        a += elapsed_ns(|| {
            alpha.clear();
            black_box(alpha.len());
        });
        let (_, mut pool) = build_alpha(&facts, &gkey, &vkey);
        b += elapsed_ns(|| {
            pool.clear();
            black_box(pool.len());
        });
        let mut match_pool: Vec<(u32, i64)> = (0..N).map(|i| (i as u32, 1i64)).collect();
        m += elapsed_ns(|| {
            match_pool.clear();
            black_box(match_pool.len());
        });
        let mut tokens: Vec<super::Token> = (0..N)
            .map(|_| super::Token {
                matches: super::empty_span(),
                binds: super::empty_span(),
            })
            .collect();
        t += elapsed_ns(|| {
            tokens.clear();
            black_box(tokens.len());
        });
        let (mut alpha, mut pool) = build_alpha(&facts, &gkey, &vkey);
        let mut match_pool: Vec<(u32, i64)> = (0..N).map(|i| (i as u32, 1i64)).collect();
        let mut tokens: Vec<super::Token> = (0..N)
            .map(|_| super::Token {
                matches: super::empty_span(),
                binds: super::empty_span(),
            })
            .collect();
        d += elapsed_ns(|| {
            alpha.clear();
            tokens.clear();
            pool.clear();
            match_pool.clear();
            black_box(alpha.len() + tokens.len() + pool.len() + match_pool.len());
        });
    }
    let r = RUNS as f64;
    a /= r;
    b /= r;
    m /= r;
    t /= r;
    d /= r;
    assert!(d > 0.0, "drop-all recorded 0 ns — the loop never ran");

    println!(
        "\ndrop-memories split — {N} Elements / pairs / matches, mean of {RUNS}\n\
             construction untimed; this times clear() only\n\
             \n\
             A  drop Vec<Element>                  {:>7.2} ms\n\
             B  drop bind_pool                     {:>7.2} ms\n\
             M  drop match_pool                    {:>7.2} ms\n\
             T  drop Vec<Token>                    {:>7.2} ms\n\
             D  all four (authority)               {:>7.2} ms\n",
        ms(a),
        ms(b),
        ms(m),
        ms(t),
        ms(d),
    );
}

/// Unary gather index vs `Vec<Value>` keys (`DESIGN-STONE-gather-unary-index`).
#[test]
fn gather_unary_index_split() {
    use rustc_hash::FxHashMap;
    use std::hint::black_box;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let gkey = Value::String(Arc::new("?g".into()));
    let vkey = Value::String(Arc::new("?v".into()));
    let join_keys = [gkey.clone()];
    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut pool: Vec<(u32, u32)> = Vec::new();
    let mut els: Vec<super::Element> = Vec::with_capacity(N);
    for i in 0..N {
        let g = Value::i64((i / 200) as i64);
        let v = Value::i64((i % 200) as i64);
        let binds = super::span_from_pairs(
            &mut crate::rete::compiled_cond::BindIntern {
                keys: &mut keys,
                vals: &mut vals,
                ids: &mut ids,
                pool: &mut pool,
            },
            [(gkey.clone(), g), (vkey.clone(), v)],
        );
        els.push(super::Element { fact: 0, binds });
    }


    black_box(super::build_gather_index(
        &els,
        &join_keys,
        super::GatherIntern::of(&keys, &vals, &pool, &ids),
    ));

    let mut k = 0.0;
    let mut v = 0.0;
    let mut u = 0.0;
    let mut b = 0.0;
    let mut s = 0.0;
    for _ in 0..RUNS {
        k += elapsed_ns(|| {
            for el in &els {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                black_box(super::key_of(&pairs, &join_keys, &ids));
            }
        });
        v += elapsed_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<super::JoinKey, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                let key = super::key_of(&pairs, &join_keys, &ids);
                idx.entry(key).or_default().push(i);
            }
            black_box(idx.len());
        });
        u += elapsed_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<Value, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                if let Some(val) = Bindings::get(&pairs, &gkey) {
                    idx.entry(val.clone()).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
        b += elapsed_ns(|| {
            black_box(super::build_gather_index(
                &els,
                &join_keys,
                super::GatherIntern::of(&keys, &vals, &pool, &ids),
            ));
        });
        s += elapsed_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<Value, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                if let Some(val) = Bindings::get(&pairs, &gkey) {
                    idx.entry(val.clone()).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
    }
    let r = RUNS as f64;
    k /= r;
    v /= r;
    u /= r;
    b /= r;
    s /= r;
    assert!(
        b > 0.0,
        "build_gather_index recorded 0 ns — the loop never ran"
    );

    println!(
        "\ngather unary-index split — {N} Readings, join_keys=[?g], mean of {RUNS}\n\
             unscaled (one build; the cell pays two)\n\
             \n\
             K  40k key_of                         {:>7.2} ms\n\
             V  40k HashMap<Vec<Value>> insert     {:>7.2} ms\n\
             U  40k HashMap<Value> insert          {:>7.2} ms\n\
             B  build_gather_index (authority)     {:>7.2} ms\n\
             S  get + unary insert                 {:>7.2} ms\n\
             \n\
             B−S  predicted cut per build          {:>7.2} ms\n\
             ×2 builds on the cell                 {:>7.2} ms\n",
        ms(k),
        ms(v),
        ms(u),
        ms(b),
        ms(s),
        ms(b - s),
        ms((b - s) * 2.0),
    );
}

/// Unary gather `HashMap<Value>` vs interned filler id
/// (`DESIGN-STONE-gather-val-id`).
#[test]
fn gather_val_id_split() {
    use rustc_hash::FxHashMap;
    use std::hint::black_box;

    const N: usize = 40_200;
    const RUNS: usize = 3;

    let gkey = Value::String(Arc::new("?g".into()));
    let vkey = Value::String(Arc::new("?v".into()));
    let join_keys = [gkey.clone()];
    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut pool: Vec<(u32, u32)> = Vec::new();
    let mut els: Vec<super::Element> = Vec::with_capacity(N);
    for i in 0..N {
        let g = Value::i64((i / 200) as i64);
        let v = Value::i64((i % 200) as i64);
        let binds = super::span_from_pairs(
            &mut crate::rete::compiled_cond::BindIntern {
                keys: &mut keys,
                vals: &mut vals,
                ids: &mut ids,
                pool: &mut pool,
            },
            [(gkey.clone(), g), (vkey.clone(), v)],
        );
        els.push(super::Element { fact: 0, binds });
    }
    let kid = super::intern_key(&mut keys, &gkey);


    let mut u = 0.0;
    let mut iarm = 0.0;
    let mut b = 0.0;
    for _ in 0..RUNS {
        u += elapsed_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<Value, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::element_fact_bindings(el, &keys, &vals, &pool);
                if let Some(val) = Bindings::get(&pairs, &gkey) {
                    idx.entry(val.clone()).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
        iarm += elapsed_ns(|| {
            // rune:perspicere(read-once) — gather microbench index; not a domain noun.
            let mut idx: FxHashMap<u32, Vec<usize>> = FxHashMap::default();
            for (i, el) in els.iter().enumerate() {
                let pairs = super::pool_slice(&pool, el.binds);
                if let Some((_, vid)) = pairs.iter().find(|(k, _)| *k == kid) {
                    idx.entry(*vid).or_default().push(i);
                }
            }
            black_box(idx.len());
        });
        b += elapsed_ns(|| {
            black_box(super::build_gather_index(
                &els,
                &join_keys,
                super::GatherIntern::of(&keys, &vals, &pool, &ids),
            ));
        });
    }
    let r = RUNS as f64;
    u /= r;
    iarm /= r;
    b /= r;
    assert!(iarm > 0.0, "vid insert recorded 0 ns — the loop never ran");
    println!(
        "\ngather val-id split — {N} Readings, join_keys=[?g], mean of {RUNS}\n\
             unscaled (one build; the cell pays two)\n\
             \n\
             U  HashMap<Value> clone+insert        {:>7.2} ms\n\
             I  HashMap<u32> vid insert            {:>7.2} ms\n\
             B  build_gather_index (authority)     {:>7.2} ms\n\
             \n\
             U−I  predicted cut per build          {:>7.2} ms\n\
             ×2 builds on the cell                 {:>7.2} ms\n",
        ms(u),
        ms(iarm),
        ms(b),
        ms(u - iarm),
        ms((u - iarm) * 2.0),
    );
}

/// Apportion leftover `hj:catchup:probe` (12.30 ms / 40k ≈ 307 ns)
/// without nesting 40k marks (`DESIGN-STONE-probe-extend-split`).
#[test]
fn probe_extend_cost_split() {
    use std::hint::black_box;

    const N: usize = 300_000;
    const RUNS: usize = 3;
    const EXTENDS: f64 = 40_000.0;
    const LEFTS: f64 = 2_000.0;

    let k = Value::String(Arc::new("?k".into()));
    let l = Value::String(Arc::new("?l".into()));
    let r = Value::String(Arc::new("?r".into()));
    let join_keys = [k.clone()];

    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut bind_pool: Vec<(u32, u32)> = Vec::new();
    let mut match_pool: Vec<(u32, i64)> = Vec::new();
    let left_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (l.clone(), Value::i64(2))],
    );
    let right_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (r.clone(), Value::i64(3))],
    );
    let left_matches = super::push_match(&mut match_pool, 0, 1);
    let tok = super::Token {
        matches: left_matches,
        binds: left_binds,
    };

    // rune:perspicere(read-once) — gather microbench index; not a domain noun.
    let mut idx: HashMap<Vec<Value>, usize> = HashMap::new();
    idx.insert(vec![Value::i64(1)], 20);


    // Warm.
    {
        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        black_box(super::extend_token(
            &tok,
            0,
            right_binds,
            2,
            &mut bp,
            &mut mp,
        ));
        black_box(super::key_of(
            &super::bind_view(&keys, &vals, &bind_pool, left_binds),
            &join_keys,
            &ids,
        ));
        black_box(idx.get(&vec![Value::i64(1)]));
    }

    let mut b = 0.0;
    let mut m = 0.0;
    let mut e = 0.0;
    let mut kk = 0.0;
    let mut h = 0.0;
    for _ in 0..RUNS {
        let mut bp = bind_pool.clone();
        bp.reserve(N * 4);
        b += ns_per_iter(N, || {
            let lo = left_binds.off as usize;
            let ln = left_binds.len as usize;
            let eo = right_binds.off as usize;
            let en = right_binds.len as usize;
            let start = bp.len();
            for i in 0..ln {
                let p = bp[lo + i];
                bp.push(p);
            }
            for i in 0..en {
                let (kk, vv) = bp[eo + i];
                let already = (start..start + ln).any(|j| bp[j].0 == kk);
                if !already {
                    bp.push((kk, vv));
                }
            }
            black_box(bp.len());
        });

        let mut mp = match_pool.clone();
        mp.reserve(N * 2);
        m += ns_per_iter(N, || {
            let mo = left_matches.off as usize;
            let mn = left_matches.len as usize;
            for i in 0..mn {
                let e = mp[mo + i];
                mp.push(e);
            }
            mp.push((0, 2));
            black_box(mp.len());
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        bp.reserve(N * 4);
        mp.reserve(N * 2);
        e += ns_per_iter(N, || {
            black_box(super::extend_token(
                &tok,
                0,
                right_binds,
                2,
                &mut bp,
                &mut mp,
            ));
        });

        kk += ns_per_iter(N, || {
            black_box(super::key_of(
                &super::bind_view(&keys, &vals, &bind_pool, left_binds),
                &join_keys,
                &ids,
            ));
        });

        h += ns_per_iter(N, || {
            black_box(idx.get(&vec![Value::i64(1)]));
        });
    }
    let runs = RUNS as f64;
    b /= runs;
    m /= runs;
    e /= runs;
    kk /= runs;
    h /= runs;
    assert!(e > 0.0, "extend_token recorded 0 ns — the loop never ran");

    let scale_e = |ns: f64| ns * EXTENDS / 1e6;
    let scale_l = |ns: f64| ns * LEFTS / 1e6;
    println!(
        "\nprobe extend split — fanout shape, {N} iters, mean of {RUNS}\n\
             treat the RATIO as the finding; scaled ms is a projection, not a fire\n\
             \n\
             B  bind append only                 {b:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             M  match append + fact.clone        {m:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             E  extend_token (authority)         {e:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             K  key_of one join key              {kk:>7.1} ns/op   {:>6.2} ms @ 2k\n\
             H  HashMap::get(Vec<Value>)         {h:>7.1} ns/op   {:>6.2} ms @ 2k\n\
             \n\
             B+M                                 {:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             K+H                                 {:>7.1} ns/op   {:>6.2} ms @ 2k\n",
        scale_e(b),
        scale_e(m),
        scale_e(e),
        scale_l(kk),
        scale_l(h),
        b + m,
        scale_e(b + m),
        kk + h,
        scale_l(kk + h),
    );
}

/// Apportion the in-fire probe gap (12.30 − E 7.08) without nesting
/// 40k marks (`DESIGN-STONE-probe-gap-split`).
#[test]
fn probe_gap_cost_split() {
    use crate::rete::compiled_cond::{CompiledCond, Op};
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 300_000;
    const G_N: usize = 40_000;
    const RUNS: usize = 3;
    const EXTENDS: f64 = 40_000.0;

    let k = Value::String(Arc::new("?k".into()));
    let l = Value::String(Arc::new("?l".into()));
    let r = Value::String(Arc::new("?r".into()));
    let names = Arc::new(vec!["key".into(), "id".into()]);
    let fact = Value::Aggregate(Arc::new(AggregateValue::record(
        "fan::Right".into(),
        names,
        Arc::new(vec![Value::i64(1), Value::i64(3)]),
    )));
    let mut keys = Vec::new();
    let mut vals = Vec::new();
    let mut ids = crate::rete::compiled_cond::ValIntern::default();
    let mut bind_pool: Vec<(u32, u32)> = Vec::new();
    let mut match_pool: Vec<(u32, i64)> = Vec::new();
    let left_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (l.clone(), Value::i64(2))],
    );
    let right_binds = super::span_from_pairs(
        &mut crate::rete::compiled_cond::BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut bind_pool,
        },
        [(k.clone(), Value::i64(1)), (r.clone(), Value::i64(3))],
    );
    let left_matches = super::push_match(&mut match_pool, 0, 1);
    let tok = super::Token {
        matches: left_matches,
        binds: left_binds,
    };
    let el = super::Element {
        fact: 0,
        binds: right_binds,
    };
    let facts_pv = Value::wat__core__PersistentVector(crate::value::pvec::PVec::new());
    let derived = [fact.clone()];
    let compiled = CompiledCond::from_parts(
        vec![
            Op::Bind {
                field_idx: 0,
                slot: 0,
            },
            Op::Bind {
                field_idx: 1,
                slot: 1,
            },
        ],
        Arc::from([]),
        Arc::from([]),
        2,
        Arc::from([]),
        None,
        crate::rust_caller_span!(),
        Box::from([]),
    );
    let mut conds: HashMap<i64, CompiledCond> = HashMap::new();
    conds.insert(2, compiled);
    let mut scratch: SlotFrame = Vec::new();
    let bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
    let cond_key_ids: CondKeyIds = HashMap::new();
    let i64_by_fact: Vec<Option<super::I64Row>> = Vec::new();


    {
        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        black_box(super::extend_token(&tok, 0, el.binds, 2, &mut bp, &mut mp));
        black_box(super::rematch_compiled(&conds, 2).expect("compiled"));
        black_box(conds.get(&2).expect("id").has_seed_cmp());
        black_box(
            super::join_extend(
                &tok,
                &el,
                2,
                &mut super::FireCtx {
            sym: crate::rete::compiled_cond::test_sym(),
                    compiled_conds: &conds,
                    scratch: &mut scratch,
                    pool: &mut bp,
                    match_pool: &mut mp,
                    keys: &keys,
                    vals: &vals,
                    val_ids: &ids,
                    facts: &facts_pv,
                    derived: &derived,
                    n_input: 0,
                    i64_by_fact: &i64_by_fact,
                    bind_only: &bind_only,
                    cond_key_ids: &cond_key_ids,
                },
            )
            .expect("join"),
        );
    }

    let mut r = 0.0;
    let mut s = 0.0;
    let mut p = 0.0;
    let mut e = 0.0;
    let mut j = 0.0;
    let mut g = 0.0;
    for _ in 0..RUNS {
        r += ns_per_iter(N, || {
            black_box(super::rematch_compiled(&conds, 2).expect("compiled"));
        });
        s += ns_per_iter(N, || {
            black_box(conds.get(&2).expect("id").has_seed_cmp());
        });
        let mut out: Vec<super::Token> = Vec::with_capacity(N);
        p += ns_per_iter(N, || {
            out.push(tok);
            black_box(out.len());
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        bp.reserve(N * 4);
        mp.reserve(N * 2);
        e += ns_per_iter(N, || {
            black_box(super::extend_token(&tok, 0, el.binds, 2, &mut bp, &mut mp));
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        bp.reserve(N * 4);
        mp.reserve(N * 2);
        scratch.clear();
        j += ns_per_iter(N, || {
            black_box(
                super::join_extend(
                    &tok,
                    &el,
                    2,
                    &mut super::FireCtx {
            sym: crate::rete::compiled_cond::test_sym(),
                        compiled_conds: &conds,
                        scratch: &mut scratch,
                        pool: &mut bp,
                        match_pool: &mut mp,
                        keys: &keys,
                        vals: &vals,
                        val_ids: &ids,
                        facts: &facts_pv,
                        derived: &derived,
                        n_input: 0,
                        i64_by_fact: &i64_by_fact,
                        bind_only: &bind_only,
                        cond_key_ids: &cond_key_ids,
                    },
                )
                .expect("join"),
            );
        });

        let mut bp = bind_pool.clone();
        let mut mp = match_pool.clone();
        let t0 = Instant::now();
        for _ in 0..G_N {
            black_box(super::extend_token(&tok, 0, el.binds, 2, &mut bp, &mut mp));
        }
        g += t0.elapsed().as_nanos() as f64;
    }
    let runs = RUNS as f64;
    r /= runs;
    s /= runs;
    p /= runs;
    e /= runs;
    j /= runs;
    g /= runs;
    assert!(j > 0.0, "join_extend recorded 0 ns — the loop never ran");

    let scale = |ns: f64| ns * EXTENDS / 1e6;
    let g_ms = g / 1e6;
    let e_ms = scale(e);
    println!(
        "\nprobe gap split — fanout shape, {N} iters (G is {G_N} unreserved), mean of {RUNS}\n\
             treat the RATIO as the finding; scaled ms is a projection, not a fire\n\
             \n\
             R  rematch_compiled                 {r:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             S  has_seed_cmp                     {s:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             P  Vec<Token>::push                 {p:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             E  extend_token reserved            {e:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             J  join_extend reserved             {j:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             G  extend_token × 40k unreserved    {:>7.1} ns/op   {g_ms:>6.2} ms @ 40k\n\
             \n\
             J−E wrapper                         {:>7.1} ns/op   {:>6.2} ms @ 40k\n\
             G−E growth                          {:>6.2} ms @ 40k\n\
             R+S+P                               {:>7.1} ns/op   {:>6.2} ms @ 40k\n",
        scale(r),
        scale(s),
        scale(p),
        e_ms,
        scale(j),
        g / G_N as f64,
        j - e,
        scale(j - e),
        g_ms - e_ms,
        r + s + p,
        scale(r + s + p),
    );
}

/// Volume of `d_beta_from_parents` — the capacity-less `Vec<Token>` gather
/// (`NEXT-STRIKES-theater-hunt.md` T8). Reports calls / tokens / allocating
/// calls / MULTI-parent calls across two workloads, so the strike is aimed at
/// measured volume rather than an assumed one.
///
/// TRIPWIRE. T8 claimed this gather grows-and-reallocs a capacity-less `Vec`.
/// It cannot while every call has at most ONE contributing parent: a single
/// `extend` from an `ExactSizeIterator` reserves the exact length in one shot,
/// and an empty gather never allocates. Both fixtures measure 0 MULTI-parent
/// calls, which is why T8 was CLEARED rather than cut. If either ever reports
/// one, that premise has changed and T8 is live again.
#[test]
fn dbeta_gather_volume() {
    const STRATA: i64 = 6;
    const ITEMS: i64 = 2000;
    const STRAT_WORLD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/wat-scripts/perf/grid/strat-neg.wat"));

    /// calls, tokens, allocating calls, multi-parent calls.
    struct Gather {
        calls: u64,
        tokens: u64,
        alloc: u64,
        multi: u64,
    }

    let run = |label: &str, world_src: &str, seed_src: String| -> Gather {
        let world = startup_from_source(world_src, None, Arc::new(InMemoryLoader::new()))
            .unwrap_or_else(|e| panic!("{label} world should freeze: {e:?}"));
        let staged = eval_in_frozen(
            &crate::parse_one!(seed_src.as_str()).expect("parse seed"),
            &world,
            &Environment::new(),
        )
        .unwrap_or_else(|e| panic!("{label} seed raised: {e:?}"))
        .value_owned();

        let (_fired, counts) = super::with_count_census(|| {
            fire_rules_on_session(&staged, world.symbols(), None)
                .unwrap_or_else(|e| panic!("{label} fire raised: {e:?}"))
        });
        let get = |n: &str| -> u64 {
            counts.iter().find(|(k, _)| *k == n).map(|(_, v)| *v).unwrap_or(0)
        };
        Gather {
            calls: get("dbeta:calls"),
            tokens: get("dbeta:tokens"),
            alloc: get("dbeta:alloc"),
            multi: get("dbeta:multi"),
        }
    };

    let strat = run(
        "strat-neg",
        STRAT_WORLD,
        format!("(:strat::seed-items (:wat::core::match (:wat::rete::compile (:strat::build-rules {STRATA})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {ITEMS})"),
    );
    let accum = run(
        "accum",
        ACCUM_AXIS_WORLD,
        "(:apx::seed (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :apx)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) 200 200)".to_string(),
    );

    let row = |label: &str, g: &Gather| -> String {
        format!(
            "{label:<20} calls {:>6}   allocating {:>6}   MULTI-parent {:>6}   tokens {:>8}\n",
            g.calls, g.alloc, g.multi, g.tokens
        )
    };
    let table = format!(
        "\nd_beta_from_parents volume\n\n{}{}",
        row("strat-neg [6 2000]", &strat),
        row("accum [200 200]", &accum),
    );
    println!("{table}");

    // Structured, exact — never a substring match on the rendering above.
    assert!(strat.calls > 0, "strat-neg gather never ran:{table}");
    assert!(accum.calls > 0, "accum gather never ran:{table}");
    assert_eq!(
        strat.multi, 0,
        "a MULTI-parent gather appeared in strat-neg — T8's premise changed, re-open it:{table}"
    );
    assert_eq!(
        accum.multi, 0,
        "a MULTI-parent gather appeared in accum — T8's premise changed, re-open it:{table}"
    );
}

/// How much does the `d_beta_from_parents` copy actually cost? The two call
/// sites document themselves as a borrow-checker workaround
/// (`NEXT-STRIKES-theater-hunt.md`), so before trying to remove one, size it.
#[test]
fn dbeta_copy_size() {
    println!(
        "\nToken is {} B; a d_beta_from_parents Vec of 2000 is {:.1} KB\n",
        std::mem::size_of::<Token>(),
        (std::mem::size_of::<Token>() * 2000) as f64 / 1024.0
    );
    assert!(std::mem::size_of::<Token>() > 0);
}

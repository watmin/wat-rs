//! The binding-key / token-bindings REPRESENTATION decision.
//!
//! `partire` verified this region names ZERO symbols from its host module — no `super::`, no
//! `FireSession`, no `to_transient`, no `eval_in`. It builds `Value`/`Arc`/`HashTrieMapSync`
//! directly, which is why it is its own module rather than part of the census.


use super::*;

// ── Inside one alpha bind: key CONSTRUCTION vs the MAP operation ──────────────────────
//
// ⛔ NEITHER ARM BELOW IS THE LIVE REPRESENTATION. The engine's token bindings are two
// `BindSpan`s into fire-scoped pools — `Token { matches, binds }` at `session.rs:64` — not a
// `String`-keyed `rpds` trie; and the `eval_clause` whose key construction arm (a) reproduces
// is off the native hot path entirely — `exec_compiled_with_key_ids` took over as the round
// loop's step 1 (`compiled_cond.rs:912` says so outright, and `match:calls` reads ZERO on a real
// fire because of it). It is not dead: `alpha_match_inner{,_local,_seeded}` is still what the
// `:wat::rete::alpha-match{,-local,-under}` primitives run, which is how the wat oracle matches
// (`wat/rete/oracle/pass.wat:21-22,193`) — it is the differential's interpreter, not the
// engine's. What this prices is therefore a live cost on the ORACLE side and EVIDENCE FOR the
// stone that chose the pooled form — not a measurement of the native path. `fire/delta.rs:725-726`
// records that stone's one premise — "a binding map holds 1-2 entries, so an rpds trie is paying
// trie prices for a pair" — and the ratios here are what one bind costs when it is spent the
// old way.
//
// `eval_clause` does `Value::String(Arc::new(var.to_string()))` per bind — a fresh String plus
// a fresh Arc, to key on a variable name that is a compile-time constant. Interning it would
// reduce that to an Arc refcount bump. Whether that is worth doing depends on its share of a
// bind, and the alternative (changing the binding map's representation) is a substrate-wide
// change shared by joins, negation, token extension and the oracle differential — so the cheap
// fix deserves to be priced first.
//
// ⚠ HONEST BOUND: this is a tight-loop microbenchmark, not the engine. Allocator state and
// cache behaviour differ from a real fire, so treat the RATIO between the three as the finding
// and not the absolute nanoseconds.
//
// ⛔ AND THERE IS NO IN-ENGINE ANCHOR TO APPORTION. This header, and the table below, used to
// quote "the ~163 ns in-engine bind" from `alpha_match_cost_per_binding`
// (`rank_and_instrument.rs:219`) — four times in prose and once in the printed output. That
// source no longer measures a positive per-binding cost at all. Driven three times at HEAD
// (2026-09-02) it reports the TWO-bind world FASTER than the one-bind world: −2, −4 and
// −10 ns/fact, from alpha arms of ~15 ms whose own run-to-run spread (14.48–15.29 ms) is wider
// than every one of those deltas. A negative cost for strictly more work is an instrument below
// its resolution, not a finding — that test declines to assert the sign for exactly that reason
// — so there is no total left to apportion INTO. The absolute is dropped rather than
// re-measured: a fresh hard-coded nanosecond figure would be the same defect with a newer
// number. The RATIO between (a), (b) and (c) is the finding, as the bound above already said.
#[test]
fn bind_key_construction_vs_map_operation() {
    use std::hint::black_box;
    const N: usize = 300_000;
    let var = "?g";
    let val = Value::i64(42);
    let interned = Value::String(Arc::new(var.to_string()));
    let empty: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();

    // (a) what we do today: build the key from scratch, every bind.
    let t0 = std::time::Instant::now();
    for _ in 0..N {
        let key = Value::String(Arc::new(var.to_string()));
        black_box(&key);
    }
    let fresh_ns = t0.elapsed().as_nanos() as f64 / N as f64;

    // (b) what interning would cost instead: an Arc refcount bump.
    let t1 = std::time::Instant::now();
    for _ in 0..N {
        let key = interned.clone();
        black_box(&key);
    }
    let interned_ns = t1.elapsed().as_nanos() as f64 / N as f64;

    // (c) the map operation itself, key supplied — get (the already-bound check) then insert
    // into a fresh empty map, which is what a first bind on an element does.
    let t2 = std::time::Instant::now();
    for _ in 0..N {
        let m = empty.clone();
        black_box(m.get(&interned));
        let m2 = m.insert(interned.clone(), val.clone());
        black_box(&m2);
    }
    let map_ns = t2.elapsed().as_nanos() as f64 / N as f64 - interned_ns; // subtract the clone (c) also pays

    assert!(
        fresh_ns > 0.0 && map_ns > 0.0,
        "microbenchmark recorded nothing"
    );

    // ⛔ `probare` classed this test hollow — the guard above is liveness on two clocks. The
    // comparison is fresh-`String` construction vs an interned `Arc` bump vs a map operation, and
    // nothing checked the map operation DID anything. An `insert` that no-ops is faster than one
    // that inserts, so the arm this test exists to price would look cheapest when it is broken.
    let probe_empty: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    let probe_after = probe_empty.insert(interned.clone(), val.clone());
    assert_eq!(
        probe_empty.size(),
        0,
        "the persistent map is not persistent — `insert` mutated the receiver, so arm (c) is \
         timing a mutation, not the copy-on-write insert it reports"
    );
    assert_eq!(
        probe_after.size(),
        1,
        "`insert` produced a map of size {}, not 1 — arm (c) is timing a no-op and would read as \
         the cheapest option precisely because it does nothing",
        probe_after.size()
    );

    println!(
            "\nbind cost apportioned — {N} iterations each (RATIOS, not absolutes)\n                 (a) fresh key   Value::String(Arc::new(var.to_string()))  {fresh_ns:>6.1} ns\n                 (b) interned    an Arc refcount bump                      {interned_ns:>6.1} ns\n                 (c) map         get + insert, key supplied                {map_ns:>6.1} ns\n                 ---------------------------------------------------------------\n                 interning would save (a)-(b) = {:>5.1} ns per bind; the map itself is\n                 {:>5.1} ns and is untouched by interning — the RATIO is the finding.\n                 (No in-engine anchor: `alpha_match_cost_per_binding` now measures a\n                  NEGATIVE per-binding cost, so there is no total to apportion into.)\n",
            fresh_ns - interned_ns, map_ns
        );
}

/// Microbenchmark — how much of a binding-map operation is the STRING KEY?
///
/// Binding keys are `Value::String(Arc<String>)` — a fresh heap String per
/// bind, hashed and memcmp'd on every lookup. **Clara's are interned Clojure keywords**
/// (`engine.cljc:23` "a map of keyword-to-values"; `compiler.clj:293` assoc's `(keyword var)`),
/// which carry a CACHED hash and compare by pointer.
///
/// `9448f012` measured "interning the bind key saves 8% — the MAP is 85% of it" and concluded
/// interning was not worth a stone. That split may be an artifact: if the map operation's cost
/// is largely *hashing the string key*, then "the map" and "the key" are not separable and the
/// 85% already contains the thing the 8% was measuring. This isolates it by changing ONLY the
/// key type on an otherwise identical map.
///
/// `Value::i64` stands in for an interned symbol id (hash of an i64, compare by value) — the
/// floor an interning scheme could reach, not a proposal for the key type itself.
///
/// ⛔ NOT ON THE RELEASE FLOOR — `#[ignore]`, and the reason is measured, not assumed.
///
/// To gate this we would have to assert the direction it exists to show: that an interned-id
/// key beats a fresh `String` key. Measured over three runs (2026-08-30), the effect is too
/// small to carry a threshold — **lookup ratio 1.0–1.1x, build ratio 1.1–1.9x, with the
/// minimum touching 1.0**. Any floor tight enough to catch a regression sits inside the noise,
/// and this repo bans known flakes absolutely; a threshold here would manufacture the very
/// thing that ban exists to prevent.
///
/// So it stays a diagnostic and leaves the floor, rather than being counted as a passing test
/// that cannot fail. Run it with:
///     cargo nextest run --release --run-ignored=only --no-capture -E 'test(binding_key_cost)'
///
/// Diagnostic. Read with `--no-capture`.
#[test]
#[ignore = "diagnostic microbenchmark: measured effect (1.0-1.9x) is too small to gate without flaking"]
fn binding_key_cost() {
    use std::hint::black_box;
    use std::time::Instant;
    const N: usize = 50_000;

    println!("\nBINDING KEY COST — Value::String (today) vs Value::i64 (an interned-id floor)");
    println!(
        "  {N} iterations; rpds::HashTrieMapSync in BOTH columns — only the KEY type differs\n"
    );
    println!(
        "  {:>4}  {:>21}  {:>21}",
        "n", "build (str / i64)", "lookup (str / i64)"
    );

    for n in [1usize, 2, 3, 5, 8] {
        let sk: Vec<(Value, Value)> = (0..n)
            .map(|i| {
                (
                    Value::String(Arc::new(format!("?v{i}"))),
                    Value::i64(i as i64),
                )
            })
            .collect();
        let ik: Vec<(Value, Value)> = (0..n)
            .map(|i| (Value::i64(i as i64), Value::i64(i as i64)))
            .collect();

        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut m = rpds::HashTrieMapSync::new_sync();
            for (k, v) in &sk {
                m.insert_mut(k.clone(), v.clone());
            }
            sink.push(m);
        }
        let bs = t.elapsed().as_nanos() as f64 / N as f64;
        let ms = sink[0].clone();
        drop(sink);

        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut m = rpds::HashTrieMapSync::new_sync();
            for (k, v) in &ik {
                m.insert_mut(k.clone(), v.clone());
            }
            sink.push(m);
        }
        let bi = t.elapsed().as_nanos() as f64 / N as f64;
        let mi = sink[0].clone();
        drop(sink);

        let ps = sk[n / 2].0.clone();
        let pi = ik[n / 2].0.clone();
        let t = Instant::now();
        for _ in 0..N {
            black_box(ms.get(black_box(&ps)));
        }
        let ls = t.elapsed().as_nanos() as f64 / N as f64;
        let t = Instant::now();
        for _ in 0..N {
            black_box(mi.get(black_box(&pi)));
        }
        let li = t.elapsed().as_nanos() as f64 / N as f64;

        println!(
            "  {:>4}  {:>9.1} /{:>9.1}  {:>9.1} /{:>9.1}   build {:>4.1}x  lookup {:>4.1}x",
            n,
            bs,
            bi,
            ls,
            li,
            bs / bi,
            ls / li
        );
    }
    println!();
}

/// Microbenchmark — rpds HAMT vs a persistent ARRAY map, at binding-map sizes.
///
/// The follow-on stone's claim is "an rpds trie pays HAMT prices on a 1-3 entry map, and
/// Clojure/Clara get an array representation for free below 8." That claim was PREDICTED, never
/// measured. This measures it, before any stone is drawn.
///
/// The comparison must be the HONEST analogue. Clojure's PersistentArrayMap is not a bare Vec —
/// it is an IMMUTABLE array behind a reference, so `clone` is a refcount bump exactly as the
/// HAMT's is, and only the LOOKUP differs (linear scan vs hash+trie descent). A bare `Vec`
/// would lose catastrophically on clone and prove nothing about the real design.
///   A = rpds::HashTrieMapSync<Value,Value>   (today)
///   B = Arc<Vec<(Value,Value)>>              (PersistentArrayMap's shape)
///
/// Five operations, chosen because they are what the kernel actually does to a binding map:
///   build   — alpha match constructs one per fact
///   lookup  — accum:fold (94 ns/element) and token_element_compatible
///   clone   — alpha:push (this REGRESSED when Element went native)
///   extend  — extend_token: clone + insert one binding (rpds shares structurally; the array copies)
///   drop    — round:drop-memories (41 ms)
///
/// Keys are real `Value::String(Arc<str>)` — hashing/comparing a wat String is the actual cost,
/// and an integer-keyed benchmark would flatter the HAMT.
///
/// ⛔ NOT ON THE RELEASE FLOOR — `#[ignore]`. Its own doc already said "not a gate"; it was on
/// the floor anyway, counted among the passing tests while asserting nothing at all.
///
/// It compares FIVE operations across two representations at four cardinalities, so there is no
/// single ordering to assert — and the sibling `binding_key_cost` measurement showed effects in
/// this family run at 1.0–1.9x, which is inside runner noise. Gating it would mean inventing a
/// threshold for a comparison nobody currently depends on.
///
///     cargo nextest run --release --run-ignored=only --no-capture -E 'test(binding_repr_microbench)'
///
/// Diagnostic, not a gate. Read with `--no-capture`.
#[test]
#[ignore = "diagnostic microbenchmark: five-way comparison with no single assertable ordering"]
fn binding_repr_microbench() {
    use std::hint::black_box;
    use std::time::Instant;

    const SIZES: [usize; 8] = [1, 2, 3, 4, 5, 8, 12, 16];
    const N: usize = 20_000;

    fn keys(n: usize) -> Vec<(Value, Value)> {
        (0..n)
            .map(|i| {
                (
                    Value::String(Arc::new(format!("?v{i}"))),
                    Value::i64(i as i64),
                )
            })
            .collect()
    }

    println!("\nBINDING REPRESENTATION — rpds HAMT (A) vs persistent array map (B)");
    println!("  {N} iterations per cell; ns/op; keys are real Value::String\n");
    println!(
        "  {:>4}  {:>19}  {:>19}  {:>19}  {:>19}  {:>19}",
        "n", "build", "lookup", "clone", "extend", "drop"
    );
    println!(
        "  {:>4}  {:>19}  {:>19}  {:>19}  {:>19}  {:>19}",
        "", "A / B", "A / B", "A / B", "A / B", "A / B"
    );

    for n in SIZES {
        let kv = keys(n);
        let probe = kv[n / 2].0.clone();
        let extra = (Value::String(Arc::new("?zz".to_string())), Value::i64(99));

        // ── build (construct into a reserved Vec; drop timed separately) ──
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink_a: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut m = rpds::HashTrieMapSync::new_sync();
            for (k, v) in &kv {
                m.insert_mut(k.clone(), v.clone());
            }
            sink_a.push(m);
        }
        let build_a = t.elapsed().as_nanos() as f64 / N as f64;

        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut sink_b: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut v = Vec::with_capacity(n);
            for (k, val) in &kv {
                v.push((k.clone(), val.clone()));
            }
            sink_b.push(Arc::new(v));
        }
        let build_b = t.elapsed().as_nanos() as f64 / N as f64;

        let ma = sink_a[0].clone();
        let mb = sink_b[0].clone();

        // ── lookup (hit, mid-map) ──
        let t = Instant::now();
        for _ in 0..N {
            black_box(ma.get(black_box(&probe)));
        }
        let look_a = t.elapsed().as_nanos() as f64 / N as f64;
        let t = Instant::now();
        for _ in 0..N {
            black_box(Bindings::get(mb.as_slice(), black_box(&probe)));
        }
        let look_b = t.elapsed().as_nanos() as f64 / N as f64;

        // ── clone ──
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut ca: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            ca.push(ma.clone());
        }
        let clone_a = t.elapsed().as_nanos() as f64 / N as f64;
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut cb: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            cb.push(Arc::clone(&mb));
        }
        let clone_b = t.elapsed().as_nanos() as f64 / N as f64;
        drop(ca);
        drop(cb);

        // ── extend (extend_token: derive a new map with one more binding) ──
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut ea: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            ea.push(ma.insert(extra.0.clone(), extra.1.clone()));
        }
        let ext_a = t.elapsed().as_nanos() as f64 / N as f64;
        // rune:perspicere(read-once) — microbench sink; alias would be a mumble.
        let mut eb: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
        let t = Instant::now();
        for _ in 0..N {
            let mut v = (*mb).clone();
            v.push(extra.clone());
            eb.push(Arc::new(v));
        }
        let ext_b = t.elapsed().as_nanos() as f64 / N as f64;
        drop(ea);
        drop(eb);

        // ── drop (the sinks built above) ──
        let t = Instant::now();
        drop(sink_a);
        let drop_a = t.elapsed().as_nanos() as f64 / N as f64;
        let t = Instant::now();
        drop(sink_b);
        let drop_b = t.elapsed().as_nanos() as f64 / N as f64;

        println!("  {:>4}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}",
                     n, build_a, build_b, look_a, look_b, clone_a, clone_b, ext_a, ext_b, drop_a, drop_b);
    }
    println!("\n  A = rpds::HashTrieMapSync (today)   B = Arc<Vec<(Value,Value)>>\n"); // rune:lint(no-angle-type-in-diagnostic) — RUST types in a bench header, not wat
}

/// Diagnostic — the binding-cardinality distribution, the PREMISE under the
/// binding-representation stone.
///
/// The stone's whole argument is that a binding map holds 1-2 entries, so an
/// `rpds::HashTrieMapSync` (heap alloc + Arc + hash + pointer-chase + dealloc) is paying trie
/// prices for a pair. If the distribution is wide, an inline small-vec is WORSE and the stone
/// inverts. Nobody had measured it.
///
/// Load-bearing subtlety: binding cardinality is a property of the RULE SHAPE, not the data
/// volume. A 2-condition rule binding 3 distinct vars yields 3-binding tokens at 10 facts and
/// at 10 million. So this drives SEVERAL rule shapes and reports each — a single workload
/// would answer a narrower question than the one the stone asks.
///
/// Read with `--no-capture`. Diagnostic, not a gate; the assertion only stops it reporting an
/// artifact (a census that counted nothing would print an empty table reading as "all zero").
#[test]
fn binding_cardinality_distribution() {
    fn dist(label: &str, rows: &[(&'static str, u64)]) -> String {
        let get = |k: &str| {
            rows.iter()
                .find(|(n, _)| *n == k)
                .map(|(_, c)| *c)
                .unwrap_or(0)
        };
        let els = get("bind-card:ELEMENTS");
        let toks = get("bind-card:TOKENS");
        let total = els + toks;
        let mut out = format!("\n  {label}  —  {els} elements, {toks} tokens\n");
        if total == 0 {
            out.push_str("    (nothing counted)\n");
            return out;
        }
        for (kind, tot, pfx) in [("ELEMENT", els, "elem-card:"), ("TOKEN", toks, "tok-card:")] {
            if tot == 0 {
                continue;
            }
            out.push_str(&format!("    {kind}S ({tot})\n"));
            for suf in ["0", "1", "2", "3", "4", "5", "6-7", "8+"] {
                let key = format!("{pfx}{suf}");
                let n = rows
                    .iter()
                    .find(|(nm, _)| *nm == key)
                    .map(|(_, c)| *c)
                    .unwrap_or(0);
                if n == 0 {
                    continue;
                }
                out.push_str(&format!(
                    "      {:<6} {:>9}  {:>5.1}%\n",
                    suf,
                    n,
                    100.0 * n as f64 / tot as f64
                ));
            }
        }
        out
    }

    let mut report = String::from("\nBINDING CARDINALITY — the premise under the small-vec stone");

    // Shape A — accumulate: conditions bind ?g / ?g,?v; tokens carry the group key.
    let rows_accum = accum_count_census(60, 60);
    report.push_str(&dist("accumulate  (accum axis, G=60 W=60)", &rows_accum));

    // Shape B — a 2-condition JOIN binding THREE distinct vars across the conditions
    // (?loc shared, ?t from one, ?w from the other). This is the shape that grows a token's
    // binding map, and the one an accumulate-only measurement would never show.
    const J: &str = "\
(:wat::core::defrecord :bcd::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::i64])\n\
(:wat::core::defrecord :bcd::WindSpeed   [kph      <- :wat::core::i64  location <- :wat::core::i64])\n\
(:wat::core::defrecord :bcd::Cw          [loc <- :wat::core::i64  t <- :wat::core::i64  w <- :wat::core::i64])\n\
(:wat::core::defn :bcd::seed [n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::let [c1   (:wat::core::quote (:bcd::Temperature (?loc <- :location) (?t <- :celsius)))\n\
                    c2   (:wat::core::quote (:bcd::WindSpeed (?loc <- :location) (?w <- :kph)))\n\
                    rhs1 (:wat::core::quote (:bcd::Cw ?loc ?t ?w))\n\
                    rule (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))\n\
                    s0   (:wat::core::match (:wat::rete::compile (:wat::core::PersistentVector rule)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))]\n\
    (:wat::core::foldl\n\
      (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
        (:wat::core::let [a (:wat::core::match (:wat::rete::insert acc (:bcd::Temperature :celsius i :location i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))]\n\
          (:wat::core::match (:wat::rete::insert a (:bcd::WindSpeed :kph i :location i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))))\n\
      s0 (:wat::core::range 0 n))))\n\
";
    let wj = startup_from_source(J, None, Arc::new(InMemoryLoader::new()))
        .expect("join world should freeze");
    let ast = crate::parse_one!("(:wat::core::match (:wat::rete::fire-rules (:bcd::seed 400)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))").expect("parse");
    let (_f, rows_join) = super::with_count_census(|| {
        eval_in_frozen(&ast, &wj, &Environment::new())
            .unwrap_or_else(|e| panic!("join fire raised: {e:?}"))
            .value_owned()
    });
    report.push_str(&dist("2-cond join, 3 distinct vars (N=400)", &rows_join));

    let counted: u64 = rows_accum
        .iter()
        .chain(rows_join.iter())
        .filter(|(n, _)| {
            n.starts_with("bind-card:") || n.starts_with("elem-card:") || n.starts_with("tok-card:")
        })
        .map(|(_, c)| *c)
        .sum();
    // ⛔ `probare` classed this test hollow — the guard below is liveness. `counted` is summed
    // from the engine's own census over the FIXED (60, 60) accum axis, so it is deterministic.
    // The exact figure is asserted by the sibling check that follows; this one keeps the
    // original guard because a zero here has its own, clearer message.
    // ⛔ MEASURED, NOT DERIVED — 16,920 over the fixed (60, 60) accum axis. I did not compute
    // this from the 7,260 elements the report prints; an earlier assertion in this batch was
    // reasoned rather than probed and came out 3x wrong, so this one is the number the census
    // actually produced.
    assert_eq!(
        counted, 16_920,
        "the (60, 60) accum axis counted {counted} binding cells, not 16,920 — the cardinality \
         distribution below is describing a different population than the stone it tests"
    );
    assert!(
        counted > 0,
        "the binding census counted NOTHING — the walk never ran, so an all-zero table \
             would be an artifact, not a distribution"
    );

    println!("{report}");
}

// ── Token.bindings representation — the DOMINANCE probe ──────────────────────────────
//
// ⛔ NEITHER ARM HERE IS THE LIVE REPRESENTATION, and `Token.bindings` no longer exists at
// all. The engine's token bindings are a `BindSpan` into the fire-scoped bind pool —
// `Token { matches, binds }` at `session.rs:64` — while this probe compares an `rpds` trie
// against a persistent array, the two candidates that were on the table when the pooled form
// was chosen. So this is the EVIDENCE FOR that stone rather than a measurement of it:
// `fire/delta.rs:725-726` records the single premise the comparison tests — "a binding map
// holds 1-2 entries, so an rpds trie ... is paying trie prices for a pair" — and the
// binding-cardinality census that sits beside that stone measures the distribution this probe
// assumes. Both arms stay for that reason: they are why the pooled form is defensible.
//
// 41c59cde made `Element.bindings` an array and left `Token.bindings` a trie, with the
// reason: *"the trie's sole advantage is extend, which an Element never does."* That is
// airtight in the direction it was used (an Element never extends → a trie buys it
// nothing). Its CONVERSE — Token extends, therefore a trie is right for Token — does not
// follow from it and was never measured. This probe measures it.
//
// ⚠ THE QUESTION IS DOMINANCE, NOT A THRESHOLD. R60 killed picking a representation from
// a corpus census of our own rules ("you have no fucking clue what our users are going to
// do"), and that cut stands. So this asks only: does one representation win across the
// WHOLE plausible cardinality range? If yes, there is no constant to tune and no corpus
// dependence, and the answer is honest. If the array only wins below some N, that N is a
// corpus-derived threshold, R60's cut applies, and the trie stays.
//
// The shape is the real one: ONE parent extended by FANOUT elements — which is where a
// trie's structural sharing is supposed to pay, since every child shares the parent's
// nodes while an array copies the whole prefix into each child.

/// Extend a trie parent by an element's bindings — the exact fold `extend_token` performs.
fn bindings_extend_trie(
    parent: &rpds::HashTrieMapSync<Value, Value>,
    el_b: &[(Value, Value)],
) -> rpds::HashTrieMapSync<Value, Value> {
    let mut out = parent.clone();
    for (k, v) in el_b {
        if out.get(k) != Some(v) {
            out.insert_mut(k.clone(), v.clone());
        }
    }
    out
}

/// The array twin — same semantics (idempotent skip for a shared key already equal).
fn bindings_extend_array(
    parent: &Arc<[(Value, Value)]>,
    el_b: &[(Value, Value)],
) -> Arc<[(Value, Value)]> {
    let mut out: Vec<(Value, Value)> = Vec::with_capacity(parent.len() + el_b.len());
    out.extend_from_slice(parent);
    for (k, v) in el_b {
        if !out.iter().any(|(ek, ev)| ek == k && ev == v) {
            out.push((k.clone(), v.clone()));
        }
    }
    out.into()
}

fn kv(i: usize) -> (Value, Value) {
    (
        Value::String(Arc::new(format!("?v{i}"))),
        Value::i64(i as i64),
    )
}

#[test]
fn token_bindings_representation_dominance() {
    use std::hint::black_box;

    const FANOUT: usize = 20; // one parent, 20 children — the fanout cell's shape
    const REPS: usize = 400;
    let cards = [1usize, 2, 3, 4, 8, 16, 32, 64];

    let mut table = String::from(
            "\n  TOKEN.BINDINGS REPRESENTATION — one parent x 20 children, 400 reps\n\
             \n  card    EXTEND trie   EXTEND array   ratio      GET trie    GET array   ratio\n\
             \x20 -------------------------------------------------------------------------------\n",
        );
    let mut extend_array_wins = 0usize;
    let mut get_array_wins = 0usize;
    // Every row's four timings, kept so the assertions below can name a SPECIFIC cardinality
    // rather than a counter. A counter says how often the array won; only the row says where
    // the crossover is, and the crossover is the finding.
    // (card, extend trie ns, extend array ns, get trie ns, get array ns)
    let mut rows: Vec<(usize, f64, f64, f64, f64)> = Vec::with_capacity(cards.len());

    for &c in &cards {
        // The parent: `c` existing bindings, built once, in both representations.
        let mut trie: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
        let mut arr: Vec<(Value, Value)> = Vec::new();
        for i in 0..c {
            let (k, v) = kv(i);
            trie.insert_mut(k.clone(), v.clone());
            arr.push((k, v));
        }
        let arr: Arc<[(Value, Value)]> = arr.into();

        // Each child contributes one shared key (skipped) + one new key — the real shape:
        // a join key already bound by the parent, plus the element's own variable.
        // rune:perspicere(read-once) — microbench fanout rows; alias would be a mumble.
        let el: Vec<Vec<(Value, Value)>> = (0..FANOUT).map(|f| vec![kv(0), kv(1000 + f)]).collect();

        // Faithfulness gate FIRST: the twin must produce the same logical binding set, or
        // the timings below are comparing two different computations.
        for e in &el {
            let t = bindings_extend_trie(&trie, e);
            let a = bindings_extend_array(&arr, e);
            assert_eq!(
                t.size(),
                a.len(),
                "card {c}: the array twin is not faithful — trie {} keys vs array {}",
                t.size(),
                a.len()
            );
            for (k, v) in a.iter() {
                assert_eq!(
                    t.get(k),
                    Some(v),
                    "card {c}: key {k:?} disagrees between reps"
                );
            }
        }

        let mut warm = 0usize;
        for e in &el {
            warm += bindings_extend_trie(&trie, e).size() + bindings_extend_array(&arr, e).len();
        }
        black_box(warm);

        let t0 = std::time::Instant::now();
        for _ in 0..REPS {
            for e in &el {
                black_box(bindings_extend_trie(black_box(&trie), black_box(e)));
            }
        }
        let ext_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        let t0 = std::time::Instant::now();
        for _ in 0..REPS {
            for e in &el {
                black_box(bindings_extend_array(black_box(&arr), black_box(e)));
            }
        }
        let ext_arr = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        // GET is the other half: the matcher reads bindings constantly, and the array pays
        // a linear scan. A representation that extends faster but reads slower is not a win.
        // Probe the WORST key (last inserted) so the scan is not flattered.
        let probe = kv(c.saturating_sub(1)).0;
        let t0 = std::time::Instant::now();
        for _ in 0..REPS * FANOUT {
            black_box(black_box(&trie).get(black_box(&probe)));
        }
        let get_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        let t0 = std::time::Instant::now();
        for _ in 0..REPS * FANOUT {
            black_box(Bindings::get(black_box(arr.as_ref()), black_box(&probe)));
        }
        let get_arr = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        if ext_arr < ext_trie {
            extend_array_wins += 1;
        }
        if get_arr < get_trie {
            get_array_wins += 1;
        }
        rows.push((c, ext_trie, ext_arr, get_trie, get_arr));

        table.push_str(&format!(
                "  {c:>4}  {ext_trie:>10.1}ns  {ext_arr:>11.1}ns  {:>6.2}x  {get_trie:>10.1}ns  {get_arr:>10.1}ns  {:>6.2}x\n",
                ext_trie / ext_arr,
                get_trie / get_arr,
            ));
    }

    table.push_str(&format!(
        "\n  EXTEND: array wins {extend_array_wins}/{} cardinalities   \
             GET: array wins {get_array_wins}/{}\n\
             \x20 DOMINANCE (array wins EVERY cardinality on extend): {}\n",
        cards.len(),
        cards.len(),
        if extend_array_wins == cards.len() {
            "YES"
        } else {
            "NO — a threshold, so R60's cut stands"
        },
    ));
    println!("{table}");

    // ── What must hold ──────────────────────────────────────────────────────────────────
    //
    // The assertion that stood here was `extend_array_wins + get_array_wins < usize::MAX`,
    // message `"unreachable"` — true for every pair of `usize`, under a comment declaring the
    // check it did not make. Below is the check that comment declares, plus the three orderings
    // the printed verdict rests on. Every failure interpolates the WHOLE table, so a red arrives
    // carrying the measurement that produced it and nobody has to re-run to see what happened —
    // a re-run being the one move that destroys the evidence.
    assert_eq!(
        rows.len(),
        cards.len(),
        "the probe recorded {} rows for {} cardinalities — the table below is not the population \
         the counters were taken over\n{table}",
        rows.len(),
        cards.len()
    );

    // (1) NON-VACUITY — the check the comment above the old assertion always declared. A zero
    //     here means the probe timed nothing: on a working clock the array cannot lose EVERY
    //     cell, because at the smallest cardinality it is one compare against a hash plus a
    //     trie descent.
    assert!(
        extend_array_wins + get_array_wins > 0,
        "the probe measured NOTHING — across {} cardinalities and both operations the array \
         representation did not come out ahead in a single cell. That is not a result, it is a \
         dead clock or a broken twin: a one-entry array beats a HAMT lookup by construction\n{table}",
        cards.len()
    );

    // The two ends of the range. The EXTEND column at the small end is measured and printed but
    // deliberately NOT asserted: over twelve drives (2026-09-02) its ratio ran 1.19-2.02x, and a
    // 19% margin does not clear the ~16% the absolutes in this family reproduce to. That is the
    // same reasoning that keeps `binding_key_cost` off the floor two tests above, and this repo
    // bans known flakes absolutely — a gate inside the noise would manufacture one. The three
    // orderings below all carry margins measured at 2.6x or better across those same drives.
    let (lo_c, _lo_ext_trie, _lo_ext_arr, lo_get_trie, lo_get_arr) = rows[0];
    let (hi_c, hi_ext_trie, hi_ext_arr, hi_get_trie, hi_get_arr) = rows[rows.len() - 1];

    // (2) THE SMALL END, GET — the array must be read faster at the smallest cardinality. This
    //     is the half of the threshold that makes the stone's premise pay: at 1-2 bindings an
    //     array read is a scan of one key against a hash plus a trie descent. If the trie ever
    //     wins HERE, `fire/delta.rs:725-726`'s premise is what broke, and that is the news, not
    //     this test. (Measured 4.53-10.15x; it is also the row that makes (1) hold, so a red in
    //     (1) and a red here name the same collapse from two distances.)
    assert!(
        lo_get_arr < lo_get_trie,
        "at the smallest cardinality ({lo_c}) the trie was READ faster than the array \
         ({lo_get_trie:.1}ns vs {lo_get_arr:.1}ns) — a linear scan of {lo_c} entry/entries lost \
         to a hash plus a trie descent, which cannot be right\n{table}"
    );

    // (3)+(4) THE LARGE END — the array must lose both operations at the largest cardinality.
    //     This is what makes the printed verdict ("DOMINANCE: NO — a threshold, so R60's cut
    //     stands") a measurement instead of a caption: array wins at the small end and loses at
    //     the large end IS the threshold, and a threshold is exactly what R60 refuses to tune
    //     from our own corpus. Were the array to win here too, the verdict would flip to
    //     dominance and the representation question would be reopened — so these two are the
    //     load-bearing half of the conclusion, not decoration.
    assert!(
        hi_ext_trie < hi_ext_arr,
        "at the largest cardinality ({hi_c}) the array EXTENDED faster than the trie \
         ({hi_ext_arr:.1}ns vs {hi_ext_trie:.1}ns) — copying {hi_c} pairs beat structural \
         sharing. If that reproduces, the array DOMINATES and the verdict printed above is \
         wrong\n{table}"
    );
    assert!(
        hi_get_trie < hi_get_arr,
        "at the largest cardinality ({hi_c}) the array was READ faster than the trie \
         ({hi_get_arr:.1}ns vs {hi_get_trie:.1}ns) — a linear scan over {hi_c} entries beat a \
         hashed lookup, so either the probe key stopped being the worst-case one or the twins \
         are no longer the same computation\n{table}"
    );
}

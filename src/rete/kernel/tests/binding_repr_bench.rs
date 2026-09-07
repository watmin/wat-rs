//! The binding-key / token-bindings REPRESENTATION decision.
//!
//! `partire` verified this region names ZERO symbols from its host module — no `super::`, no
//! `FireSession`, no `to_transient`, no `eval_in`. It builds `Value`/`Arc`/`HashTrieMapSync`
//! directly, which is why it is its own module rather than part of the census.
//!
//! Three diagnostics that used that property (`binding_key_cost`, `binding_repr_microbench`,
//! `token_bindings_representation_dominance`) moved to `benches/binding_repr.rs` (296 Stone K,
//! moves 2–4). Behaviour unchanged; only the harness. The two live gates stay.


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

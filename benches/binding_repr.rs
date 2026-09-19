//! Binding-key / token-bindings representation diagnostics (296 Stone K, moves 2–4).
//!
//! Relocated from `src/rete/kernel/tests/binding_repr_bench.rs`. Behaviour is UNCHANGED;
//! only the harness. A benchmark is not in the test binary at all, so the category is
//! structural rather than an `#[ignore]` excused by a string.
//!
//! `partire` verified the source region names ZERO symbols from its host module — no
//! `super::`, no `FireSession`, no `to_transient`, no `eval_in`. Two of the three
//! diagnostics called `Bindings::get` (`pub(crate)` in `matcher.rs`). That call is the
//! slice linear-scan already written below as [`array_get`] — the same body as the
//! `[(Value, Value)]` impl — so this crate reaches only `wat::Value`, `std`, and `rpds`.
//! The public API was not widened.
//!
//! The three `rune:excusare` reasons survive here as doc comments. They are measured
//! evidence: without them a future hand reads three unasserted benchmarks and "fixes"
//! them into assertions. `no_unknown_ward_rune` walks `src`/`crates`/`tests`/`wat`/
//! `wat-scripts`/`wat-tests`, not `benches/`; the literal rune text is kept so the
//! category and the reason stay on the record.
//!
//! Run: cargo bench --bench binding_repr

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;
use wat::Value;

/// Linear scan over an array map. Same body as the `Bindings` impl for `[(Value, Value)]`
/// (`matcher.rs`), inlined so this crate does not reach the `pub(crate)` trait.
fn array_get<'a>(pairs: &'a [(Value, Value)], k: &Value) -> Option<&'a Value> {
    pairs.iter().find(|(kk, _)| kk == k).map(|(_, v)| v)
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
/// rune:excusare(below-resolution) — lookup 1.0–1.1× / build 1.1–1.9× (three runs 2026-08-30);
/// this floor's rete-cohort contention band is 3.5×–4.4× (.config/nextest.toml: 8.13s→35.39s,
/// 7.98s→29.42s, 13.77s→48.72s). A 1.9× ceiling sits inside that band, so any floor tight
/// enough to catch a regression is a threshold inside the noise.
///
/// To gate this we would have to assert the direction it exists to show: that an interned-id
/// key beats a fresh `String` key. The effect is too small to carry a threshold. So it stays
/// a diagnostic. Run: cargo bench --bench binding_repr
fn binding_key_cost() {
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
/// rune:excusare(no-falsifier) — tried asserting array-wins-extend at every cardinality (the
/// dominance question): the sibling token_bindings_representation_dominance already prints
/// DOMINANCE: NO on that exact question, so the assert would be a known-false gate. Tried a
/// single-cell ordering: one cell of a 5×2×4 grid, and a green is not evidence about the table.
/// The test sat on the floor asserting nothing, counted as a passing test that cannot fail.
/// Nothing achievable reds the rest of the grid without inventing a crossover N, which R60
/// refuses.
///
/// It compares FIVE operations across two representations at four cardinalities, so there is no
/// single ordering to assert — and the sibling `binding_key_cost` measurement showed effects in
/// this family run at 1.0–1.9x, which is inside runner noise.
///
/// Run: cargo bench --bench binding_repr
fn binding_repr_microbench() {
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
            black_box(array_get(mb.as_slice(), black_box(&probe)));
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

/// Token.bindings representation — the dominance probe. Diagnostic, not a gate.
///
/// NOT a correctness gate — timings are machine-relative, so there is no hard
/// timing assertion. The four ordering assertions that used to sit under the
/// table compared two sequential wall-clock windows with a bare `<`; under this
/// floor's 3.5×–4.4× contention band that cannot separate the hypotheses.
///
/// Faithfulness (the twins produce the same binding set) stays; it is the
/// benchmark's precondition, not a production guard (`bindings_extend_array`
/// does not ship).
///
/// measured 2026-09-07, 6 isolated samples, largest cardinality (64):
///   EXTEND median trie 676.3 ns (range 642.3–716.7) vs array 2028.8 ns (range 1989.4–2569.8);
///   GET    median trie 35.1 ns (range 28.4–69.9) vs array 346.2 ns (range 340.9–662.5);
///   DOMINANCE: NO, so R60's cut stands. Trie won EXTEND at card 64 in 6/6.
///
/// rune:excusare(below-resolution) — margin: the array/trie EXTEND ratio at card 64 —
/// medians 2028.8 ns / 676.3 ns = 3.0×, six isolated samples 2026-09-07. Noise floor: this
/// floor's rete-cohort contention band, measured 3.5×–4.4× (.config/nextest.toml:
/// 8.13s→35.39s, 7.98s→29.42s, 13.77s→48.72s). The margin sits inside the floor. Captured
/// red .floor/2026-09-07T03-20-25Z (trie 5860.1 ns vs array 3995.9 ns at card 64) is the
/// consequence, not the margin.
///
/// Run: cargo bench --bench binding_repr
fn token_bindings_representation_dominance() {
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

        let t0 = Instant::now();
        for _ in 0..REPS {
            for e in &el {
                black_box(bindings_extend_trie(black_box(&trie), black_box(e)));
            }
        }
        let ext_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        let t0 = Instant::now();
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
        let t0 = Instant::now();
        for _ in 0..REPS * FANOUT {
            black_box(black_box(&trie).get(black_box(&probe)));
        }
        let get_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

        let t0 = Instant::now();
        for _ in 0..REPS * FANOUT {
            black_box(array_get(black_box(arr.as_ref()), black_box(&probe)));
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

    // Structural: the table is the population the counters were taken over.
    // Timing-ordering assertions are gone — they cannot separate the hypotheses
    // under this floor's contention band. Faithfulness, above the loop, is the
    // remaining gate.
    assert_eq!(
        rows.len(),
        cards.len(),
        "the probe recorded {} rows for {} cardinalities — the table below is not the population \
         the counters were taken over\n{table}",
        rows.len(),
        cards.len()
    );
    assert!(
        extend_array_wins + get_array_wins > 0,
        "the probe measured NOTHING — across {} cardinalities and both operations the array \
         representation did not come out ahead in a single cell. That is not a result, it is a \
         dead clock or a broken twin: a one-entry array beats a HAMT lookup by construction\n{table}",
        cards.len()
    );
}

fn main() {
    binding_key_cost();
    binding_repr_microbench();
    token_bindings_representation_dominance();
}

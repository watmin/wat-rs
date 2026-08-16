//! Arc 278 — fire-rules throughput BASELINE (kept perf measurement, run on demand).
//!
//! This is the reference measuring stick for the engine perf arc: it times the current **wat-eval**
//! `fire-rules` (the re-run-from-scratch reference engine) over a 2-condition join (cold-and-windy) at
//! growing fact counts. When the **Rust fire kernel** (delta propagation + join-bindings-keyed joins +
//! native mutable memories) lands, extend this file to print the rust-eval column alongside — the wat/rust
//! ratio is the speedup, and the bar is Clara-parity-or-superior.
//!
//! NOT a correctness gate — timings are machine-relative, so there is no hard timing assertion (only a
//! sanity check that the engine still derives). `#[ignore]`d so it stays out of the normal suite.
//!
//! Run: cargo test --release -p wat --test perf_arc278_fire_baseline -- --ignored --nocapture
//!
//! First baseline (2026-06-19, re-run-from-scratch wat engine — QUADRATIC, the wasteful tree we replace):
//!   N= 25  (  50 facts)   ~61ms     ~820 facts/s
//!   N= 50  ( 100 facts)  ~201ms     ~500 facts/s
//!   N=100  ( 200 facts)  ~762ms     ~260 facts/s
//!   N=200  ( 400 facts) ~1799ms     ~220 facts/s
//!   N=400  ( 800 facts) ~6134ms     ~130 facts/s   (per-fact cost climbs 1.2ms→7.7ms = O(N^2))

use std::time::Instant;
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

// rune:lint(no-inlined-wat) — perf-sweep world generated at runtime from N (25/50/100/200/400 and
// 100/200/400/800/1600 fact counts); the whole point is the SWEEP across growing N, so no fixed
// .wat fixture can stand in without losing the scaling measurement itself. #[ignore]d, non-gating.

// N Temperatures + N WindSpeeds at N distinct locations → N same-loc joins → N derived ColdAndWindy.
fn run_for(n: usize) {
    let world = startup_beside(file!()).expect("startup");

    let mut binds = String::from(
        "   c1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))\
            c2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))\
            rhs1 (:wat::core::quote (:weather::ColdAndWindy ?loc))\
            rule (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))\
            s0   (:wat::rete::compile (:wat::core::PersistentVector rule))\n",
    );
    let mut prev = 0usize;
    let mut idx = 1usize;
    for j in 0..n {
        binds.push_str(&format!(
            "   s{idx} (:wat::rete::insert s{prev} (:weather::Temperature :celsius 15 :location \"loc{j}\"))\n"
        ));
        prev = idx;
        idx += 1;
    }
    for j in 0..n {
        binds.push_str(&format!(
            "   s{idx} (:wat::rete::insert s{prev} (:weather::WindSpeed :kph 45 :location \"loc{j}\"))\n"
        ));
        prev = idx;
        idx += 1;
    }
    let expr = format!(
        "(:wat::core::let [{binds}\n fired (:wat::rete::fire-rules-spec s{prev})\n pmem (:wat::rete::Session/production-memory fired)]\
           (:wat::core::length (:wat::core::PersistentMap/keys pmem)))"
    );
    let ast = wat::parse_one!(&expr).expect("parse");

    let t = Instant::now();
    let out = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned();
    let dt = t.elapsed();
    let facts = 2 * n;
    eprintln!(
        "N={n:>4} facts={facts:>5}  fire-rules={:>9.2}ms  {:>8.1}us/fact  ({:>9.0} facts/s)  [prodnodes={out:?}]",
        dt.as_secs_f64() * 1e3,
        dt.as_secs_f64() * 1e6 / facts as f64,
        facts as f64 / dt.as_secs_f64()
    );
}

#[test]
#[ignore = "ON-DEMAND (not debt) — PERF BASELINE, deliberately outside the floor: it measures, it \
            does not gate. Run: cargo nextest run --release --run-ignored only \
            -E 'test(fire_throughput_baseline)' --no-capture. HOME: needs a real mechanism (a nextest profile + default-filter in .config/nextest.toml, which already carries profiles and per-test overrides) so ON-DEMAND stops inflating the ignore count. Until then this marker makes the two populations mechanically separable."]
fn fire_throughput_baseline() {
    eprintln!("--- wat-eval fire-rules throughput (re-run-from-scratch reference engine) ---");
    for &n in &[25usize, 50, 100, 200, 400] {
        run_for(n);
    }
}

// ─── Native fire-once' join-scaling (the P3 curve-bend measure) ──────────────────
// N Temps + N Winds at N DISTINCT locations → N same-loc joins out of N×N candidate pairs.
// The native hash-join cost is the variable: P2 (cross) is O(N²); P3 (keyed) is O(N).
// Times `(:wat::rete::fire-once' s)` — the per-fact us should stay ~flat under keying, climb under cross.
fn run_native(n: usize) {
    let world = startup_beside(file!()).expect("startup");
    let mut binds = String::from(
        "   c1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))\
            c2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))\
            rhs1 (:wat::core::quote (:weather::ColdAndWindy ?loc))\
            rule (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))\
            s0   (:wat::rete::compile (:wat::core::PersistentVector rule))\n",
    );
    let mut prev = 0usize;
    let mut idx = 1usize;
    for j in 0..n {
        binds.push_str(&format!("   s{idx} (:wat::rete::insert s{prev} (:weather::Temperature :celsius 15 :location \"loc{j}\"))\n"));
        prev = idx; idx += 1;
    }
    for j in 0..n {
        binds.push_str(&format!("   s{idx} (:wat::rete::insert s{prev} (:weather::WindSpeed :kph 45 :location \"loc{j}\"))\n"));
        prev = idx; idx += 1;
    }
    let expr = format!(
        "(:wat::core::let [{binds}\n fired (:wat::rete::fire-once' s{prev})\n pmem (:wat::rete::Session/production-memory fired)]\
           (:wat::core::length (:wat::core::PersistentMap/keys pmem)))"
    );
    let ast = wat::parse_one!(&expr).expect("parse");
    let t = Instant::now();
    let _ = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}")).value_owned();
    let dt = t.elapsed();
    let facts = 2 * n;
    eprintln!(
        "N={n:>4} facts={facts:>5}  fire-once'={:>9.2}ms  {:>8.1}us/fact  ({:>9.0} facts/s)",
        dt.as_secs_f64() * 1e3, dt.as_secs_f64() * 1e6 / facts as f64, facts as f64 / dt.as_secs_f64()
    );
}

#[test]
#[ignore = "ON-DEMAND (not debt) — PERF: native fire-once join scaling (P2 cross = O(N^2); \
            P3 keyed = O(N)). Measures, does not gate. Run: cargo nextest run --release \
            --run-ignored only -E 'test(native_fire_once_join_scaling)' --no-capture. HOME: needs a real mechanism (a nextest profile + default-filter in .config/nextest.toml, which already carries profiles and per-test overrides) so ON-DEMAND stops inflating the ignore count. Until then this marker makes the two populations mechanically separable."]
fn native_fire_once_join_scaling() {
    eprintln!("--- native fire-once' join scaling (N distinct locs: N joins of N×N candidate pairs) ---");
    for &n in &[100usize, 200, 400, 800, 1600] {
        run_native(n);
    }
}

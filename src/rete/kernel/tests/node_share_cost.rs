//! Node-sharing cost — what two rules sharing a node actually save, decomposed.


use super::*;

/// Compile N node-share rules, seed M×2 facts, fire through the NATIVE path, return the census.
///
/// Fires `:wat::rete::fire-rules` — the public production verb, which delegates to
/// `fire-rules$native` (`wat/rete/oracle/fire.wat`) — so this is the same path the
/// grid harness times.
fn node_share_census(n: i64, m: i64) -> Vec<super::RoundCensus> {
    let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("node-share world should freeze");
    let src = format!(
        "(:wat::core::match (:wat::rete::fire-rules (:nsh::seed (:wat::core::match (:wat::rete::compile (:nsh::build-rules {n})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {m})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, census) = super::with_fire_census(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
            .value_owned()
    });
    census
}

/// Sum the tokens held by every beta node of a given kind in a census row.
fn tokens_of_kind(row: &super::RoundCensus, kind: &str) -> usize {
    row.beta_by_node
        .iter()
        .filter(|(_, k, _)| *k == kind)
        .map(|(_, _, t)| *t)
        .sum()
}

/// Tokens PRODUCED by nodes of `kind` across the whole fire, read off the per-round delta.
///
/// Since the beta-readers guard (`DESIGN-STONE-beta-is-written-only-for-readers`), a node
/// nothing reads has no materialised `wm.beta`, so `tokens_of_kind` reports 0 for it — a fact
/// about the guard, not about the join. `d_beta` still carries every token the node produced.
///
/// This is the SAME NUMBER the beta reading used to give, not a softer one: before the guard
/// both stores were fed by one unconditional statement pair, so summing the deltas across
/// rounds reconstructs exactly what the cumulative beta held.
fn produced_of_kind(census: &[super::RoundCensus], kind: &str) -> usize {
    census
        .iter()
        .flat_map(|r| r.d_beta_by_node.iter())
        .filter(|(_, k, _)| *k == kind)
        .map(|(_, _, t)| *t)
        .sum()
}

/// ★ STEP 0 of DESIGN-STONE-compiled-where — the DECOMPOSITION, before anything is built.
///
/// The counters (`node_share_filter_eval_census`, below) proved the MECHANISM exactly — AS IT
/// WAS ON 2026-08-01: 10,000 `Environment` builds and 10,000 key allocations per fire at
/// `[50 200]`, 98% of them for a predicate about to fail. They say NOTHING about the SHARE —
/// and a cost read is not a cost measured (`[[feedback_measure_the_decomposition_never_read_it]]`,
/// four wrong attributions in one session doing exactly that).
///
/// ⛔ THOSE TWO 10,000s ARE HISTORY, NOT THE PRESENT, and this sentence stood in the present
/// tense until 2026-09-02. Re-driven that day, the SAME census at `[50 200]` reads
/// `evals 0 · reuse 200 · passes 200 · envs 0 · keyallocs 0`: the where-tree proves every
/// candidate a pure comparison, so the fire builds NO environments, allocates NO keys, and
/// calls `exec_where` ZERO times. Every arm below is still scaled to `evals_per_round`
/// = N x tokens.len() = 10,000, which is that dead pre-where-tree count — see the ⛔ block
/// beside the reconstruction for what that costs.
///
/// Two things the `filter` phase's 89.5% actually contains, unsplit until now:
///   1. the per-TestNode `new_tokens = ts.clone()` (`:2701`) — on a SHARED-prefix axis every
///      one of the N TestNodes has the same parent, so the same 200-token vector is cloned N
///      times per round. NOT the predicate. (Task #50.)
///   2. the predicate itself, which splits again into the env build and the `eval_inner` walk.
///
/// So three arms, at what WAS one round's worth of work each so the numbers would land on the
/// same scale as the then-6.83 ms `filter` reading (that constant is gone; the phase is now read
/// live, and the scale premise no longer holds — the ⛔ block below the table says why),
/// **interleaved** — never blocks; a block-ordered A/B produced a
/// clean, disjoint, WRONG −7 ms on 2026-08-01 that a B-A-B drift check destroyed
/// (`[[feedback_a_benchmarks_shape_manufactures_its_result]]`).
///
/// Inputs are the PRODUCTION values, captured out of a real fire — not fabricated.
///
/// STOP-0 (in the stone): if `walk ≫ env`, the seam's gate (`env-builds → 0`) is a mechanism
/// win with no timing behind it and the stone's shape is wrong.
/// STOP-0b: if `clone` is comparable to `env + walk`, task #50 is a peer cost and cheaper.
// rune:complectens(inline-fixtures) — interleaved timing arms ARE the measurement fixture;
// extracting them would collapse the A–F reconstruction this probe exists to document.
#[test]
fn node_share_where_cost_decomposition() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: i64 = 50;
    const M: i64 = 200;
    const REPS: usize = 15;

    // ── capture the real inputs out of a real fire ────────────────────────────────────────
    let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("node-share world should freeze");
    let src = format!(
        "(:wat::core::match (:wat::rete::fire-rules (:nsh::seed (:wat::core::match (:wat::rete::compile (:nsh::build-rules {N})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {M})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
    let (_fired, sample) = super::with_where_sample(|| {
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at N={N} M={M}: {e:?}"))
            .value_owned()
    });
    let (expr, tokens) = sample.expect(
        "the fire never reached a TestNode, so nothing was captured — every number below \
             would be measuring a fabricated input, which is the one thing this probe exists to \
             avoid",
    );

    // ── non-vacuity, BEFORE any timing ────────────────────────────────────────────────────
    // A benchmark over an empty token vector or a zero-binding token would run fast and mean
    // nothing. Assert the shape production actually produced, and that the predicate really
    // evaluates (both verdicts must be reachable across the captured tokens: node-share's
    // `i == k mod N` passes exactly one k in N).
    let bindings_per_token = tokens[0].len();
    assert!(
        tokens.len() as i64 == M && bindings_per_token > 0,
        "captured {} tokens with {bindings_per_token} bindings each; expected {M} tokens \
             carrying at least one ?var — the capture did not see node-share's real parent delta",
        tokens.len(),
    );
    let verdicts: Vec<bool> = tokens
        .iter()
        .map(|t| {
            crate::rete::eval_test::eval_test_core(&expr, t, &Environment::new(), &world.symbols)
                .expect("the captured predicate must evaluate on the captured bindings")
        })
        .collect();
    let passes = verdicts.iter().filter(|b| **b).count();
    assert!(
        passes > 0 && passes < tokens.len(),
        "captured predicate returned the SAME verdict for all {} tokens ({passes} passes) — \
             a constant-folded predicate would make arm B's walk unrepresentative",
        tokens.len(),
    );

    // ── the three arms, one round's worth each, interleaved ───────────────────────────────
    // Arm A calls `build_test_env`, which IS the block `eval_test_core` runs — extracted, not
    // copied, so the arm cannot drift from the path it claims to measure.
    let evals_per_round = (N as usize) * tokens.len(); // 50 TestNodes x 200 tokens = 10,000
    let mut a_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut b_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut c_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut d_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut e_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut f_ns: Vec<u128> = Vec::with_capacity(REPS);
    let empty = Environment::new();
    let program = crate::rete::expr_ir::lower(&expr, &world.symbols)
        .unwrap_or_else(|e| panic!("captured where must lower: {e:?}"));
    let compiled_verdicts: Vec<bool> = tokens
        .iter()
        .map(|t| {
            crate::rete::expr_ir::exec_where(&program, t, &world.symbols, &program.span)
                .expect("compiled where must exec on captured bindings")
        })
        .collect();
    assert_eq!(
        compiled_verdicts, verdicts,
        "compiled exec_where must agree with eval_test_core on the captured tokens"
    );

    // Arm D's input — the SAME predicate with its two `?k` reads replaced by the literal they
    // would resolve to. Identical node count, identical operators, ZERO name lookups: the
    // identity control that separates "the interpreter's per-node dispatch" from "resolving a
    // ?var through the Environment" inside the walk.
    let const_src = "(:wat::core::= 7 (:wat::core::i64::- 9 \
               (:wat::core::i64::* (:wat::core::i64::/ 9 50) 50)))";
    let const_expr = crate::parse_one!(const_src).expect("parse the var-free control predicate");
    // The control must actually EVALUATE, or arm D measures an error path, not a walk.
    assert!(
        crate::rete::eval_test::eval_test_core(&const_expr, &tokens[0], &empty, &world.symbols,)
            .is_ok(),
        "the var-free control predicate did not evaluate — arm D would be timing a failure"
    );
    // Arm E's key — the one binding node-share's predicate reads.
    let k_key = tokens[0]
        .iter()
        .next()
        .map(|(k, _)| k.clone())
        .expect("the captured token carries at least one binding (asserted above)");
    for _ in 0..REPS {
        // A — the env build alone.
        let t = Instant::now();
        for i in 0..evals_per_round {
            let e = crate::rete::eval_test::build_test_env(&tokens[i % tokens.len()], &empty);
            black_box(&e);
        }
        a_ns.push(t.elapsed().as_nanos());

        // B — the env build PLUS the eval_inner walk (the whole of `eval_test_core`).
        let t = Instant::now();
        for i in 0..evals_per_round {
            let v = crate::rete::eval_test::eval_test_core(
                &expr,
                &tokens[i % tokens.len()],
                &empty,
                &world.symbols,
            );
            black_box(&v);
        }
        b_ns.push(t.elapsed().as_nanos());

        // C — the per-TestNode token clone: N clones of the parent's M-token delta.
        let t = Instant::now();
        for _ in 0..N {
            let c = tokens.clone();
            black_box(&c);
        }
        c_ns.push(t.elapsed().as_nanos());

        // D — env build + walk of the VAR-FREE control (same nodes, no name lookups).
        let t = Instant::now();
        for i in 0..evals_per_round {
            let v = crate::rete::eval_test::eval_test_core(
                &const_expr,
                &tokens[i % tokens.len()],
                &empty,
                &world.symbols,
            );
            black_box(&v);
        }
        d_ns.push(t.elapsed().as_nanos());

        // E — THE FLOOR. The same predicate as hand-written Rust against the same trie: one
        // binding read, then the arithmetic. This is what a perfectly compiled IR could reach,
        // so it BOUNDS the prize instead of leaving it to a prediction (and today's
        // predictions have a bad record — `[[feedback_measure_the_decomposition_never_read_it]]`).
        let t = Instant::now();
        for i in 0..evals_per_round {
            let bs = &tokens[i % tokens.len()];
            let v = match bs.get(&k_key) {
                Some(Value::i64(k)) => 7 == k - (k / 50) * 50,
                _ => false,
            };
            black_box(v);
        }
        e_ns.push(t.elapsed().as_nanos());

        // F — the native fire path: lower-once (outside this loop) + exec_where.
        let t = Instant::now();
        for i in 0..evals_per_round {
            let v = crate::rete::expr_ir::exec_where(
                &program,
                &tokens[i % tokens.len()],
                &world.symbols,
                &program.span,
            );
            black_box(&v);
        }
        f_ns.push(t.elapsed().as_nanos());
    }
    let median = |mut v: Vec<u128>| -> f64 {
        v.sort_unstable();
        v[v.len() / 2] as f64
    };
    let a = median(a_ns);
    let b = median(b_ns);
    let c = median(c_ns);
    let d = median(d_ns);
    let e = median(e_ns);
    let f = median(f_ns);
    let walk = b - a;
    let walk_novars = d - a;
    let lookups = walk - walk_novars;
    // ── the `filter` phase this reconstructs, READ LIVE ───────────────────────────────────
    // NEVER a constant. `FILTER_MS_MEASURED_IN_FIRE = 6.83` stood here from 2026-08-01 while
    // the compiled-where work drove the real phase to ~0.39 ms, and the check its own comment
    // declared ("if it does not land near it, the harness is measuring something the fire does
    // not do") was a `println!` with nothing behind it — printing 146% accounted, its own
    // stated failure condition, for a month.
    //
    // Same axis, same [N M] the arms are scaled to, same helper `node_share_fire_phase_census`
    // reads at its own `node_share_phase_census(50, 200)` call.
    let phase_rows = node_share_phase_census(N, M);
    let filter_ms = phase_rows
        .iter()
        .find(|(n, _, _)| *n == "filter")
        .map(|(_, ns, _)| *ns as f64 / 1e6)
        .unwrap_or_else(|| {
            panic!(
                "no `filter` row in the node-share phase census at [{N} {M}] — this axis is \
                 TestNode-heavy, so its absence means the fire never entered the filter pass \
                 and there is nothing for these arms to reconstruct. Rows: {:?}",
                phase_rows.iter().map(|(n, _, _)| *n).collect::<Vec<_>>()
            )
        });
    // The reconstruction is F + C, the NATIVE path: `fire/pass/filter.rs` builds from
    // `arm.compiled_wheres` and dispatches to `exec_where` (`fire/mod.rs:1996`); it never
    // calls `eval_test_core`. B stays in the table as the interpreter HEADROOM study (B-E),
    // which is honest and useful — but it is not the arm the fire runs, and `B/F` printed
    // three rows down has always said so.
    let reconstruction = (f + c) / 1e6;

    let table = format!(
            "\nSTEP 0 — where-predicate cost decomposition, node-share [{N} {M}], \
             ONE ROUND's worth per arm, {REPS} interleaved reps, medians\n\
             \x20 captured from a real fire: 1 predicate x {} tokens x {bindings_per_token} \
             binding(s); {passes}/{} pass\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 A  env build alone         ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 B  env build + walk        ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 C  token clone             ({:>6} x)  {:>8.3} ms\n\
             \x20 D  env + walk, VAR-FREE    ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 E  hand-written Rust       ({evals_per_round:>6} x)  {:>8.3} ms   <- THE FLOOR\n\
             \x20 F  compiled exec_where     ({evals_per_round:>6} x)  {:>8.3} ms   {:>6.1} ns/eval\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 the walk        B-A   {:>8.3} ms  {:>5.1}% of B   {:>6.1} ns/eval\n\
             \x20   of which:\n\
             \x20     ?var lookup (B-A)-(D-A)  {:>8.3} ms  {:>5.1}% of the walk\n\
             \x20     node dispatch    D-A     {:>8.3} ms  {:>5.1}% of the walk\n\
             \x20 the env build   A     {:>8.3} ms  {:>5.1}% of B   {:>6.1} ns/eval\n\
             \x20 the token clone C     {:>8.3} ms\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 RECONSTRUCTION  F+C = {:>6.3} ms  vs a LIVE `filter` of \
             {filter_ms:>6.3} ms  ({:>4.0}% accounted)\n\
             \x20 HEADROOM        B-E = {:>6.3} ms is what a PERFECT compile could remove\n\
             \x20 COMPILED vs B   B/F = {:>5.2}x    F-E leftover {:>6.1} ns/eval\n",
            tokens.len(),
            tokens.len(),
            a / 1e6,
            b / 1e6,
            N,
            c / 1e6,
            d / 1e6,
            e / 1e6,
            f / 1e6,
            f / evals_per_round as f64,
            walk / 1e6,
            100.0 * walk / b,
            walk / evals_per_round as f64,
            lookups / 1e6,
            100.0 * lookups / walk,
            walk_novars / 1e6,
            100.0 * walk_novars / walk,
            a / 1e6,
            100.0 * a / b,
            a / evals_per_round as f64,
            c / 1e6,
            reconstruction,
            100.0 * reconstruction / filter_ms,
            (b - e) / 1e6,
            b / f,
            (f - e) / evals_per_round as f64,
        );
    println!("{table}");

    // Non-vacuity on the LIVE READ: a zero `filter` makes every ratio above a division by
    // nothing, and the `% accounted` column would render as `inf` rather than as a fault.
    assert!(
        filter_ms > 0.0,
        "the live `filter` phase read 0 ns at [{N} {M}] — the census never entered the \
             filter pass, so the reconstruction above is measured against nothing{table}"
    );

    // ⛔ THE DECLARED CHECK IS **NOT** ASSERTED HERE, AND THAT IS THE FINDING — not an omission.
    //
    // The old comment declared it: "if B + C does not land near it, the harness is measuring
    // something the fire does not do." Reading the phase LIVE and reconstructing from the
    // NATIVE arm (both fixed above) makes that check runnable for the first time, and it FAILS
    // — structurally, not noisily. Six consecutive runs, 2026-09-02, this box, release:
    //
    //     684%  693%  734%  723%  686%  698%   accounted  (F+C vs the live `filter`)
    //
    // A 7% spread across six samples: this is not the ~16% run-to-run noise, it is a stable
    // over-count. NO honest band admits 7x, and a band widened to admit it would re-create the
    // very defect this instrument was cleaned to remove.
    //
    // THE MECHANISM, measured — `node_share_filter_eval_census` at [50 200] on this same tree:
    //
    //     rules items |  evals  reuse  passes | envs  keyallocs
    //        50   200 |      0    200     200 |    0          0
    //
    // The fire calls `exec_where` **ZERO** times. `dispatch_where_tests` (`fire/mod.rs:2012`)
    // finds every candidate `proven` AND `is_pure_cmp` (`:2039`), takes the reuse branch
    // (`:2040`, `filter:test-reuse`), and skips the eval entirely. Arm F is scaled to `evals_per_round` = N x tokens.len() = 10,000 — the
    // PRE-where-tree count. So "ONE ROUND'S WORTH" is itself stale: a round's worth of
    // `exec_where` is now 0, not 10,000, and no rescaling of F rescues the reconstruction,
    // because the correct scale drives F's contribution to zero. C alone is then ~0.13 ms
    // against a ~0.39 ms phase — ~34%, still not a reconstruction.
    //
    // What the remaining ~66% is, no arm here measures: the per-token `where_tree.candidates`
    // walk, the `bind_view`, the two per-token `HashSet` builds (`proven`/`maybe`), and the
    // `d_beta` pushes. Those are the filter phase today. Adding arms for them is a strike of
    // its own; asserting a number over arms that do not cover them would not be one.
    //
    // Until an arm set covers the phase, the honest guards are the non-vacuity ones: the live
    // read found its row and is non-zero (above), and the instrument is not optimised away
    // (below). Both carry the whole table so a red arrives with its own evidence.

    // Non-vacuity on the INSTRUMENT itself: a zero reading means the optimiser removed the
    // arm, and every share above would be an artifact.
    assert!(
        a > 0.0 && b > 0.0 && c > 0.0 && d > 0.0 && e > 0.0 && f > 0.0 && b > a && b > e,
        "an arm measured zero, or the orderings that MUST hold do not — the loop was \
             optimised away and the shares above are artifacts \
             (A={a}ns B={b}ns C={c}ns D={d}ns E={e}ns){table}"
    );
}

/// (b) landed — this census now gates the index, not the pre-index waste.
///
/// Node-share: M tokens, N rules, one shared dim `(= i (k rem n))`. Linear eval is
/// M×N with ~98% waste. The where-tree must cut that to ~1 eval/token so
/// `evals ≈ passes ≈ M`. If `evals` climbs back toward M×N the tree stopped
/// discriminating (analysis miss, or dispatch still walking every sibling).
#[test]
fn node_share_filter_eval_census() {
    let mut table = String::from(
        "\nnode-share — `where` evaluations vs passes (the (b) WhereDiscNode gate)\n\
             \x20 rules  items |    evals    reuse    passes   wasted  waste%   evals/token\n\
             \x20 -----------------------------------------------------------------------------\n",
    );
    let mut worst_waste = 0.0f64;
    for (n, m) in [(10i64, 200i64), (25, 200), (50, 200)] {
        let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("node-share world should freeze");
        let src = format!(
                "(:wat::core::match (:wat::rete::fire-rules (:nsh::seed (:wat::core::match (:wat::rete::compile (:nsh::build-rules {n})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {m})) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
            );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, rows) = super::with_count_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
                .value_owned()
        });
        let get = |k: &str| {
            rows.iter()
                .find(|(a, _)| *a == k)
                .map(|(_, v)| *v)
                .unwrap_or(0)
        };
        let evals = get("filter:test-evals");
        let reuse = get("filter:test-reuse");
        let passes = get("filter:test-pass");
        let envs = get("filter:test-env-builds");
        let keys = get("filter:test-key-alloc");
        // Non-vacuity FIRST: a fire that never reached a TestNode would report 0 evals and
        // 0 passes, and a "0% waste" reading would look like the best possible news.
        // Proven `(= dim lit)` or range skip `exec_where` (`filter:test-reuse`).
        assert!(
                evals > 0 || reuse > 0,
                "node-share N={n} M={m} recorded ZERO `where` evaluations and ZERO reuse — the \
                 filter pass never ran, so any ratio taken from this is an artifact, not a measurement"
            );
        assert!(
            passes > 0,
            "node-share N={n} M={m} recorded ZERO passes — the tree pruned every TestNode \
                 (under-approx) or nothing fired"
        );
        let wasted = evals.saturating_sub(passes);
        let waste_pct = if evals == 0 {
            0.0
        } else {
            100.0 * wasted as f64 / evals as f64
        };
        worst_waste = worst_waste.max(waste_pct);
        table.push_str(&format!(
                "  {n:>5}  {m:>5} | {evals:>8}  {reuse:>8}  {passes:>8} {wasted:>8}  {waste_pct:>5.1}%  \
                 {:>10.2}  | envs {envs:>7}  keyallocs {keys:>7}\n",
                evals as f64 / m as f64,
            ));
        // ~1 candidate per token. Slack of 2× covers a second filter pass / mild over-approx.
        // Linear scan is N×M (10_000 at [50 200]) — that must not pass.
        assert!(
            evals <= passes.saturating_mul(2),
            "where-tree should eval about as many predicates as pass (one matching residue \
                 per token). N={n} M={m} evals={evals} passes={passes}.{table}"
        );
        assert!(
            evals <= (m as u64).saturating_mul(4),
            "where-tree evals should sit near M (one token → one residue), not N×M. \
                 N={n} M={m} evals={evals}.{table}"
        );
    }
    println!("{table}");
    assert!(
        worst_waste < 50.0,
        "(b) must collapse wasted `where` evals (a token tested by every rule, matching \
             at most one) — peak waste {worst_waste:.1}%. If this rose, dispatch is linear \
             again or DimKey failed to unify the node-share residue.{table}"
    );
}

/// The node-share phase table, at the GRID's own ladder ([10|25|50] x 200).
#[test]
fn node_share_fire_phase_census() {
    const TOP: [&str; 4] = [
        "IN: to_transient",
        "SETUP: indexes",
        "ROUND LOOP",
        "OUT: to_persistent",
    ];
    // Floor only — the table discovers the rest. node-share has no accumulate/filter, so its
    // required set is deliberately smaller than accum's; asserting accum's list here would
    // fail on phases this axis never reaches.
    const REQUIRED: [&str; 6] = [
        "SETUP: indexes",
        "ROUND LOOP",
        "alpha",
        "root-join",
        "hash-join",
        "production",
    ];
    let table = render_phase_table(
        "node-share fire",
        &[(10, 200), (25, 200), (50, 200)],
        &TOP,
        &REQUIRED,
        |_n, m| m * 2, // M A-facts + M B-facts
        node_share_phase_census,
    );
    println!("{table}");

    // Assert on the DATA, not the rendered text. A `table.contains("ROUND LOOP")` passes on a
    // table whose every number is zero. Non-vacuity: the axis fired, and `filter` still
    // recorded (this world has TestNodes). WhereDiscNode already killed filter-dominates
    // (89.5% on 2026-08-01); do not wall-gate that share.
    let rows = node_share_phase_census(50, 200);
    let ns_of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _, _)| *n == name)
            .map(|(_, ns, _)| *ns)
            .unwrap_or(0)
    };
    let round_loop = ns_of("ROUND LOOP");
    let filter = ns_of("filter");
    assert!(
        round_loop > 0,
        "ROUND LOOP recorded 0ns at 50/200 — the fire never ran, and a\n\
                                 table of zeroes would still have rendered every row:\n{table}"
    );
    // ⛔ The guards here are LIVENESS — `probare` classed this test hollow. The node-share axis
    // is TestNode-heavy, so `filter` must appear alongside the join phases.
    super::assert_phases_present(
        rows.iter().map(|r| r.0),
        &["alpha", "root-join", "hash-join", "production", "filter"],
        &table,
    );
    assert!(
        filter > 0,
        "filter recorded 0ns at 50/200 — this axis has TestNodes:\n{table}"
    );
}

/// A8 — census the native fire path as rule-count N grows against a fixed fact set.
///
/// M is deliberately tiny (50 of each type). The axis blew a machine's RAM at N=20/M=500;
/// nothing here can approach that, and the growth SHAPE is what the diagnosis needs, not the
/// magnitude. Prints the full per-N table (`--no-capture` to read it) and asserts the
/// invariants that must hold for the shared-prefix story to be true at fire time.
///
/// What would turn this red — the R59 question, answered before the assertions were written:
///   (a) the instrument recording nothing (an unarmed or never-entered loop),
///   (b) the derived-fact count drifting from M (the axis's documented N-invariance breaking),
///   (c) the shared HashJoin's token count growing with N — which IS the fire-path smoking gun:
///       one compiled join node re-materialising its tokens per rule.
#[test]
fn a8_node_share_fire_census() {
    const M: i64 = 50;
    const NS: [i64; 4] = [1, 2, 4, 8];

    let mut table = String::new();
    table.push_str(&format!(
        "\nA8 node-share — native fire census (M={M} A-facts + {M} B-facts)\n\
             \n  N | edges | rnds | dIn | aNodes aEls | bNodes bToks bMatches | dbNodes dbToks \
             | lIdx rIdx | prod seen | HashJoin RootJoin Test\n"
    ));

    let mut hash_join_tokens: Vec<(i64, usize)> = Vec::new();

    for n in NS {
        let census = node_share_census(n, M);
        assert!(
            !census.is_empty(),
            "A8 census recorded ZERO rounds at N={n} — the instrument never fired, so any \
                 reading taken from it would be an artifact, not a measurement"
        );

        // The final round carries the cumulative totals for the whole fire.
        let last = census.last().expect("census is non-empty");
        // PRODUCED, not HELD. Post-guard a terminal HashJoinNode deliberately materialises no
        // beta, so `tokens_of_kind(last, "HashJoin")` would read 0 for every N and the sharing
        // assertion below would be vacuously true — the gate would keep its green and stop
        // meaning anything. The delta carries the same tokens (see `produced_of_kind`), and it
        // is the better witness for this claim anyway: the defect under test is the join
        // RE-RUNNING per rule, which shows up as tokens produced, not tokens stored.
        let hj = produced_of_kind(&census, "HashJoin");
        let rj = tokens_of_kind(last, "RootJoin");
        let tn = tokens_of_kind(last, "Test");

        table.push_str(&format!(
            "  {:<2}| {:<6}| {:<5}| {:<4}| {:<7}{:<5}| {:<7}{:<6}{:<10}| {:<8}{:<7}| \
                 {:<5}{:<5}| {:<5}{:<5}| {:<9}{:<9}{}\n",
            n,
            last.network_edges,
            census.len(),
            last.delta_facts_in,
            last.alpha_nodes,
            last.alpha_elements,
            last.beta_nodes,
            last.beta_tokens,
            last.beta_token_matches,
            last.d_beta_nodes,
            last.d_beta_tokens,
            last.left_idx_tokens,
            last.right_idx_elements,
            last.production_facts,
            last.seen_facts,
            hj,
            rj,
            tn,
        ));

        // Per-round detail: the fixpoint's shape over time. A structure that grows across
        // rounds reads differently from one that is over-allocated in a single round, and the
        // summary row above (cumulative totals) cannot tell them apart.
        for row in &census {
            table.push_str(&format!(
                "     |- round {:<2} dIn={:<5} beta={:<6} dBeta={:<6} matches={:<8} prod={}\n",
                row.round,
                row.delta_facts_in,
                row.beta_tokens,
                row.d_beta_tokens,
                row.beta_token_matches,
                row.production_facts,
            ));
        }

        // (b) The axis's own N-invariance: every k in [0, M) satisfies exactly one rule, so the
        // derived set is {Out(k)} of size M no matter how many rules split it.
        assert_eq!(
            last.production_facts, M as usize,
            "A8 derived-fact count must be N-invariant (M={M}), got {} at N={n}{table}",
            last.production_facts
        );

        hash_join_tokens.push((n, hj));
    }

    println!("{table}");

    // (c) Fire-time sharing: the ONE compiled HashJoinNode must PRODUCE the same token set no
    // matter how many rules hang off it. If this grows with N, the fire path is re-doing the
    // join per rule — the shared network collapsing back into N copies at run time, which is
    // exactly the mechanism the >4 GiB blow-up would need.
    //
    // Reworded from "must HOLD" on 2026-08-01: the beta-readers guard stopped materialising a
    // terminal join's `wm.beta`, so "holds" became vacuous by design. The quantity is
    // unchanged — before the guard, beta and the delta were fed by one unconditional
    // statement pair, so the summed delta IS what beta held — but the gate now says what it
    // actually proves rather than keeping a name the code had made false.
    let (_, baseline) = hash_join_tokens[0];
    for &(n, tokens) in &hash_join_tokens {
        assert_eq!(
            tokens, baseline,
            "A8 fire-time sharing broken: the shared HashJoinNode produced {tokens} tokens at \
                 N={n} but {baseline} at N={}. One compiled join node is materialising per-rule \
                 token sets — the fire-path defect the compiler census (4 + 2N nodes) ruled out at \
                 compile time.{table}",
            hash_join_tokens[0].0
        );
    }
    assert!(
        baseline > 0,
        "A8 census read 0 HashJoin tokens — the join never ran, so the sharing assertion above \
             would pass vacuously.{table}"
    );
}

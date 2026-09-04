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
/// calls `exec_where` ZERO times.
///
/// ⛔ THE ARMS COME IN TWO LADDERS, AND THEY MEASURE DIFFERENT BRANCHES.
///
///   A/B/D/E/F — the `exec_where` branch, scaled to `evals_per_round` = N x tokens.len()
///   = 10,000. THAT IS THE DEAD PRE-WHERE-TREE COUNT: the fire's `exec_where` calls on this
///   axis number ZERO, so this ladder measures a branch `dispatch_where_tests` does not enter
///   here. It is KEPT because `B-E` bounds what a perfect compile could remove and that is a
///   real study; it is NOT summed into a reconstruction of the `filter` phase. It was, until
///   2026-09-04, and `F+C` printed ~670% accounted.
///
///   G/H/I/J/K/L — the branch the fire DOES take, cumulative, ONE FIRE's worth per arm, with
///   every scale read off the fire's own counters (`filter:test-reuse`, `dbeta:alloc`,
///   `dbeta:tokens`) and asserted against a replay before any clock starts. `bind_view` ->
///   `where_tree.candidates` -> the two `HashSet` builds -> the `tid` loop -> the `d_beta`
///   pushes, plus L for the `d_beta_from_parents` gather that feeds them.
///
/// The two things the `filter` phase's 89.5% was once said to contain — the per-TestNode token
/// clone and the predicate — ARE NOT WHAT IS IN IT. Measured 2026-09-04: the predicate is not
/// evaluated at all, the clone is ~8 us of a ~0.39 ms phase, and ~80% of the phase is the
/// `tid` loop: 10,000 (token, tid) pairs, three `HashSet<i64>` probes each, 9,800 of them
/// reaching `continue`. Arm C times a `Vec<PMap>` clone; the fire clones `Vec<Token>`.
///
/// Every arm is **interleaved** — never blocks; a block-ordered A/B produced a
/// clean, disjoint, WRONG −7 ms on 2026-08-01 that a B-A-B drift check destroyed
/// (`[[feedback_a_benchmarks_shape_manufactures_its_result]]`).
///
/// Inputs are the PRODUCTION values, captured out of a real fire — not fabricated.
///
/// STOP-0 (in the stone): if `walk ≫ env`, the seam's gate (`env-builds → 0`) is a mechanism
/// win with no timing behind it and the stone's shape is wrong.
/// STOP-0b: if `clone` is comparable to `env + walk`, task #50 is a peer cost and cheaper.
// rune:complectens(inline-fixtures) — interleaved timing arms ARE the measurement fixture;
// extracting them would collapse the A–L reconstruction this probe exists to document.
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
    // The taken branch's arms — cumulative, ONE FIRE's worth each (the phase they are divided
    // into is also one fire's worth). NOT scaled to `evals_per_round`.
    let mut g_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut h_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut i_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut j_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut k_ns: Vec<u128> = Vec::with_capacity(REPS);
    let mut l_ns: Vec<u128> = Vec::with_capacity(REPS);
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
    // ── THE BRANCH THE FIRE ACTUALLY TAKES — its real inputs, built by production code ────
    //
    // ⛔ A/B/D/E/F ABOVE MEASURE `exec_where`, AND THIS AXIS CALLS IT ZERO TIMES.
    // `dispatch_where_tests` (`fire/mod.rs`) finds every candidate `proven` AND `is_pure_cmp`,
    // takes the reuse branch, and never reaches the `else` those five arms live in. They stay
    // because `B-E` is a real compile-headroom study; they are NOT summed into this phase.
    //
    // The taken branch, per token: `bind_view` -> `where_tree.candidates` -> two `HashSet`
    // builds -> the `tid` loop -> the `d_beta` push. Arms G..K below are that branch, cumulative,
    // so each row is a delta from the one above; arm L is the parent-delta gather that feeds it.
    //
    // NOTHING HERE IS FABRICATED AND NOTHING HERE EDITS THE FIRE. The `where_tree`, the sibling
    // `tid` group and the `beta_readers` set come from `rete_arm_get_or_build` — the same
    // constructor `fire/delta.rs` calls — over the same compiled network. The tokens are the
    // captured production bindings re-interned through `span_from_pairs`, the same door the
    // filter pass interns through.
    let staged = format!(
        "(:nsh::seed (:wat::core::match (:wat::rete::compile (:nsh::build-rules {N})) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None))) {M})"
    );
    let staged_ast = crate::parse_one!(staged.as_str()).expect("parse the staged compile+seed");
    let session = eval_in_frozen(&staged_ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile+seed raised at N={N} M={M}: {e:?}"))
        .value_owned();
    let mut wm =
        super::to_transient(&session).expect("to_transient of the seeded node-share session");
    let arm = super::rete_arm_get_or_build(&wm.network, &wm.rules, world.symbols())
        .expect("arm for the node-share network");
    // The sibling group `filter_pass` hands `dispatch_where_tests`: TestNodes sharing a parent
    // set dispatch ONCE through the interned where-tree (`arm.test_sibs`), and `tests_done`
    // stops the rest. So the per-token half of the branch runs once per token per GROUP, not
    // once per TestNode — which is exactly the scale the old ladder got wrong.
    let mut test_ids: Vec<i64> = arm.compiled_wheres.keys().copied().collect();
    test_ids.sort_unstable();
    let first_test = *test_ids
        .first()
        .expect("the node-share network has TestNodes — `compiled_wheres` cannot be empty here");
    let tids: Vec<i64> = arm
        .test_sibs
        .get(&first_test)
        .cloned()
        .unwrap_or_else(|| vec![first_test]);
    // Re-intern the captured production bindings into this session's bind pool, so `bind_view`
    // reads exactly what the fire's `bind_view` reads: ids into `bind_keys`/`bind_vals`.
    let toks: Vec<super::Token> = tokens
        .iter()
        .map(|pm| super::Token {
            matches: super::empty_span(),
            binds: super::span_from_pairs(
                &mut crate::rete::compiled_cond::BindIntern {
                    keys: &mut wm.bind_keys,
                    vals: &mut wm.bind_vals,
                    ids: &mut wm.bind_val_ids,
                    pool: &mut wm.bind_pool,
                },
                pm.iter().map(|(k, v)| (k.clone(), v.clone())),
            ),
        })
        .collect();
    let cand_span = crate::rust_caller_span!();

    // ── THE SCALE IS MEASURED, NEVER ASSUMED ──────────────────────────────────────────────
    // The old ladder scaled every arm to `evals_per_round` = N x tokens.len() = 10,000 — a
    // PRE-where-tree eval count that is now 0. These arms are scaled from the fire's OWN
    // counters, read off the same axis, so a scale that drifts REDs instead of rotting.
    let count_src = format!(
        "(:wat::core::match (:wat::rete::fire-rules {staged}) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))"
    );
    let count_ast = crate::parse_one!(count_src.as_str()).expect("parse the counted fire driver");
    let (_fired2, count_rows) = super::with_count_census(|| {
        eval_in_frozen(&count_ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("counted fire raised at N={N} M={M}: {e:?}"))
            .value_owned()
    });
    let counted = |k: &str| -> u64 {
        count_rows
            .iter()
            .find(|(a, _)| *a == k)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    };
    let fire_reuse = counted("filter:test-reuse");
    let fire_evals = counted("filter:test-evals");
    let fire_gathers = counted("dbeta:alloc");
    let fire_gather_tokens = counted("dbeta:tokens");

    // Non-vacuity and scale, BEFORE any timing.
    //
    // (i) THE BRANCH. `evals == 0` is not a threshold and not a tolerance — it is the fact the
    // whole ladder below rests on. If it ever goes non-zero the fire has started calling
    // `exec_where` on this axis, arms A/B/D/E/F become relevant again, and the reconstruction
    // must be re-derived rather than re-tuned.
    assert!(
        fire_reuse > 0 && fire_evals == 0,
        "the node-share fire at [{N} {M}] recorded reuse={fire_reuse} evals={fire_evals}. \
         Arms G..K reconstruct the REUSE branch of `dispatch_where_tests` only. A non-zero \
         `filter:test-evals` means the fire now calls `exec_where` here, so the branch these \
         arms measure is no longer the branch the fire takes — re-derive the reconstruction, \
         do not widen it"
    );
    // (ii) THE GROUP. One dispatch per sibling group, so the per-token half runs once per token.
    assert!(
        tids.len() == N as usize && tids.iter().all(|t| arm.where_tree.covers(*t)),
        "expected ONE sibling group of {N} covered TestNodes (so `filter_pass` dispatches once \
         and the per-token arms are scaled per token, not per TestNode); got {} tids, {} covered",
        tids.len(),
        tids.iter().filter(|t| arm.where_tree.covers(**t)).count(),
    );
    // (iii) THE GATHER. Arm L's replay count and width come from the census, not from N.
    assert!(
        fire_gathers > 0 && fire_gather_tokens == fire_gathers * tokens.len() as u64,
        "`d_beta_from_parents` reported {fire_gathers} non-empty gathers carrying \
         {fire_gather_tokens} tokens; arm L replays {fire_gathers} clones of a \
         {}-token vector and that only reconstructs the gather if the width divides out",
        tokens.len(),
    );
    // (iv) THE REPLICA AGREES WITH THE FIRE. Run the taken branch once, off the clock, and
    // count the reuse-arm hits: they must equal what the WHOLE fire counted. This is what
    // makes "one dispatch of {M} tokens x {N} tids" a measurement rather than a reading of
    // the source ([[a-reading-cannot-see-an-execution-defect]]).
    let mut replica_reuse = 0u64;
    let mut replica_eval_arm = 0u64;
    for tok in &toks {
        let binds = super::bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
        let cands = arm.where_tree.candidates(&binds, &cand_span);
        let proven: std::collections::HashSet<i64> = cands.proven.into_iter().collect();
        let maybe: std::collections::HashSet<i64> = cands.maybe.into_iter().collect();
        for &tid in &tids {
            if arm.where_tree.covers(tid) && !proven.contains(&tid) && !maybe.contains(&tid) {
                continue;
            }
            if proven.contains(&tid) && arm.where_tree.is_pure_cmp(tid) {
                replica_reuse += 1;
                continue;
            }
            replica_eval_arm += 1;
        }
    }
    assert!(
        replica_reuse == fire_reuse && replica_eval_arm == fire_evals,
        "the replica took reuse={replica_reuse} eval={replica_eval_arm} over one dispatch of \
         {} tokens x {} tids; the FIRE counted reuse={fire_reuse} evals={fire_evals}. These \
         must match, or arms G..K are running a different amount of work than the phase they \
         are about to be divided into",
        toks.len(),
        tids.len(),
    );
    // The `wm.beta` write inside the reuse arm is guarded by `beta_readers`, and on this axis
    // no TestNode is read — so that push is not part of what arms J/K replay. Stated, not
    // assumed: a beta-reading TestNode would put a push in the branch these arms do not have.
    let beta_read_tids = tids.iter().filter(|t| arm.beta_readers.contains(t)).count();
    assert_eq!(
        beta_read_tids, 0,
        "{beta_read_tids} of the dispatched TestNodes are beta_readers, so the reuse arm also \
         pushes into `wm.beta` — a write arms J/K do not replay. Add it before dividing"
    );

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

        // ── G..K — the branch the fire TAKES, cumulative ───────────────────────────────────
        // G — `bind_view` alone, once per token.
        let t = Instant::now();
        for tok in &toks {
            let binds = super::bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
            black_box(&binds);
        }
        g_ns.push(t.elapsed().as_nanos());

        // H — G + the where-tree walk.
        let t = Instant::now();
        for tok in &toks {
            let binds = super::bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
            let cands = arm.where_tree.candidates(&binds, &cand_span);
            black_box(&cands);
        }
        h_ns.push(t.elapsed().as_nanos());

        // I — H + the two per-token `HashSet` builds (`proven` / `maybe`).
        let t = Instant::now();
        for tok in &toks {
            let binds = super::bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
            let cands = arm.where_tree.candidates(&binds, &cand_span);
            let proven: std::collections::HashSet<i64> = cands.proven.into_iter().collect();
            let maybe: std::collections::HashSet<i64> = cands.maybe.into_iter().collect();
            black_box(&proven);
            black_box(&maybe);
        }
        i_ns.push(t.elapsed().as_nanos());

        // J — I + the `tid` loop: the three set probes per (token, tid) pair, and the reuse
        // arm's `is_pure_cmp` + census bumps + `beta_readers` probe. NO push.
        let t = Instant::now();
        for tok in &toks {
            let binds = super::bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
            let cands = arm.where_tree.candidates(&binds, &cand_span);
            let proven: std::collections::HashSet<i64> = cands.proven.into_iter().collect();
            let maybe: std::collections::HashSet<i64> = cands.maybe.into_iter().collect();
            for &tid in &tids {
                if arm.where_tree.covers(tid) && !proven.contains(&tid) && !maybe.contains(&tid) {
                    continue;
                }
                if proven.contains(&tid) && arm.where_tree.is_pure_cmp(tid) {
                    super::census_count("filter:test-reuse");
                    super::census_count("filter:test-pass");
                    black_box(arm.beta_readers.contains(&tid));
                    continue;
                }
                black_box(tid);
            }
        }
        j_ns.push(t.elapsed().as_nanos());

        // K — J + the `d_beta` push. THE WHOLE TAKEN BRANCH of `dispatch_where_tests`.
        // The map is built empty (as the fire's round-local `d_beta` is for these ids), so
        // `or_default()` allocates its Vec here exactly as it does in the fire.
        let mut d_beta: super::BetaMemory = super::BetaMemory::default();
        let t = Instant::now();
        for tok in &toks {
            let binds = super::bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
            let cands = arm.where_tree.candidates(&binds, &cand_span);
            let proven: std::collections::HashSet<i64> = cands.proven.into_iter().collect();
            let maybe: std::collections::HashSet<i64> = cands.maybe.into_iter().collect();
            for &tid in &tids {
                if arm.where_tree.covers(tid) && !proven.contains(&tid) && !maybe.contains(&tid) {
                    continue;
                }
                if proven.contains(&tid) && arm.where_tree.is_pure_cmp(tid) {
                    super::census_count("filter:test-reuse");
                    super::census_count("filter:test-pass");
                    black_box(arm.beta_readers.contains(&tid));
                    d_beta.entry(tid).or_default().push(*tok);
                    continue;
                }
                black_box(tid);
            }
        }
        k_ns.push(t.elapsed().as_nanos());
        black_box(&d_beta);
        drop(d_beta);

        // L — the parent-delta gather that FEEDS the branch: `d_beta_from_parents` extends a
        // fresh `Vec<Token>` with the parent's delta, once per TestNode. `fire_gathers` and the
        // width both come off the fire's own counters (asserted above), not from N.
        let t = Instant::now();
        for _ in 0..fire_gathers {
            let mut out: Vec<super::Token> = Vec::new();
            out.extend(toks.iter().cloned());
            black_box(&out);
        }
        l_ns.push(t.elapsed().as_nanos());
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
    // The taken branch, paired per rep BEFORE the medians consume the vectors — K and L are
    // timed in the same rep, so their sum has a per-rep spread and not a spread of medians
    // ([[six-samples-or-no-number]]: a band is what makes a coverage figure readable).
    let mut kl_ns: Vec<u128> = k_ns.iter().zip(l_ns.iter()).map(|(x, y)| x + y).collect();
    kl_ns.sort_unstable();
    // ⛔ NOT min/max. The extremes of 15 reps are outlier-driven — driven 2026-09-04, a single
    // scheduling stall put one rep at ~0.65 ms against a ~0.35 ms median and rendered the band
    // as `84.9-153.3%`, which says nothing about the coverage figure's uncertainty. The band
    // below is the INTERQUARTILE spread of K+L (p25/p75 of the reps) against the min/max of the
    // phase reads, and the table labels it as exactly that.
    let kl_lo = kl_ns[kl_ns.len() / 4] as f64;
    let kl_hi = kl_ns[kl_ns.len() - 1 - kl_ns.len() / 4] as f64;
    let g = median(g_ns);
    let h = median(h_ns);
    let i_arm = median(i_ns);
    let j = median(j_ns);
    let k = median(k_ns);
    let l = median(l_ns);
    let kl = kl_ns[kl_ns.len() / 2] as f64;
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
    //
    // READ FIVE TIMES, not once. The denominator of every share below is this number, and one
    // reading of it cannot show its own spread — the defect that put a single 656% against a
    // six-sample 684-734 band and made a 4% difference look like a movement.
    const PHASE_READS: usize = 7;
    let mut filter_samples: Vec<f64> = Vec::with_capacity(PHASE_READS);
    // The phase reading CONTAINS ITS OWN INSTRUMENT: `d_beta_from_parents` opens and closes a
    // `dbeta:gather` mark on every call, inside the `filter` mark. A pair is ~75-80 ns
    // (`census.rs`, measured 2026-08-01), so this count is a NAMED part of the remainder rather
    // than an unexplained one.
    let mut gather_pairs = 0u64;
    for _ in 0..PHASE_READS {
        let phase_rows = node_share_phase_census(N, M);
        gather_pairs = phase_rows
            .iter()
            .find(|(n, _, _)| n.trim_start().trim_start_matches(['\u{251c}', '\u{2514}']).trim() == "dbeta:gather")
            .map(|(_, _, k)| *k)
            .unwrap_or(0);
        filter_samples.push(
            phase_rows
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
                }),
        );
    }
    filter_samples.sort_by(|x, y| x.partial_cmp(y).expect("phase readings are finite"));
    let filter_ms = filter_samples[PHASE_READS / 2];
    // The RAW extremes are printed (nothing is hidden), but the band is computed from the
    // TRIMMED range — one reading dropped from each end. A single scheduling stall in one of
    // seven fires is not information about coverage: driven 2026-09-04 it put one `filter` read
    // at 0.687 ms against a 0.395 ms median and dragged the printed band's floor to 49.8%.
    let filter_raw_lo = filter_samples[0];
    let filter_raw_hi = filter_samples[PHASE_READS - 1];
    let filter_lo = filter_samples[1];
    let filter_hi = filter_samples[PHASE_READS - 2];
    // ⛔ `F+C` IS NOT A RECONSTRUCTION OF THIS PHASE AND IS NO LONGER PRESENTED AS ONE.
    // It is kept, printed, and labelled — deleting it would discard a real measurement — but
    // it sums five arms in the `exec_where` branch (entered ZERO times here, asserted above)
    // with a clone of the wrong type (see arm C's note). The reconstruction is `K+L`.
    let not_a_reconstruction = (f + c) / 1e6;
    let taken = kl / 1e6;
    let accounted = 100.0 * taken / filter_ms;
    // The band pairs the WORST arm reading with the BEST phase reading and vice versa, so it
    // brackets the coverage figure rather than flattering it.
    let accounted_lo = 100.0 * (kl_lo / 1e6) / filter_hi;
    let accounted_hi = 100.0 * (kl_hi / 1e6) / filter_lo;

    let a_ms = a / 1e6;
    let b_ms = b / 1e6;
    let c_ms = c / 1e6;
    let d_ms = d / 1e6;
    let e_ms = e / 1e6;
    let f_ms = f / 1e6;
    // The taken branch is a ~0.36 ms phase read in MICROSECONDS: at ms precision arm G renders
    // as `0.000` and its rung would be unreadable.
    let g_us = g / 1e3;
    let h_us = h / 1e3;
    let i_us = i_arm / 1e3;
    let j_us = j / 1e3;
    let k_us = k / 1e3;
    let l_us = l / 1e3;
    let dh = (h - g) / 1e3;
    let di = (i_arm - h) / 1e3;
    let dj = (j - i_arm) / 1e3;
    let dk = (k - j) / 1e3;
    // Rung shares of the whole taken branch, so the table apportions instead of only listing.
    let dj_pct = 100.0 * (j - i_arm) / k;
    let instrument_us = gather_pairs as f64 * 0.078;
    let f_per = f / evals_per_round as f64;
    let walk_ms = walk / 1e6;
    let walk_pct = 100.0 * walk / b;
    let walk_per = walk / evals_per_round as f64;
    let look_ms = lookups / 1e6;
    let look_pct = 100.0 * lookups / walk;
    let nov_ms = walk_novars / 1e6;
    let nov_pct = 100.0 * walk_novars / walk;
    let a_pct = 100.0 * a / b;
    let a_per = a / evals_per_round as f64;
    let head_ms = (b - e) / 1e6;
    let b_over_f = b / f;
    let fe_per = (f - e) / evals_per_round as f64;
    let ntok = tokens.len();
    let ntid = tids.len();
    let pairs = ntok * ntid;
    let not_pct = 100.0 * not_a_reconstruction / filter_ms;
    let unaccounted = 100.0 - accounted;

    let table = format!(
            "\nSTEP 0 — where-predicate cost decomposition, node-share [{N} {M}], \
             {REPS} interleaved reps, medians\n\
             \x20 captured from a real fire: 1 predicate x {ntok} tokens x {bindings_per_token} \
             binding(s); {passes}/{ntok} pass\n\
             \x20 fire counters, same axis: reuse {fire_reuse}  evals {fire_evals}  \
             gathers {fire_gathers} x {ntok} tokens\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 THE `exec_where` BRANCH — the fire enters it ZERO times here (evals = \
             {fire_evals})\n\
             \x20 A  env build alone         ({evals_per_round:>6} x)  {a_ms:>8.3} ms\n\
             \x20 B  env build + walk        ({evals_per_round:>6} x)  {b_ms:>8.3} ms\n\
             \x20 C  Vec<PMap> clone         ({N:>6} x)  {c_ms:>8.3} ms   <- NOT the fire's \
             clone; it clones Vec<Token> (arm L)\n\
             \x20 D  env + walk, VAR-FREE    ({evals_per_round:>6} x)  {d_ms:>8.3} ms\n\
             \x20 E  hand-written Rust       ({evals_per_round:>6} x)  {e_ms:>8.3} ms   <- THE FLOOR\n\
             \x20 F  compiled exec_where     ({evals_per_round:>6} x)  {f_ms:>8.3} ms   \
             {f_per:>6.1} ns/eval\n\
             \x20   the walk      B-A   {walk_ms:>8.3} ms  {walk_pct:>5.1}% of B   \
             {walk_per:>6.1} ns/eval\n\
             \x20     ?var lookup (B-A)-(D-A)  {look_ms:>8.3} ms  {look_pct:>5.1}% of the walk\n\
             \x20     node dispatch    D-A     {nov_ms:>8.3} ms  {nov_pct:>5.1}% of the walk\n\
             \x20   the env build A     {a_ms:>8.3} ms  {a_pct:>5.1}% of B   {a_per:>6.1} ns/eval\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 THE BRANCH THE FIRE TAKES — cumulative, ONE FIRE's worth per arm\n\
             \x20 G  bind_view               ({ntok:>6} x)  {g_us:>8.1} us\n\
             \x20 H  + where_tree.candidates ({ntok:>6} x)  {h_us:>8.1} us   + {dh:>7.1}\n\
             \x20 I  + proven/maybe HashSets ({ntok:>6} x)  {i_us:>8.1} us   + {di:>7.1}\n\
             \x20 J  + the tid loop          ({pairs:>6} x)  {j_us:>8.1} us   + {dj:>7.1}   \
             <- {dj_pct:.0}% of the branch\n\
             \x20 K  + the d_beta pushes     ({fire_reuse:>6} x)  {k_us:>8.1} us   + {dk:>7.1}   \
             <- the whole taken branch\n\
             \x20 L  d_beta parent gather    ({fire_gathers:>6} x)  {l_us:>8.1} us   \
             <- Vec<Token>, what the fire clones ({c_ms:.3} ms as Vec<PMap>)\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 RECONSTRUCTION  K+L = {taken:>6.3} ms  vs a LIVE `filter` of \
             {filter_ms:>6.3} ms  [{filter_raw_lo:.3}-{filter_raw_hi:.3} raw over {PHASE_READS}]\n\
             \x20                 {accounted:>5.1}% accounted  (p25/p75 band \
             {accounted_lo:.1}-{accounted_hi:.1}%)   UNACCOUNTED {unaccounted:>5.1}%\n\
             \x20   of which NAMED: the phase's own clock, {gather_pairs} dbeta:gather \
             mark pairs ~= {instrument_us:.1} us\n\
             \x20 ⛔ NOT a reconstruction  F+C = {not_a_reconstruction:>6.3} ms = \
             {not_pct:>4.0}% — five arms in a branch the fire does not enter\n\
             \x20 HEADROOM        B-E = {head_ms:>6.3} ms is what a PERFECT compile could remove\n\
             \x20 COMPILED vs B   B/F = {b_over_f:>5.2}x    F-E leftover {fe_per:>6.1} ns/eval\n",
        );
    println!("{table}");

    // Non-vacuity on the LIVE READ: a zero `filter` makes every ratio above a division by
    // nothing, and the `% accounted` column would render as `inf` rather than as a fault.
    assert!(
        filter_ms > 0.0,
        "the live `filter` phase read 0 ns at [{N} {M}] — the census never entered the \
             filter pass, so the reconstruction above is measured against nothing{table}"
    );

    // ⛔ THE COVERAGE FIGURE IS PRINTED AND NOT ASSERTED, AND THAT IS DELIBERATE.
    //
    // C6 measured the declared `F+C` check failing at ~7x and REFUSED to assert it. An arm set
    // that lands nearer and gets asserted BECAUSE it looks better is the same unfalsifiable
    // claim with a prettier number, so the same refusal applies here for a stated reason:
    //
    //   K+L covers ~89% of the live `filter` phase, and each run's own band spans ~13 points.
    //   Driven 2026-09-04, this box, release, six consecutive runs of this test:
    //
    //     87.3%  92.4%  91.2%  93.5%  87.1%  84.2%   accounted (K+L vs the live `filter`)
    //     bands  82.4-93.5  90.5-97.5  87.5-97.2  86.2-98.1  85.1-98.0  81.0-94.3
    //
    // The arms replay a strict SUBSET of the phase's statements, so `K+L <= filter` is a
    // structural necessity — and the measurement CANNOT RESOLVE IT: the run-to-run spread of
    // the headline is ~9 points (84.2-93.5) and each run's own p25/p75 band reaches ~98%, so
    // the ~11% remainder is roughly the size of the instrument's own scatter. Any bound tight
    // enough to be interesting would be a number chosen to pass, which is the one thing this
    // file exists to not do. So the coverage is STATED with its band and the remainder is
    // NAMED, and no threshold is asserted over either.
    //
    // THE REMAINDER IS TOKEN-INDEPENDENT, and that was MEASURED, not reasoned. Halving the
    // token count to [50 100] (2026-09-04) halves every taken-branch arm — K 354 -> 171 us,
    // J-I 275 -> 137, H-G 46 -> 22 — but the phase falls only 0.393 -> 0.225 ms, so coverage
    // drops ~88% -> 78.3%. The gap is a ~40-55 us component that does not scale with M: the
    // per-node walk over all N filter nodes (twice, once per round) and a `dbeta:gather` mark
    // count that stays at 100 pairs because it is per NODE, not per token.
    //
    // WHAT THE ~11% IS, as far as it can be named without a new mark:
    //   - the phase's OWN CLOCK: `d_beta_from_parents` fires 100 `dbeta:gather` mark pairs
    //     inside the `filter` mark, ~78 ns each = ~7.8 us = ~2 points of the 8. Printed above.
    //   - `filter_pass`'s per-node loop over all {N} filter nodes: `get_node`, `kind_of`, the
    //     `parents_of` lookup, the `tests_done` probe, and — in a `cfg(test)` build, which is
    //     what the phase census reads — `capture_where_sample`'s `node_named_ast(node, "expr")`.
    //   - the SECOND round: `filter` fired 2 mark pairs, and round 2 re-walks all {N} nodes for
    //     an empty parent delta (50 of the fire's 100 `dbeta:calls` return nothing).
    // None of these is the dispatch; all of them are inside the mark.
    //
    // ⛔ AND THE HEADLINE THE OLD LADDER COULD NOT SEE: the `tid` loop is the phase.
    // J-I is ~78-83% of the whole taken branch and ~70% of the live `filter` phase — 10,000
    // (token, tid) pairs, three `HashSet<i64>` probes each, 9,800 of them reaching `continue`.
    // It is the ONLY rung that scales with N x M, and until this strike it had no arm at all.
    // The arm that used to be presented as the phase's honest half, C, is a `Vec<PMap>` clone:
    // the fire clones `Vec<Token>` (`d_beta_from_parents`, 16-byte `Copy`), which arm L
    // measures at ~8 us against C's ~125 us. Same count, wrong type, ~16x
    // ([[a-count-cannot-see-a-value-defect]]).


    // Non-vacuity on the INSTRUMENT itself: a zero reading means the optimiser removed the
    // arm, and every share above would be an artifact.
    assert!(
        a > 0.0 && b > 0.0 && c > 0.0 && d > 0.0 && e > 0.0 && f > 0.0 && b > a && b > e,
        "an arm measured zero, or the orderings that MUST hold do not — the loop was \
             optimised away and the shares above are artifacts \
             (A={a}ns B={b}ns C={c}ns D={d}ns E={e}ns){table}"
    );

    // Non-vacuity on the taken branch's arms: a zero reading is an arm the optimiser removed,
    // and every share above would then be an artifact.
    assert!(
        g > 0.0 && h > 0.0 && i_arm > 0.0 && j > 0.0 && k > 0.0 && l > 0.0,
        "a taken-branch arm measured zero — the loop was optimised away and the rungs above are \
         artifacts (G={g}ns H={h}ns I={i_arm}ns J={j}ns K={k}ns L={l}ns){table}"
    );

    // ⛔ ADJACENT-RUNG MONOTONICITY IS **NOT** ASSERTED, AND THE REASON IS A RED THIS
    // INSTRUMENT ALREADY PRODUCED. `G <= H <= I <= J <= K` is structurally true — each rung
    // ADDS a statement of the taken branch — but this instrument cannot see it. Asserted as
    // `h >= g && i >= h && j >= i && k >= j`, it went RED on the SIXTH consecutive drive,
    // 2026-09-04, this box, release:
    //
    //     the cumulative ladder is not monotone, so a rung's added work was optimised away
    //     and its delta is an artifact
    //     (G=338ns H=46755ns I=56527ns J=349781ns K=345199ns L=6697ns)
    //
    // K came in 4,582 ns BELOW J while the K-J rung is only ~8,000 ns — 2.3% of a 345 us arm.
    // J and K are timed in different parts of the same rep, so their medians drift against each
    // other by more than a 2% rung is wide. THE SMALL RUNGS ARE BELOW THIS INSTRUMENT'S
    // RESOLUTION, and a gate on a delta smaller than the spread that produces it is a coin
    // toss wearing an invariant's name ([[gate-the-ratio-not-the-millisecond]]).
    //
    // So the table PRINTS every rung, negative ones included — `+ -4.6` is exactly what that
    // red rendered — and the only ordering asserted is the one with a wide margin: the ★ below.
    // ⚠ Do NOT re-add the adjacent chain "because it passed five times". It did.

    // ★ THE FINDING, ASSERTED AS AN ORDERING RATHER THAN A NUMBER. The (token x tid) set-probe
    // loop is the largest rung of the taken branch and the majority of it — ~275-295 us of a
    // ~350 us branch across six drives, against ~44 us for the next rung. That is not a
    // tolerance: it is the mechanism this strike found, and it is what makes zeroing that loop
    // RED rather than merely shifting a printed percentage. If it stops holding, the filter
    // phase's cost has MOVED: re-derive which rung dominates and say so, do not relax this.
    let branch_rungs = format!(
        "J-I {dj:.1} us of K {k_us:.1} us; H-G {dh:.1}, I-H {di:.1}, K-J {dk:.1}, L {l_us:.1}"
    );
    assert!(
        dj > dh && dj > di && dj > dk && dj > l_us && (j - i_arm) > k / 2.0,
        "the (token x tid) set-probe loop is no longer the majority rung of the taken branch \
         ({branch_rungs}). The filter phase's cost has moved to a different statement — name \
         it{table}"
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

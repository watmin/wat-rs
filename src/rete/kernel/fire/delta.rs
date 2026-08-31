//! Semi-naive delta fixpoint and opt-in explain.

use super::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;
use crate::value::value::AggregateValue;

/// Acc `:from` leftover binds → elements in that group.
pub(crate) type AccGroupKey = Vec<(Value, Value)>;
pub(crate) type AccGroupBuckets<'a> = HashMap<AccGroupKey, Vec<&'a Element>>;
pub(crate) type AccGroupOrder<'a> = Vec<(crate::value::pmap::PMap, Vec<&'a Element>)>;

/// Step-1 alpha activate for one fact. Shared by the seed worklist (`wm.facts`)
/// and later owned deltas (`DESIGN-STONE-setup-seen-once`). Split-borrow bundle:
/// the refs one fact-activate needs. The P4b/P6 round loop lives on
/// [`fire_fixpoint_delta_armed`].
pub(crate) struct AlphaActivateCx<'a> {
    /// Needed by `Op::Eval` (fix-list F) — a computed inline operand runs through the one
    /// expression core.
    pub(crate) sym: &'a SymbolTable,
    pub(crate) wm: &'a mut FireSession,
    pub(crate) d_alpha: &'a mut AlphaDelta,
    pub(crate) alpha_tree: &'a crate::rete::alpha_tree::AlphaTree,
    pub(crate) compiled_conds: &'a HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    pub(crate) match_scratch: &'a mut SlotFrame,
    pub(crate) cand_scratch: &'a mut Vec<i64>,
    pub(crate) cond_key_ids: &'a CondKeyIds,
    /// Bind-only alphas: output field indexes into the packed row
    /// (`DESIGN-STONE-fire-i64-columns`). Absent → compiled exec.
    pub(crate) bind_only: &'a BindOnlyFields,
}

/// Push one fact through the alpha tree, writing matches into the round's alpha delta.
///
/// A non-record value returns `Ok(())` rather than raising: the fact bag is filtered at the
/// insert door, so anything else reaching here is not a user error to report but a value this
/// pass has nothing to say about.
pub(crate) fn alpha_activate_fact(
    fact: &Value,
    fact_idx: u32,
    cx: &mut AlphaActivateCx<'_>,
) -> Result<(), EvalBreak> {
    let (fact_class, fact_fields) = match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            (a.class.as_ref(), a.fields.as_slice())
        }
        _ => return Ok(()),
    };
    cx.alpha_tree
        .candidates_into(fact_class, fact_fields, cx.cand_scratch);
    if cx.cand_scratch.is_empty() {
        return Ok(());
    }
    let i = fact_idx as usize;
    if i >= cx.wm.i64_by_fact.len() {
        let packed = pack_i64_row(fact_fields, &mut cx.wm.bind_vals, &mut cx.wm.bind_val_ids);
        cx.wm.i64_by_fact.resize(i, None);
        cx.wm.i64_by_fact.push(packed);
    }
    let row = cx.wm.i64_by_fact[i];
    for aid in cx.cand_scratch.iter().copied() {
        let compiled = rematch_compiled(cx.compiled_conds, aid)?;
        let key_ids = cx.cond_key_ids.get(&aid).map(|v| v.as_slice());
        let fields = cx.bind_only.get(&aid).map(Vec::as_slice);
        let skip_span = compiled.fact_bind().is_none()
            && match fields {
                Some([]) => true,
                Some(_) => row.is_some(),
                None => false,
            };
        let matched = if skip_span {
            census_count("compiled:calls");
            Some((0u32, 0u16))
        } else {
            let mut intern = crate::rete::compiled_cond::BindIntern {
                keys: &mut cx.wm.bind_keys,
                vals: &mut cx.wm.bind_vals,
                ids: &mut cx.wm.bind_val_ids,
                pool: &mut cx.wm.bind_pool,
            };
            crate::rete::compiled_cond::exec_compiled_with_key_ids(
                cx.sym,
                compiled,
                fact_fields,
                cx.match_scratch,
                &mut intern,
                fact,
                key_ids,
            )
        };
        if let Some((off, len)) = matched {
            let el = make_element(fact_idx, off, len);
            let slot = {
                let v = Arc::make_mut(cx.wm.alpha.entry(aid).or_default());
                v.push(el);
                v.len() - 1
            };
            cx.d_alpha.entry(aid).or_default().push(slot);
        }
    }
    Ok(())
}

/// Compare leaf-set predicted occupancy to what seed installed in `wm.alpha`
/// (leaf-fill or activate). Does not change memories
/// (`DESIGN-STONE-occupancy-leaf-column` recolligere).
#[cfg(test)]
pub(crate) fn record_seed_leaf_vs_alpha(
    wm: &FireSession,
    alpha_tree: &crate::rete::alpha_tree::AlphaTree,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    bind_only: &BindOnlyFields,
    input_facts: &crate::value::pvec::PVec,
) {
    if !crate::rete::kernel::census::leaf_occ_armed() {
        return;
    }
    let mut predicted: FxHashSet<(i64, u32)> = FxHashSet::default();
    let mut leaf_aids: Vec<i64> = Vec::new();
    for (class, leaves) in alpha_tree.undiscriminated_leaves() {
        let batchable = leaves.iter().all(|&id| {
            compiled_conds
                .get(&id)
                .is_some_and(|c| c.fact_bind().is_none())
                && bind_only.contains_key(&id)
        });
        if !batchable {
            continue;
        }
        leaf_aids.extend_from_slice(leaves);
        for (i, fact) in input_facts.iter().enumerate() {
            let Value::Aggregate(a) = fact else {
                continue;
            };
            if a.nature == Nature::Struct || a.class.as_ref() != class {
                continue;
            }
            if wm
                .i64_by_fact
                .get(i)
                .and_then(|o| o.as_ref())
                .is_none()
            {
                continue;
            }
            for &aid in leaves {
                predicted.insert((aid, i as u32));
            }
        }
    }
    let mut actual: FxHashSet<(i64, u32)> = FxHashSet::default();
    for &aid in &leaf_aids {
        if let Some(els) = wm.alpha.get(&aid) {
            for el in els.iter() {
                actual.insert((aid, el.fact));
            }
        }
    }
    let mut extra: Vec<(i64, u32)> = predicted.difference(&actual).copied().collect();
    let mut missing: Vec<(i64, u32)> = actual.difference(&predicted).copied().collect();
    extra.sort_unstable();
    missing.sort_unstable();
    crate::rete::kernel::census::record_leaf_occ_diff(crate::rete::kernel::census::LeafOccDiff {
        predicted: predicted.len(),
        actual: actual.len(),
        extra,
        missing,
        n_facts: input_facts.len(),
        n_leaf_aids: leaf_aids.len(),
    });
}

/// Stamped Aggregates membership is the construction fingerprint
/// (`DESIGN-STONE-seen-identity-set`). `identity == 0` still stores `Value`.
pub(crate) fn seen_insert(
    ids: &mut FxHashSet<u64>,
    rest: &mut FxHashSet<Value>,
    v: &Value,
) -> bool {
    match v {
        Value::Aggregate(a) if a.identity() != 0 => ids.insert(a.identity()),
        _ => rest.insert(v.clone()),
    }
}

/// fire-rules vs fire-once share the delta walk. Once is one round and
/// keeps alpha/beta (oracle `fire-once$oracle`); Rules cascades and drops
/// scratch memories before freeze.
#[derive(Clone, Copy)]
pub(crate) enum FireKind {
    Rules,
    Once,
}

/// Semi-naive delta fixpoint: persistent memories, per-round delta sets, linear depth.
///
/// Implements the algorithm from DESIGN-STONE-P4b-delta-fire.md:
/// - Memories (`wm.alpha`, `wm.beta`, `wm.production`) accumulate across rounds (never cleared).
/// - Each round propagates only `delta_facts` (the facts derived last round).
/// - Hash-join uses the semi-naive formula:
///   `Δbeta[J] = (Δbeta[P] ⋈ all wm.alpha[A]) ∪ (old_left[P] ⋈ Δalpha[A])`
///   where `old_left[P] = wm.beta[P]` before this round's root-join/hash-join appends.
/// - Terminates when `next_delta_facts` is empty (monotone-finite / datalog).
/// - Returns the persistent session with `facts = input` (same contract as P4a).
///
/// Observationally identical to a naive re-run fixpoint: same token multiset produced,
/// same `wm.production` multiset → identical `query` counts. O(depth²) → linear.
///
/// P6: the hash-join delta step uses persistent per-node `left_idx`/`right_idx`/`join_keys`
/// maintained incrementally across rounds (never rebuilt) — same observable result, O(1)
/// probe cost per match instead of O(W) rebuild per round per node.
pub(crate) fn fire_fixpoint_delta(
    session: &Value,
    sym: &SymbolTable,
    support: Option<&mut ExplainSupport>,
) -> Result<Value, EvalBreak> {
    fire_fixpoint_delta_armed(session, sym, support, None, FireKind::Rules)
}

/// Same as [`fire_fixpoint_delta`], with a prebuilt arm. Stratify holds the
/// slice `Arc` as a value and does not intern the slice network.
pub(crate) fn fire_fixpoint_delta_armed(
    session: &Value,
    sym: &SymbolTable,
    mut support: Option<&mut ExplainSupport>,
    pre_arm: Option<Arc<crate::rete::kernel::InternedNetwork>>,
    kind: FireKind,
) -> Result<Value, EvalBreak> {
    let __in = phase_start();
    let mut wm = to_transient_for_fire(session)?;
    phase_end("IN: to_transient", __in);
    let __setup = phase_start();

    // Start with empty memories (staged session may carry stale state from prior calls).
    wm.alpha.clear();
    wm.beta.clear();
    wm.production.clear();
    wm.bind_pool.clear();
    wm.bind_keys.clear();
    wm.bind_vals.clear();
    wm.bind_val_ids.clear();
    wm.match_pool.clear();
    wm.derived_facts.clear();
    wm.i64_by_fact.clear();
    wm.bind_only.clear();
    wm.cond_key_ids.clear();
    wm.input_has_scan_class = false;

    // `seen`: every fact ever in the working set. Seed with all input facts.
    // Mirrors `merge-facts`'s `contains?` guard — ensures each derived fact is processed once.
    // A HashSet (not Vec) so the membership check is O(1): with N derived facts, a Vec + `.contains`
    // is O(N) per check = O(N²) total (the fan-out blow-up); the set makes dedup O(N). Order does not
    // matter — RETE's final fact set is order-independent and the differential gates counts.
    // First worklist IS wm.facts. `seen` is filled once (one clone+hash per
    // input). Later rounds own a Vec of derived facts
    // (`DESIGN-STONE-setup-seen-once`).
    let input_facts: crate::value::pvec::PVec = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => crate::value::pvec::PVec::new(),
    };
    wm.n_input = input_facts.len() as u32;
    wm.bind_pool.reserve(input_facts.len().saturating_mul(4));
    wm.i64_by_fact.reserve(input_facts.len());
    let __seen = phase_start();
    let __seen_alloc = phase_start();
    let mut seen_ids: FxHashSet<u64> =
        FxHashSet::with_capacity_and_hasher(input_facts.len(), Default::default());
    let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
    phase_end("  │  setup:seen:alloc", __seen_alloc);
    phase_end("  ├ setup:seen", __seen);
    let mut owned_delta: Vec<u32> = Vec::new();
    let mut seed_round = true;

    // Item 12 — the arm lives next to the network. Hit: skip lower/classify.
    // Miss: build once, intern under the network's rust identity. insert/clone
    // share that identity (facts overlay). Stratify passes a prebuilt slice arm.
    let __arm = phase_start();
    let arm = match pre_arm {
        Some(a) => a,
        None => rete_arm_get_or_build(&wm.network, &wm.rules, sym)?,
    };
    phase_end("  ├ setup:arm", __arm);
    #[cfg(test)]
    let node_ids = arm.node_ids.as_slice();
    let kind_ids = &arm.kind_ids;
    let compiled_conds = &arm.compiled_conds;
    let q_scans = query_class_scans(&arm, &wm.network);
    let q_only_alphas: HashSet<i64> = q_scans.keys().copied().collect();
    // Occupancy tree does not contain query-only alphas
    // (`DESIGN-STONE-query-only-out-of-occupancy`). Harvest is the closed bag.
    let occupancy_tree = if q_only_alphas.is_empty() {
        None
    } else {
        let keep: HashSet<i64> = kind_ids
            .alpha
            .iter()
            .copied()
            .filter(|id| !q_only_alphas.contains(id))
            .collect();
        Some(arm.alpha_tree.restrict(&keep))
    };
    let alpha_tree = occupancy_tree.as_ref().unwrap_or(&arm.alpha_tree);
    let scan_classes: HashSet<&str> = q_scans.values().map(|s| s.class.as_str()).collect();

    // P6 — persistent join indexes, maintained ACROSS rounds (never rebuilt).
    // Keyed by HashJoinNode id J.
    // left_idx[J]:  key → Vec<Token>   (all left tokens seen so far for J)
    // right_idx[J]: key → Vec<Element> (all right elements seen so far for J)
    // join_keys[J]: the sorted shared-variable list (cached lazily on first use)
    let mut left_idx: JoinLeftIndex = HashMap::new();
    let mut right_idx: JoinRightIndex = HashMap::new();
    let mut join_keys_cache: JoinKeysCache = HashMap::new();
    let mut right_idx_n: HashMap<i64, usize> = HashMap::new();
    // P6-for-gathers: persist across rounds, append d_alpha
    // (`DESIGN-STONE-persist-gather-across-rounds`). Not a Session field.
    let mut gather_cache: GatherCache = FxHashMap::default();
    // Fire-scoped, NOT round-scoped, and that distinction is the whole point: a
    // leading (parentless) `:not`/`:exists` is re-evaluated every round, `wm.beta`
    // is cumulative, and the dedup set used to live inside the round body — so a
    // query over such a rule returned one row PER ROUND. See `LeadingEmitted`.
    let mut leading_emitted: LeadingEmitted = HashMap::new();

    // One scratch buffer, reused for every compiled-condition call this whole fire pass: sized
    // once to the largest `n_slots` any compiled alpha needs, so `exec_compiled_with_key_ids`'s
    // `clear` + `resize` back up never reallocates after this point — the failure path it
    // guards allocates nothing (row 2 of the DESIGN-STONE's scorecard).
    let mut match_scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
    let mut cand_scratch: Vec<i64> = Vec::new();
    // Written by pass 3.20, read by pass 3.5, cleared per round by 3.20 — see
    // `RoundScratch::pre_dispatched`. Declared beside the other scratch so it is allocated once.
    let mut pre_dispatched: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut cond_key_ids: CondKeyIds = HashMap::new();
    let mut bind_only: BindOnlyFields = HashMap::new();
    for (&id, c) in compiled_conds {
        cond_key_ids.insert(
            id,
            crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
        );
        if let Some(fields) = crate::rete::compiled_cond::bind_only_fields(c) {
            bind_only.insert(id, fields);
        }
    }
    // ── WHY THERE ARE TWO COPIES OF EACH — a borrow split, not an oversight ──────────────
    // The round locals above and the `wm.*` fields below hold the SAME data for the whole
    // fire. Both are live and both are read: passes reached from here take `&bind_only` /
    // `&cond_key_ids` directly (see the `AlphaNews` construction sites below), while passes
    // that receive only `&mut FireSession` — `pass/mod.rs`'s `left_activate_join` setup and
    // `filter_after_join.rs` — read `&wm.bind_only` / `&wm.cond_key_ids` instead. A single
    // copy cannot serve both: borrowck refuses an immutable borrow of a `wm` field held
    // across a call that takes `&mut wm`. The clone is what buys the split.
    //
    // THEY CANNOT DIVERGE, and that is structural rather than a discipline: both `wm` fields
    // are cleared at the top of this same `fire_fixpoint_delta_armed`, in the fire-scoped
    // `clear()` block beside `wm.alpha` / `wm.beta`; this `clone_from` pair is then their
    // ONLY writer, and neither copy is mutated afterwards. So "are they still in sync?" is not
    // a question a reader has to carry.
    //
    // `sequi` flagged this on 2026-08-25 as two live copies with the reason stated nowhere —
    // correct, and by then the silence had already sent two scans down the wrong path, each
    // reading a `bind_only` that was not the one it thought (see the notes in
    // `pass/filter.rs` and `pass/filter_after_join.rs`, both written by a scan that had been
    // misled). The comment is the fix; the shape is deliberate.
    wm.bind_only.clone_from(&bind_only);
    wm.cond_key_ids.clone_from(&cond_key_ids);

    // Leaf-set fill: pack every fact (activate side effect), occupancy from
    // the column (`DESIGN-STONE-occupancy-leaf-column` recolligere).
    let mut leaf_aids: LeafAidsByClass = HashMap::new();
    for (class, leaves) in alpha_tree.undiscriminated_leaves() {
        let batchable = leaves.iter().all(|&id| {
            compiled_conds
                .get(&id)
                .is_some_and(|c| c.fact_bind().is_none())
                && bind_only.contains_key(&id)
        });
        if batchable {
            leaf_aids.insert(class.to_string(), leaves.to_vec());
        }
    }

    // A8 instrument: the round counter the census stamps its rows with (test-only).
    #[cfg(test)]
    let mut round_no: usize = 0;

    // ── THE TERMINATION CAP ──────────────────────────────────────────────────────────────────
    //
    // THIS IS THE STONE 4b DEFERRED, AND IT NAMED IT EXACTLY. `DESIGN-STONE-4b-cascade-fixpoint`
    // § Termination does not claim divergence is impossible — it names the boundary and defers:
    // "a rule that derives an unbounded stream of distinct facts (e.g. arithmetic in a fact-arg
    // producing X(n) -> X(n+1)) would not terminate ... if one is ever needed, a depth/round
    // safety cap is its own future stone (let need reveal)."
    //
    // The need revealed. `circumspicere` found nothing protects an embedder, and the shape 4b
    // predicted was measured 2026-08-27 in 11 lines of legal wat: the process died on
    // `memory allocation of 545259536 bytes failed` — no wat error, no span, no rule named, and
    // with no ulimit that is the machine's memory rather than one test's.
    //
    // ★ 4b's OBJECTION TO A CAP, ANSWERED RATHER THAN IGNORED: "a cap would mask a genuine
    // user-rule error and pick an arbitrary N".
    //   - MASKING: it does the opposite. Without the cap the user-rule error surfaced as an
    //     allocator abort naming nothing; with it, the error names the cap, the still-growing
    //     count, and the rule shape to look for. The cap is what makes the error VISIBLE.
    //   - ARBITRARY N: yes, and it is chosen against measurement rather than taste — see below.
    //
    // WHAT THIS BOUNDS, AND WHAT IT DOES NOT. It bounds NON-TERMINATION, not memory. One round may
    // still derive without bound — `fanout` derives 40_000 facts in a SINGLE round — and that is a
    // legitimate workload shape this deliberately does not limit. Capping rounds catches the
    // qualitative bug (a fixpoint that never converges) without putting a ceiling on honest volume.
    //
    // THE VALUE IS PER-PROGRAM: `(:wat::config::rete::set-max-fire-rounds! n)`, defaulting to
    // `crate::config::DEFAULT_MAX_FIRE_ROUNDS` (which carries why a single number cannot be right
    // for everyone). It is read through `config`, not through an encoding field, so it inherits
    // into spawned sub-programs like every other config value.
    //
    // ⚠ THIS IS A BACKSTOP, NOT THE GUARANTEE. The real answer is a load-time verifier that
    // REFUSES a rule set it cannot prove terminates — the eBPF-verifier shape, and the rung above
    // this one. `stratify` already refuses un-stratifiable sets at load ("negation cycle detected
    // — rule set is not stratifiable"), so half the machinery exists. Do not let this diagnostic
    // become the reason that never gets built: "I gave up after N rounds" is not "this program
    // cannot diverge".
    //
    // ⚠ THE ORACLE HAS NO SUCH CAP and will still hang on the same input. That asymmetry is
    // deliberate and bounded: the cap fires only on rule sets that are already broken, the
    // differential fuzzers cannot generate one (the case would hang the suite rather than fail
    // it), and `$oracle` is the slow-but-correct reference an embedder never runs.
    // ⛔ NO PER-FIRE SNAPSHOT. This used to snapshot `thread_bytes()` at fire entry and measure
    // growth from there, which bounded ONE FIRE — a FIRE ceiling wearing a SESSION ceiling's name.
    // The builder's ruling is that the SESSION is the boundary (*"it may not consume more than the
    // configured amount of memory, 1G by default"*), and a per-fire zero cannot express that: it
    // forgets everything `insert` staged before it, which is how 2.5M staged facts reached 4.0 GB
    // against a 1 GiB contract with no diagnostic. The origin is now marked once, at `arm-session`
    // (`alloc_counter::mark_session_origin`), and BOTH doors measure from it.
    // WHICH SESSION THE CEILING BELOW IS JUDGING. The zero point is filed per session
    // (`alloc_counter::SessionOriginKey` — the network's rust identity), not per thread, so this
    // door has to name the session it is standing at. Taken ONCE, above the loop: a fire does not
    // re-intern the network, and `insert`'s overlay clones it, which preserves the intern.
    let origin_key = crate::rete::kernel::network_identity(&wm.network);
    let max_fire_rounds: usize = sym
        .encoding_ctx()
        .map(|c| c.config.max_fire_rounds)
        .unwrap_or(crate::config::DEFAULT_MAX_FIRE_ROUNDS);
    let mut rounds_run: usize = 0;

    phase_end("SETUP: indexes", __setup);
    let __rounds = phase_start();
    loop {
        // ROUND LOOP scaffolding. Named phases inside the body are Alpha, Root-join,
        // Hash-join, Accumulate, Filter, Join-after-filter, Filter-after-join, Production,
        // then terminate — not a fixed pass count. These two marks bracket the loop's
        // own preamble so the remainder has a name instead of a parent/child subtraction.
        let __pre = phase_start();
        // Per-round delta sets (new elements/tokens created THIS round).
        // Indices into this round's wm.alpha[aid] (DESIGN-STONE-delta-alpha-indices).
        let mut d_alpha: AlphaDelta = FxHashMap::default();
        let mut d_beta: BetaMemory = HashMap::new();
        // Packed seed occupancy is dirty in full — walk 0..len, do not
        // materialize 0..n (`DESIGN-STONE-seed-d-alpha-range`).
        let mut packed_full: HashSet<i64> = HashSet::new();

        phase_end("  ├ round:preamble", __pre);

        // ── 1. Alpha delta (type-indexed): each delta fact probes ONLY its type's alphas. ──
        #[cfg(test)]
        let this_round_in = if seed_round {
            input_facts.len()
        } else {
            owned_delta.len()
        };
        let __pt0 = phase_start();
        if seed_round {
            crate::rete::kernel::fire::pass::alpha_seed(
                sym,
                &mut wm,
                &mut crate::rete::kernel::fire::pass::RoundScratch {
                    d_alpha: &mut d_alpha,
                    packed_full: &mut packed_full,
                    bind_only: &bind_only,
                    cond_key_ids: &cond_key_ids,
                    cand_scratch: &mut cand_scratch,
                    match_scratch: &mut match_scratch,
                    seen_ids: &mut seen_ids,
                    seen_rest: &mut seen_rest,
                    leaf_aids: &leaf_aids,
                    pre_dispatched: &mut pre_dispatched,
                },
                &input_facts,
                &arm.compiled_conds,
                alpha_tree,
                &scan_classes,
            )?;
            seed_round = false;
        } else {
            crate::rete::kernel::fire::pass::alpha_delta(
                sym,
                &mut wm,
                &mut crate::rete::kernel::fire::pass::RoundScratch {
                    d_alpha: &mut d_alpha,
                    packed_full: &mut packed_full,
                    bind_only: &bind_only,
                    cond_key_ids: &cond_key_ids,
                    cand_scratch: &mut cand_scratch,
                    match_scratch: &mut match_scratch,
                    seen_ids: &mut seen_ids,
                    seen_rest: &mut seen_rest,
                    leaf_aids: &leaf_aids,
                    pre_dispatched: &mut pre_dispatched,
                },
                &owned_delta,
                &arm.compiled_conds,
                alpha_tree,
            )?;
        }

        phase_end("alpha", __pt0);
        append_d_alpha(&mut gather_cache, &d_alpha, &wm, &packed_full);

        // ── 2. Root-join delta: seed tokens from NEW elements (d_alpha) only. ───
        crate::rete::kernel::fire::pass::root_join_delta(
            &mut wm,
            &arm,
            &d_alpha,
            &mut d_beta,
            &packed_full,
        );

        crate::rete::kernel::fire::pass::hash_join_delta(
            sym,
            &mut wm,
            &arm,
            &mut crate::rete::kernel::fire::pass::RoundScratch {
                d_alpha: &mut d_alpha,
                packed_full: &mut packed_full,
                bind_only: &bind_only,
                cond_key_ids: &cond_key_ids,
                cand_scratch: &mut cand_scratch,
                match_scratch: &mut match_scratch,
                seen_ids: &mut seen_ids,
                seen_rest: &mut seen_rest,
                leaf_aids: &leaf_aids,
                pre_dispatched: &mut pre_dispatched,
            },
            &mut d_beta,
            &mut left_idx,
            &mut right_idx,
            &mut join_keys_cache,
        )?;

        crate::rete::kernel::fire::pass::accumulate_pass(
            &mut wm,
            &arm,
            &mut crate::rete::kernel::fire::pass::RoundScratch {
                d_alpha: &mut d_alpha,
                packed_full: &mut packed_full,
                bind_only: &bind_only,
                cond_key_ids: &cond_key_ids,
                cand_scratch: &mut cand_scratch,
                match_scratch: &mut match_scratch,
                seen_ids: &mut seen_ids,
                seen_rest: &mut seen_rest,
                leaf_aids: &leaf_aids,
                pre_dispatched: &mut pre_dispatched,
            },
            &mut d_beta,
            &mut gather_cache,
            sym,
        )?;

        crate::rete::kernel::fire::pass::filter_pass(
            &mut wm,
            &arm,
            &mut crate::rete::kernel::fire::pass::RoundScratch {
                d_alpha: &mut d_alpha,
                packed_full: &mut packed_full,
                bind_only: &bind_only,
                cond_key_ids: &cond_key_ids,
                cand_scratch: &mut cand_scratch,
                match_scratch: &mut match_scratch,
                seen_ids: &mut seen_ids,
                seen_rest: &mut seen_rest,
                leaf_aids: &leaf_aids,
                pre_dispatched: &mut pre_dispatched,
            },
            &mut d_beta,
            &mut gather_cache,
            &mut leading_emitted,
            sym,
        )?;

        let after_join_frontier = crate::rete::kernel::fire::pass::join_after_filter(
            sym,
            &mut wm,
            &arm,
            &mut d_beta,
            &mut right_idx,
            &mut right_idx_n,
            &mut join_keys_cache,
            &mut match_scratch,
        )?;

        crate::rete::kernel::fire::pass::filter_after_join(
            &mut wm,
            &arm,
            &mut crate::rete::kernel::fire::pass::RoundScratch {
                d_alpha: &mut d_alpha,
                packed_full: &mut packed_full,
                bind_only: &bind_only,
                cond_key_ids: &cond_key_ids,
                cand_scratch: &mut cand_scratch,
                match_scratch: &mut match_scratch,
                seen_ids: &mut seen_ids,
                seen_rest: &mut seen_rest,
                leaf_aids: &leaf_aids,
                pre_dispatched: &mut pre_dispatched,
            },
            &mut d_beta,
            &mut right_idx,
            &mut right_idx_n,
            &mut join_keys_cache,
            &mut gather_cache,
            after_join_frontier,
            sym,
        )?;

        // ── 4. Production delta: fire production nodes on NEW tokens only. ────────
        let __pt5 = phase_start();
        let next_delta = crate::rete::kernel::fire::pass::production_delta(
            &mut wm,
            &arm,
            &d_beta,
            &mut seen_ids,
            &mut seen_rest,
            &mut support,
            sym,
        )?;
        // NARROWED 2026-08-24: this mark used to close AFTER the A8 census
        // below, so `production` reported the pass PLUS an 85-line test-only
        // node walk. It now brackets the pass and nothing else.
        phase_end("production", __pt5);

        // ── A8 instrument: census this round BEFORE the terminate check. ─────────
        #[cfg(test)]
        crate::rete::kernel::fire::pass::record_round_census(
            &wm,
            node_ids,
            &d_beta,
            &left_idx,
            &right_idx,
            &seen_ids,
            &seen_rest,
            round_no,
            this_round_in,
        );
        #[cfg(test)]
        {
            round_no += 1;
        }

        // ── 5. Terminate or loop. ─────────────────────────────────────────────────
        let __ep = phase_start();
        let __done = next_delta.is_empty();
        // Captured BEFORE the move below — it is the evidence the fixpoint was still GROWING at
        // the cap rather than merely deep, which is what separates a runaway rule from a big one.
        let __still_deriving = next_delta.len();
        if !__done {
            owned_delta = next_delta;
        }
        phase_end("  └ round:epilogue", __ep);
        // ── THE MEMORY CEILING, CHECKED ON EVERY ROUND INCLUDING THE LAST ───────────────────
        //
        // ⛔ THIS SITS ABOVE THE `break`, AND THE FIRST CUT DID NOT. Placed below it, the check was
        // unreachable for a fire that CONVERGED (`__done`) and for every `fire-once` — so it only
        // covered a multi-round fire still in progress, which is far narrower than the per-session
        // contract it is named for. A single round can allocate without bound (this file's own
        // header: "`fanout` derives 40_000 facts in one round"), so "it converged" is not evidence
        // it was cheap. Found 2026-08-29 when the builder asked whether `insert` could exceed the
        // limit; the answer sent me back to my own check, which had the same hole one level in.
        //
        // The DECISION is `session::session_ceiling_breach` — shared with the insert door so the
        // two cannot drift on what "over the ceiling" means. Only the diagnostic is local: rounds
        // completed is what THIS door can honestly say about how far the session had got.
        if let Some(breach) = crate::rete::kernel::session::session_ceiling_breach(sym, origin_key) {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::SessionMemoryCeilingExceeded {
                    limit: breach.limit,
                    used: breach.used,
                    rounds: rounds_run,
                },
            )
            .into());
        }
        if __done || matches!(kind, FireKind::Once) {
            break;
        }
        rounds_run += 1;
        if rounds_run >= max_fire_rounds {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::FixpointRoundCapExceeded {
                    cap: max_fire_rounds,
                    still_deriving: __still_deriving,
                },
            )
            .into());
        }
    }

    // Drop alpha elements before freeze — alpha is fire-scoped scratch, not session state.
    // The wat oracle's fire-rules$oracle returns an EMPTY alpha (fire-stratified),
    // so carrying one here is a divergence as well as a cost: both engines rebuild alpha from
    // `facts` every fire and never read a frozen one. It was ~31% of fire to serialize.
    // (fire_once_session deliberately keeps its alpha — it mirrors the oracle's fire-once,
    //  which does populate it.)
    // ── Binding-cardinality census (test-only) ───────────────────────────────────────────
    // The binding-representation stone rests on ONE premise: a binding map holds 1-2 entries,
    // so an rpds trie (heap alloc + Arc + hash + pointer-chase + dealloc) is paying trie prices
    // for a pair. If the real distribution is wide, a small-vec is WORSE and the stone inverts.
    // Measured on the LIVE population at end of fire — one walk, no hot-path instrumentation to
    // distort the very thing being measured.
    #[cfg(test)]
    {
        // Buckets are PER KIND. Element and Token have different operation profiles and are
        // getting different representations (DESIGN-STONE-element-bindings-array), so a combined
        // histogram cannot answer the question either of them asks. An earlier version of this
        // census shared one bucket set across both and a design doc then claimed it "separates
        // elements from tokens" — it separated only the totals.
        fn ebucket(n: usize) -> &'static str {
            match n {
                0 => "elem-card:0",
                1 => "elem-card:1",
                2 => "elem-card:2",
                3 => "elem-card:3",
                4 => "elem-card:4",
                5 => "elem-card:5",
                6..=7 => "elem-card:6-7",
                _ => "elem-card:8+",
            }
        }
        fn tbucket(n: usize) -> &'static str {
            match n {
                0 => "tok-card:0",
                1 => "tok-card:1",
                2 => "tok-card:2",
                3 => "tok-card:3",
                4 => "tok-card:4",
                5 => "tok-card:5",
                6..=7 => "tok-card:6-7",
                _ => "tok-card:8+",
            }
        }
        for els in wm.alpha.values() {
            for el in els.iter() {
                let b = element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                census_count(ebucket(b.len()));
                census_count("bind-card:ELEMENTS");
            }
        }
        for toks in wm.beta.values() {
            for t in toks {
                census_count(tbucket(t.binds.len as usize));
                census_count("bind-card:TOKENS");
            }
        }
    }

    let __hq = phase_start();
    harvest_query_memory(&mut wm, &arm, &q_scans);
    phase_end("  ├ harvest:query", __hq);
    let __drop = phase_start();
    if matches!(kind, FireKind::Rules) {
        wm.alpha.clear();
        // Drop ephemeral beta tokens before freeze — derived facts live in production-memory.
        // (Re-generated on every fire; never read from a frozen Session's beta-memory by native fire.)
        wm.beta.clear();
        // Pairs last — Element spans must not dangle (`DESIGN-STONE-bind-pool`).
        wm.bind_pool.clear();
        wm.bind_keys.clear();
        wm.bind_vals.clear();
        wm.bind_val_ids.clear();
        wm.match_pool.clear();
    }
    phase_end("  └ round:drop-memories", __drop);
    phase_end("ROUND LOOP", __rounds);

    // Return persistent session with facts = input (fire-rules contract).
    // The input facts are already in wm.facts (never modified during delta fire).
    let input_facts = wm.facts.clone();
    // The Value<->native conversions and the tail are OUTSIDE the round loop and were
    // never marked — the six phases covered only ~28% of fire, so everything apportioned
    // within them was apportioned within a quarter of the work.
    let __out = phase_start();
    let __res = Ok(session_with_facts(&to_persistent(wm), input_facts));
    phase_end("OUT: to_persistent", __out);
    __res
}

// ── Public entry: native fire-rules-explain ──────────────────────────────────

/// `(:wat::rete::fire-rules-explain <session>) -> :wat::rete::Explained`
///
/// P12a: OPT-IN diagnostic fire. Enters the same stratify-or-delta door as
/// `fire-rules` and additionally records, for each derived fact, the token
/// that produced it (and the rule name). Returns `Explained { session, support }`
/// — `session` is the same frozen Session the fast path produces; `support` is a
/// `PersistentMap<derived-fact, Support>`.
pub(crate) fn eval_fire_rules_explain(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-rules-explain";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }

    // Evaluate the session argument (mirrors eval_fire_rules_native).
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();

    // ⛔ THE CEILING ARMS SHORT-CIRCUIT HERE, through the ONE conversion site. `explain` is a fire
    // like any other and carries both ceilings; what differs is only its payload, which is why
    // `FireOutcome` is parametric — this answers `(FireOutcome :- [Explained])` while `fire-rules`
    // answers `(FireOutcome :- [Session])`, out of the same enum and the same converter.
    //
    // Same stratify-or-delta door as fire-rules; support records on both arms.
    let mut idx: HashMap<Value, (String, Value)> = HashMap::new();
    let session_out = match fire_rules_on_session(&session, sym, Some(&mut idx)) {
        Ok(s) => s,
        // A breach: hand the Err straight to the converter, which turns it into the matchable
        // ceiling arm. There is no half-built `Explained` to discard — the fire never returned one.
        Err(e) => return crate::rete::kernel::outcome::fire_result_to_outcome(Err(e)),
    };

    // Build the support PersistentMap: derived-fact → Support{rule, token_value}.
    let mut support_pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (derived_fact, (rule_name, token_value)) in idx {
        let support_value = Value::Aggregate(Arc::new(AggregateValue::record(
            (*support_class_fqdn()).clone(),
            support_names(),
            Arc::new(vec![Value::String(Arc::new(rule_name)), token_value]),
        )));
        support_pm.insert_mut(derived_fact, support_value);
    }

    // Build Explained { session, support }.
    let explained = Value::Aggregate(Arc::new(AggregateValue::record(
        (*explained_class_fqdn()).clone(),
        explained_names(),
        Arc::new(vec![
            session_out,
            // Never wrap a built trie directly — choose the arm by size.
            Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(support_pm)),
        ]),
    )));

    crate::rete::kernel::outcome::fire_result_to_outcome(Ok(explained))
}

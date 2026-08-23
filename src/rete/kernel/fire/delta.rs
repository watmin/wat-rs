//! Semi-naive delta fixpoint and opt-in explain.

use super::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::ast::WatAST;
use crate::rete::compiled_cond::BindIntern;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;
use crate::value::value::AggregateValue;

/// Acc `:from` leftover binds → elements in that group.
type AccGroupKey = Vec<(Value, Value)>;
type AccGroupBuckets<'a> = HashMap<AccGroupKey, Vec<&'a Element>>;
type AccGroupOrder<'a> = Vec<(crate::value::pmap::PMap, Vec<&'a Element>)>;

/// Step-1 alpha activate for one fact. Shared by the seed worklist (`wm.facts`)
/// and later owned deltas (`DESIGN-STONE-setup-seen-once`). Split-borrow bundle:
/// the refs one fact-activate needs. The P4b/P6 round loop lives on
/// [`fire_fixpoint_delta_armed`].
pub(crate) struct AlphaActivateCx<'a> {
    pub(crate) wm: &'a mut FireSession,
    pub(crate) d_alpha: &'a mut AlphaDelta,
    pub(crate) alpha_tree: &'a crate::rete::alpha_tree::AlphaTree,
    pub(crate) compiled_conds: &'a HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    pub(crate) match_scratch: &'a mut SlotFrame,
    pub(crate) cand_scratch: &'a mut Vec<i64>,
    pub(crate) cond_key_ids: &'a CondKeyIds,
    /// Bind-only alphas: output field indexes into the packed row
    /// (`DESIGN-STONE-fire-i64-columns`). Absent → compiled exec.
    pub(crate) bind_only: &'a HashMap<i64, Vec<u8>>,
}

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
fn record_seed_leaf_vs_alpha(
    wm: &FireSession,
    alpha_tree: &crate::rete::alpha_tree::AlphaTree,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    bind_only: &HashMap<i64, Vec<u8>>,
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
    let compiled_drivers = &arm.compiled_drivers;
    let compiled_wheres = &arm.compiled_wheres;
    let where_tree = &arm.where_tree;
    let compiled_acc_folds = &arm.compiled_acc_folds;
    let compiled_rhs_cache = &arm.compiled_rhs;
    let feeding_alpha_of = &arm.feeding_alpha_of;
    let parents_of = &arm.parents_of;
    let beta_readers = &arm.beta_readers;
    let test_sibs_of = &arm.test_sibs;
    let test_children = &arm.test_children;
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

    // One scratch buffer, reused for every compiled-condition call this whole fire pass: sized
    // once to the largest `n_slots` any compiled alpha needs, so `exec_compiled_with_key_ids`'s
    // `clear` + `resize` back up never reallocates after this point — the failure path it
    // guards allocates nothing (row 2 of the DESIGN-STONE's scorecard).
    let mut match_scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
    let mut cand_scratch: Vec<i64> = Vec::new();
    let mut cond_key_ids: CondKeyIds = HashMap::new();
    let mut bind_only: HashMap<i64, Vec<u8>> = HashMap::new();
    for (&id, c) in compiled_conds {
        cond_key_ids.insert(
            id,
            crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
        );
        if let Some(fields) = crate::rete::compiled_cond::bind_only_fields(c) {
            bind_only.insert(id, fields);
        }
    }
    wm.bind_only.clone_from(&bind_only);
    wm.cond_key_ids.clone_from(&cond_key_ids);

    // Leaf-set fill: pack every fact (activate side effect), occupancy from
    // the column (`DESIGN-STONE-occupancy-leaf-column` recolligere).
    let mut leaf_aids: HashMap<String, Vec<i64>> = HashMap::new();
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
            // Two pairs / fire, not per fact (`DESIGN-STONE-alpha-leftover-split`).
            let __seed = phase_start();
            let mut class_ids: HashMap<String, Vec<u32>> = HashMap::new();
            for class in leaf_aids.keys() {
                class_ids.insert(class.clone(), Vec::with_capacity(input_facts.len()));
            }
            for (i, fact) in input_facts.iter().enumerate() {
                seen_insert(&mut seen_ids, &mut seen_rest, fact);
                let (class, fields) = match fact {
                    Value::Aggregate(a) if a.nature != Nature::Struct => {
                        (a.class.as_ref(), a.fields.as_slice())
                    }
                    _ => {
                        alpha_activate_fact(
                            fact,
                            i as u32,
                            &mut AlphaActivateCx {
                                wm: &mut wm,
                                d_alpha: &mut d_alpha,
                                alpha_tree,
                                compiled_conds,
                                match_scratch: &mut match_scratch,
                                cand_scratch: &mut cand_scratch,
                                cond_key_ids: &cond_key_ids,
                                bind_only: &bind_only,
                            },
                        )?;
                        continue;
                    }
                };
                if !wm.input_has_scan_class && scan_classes.contains(class) {
                    wm.input_has_scan_class = true;
                }
                if wm.i64_by_fact.len() == i {
                    wm.i64_by_fact.push(pack_i64_row(
                        fields,
                        &mut wm.bind_vals,
                        &mut wm.bind_val_ids,
                    ));
                }
                let packed = wm.i64_by_fact.get(i).and_then(|o| o.as_ref()).is_some();
                if packed {
                    if let Some(ids) = class_ids.get_mut(class) {
                        ids.push(i as u32);
                        continue;
                    }
                }
                alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut AlphaActivateCx {
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree,
                        compiled_conds,
                        match_scratch: &mut match_scratch,
                        cand_scratch: &mut cand_scratch,
                        cond_key_ids: &cond_key_ids,
                        bind_only: &bind_only,
                    },
                )?;
            }
            for (class, aids) in &leaf_aids {
                let Some(ids) = class_ids.get(class) else {
                    continue;
                };
                if ids.is_empty() {
                    continue;
                }
                census_count_n("compiled:calls", ids.len() as u64 * aids.len() as u64);
                // rune:perspicere(intentional-structure) — Arc vs owned Vec is the occupancy-share door
                let els: Arc<Vec<Element>> = Arc::from(
                    ids.iter()
                        .map(|&idx| make_element(idx, 0, 0))
                        .collect::<Vec<_>>(),
                );
                for &aid in aids {
                    wm.alpha.insert(aid, Arc::clone(&els));
                    packed_full.insert(aid);
                }
            }
            phase_end("  ├ alpha:seed", __seed);
            #[cfg(test)]
            record_seed_leaf_vs_alpha(
                &wm,
                alpha_tree,
                compiled_conds,
                &bind_only,
                &input_facts,
            );
            seed_round = false;
        } else {
            let __delta = phase_start();
            for &idx in &owned_delta {
                let fact = fact_at(&wm.facts, &wm.derived_facts, wm.n_input, idx).clone();
                alpha_activate_fact(
                    &fact,
                    idx,
                    &mut AlphaActivateCx {
                        wm: &mut wm,
                        d_alpha: &mut d_alpha,
                        alpha_tree,
                        compiled_conds,
                        match_scratch: &mut match_scratch,
                        cand_scratch: &mut cand_scratch,
                        cond_key_ids: &cond_key_ids,
                        bind_only: &bind_only,
                    },
                )?;
            }
            phase_end("  └ alpha:delta", __delta);
        }

        phase_end("alpha", __pt0);
        append_d_alpha(&mut gather_cache, &d_alpha, &wm, &packed_full);

        // ── 2. Root-join delta: seed tokens from NEW elements (d_alpha) only. ───
        let __pt1 = phase_start();
        for node_id in &kind_ids.alpha {
            // New this round: indices into wm.alpha[node_id]. Packed seed
            // is 0..len (`DESIGN-STONE-seed-d-alpha-range`).
            let news = AlphaNews::of(&d_alpha, &wm.alpha, *node_id, &packed_full);
            if news.is_empty() {
                continue;
            }
            let child_ids: &[i64] = arm
                .children_of
                .get(node_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            // rune:temperare(simplicity-win) — kind_of filters mixed children_of; typed child
            // lists at intern would drop the Value-network probe. n children × rounds is small.
            for child_id in child_ids {
                // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != NodeKind::RootJoin {
                    continue;
                }
                for ei in news.iter() {
                    let el = wm.alpha[node_id][ei];
                    // Seed native Token: one matches edge (fact idx, alpha_id).
                    let binds = if el.binds.len > 0 {
                        seed_token_binds(&el)
                    } else {
                        span_from_row(
                            &mut wm.bind_pool,
                            &el,
                            *node_id,
                            &wm.i64_by_fact,
                            &wm.bind_only,
                            &wm.cond_key_ids,
                        )
                    };
                    let tok = Token {
                        matches: push_match(&mut wm.match_pool, el.fact, *node_id),
                        binds,
                    };
                    if beta_readers.contains(child_id) {
                        beta_written(*child_id, 1);
                        wm.beta.entry(*child_id).or_default().push(tok);
                    }
                    d_beta.entry(*child_id).or_default().push(tok);
                }
            }
        }

        phase_end("root-join", __pt1);

        // ── 3. Hash-join delta (ascending id — topological). ─────────────────────
        let __pt2 = phase_start();
        // P6 persistent-index algorithm (DESIGN-STONE-P6, 6-step ordering):
        //
        // For each parent P (Root/HashJoin) with HashJoinNode child J (feeding alpha A):
        //   dl = d_beta[P]  (Δleft:  tokens new this round at P)
        //   dr = d_alpha[A] (Δright: elements new this round at A)
        //
        //   Step 2: add dr → right_idx[J]   (right_idx now holds ALL right incl. this round's)
        //   Step 3: term1 = Δleft ⋈ all_right   (probe right_idx[J] with dl)
        //   Step 4: term2 = old_left ⋈ Δright   (probe left_idx[J] — still OLD — with dr)
        //   Step 5: add dl → left_idx[J]    (AFTER term2: left_idx now holds ALL left incl. this round's)
        //   Step 6: new tokens → wm.beta[J] + d_beta[J]
        //
        // Invariant: (Δleft×Δright) appears in term1 only (right_idx already has Δright at step 3);
        //            old_left×Δright appears in term2 only (left_idx lacks Δleft at step 4).
        //            No double-count, no miss — same semi-naive result as the keyed_join rebuild.
        // Dirty join-parents only (`DESIGN-STONE-dirty-join-parents`): left d_beta
        // or a HashJoin child whose feeding alpha has d_alpha. First-keying runs
        // the round the second side arrives (that delta is non-empty). Grow the
        // set as we emit so a middle join (J1→J2) is visited this round.
        let mut dirty_parents = seed_dirty_join_parents(
            &kind_ids.join_parent,
            &d_beta,
            &d_alpha,
            &packed_full,
            &arm.joins_fed_by,
            parents_of,
        );
        for node_id in &kind_ids.join_parent {
            if !dirty_parents.contains(node_id) {
                continue;
            }
            // kind_ids.join_parent already filtered; children interned so fire
            // does not re-scan names (`InternedNetwork.children_of`).
            let child_ids: &[i64] = arm
                .children_of
                .get(node_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            for child_id in child_ids {
                // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != NodeKind::HashJoin {
                    continue;
                }
                let alpha_id = feeding_alpha_of.get(child_id).copied().unwrap_or(-1);

                // Step 1: Ensure join_keys[J] is cached.
                // Compute from a sample token at P and a sample element at A (if both exist).
                // first_keying=true means J was previously skipped while one side was empty;
                // a one-time catch-up full-join is required to populate right_idx[J] from ALL
                // cumulative wm.alpha[alpha_id] (not just the current round's dr).
                let first_keying = if !join_keys_cache.contains_key(child_id) {
                    let sample_tok = wm.beta.get(node_id).and_then(|v| v.first());
                    // READ #1 of 2: one sample token, to derive this join's keys.
                    if sample_tok.is_some() {
                        beta_read(*node_id, 1);
                    }
                    let sample_el = wm.alpha.get(&alpha_id).and_then(|v| v.first());
                    match (sample_tok, sample_el) {
                        (Some(tok), Some(el)) => {
                            let keys = gather_join_keys(
                                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                std::slice::from_ref(el),
                                GatherIntern::from_wm(&wm, alpha_id),
                            );
                            join_keys_cache.insert(*child_id, keys);
                            true // first keying: catch-up full-join needed
                        }
                        _ => {
                            // Neither side has data yet — skip this node for this round.
                            // The join_keys will be computed next round when both sides are populated.
                            continue;
                        }
                    }
                } else {
                    false
                };

                // Group C: borrow join_keys (pointer bump) instead of cloning (Vec alloc + copy).
                let jk: &[Value] = &join_keys_cache[child_id];

                // CATCH-UP (first keying only): J was skipped every prior round while one side
                // was empty, so right_idx[J] was never populated from those rounds' facts.
                // Rebuild from ALL cumulative wm.alpha[alpha_id] and wm.beta[parent], cross-join
                // fully, and build both indexes. Safe: J produced ZERO tokens before first keying
                // so there is nothing to double-count. On subsequent rounds the incremental
                // semi-naive path (steps 2–5 below) handles new arrivals correctly.
                //
                // Note: at this point in the round, steps 1 (alpha delta) and 2 (root-join delta)
                // have ALREADY run, so wm.alpha and wm.beta contain ALL cumulative data including
                // this round's new elements — the catch-up covers historical AND current-round facts.
                if first_keying {
                    // Occupancy is already Arc-shared. Bump the Arc; do not memcpy
                    // the Vec (`DESIGN-STONE-catchup-arc-occupancy`). all_left still
                    // clones wm.beta (HashMap split-borrow, not Arc occupancy).
                    let all_right = wm.alpha.get(&alpha_id).cloned();
                    let n_right = all_right.as_ref().map(|v| v.len()).unwrap_or(0);
                    let all_left: Vec<Token> = wm.beta.get(node_id).cloned().unwrap_or_default();
                    // READ #2 of 2: the parent's cumulative tokens, for the catch-up cross-join.
                    beta_read(*node_id, all_left.len() as u64);
                    // Key from packed occupancy (empty binds), then write BindSpan
                    // onto the indexed copy (`DESIGN-STONE-join-index-span`).
                    // Keying after materialize used the binds-path JoinKey, which
                    // missed token probes (7b/7exists/8b native=0).
                    let __cri = phase_start();
                    {
                        let ridx = right_idx.entry(*child_id).or_default();
                        if let Some(right) = all_right.as_deref() {
                            for &el in right {
                                let k = key_of_el(&el, jk, &GatherIntern::from_wm(&wm, alpha_id));
                                let el = element_with_row_span(
                                    el,
                                    &mut wm.bind_pool,
                                    alpha_id,
                                    &wm.i64_by_fact,
                                    &wm.bind_only,
                                    &wm.cond_key_ids,
                                );
                                ridx.entry(k).or_default().push(el);
                            }
                        }
                    }
                    phase_end("  ├ hj:catchup:right-idx", __cri);
                    // Reserve the 40k appends. Isolated unreserved extend paid
                    // G−E = 4.13 ms (`DESIGN-STONE-probe-gap-split`).
                    let n_join = match right_idx.get(child_id) {
                        Some(idx) if !idx.is_empty() && n_right > 0 => {
                            all_left.len().saturating_mul(n_right / idx.len())
                        }
                        _ => 0,
                    };
                    wm.bind_pool.reserve(n_join.saturating_mul(4));
                    wm.match_pool.reserve(n_join.saturating_mul(2));
                    // Full cross-join: every left token keyed against right_idx[J].
                    let __cpr = phase_start();
                    let mut new_tokens: Vec<Token> = Vec::with_capacity(n_join);
                    if let Some(ridx) = right_idx.get(child_id) {
                        for tok in &all_left {
                            let k = key_of(
                                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                jk,
                                &wm.bind_val_ids,
                            );
                            if let Some(bucket) = ridx.get(&k) {
                                for el in bucket {
                                    if let Some(new_tok) = join_extend(
                                        tok,
                                        el,
                                        alpha_id,
                                        &mut FireCtx {
                                            compiled_conds,
                                            scratch: &mut match_scratch,
                                            pool: &mut wm.bind_pool,
                                            match_pool: &mut wm.match_pool,
                                            keys: &wm.bind_keys,
                                            vals: &wm.bind_vals,
                                            val_ids: &wm.bind_val_ids,
                                            facts: &wm.facts,
                                            derived: &wm.derived_facts,
                                            n_input: wm.n_input,
                                            i64_by_fact: &wm.i64_by_fact,
                                            bind_only: &wm.bind_only,
                                            cond_key_ids: &wm.cond_key_ids,
                                        },
                                    )? {
                                        new_tokens.push(new_tok);
                                    }
                                }
                            }
                        }
                    }
                    phase_end("  ├ hj:catchup:probe", __cpr);
                    // Build left_idx[J] from ALL cumulative left tokens.
                    let __cli = phase_start();
                    {
                        let lidx = left_idx.entry(*child_id).or_default();
                        for tok in all_left {
                            let k = key_of(
                                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                jk,
                                &wm.bind_val_ids,
                            );
                            lidx.entry(k).or_default().push(tok);
                        }
                    }
                    phase_end("  ├ hj:catchup:left-idx", __cli);
                    // Emit catch-up tokens into cumulative and delta memories.
                    let __cem = phase_start();
                    // `entry()` HOISTED out of the per-token loop: the key is constant, so the
                    // old form paid two map lookups per token (80,000 on the fanout cell) where
                    // two total will do. Correct regardless of the guard below.
                    if beta_readers.contains(child_id) {
                        beta_written(*child_id, new_tokens.len() as u64);
                        let beta = wm.beta.entry(*child_id).or_default();
                        beta.reserve(new_tokens.len());
                        for t in &new_tokens {
                            beta.push(*t);
                        }
                    }
                    let n_emit = new_tokens.len();
                    let delta = d_beta.entry(*child_id).or_default();
                    delta.reserve(n_emit);
                    for new_tok in new_tokens {
                        delta.push(new_tok);
                    }
                    if n_emit > 0 {
                        dirty_parents.insert(*child_id);
                    }
                    phase_end("  ├ hj:catchup:emit", __cem);
                    continue; // Skip incremental steps 2–5 for this round.
                }

                // Group C: borrow dl; packed seed dr is 0..len, not a Vec
                // (`DESIGN-STONE-seed-d-alpha-range`). NLL ends these borrows
                // at their last use (step 5), before step 6 mutates d_beta.
                let dl: &[Token] = d_beta.get(node_id).map(Vec::as_slice).unwrap_or_default();
                let dr = AlphaNews::of(&d_alpha, &wm.alpha, alpha_id, &packed_full);

                // Skip if nothing new on either side.
                if dl.is_empty() && dr.is_empty() {
                    continue;
                }

                // Step 2: add Δright (dr) to right_idx[J] FIRST.
                // dr is indices into wm.alpha[A]; right_idx still owns Elements (P6).
                // Span once onto the indexed copy (`DESIGN-STONE-join-index-span`).
                let __s2 = phase_start();
                {
                    let ridx = right_idx.entry(*child_id).or_default();
                    let right_mem = wm.alpha.get(&alpha_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    for ei in dr.iter() {
                        let el = right_mem[ei];
                        let k = key_of_el(&el, jk, &GatherIntern::from_wm(&wm, alpha_id));
                        let el = element_with_row_span(
                            el,
                            &mut wm.bind_pool,
                            alpha_id,
                            &wm.i64_by_fact,
                            &wm.bind_only,
                            &wm.cond_key_ids,
                        );
                        ridx.entry(k).or_default().push(el);
                    }
                }
                phase_end("  ├ hj:step2-right-idx", __s2);

                // Step 3: term1 = Δleft ⋈ all_right (probe right_idx[J] — now includes Δright).
                // The mutable borrow from step 2 ended with that scope block; safe to borrow immutably.
                let __s3 = phase_start();
                let mut new_tokens: Vec<Token> = Vec::new();
                if !dl.is_empty() {
                    if let Some(ridx) = right_idx.get(child_id) {
                        for tok in dl {
                            let k = key_of(
                                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                jk,
                                &wm.bind_val_ids,
                            );
                            if let Some(bucket) = ridx.get(&k) {
                                for el in bucket {
                                    if let Some(new_tok) = join_extend(
                                        tok,
                                        el,
                                        alpha_id,
                                        &mut FireCtx {
                                            compiled_conds,
                                            scratch: &mut match_scratch,
                                            pool: &mut wm.bind_pool,
                                            match_pool: &mut wm.match_pool,
                                            keys: &wm.bind_keys,
                                            vals: &wm.bind_vals,
                                            val_ids: &wm.bind_val_ids,
                                            facts: &wm.facts,
                                            derived: &wm.derived_facts,
                                            n_input: wm.n_input,
                                            i64_by_fact: &wm.i64_by_fact,
                                            bind_only: &wm.bind_only,
                                            cond_key_ids: &wm.cond_key_ids,
                                        },
                                    )? {
                                        new_tokens.push(new_tok);
                                    }
                                }
                            }
                        }
                    }
                }
                phase_end("  ├ hj:step3-term1", __s3);

                // Step 4: term2 = old_left ⋈ Δright (probe left_idx[J] — still OLD, Δleft not yet added).
                // left_idx is a separate map from right_idx; no aliasing — safe immutable borrow.
                let __s4 = phase_start();
                if !dr.is_empty() {
                    if let Some(lidx) = left_idx.get(child_id) {
                        let right_mem = wm.alpha.get(&alpha_id).map(|v| v.as_slice()).unwrap_or(&[]);
                        for ei in dr.iter() {
                            let el = right_mem[ei];
                            let k = key_of_el(&el, jk, &GatherIntern::from_wm(&wm, alpha_id));
                            let el = element_with_row_span(
                                el,
                                &mut wm.bind_pool,
                                alpha_id,
                                &wm.i64_by_fact,
                                &wm.bind_only,
                                &wm.cond_key_ids,
                            );
                            if let Some(bucket) = lidx.get(&k) {
                                for tok in bucket {
                                    if let Some(new_tok) = join_extend(
                                        tok,
                                        &el,
                                        alpha_id,
                                        &mut FireCtx {
                                            compiled_conds,
                                            scratch: &mut match_scratch,
                                            pool: &mut wm.bind_pool,
                                            match_pool: &mut wm.match_pool,
                                            keys: &wm.bind_keys,
                                            vals: &wm.bind_vals,
                                            val_ids: &wm.bind_val_ids,
                                            facts: &wm.facts,
                                            derived: &wm.derived_facts,
                                            n_input: wm.n_input,
                                            i64_by_fact: &wm.i64_by_fact,
                                            bind_only: &wm.bind_only,
                                            cond_key_ids: &wm.cond_key_ids,
                                        },
                                    )? {
                                        new_tokens.push(new_tok);
                                    }
                                }
                            }
                        }
                    }
                }
                phase_end("  ├ hj:step4-term2", __s4);

                // Step 5: add Δleft (dl) to left_idx[J] AFTER term2 (no-double-count invariant).
                // dl is &[Token] — iterate directly.
                let __s5 = phase_start();
                {
                    let lidx = left_idx.entry(*child_id).or_default();
                    for tok in dl {
                        let k = key_of(
                            &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                            jk,
                            &wm.bind_val_ids,
                        );
                        lidx.entry(k).or_default().push(*tok);
                    }
                }
                phase_end("  ├ hj:step5-left-idx", __s5);

                // Step 6: push new tokens to wm.beta[J] and d_beta[J].
                let __s6 = phase_start();
                // Same hoist + guard as the catch-up emit above.
                if beta_readers.contains(child_id) {
                    beta_written(*child_id, new_tokens.len() as u64);
                    let beta = wm.beta.entry(*child_id).or_default();
                    beta.reserve(new_tokens.len());
                    for t in &new_tokens {
                        beta.push(*t);
                    }
                }
                let n_emit = new_tokens.len();
                let delta = d_beta.entry(*child_id).or_default();
                delta.reserve(n_emit);
                for new_tok in new_tokens {
                    delta.push(new_tok);
                }
                if n_emit > 0 {
                    dirty_parents.insert(*child_id);
                }
                phase_end("  ├ hj:step6-emit", __s6);
            }
        }

        phase_end("hash-join", __pt2);

        // ── 3.25 Accumulate-pass (8-b): dispatch AccumulateNode. ────────────────
        let __pt3 = phase_start();
        // For each AccumulateNode (topological = ascending id order): for each NEW token
        // at the parent (d_beta[parent]), gather the token-compatible elements from the
        // FULL cumulative wm.alpha[from_alpha_id] (the aggregate needs all matching facts,
        // like 7-b negation), compute the aggregate in Rust (mirroring the wat acc::* folds),
        // and — if a value — extend the token with result-var → aggregate and push to
        // wm.beta[acc] (cumulative) + d_beta[acc] (new-this-round, consumed downstream).
        // min/max/mean on an empty gather → no value → drop the token.
        // Runs BEFORE the filter-pass so a :where on the result-var sees the binding.
        for node_id in &kind_ids.acc {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(node) != NodeKind::Accumulate {
                continue;
            }
            let Some(result_var) = node_named_field(node, "result-var")
                .cloned()
                .filter(|v| matches!(v, Value::String(_)))
            else {
                continue;
            };
            let Some(acc_fold) = compiled_acc_folds.get(node_id) else {
                return Err(RuntimeError::new(
                    crate::rust_caller_span!(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::rete::fire-rules".into(),
                        reason: format!(
                            "AccumulateNode {node_id} has no compiled fold — setup should have compiled it"
                        ),
                    },
                )
                .into());
            };
            let Some(from_alpha_id) = node_named_i64(node, "from-alpha-id") else {
                continue;
            };
            // NEW tokens at EVERY parent (clone to avoid the d_beta read/write borrow conflict).
            // Leading accumulate (Clara test-count): no parent — seed one empty token.
            // count/sum emit 0 on empty gather; min/max/mean drop the token.
            let pids = parents_of.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let mut new_tokens: Vec<Token> = d_beta_from_parents(parents_of, &d_beta, *node_id);
            if new_tokens.is_empty() && pids.is_empty() {
                new_tokens = vec![Token {
                    matches: empty_span(),
                    binds: empty_span(),
                }];
            }
            if new_tokens.is_empty() {
                continue;
            }
            // Derive the join-key tuple first (cheap: elements[0] + a sample-bindings
            // intersection) so the cache can be probed BEFORE paying for a snapshot clone or an
            // index build. Reads wm.alpha through a borrow, no clone yet.
            let __ix = phase_start();
            let empty_index = GatherIndex::Nary(FxHashMap::default());
            let empty_keys: Arc<[Value]> = Arc::from([]);
            let gathered = ensure_gather(
                &mut gather_cache,
                &wm,
                from_alpha_id,
                &bind_view(
                    &wm.bind_keys,
                    &wm.bind_vals,
                    &wm.bind_pool,
                    new_tokens[0].binds,
                ),
            );
            phase_end("  ├ accum:index", __ix);
            // Empty :from is not cached (unsampled [] ≠ cartesian []). Acc still
            // walks grouping: ungrouped empty emits identity; grouped empty does not.
            let (index, join_keys) = match gathered.as_ref() {
                Some((idx, keys)) => (*idx, keys),
                None => (&empty_index, &empty_keys),
            };
            // No clone — indices name this round's wm.alpha[id] (step 1 is done).
            let __sn = phase_start();
            let from_elements = alpha_elements(&wm.alpha, from_alpha_id);
            phase_end("  ├ accum:snapshot", __sn);
            let from_compiled = rematch_compiled(compiled_conds, from_alpha_id)?;
            let leftover = from_compiled.has_seed_cmp();
            let from_keys = from_compiled.bind_keys();
            let operand_keys = acc_fold.operand_keys();
            let col_keys = from_compiled.slot_keys();
            let empty_fields: &[u8] = &[];
            let col_fields = wm
                .bind_only
                .get(&from_alpha_id)
                .map(Vec::as_slice)
                .unwrap_or(empty_fields);
            let __fd = phase_start();
            for tok in new_tokens {
                let key = key_of(
                    &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                    join_keys.as_ref(),
                    &wm.bind_val_ids,
                );
                let bucket: &[usize] = index.bucket(&key);
                let group_keys: Vec<Value> = from_keys
                    .iter()
                    .filter(|k| {
                        Bindings::get(
                            &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                            k,
                        )
                        .is_none()
                            && !operand_keys.iter().any(|o| o == *k)
                    })
                    .cloned()
                    .collect();
                // No leftover SeedCmp: the keyed bucket IS the gather (keyed-gather
                // contract). Rematch cannot reject a member or bind anything the
                // Element does not already hold. Count is len; value folds read a slot.
                if !leftover && group_keys.is_empty() {
                    if let Some(aggregate) = fold_bucket(
                        acc_fold,
                        from_elements,
                        bucket,
                        sym,
                        &acc_view(&wm, col_keys, col_fields),
                    )? {
                        let new_tok = token_assoc(
                            &tok,
                            result_var.clone(),
                            aggregate,
                            &mut BindIntern {
                                keys: &mut wm.bind_keys,
                                vals: &mut wm.bind_vals,
                                ids: &mut wm.bind_val_ids,
                                pool: &mut wm.bind_pool,
                            },
                        );
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(new_tok);
                        }
                        d_beta.entry(*node_id).or_default().push(new_tok);
                    }
                    continue;
                }
                // Gather the token-compatible :from elements (shared ?var agreement), in
                // alpha-memory insertion order (matches the wat foldl over from-els) — the
                // bucket's indices were pushed in that same order.
                let mut gathered: Vec<&Element> = Vec::new();
                if leftover {
                    for &i in bucket {
                        let el = &from_elements[i];
                        census_gather_visit();
                        let ok = fact_holds_under(
                            fact_at(&wm.facts, &wm.derived_facts, wm.n_input, el.fact),
                            &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                            from_compiled,
                            &mut match_scratch,
                        );
                        if ok {
                            gathered.push(el);
                        }
                    }
                } else {
                    gathered.extend(bucket.iter().map(|&i| &from_elements[i]));
                }
                // One fold of the whole gather when the token already holds every
                // `:from` bind (or the `:from` binds none). Otherwise group by the
                // leftover binds; empty gather + leftover keys is not a bag-wide 0.
                let groups: AccGroupOrder<'_> = if group_keys.is_empty() {
                    vec![(
                        pmap_from_span(tok.binds, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool),
                        gathered,
                    )]
                } else if gathered.is_empty() {
                    Vec::new()
                } else {
                    let mut order: Vec<AccGroupKey> = Vec::new();
                    let mut buckets: AccGroupBuckets<'_> = HashMap::new();
                    for el in gathered {
                        let el_b =
                            element_fact_bindings(el, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool);
                        let proj = project_group_keys(&el_b, &group_keys);
                        buckets
                            .entry(proj.clone())
                            .or_insert_with(|| {
                                order.push(proj);
                                Vec::new()
                            })
                            .push(el);
                    }
                    order
                        .into_iter()
                        .map(|proj| {
                            let mut nb = pmap_from_span(
                                tok.binds,
                                &wm.bind_keys,
                                &wm.bind_vals,
                                &wm.bind_pool,
                            );
                            for (k, v) in &proj {
                                nb = nb.assoc(k.clone(), v.clone());
                            }
                            let els = buckets.remove(&proj).unwrap_or_default();
                            (nb, els)
                        })
                        .collect()
                };
                for (group_bindings, group_els) in groups {
                    if let Some(aggregate) = accumulate_value(
                        acc_fold,
                        &group_els,
                        sym,
                        &acc_view(&wm, col_keys, col_fields),
                    )? {
                        let new_bindings = group_bindings.assoc(result_var.clone(), aggregate);
                        let new_tok = Token {
                            matches: tok.matches,
                            binds: span_from_pairs(
                                &mut BindIntern {
                                    keys: &mut wm.bind_keys,
                                    vals: &mut wm.bind_vals,
                                    ids: &mut wm.bind_val_ids,
                                    pool: &mut wm.bind_pool,
                                },
                                new_bindings.iter().map(|(k, v)| (k.clone(), v.clone())),
                            ),
                        };
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(new_tok);
                        }
                        d_beta.entry(*node_id).or_default().push(new_tok);
                    }
                }
            }
            phase_end("  └ accum:fold", __fd);
        }

        phase_end("accumulate", __pt3);

        // ── 3.5 Filter-pass: dispatch TestNode, NegationNode, ExistsNode. ─────
        let __pt4 = phase_start();
        // For each TestNode, NegationNode, or ExistsNode (ascending id order):
        //   TestNode     → eval-test filter: pass the token iff expr evaluates true.
        //   NegationNode → negation filter: pass the un-extended token iff ZERO elements in
        //                  wm.alpha[neg_alpha_id] (the FULL cumulative alpha-memory) are
        //                  token-element-compatible with the token's bindings.
        //   ExistsNode   → existence filter: pass iff ANY compatible element; leading exists
        //                  seeds one token per distinct inner binding (no parent).
        // New tokens still come from d_beta[parent] (the delta); only the absence/presence
        // check reads the full wm.alpha (populated in step 1 before this pass).
        // Passing tokens are pushed to wm.beta[node_id] (cumulative) and d_beta[node_id]
        // (new-this-round, consumed by production in step 4).
        // rune:temperare(simplicity-win) — 3.7 still get_node+node_children; 3.6 already
        // walks arm.children_of. n HashJoin×filter descendants is small vs intern hoist.
        let mut tests_done: HashSet<i64> = HashSet::new();
        for node_id in &kind_ids.filter {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            let kind = kind_of(node);
            if kind != NodeKind::Test && kind != NodeKind::Negation && kind != NodeKind::Exists {
                continue;
            }
            // Clone the new-this-round tokens at EVERY parent to avoid a simultaneous
            // borrow conflict (reading d_beta[parent] while writing d_beta[*node_id]).
            // A Test/:not/:exists after condition `:or` has N parents.
            let pids = parents_of.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let mut new_tokens: Vec<Token> = d_beta_from_parents(parents_of, &d_beta, *node_id);
            // Leading :not has no parent — Clara matches the empty world with one
            // empty-binding token. Do not seed when parents exist but produced nothing.
            if pids.is_empty() && kind == NodeKind::Negation {
                new_tokens = vec![Token {
                    matches: empty_span(),
                    binds: empty_span(),
                }];
            }
            // Leading :exists: one token per DISTINCT inner binding (Clara
            // test-simple-exists — two Winds at MCI → one {?loc MCI}), not an
            // empty seed. Mid-chain exists still filters parent tokens below.
            if pids.is_empty() && kind == NodeKind::Exists {
                let Some(alpha_id) = node_ref_alpha_id(node) else {
                    continue;
                };
                let driver = driver_of(compiled_drivers, alpha_id)?;
                let mut seen = std::collections::HashSet::new();
                if matches!(driver, CondDriver::Leaf(_)) {
                    let els: Vec<Element> = wm
                        .alpha
                        .get(&alpha_id)
                        .map(|v| v.as_ref().clone())
                        .unwrap_or_default();
                    // rune:perspicere(read-once) — one leaf; Clara test-simple-exists distinct inner binds; alias would be a mumble
                    let candidates: Vec<(BindSpan, Vec<(u32, u32)>)> = els
                        .iter()
                        .map(|el| {
                            let binds = if el.binds.len > 0 {
                                el.binds
                            } else {
                                span_from_row(
                                    &mut wm.bind_pool,
                                    el,
                                    alpha_id,
                                    &wm.i64_by_fact,
                                    &wm.bind_only,
                                    &wm.cond_key_ids,
                                )
                            };
                            (binds, pool_slice(&wm.bind_pool, binds).to_vec())
                        })
                        .collect();
                    // rune:perspicere(read-once) — content-keyed distinct set for this leaf
                    // rune:temperare(simplicity-win) — distinct inner bindings require a
                    // content-keyed set of already-interned (u32,u32) pairs (Clara test-simple-exists)
                    let mut seen_pairs: HashSet<Vec<(u32, u32)>> = HashSet::new(); // rune:perspicere(read-once) — one leaf; a name would be a mumble
                    for (binds, pairs) in candidates {
                        if !seen_pairs.insert(pairs) {
                            continue;
                        }
                        let tok = Token {
                            matches: empty_span(),
                            binds,
                        };
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(tok);
                        }
                        d_beta.entry(*node_id).or_default().push(tok);
                    }
                    continue;
                }
                let empty = crate::value::pmap::PMap::new();
                let exts = binding_extensions(
                    driver,
                    &wm,
                    &empty,
                    compiled_conds,
                    &mut match_scratch,
                    sym,
                    &mut gather_cache,
                )?;
                for ext in exts {
                    if !seen.insert(ext.clone()) {
                        continue;
                    }
                    let tok = Token {
                        matches: empty_span(),
                        binds: span_from_pairs(
                            &mut BindIntern {
                                keys: &mut wm.bind_keys,
                                vals: &mut wm.bind_vals,
                                ids: &mut wm.bind_val_ids,
                                pool: &mut wm.bind_pool,
                            },
                            ext.iter().map(|(k, v)| (k.clone(), v.clone())),
                        ),
                    };
                    if beta_readers.contains(node_id) {
                        beta_written(*node_id, 1);
                        wm.beta.entry(*node_id).or_default().push(tok);
                    }
                    d_beta.entry(*node_id).or_default().push(tok);
                }
                continue;
            }
            if new_tokens.is_empty() {
                continue;
            }
            if kind == NodeKind::Test {
                if tests_done.contains(node_id) {
                    continue;
                }
                // DESIGN-STONE-compiled-where Step 0 — capture the FIRST (expr, tokens) this loop
                // handles. Census only; production never reads `:expr`.
                #[cfg(test)]
                if let Some(ast) = node_named_ast(node, "expr") {
                    capture_where_sample(
                        ast,
                        &new_tokens,
                        &wm.bind_keys,
                        &wm.bind_vals,
                        &wm.bind_pool,
                    );
                }
                // Siblings that share this TestNode's parent set see the same token
                // stream — dispatch once through the interned where-tree groups.
                let sibs: &[i64] = test_sibs_of
                    .get(node_id)
                    .map(|v| v.as_slice())
                    .unwrap_or(std::slice::from_ref(node_id));
                dispatch_where_tests(
                    sibs,
                    &new_tokens,
                    &mut WhereSink {
                        where_tree,
                        compiled_wheres,
                        beta_readers,
                        wm: &mut wm,
                        d_beta: &mut d_beta,
                        sym,
                    },
                )?;
                tests_done.extend(sibs);
            } else {
                // NegationNode / ExistsNode: fire reads named fields via node_ref_alpha_id.
                // Same gather as Acc: probe gather_cache for the token's join-key bucket.
                // Verdict inverts by kind: NegationNode passes iff ZERO compatible, ExistsNode
                // iff ≥1. The index is over FULL cumulative wm.alpha (step 1 ran first).
                // ExistsNode binds nothing and passes the token at most ONCE (no multiplicity).
                let is_exists = kind == NodeKind::Exists;
                let Some(alpha_id) = node_ref_alpha_id(node) else {
                    continue;
                };
                let driver = driver_of(compiled_drivers, alpha_id)?;
                for tok in new_tokens {
                    let any_compat = token_exists_under(
                        driver,
                        &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                        &wm,
                        compiled_conds,
                        &mut match_scratch,
                        sym,
                        &mut gather_cache,
                    )?;
                    // ExistsNode passes iff any-compat; NegationNode passes iff NOT any-compat.
                    let pass = if is_exists { any_compat } else { !any_compat };
                    if pass {
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(tok);
                        }
                        d_beta.entry(*node_id).or_default().push(tok);
                    }
                }
            }
        }

        phase_end("filter", __pt4);

        // ── 3.6 Join-after-filter (A1): HashJoin children of Test/Neg/Exists/Accum. ─
        // The P6 loop above only left-activates from Root/HashJoin. Compile will parent
        // a HashJoin on a mid-chain :where (Clara does; Join → Test → Join). Filter just
        // filled d_beta[test]; push those tokens across the next join. keyed_join against
        // the full alpha is the catch-up: this child produced nothing in step 3, so there
        // is nothing to double-count.
        let __pt36 = phase_start();
        let mut after_join_frontier: Vec<i64> = Vec::new();
        for node_id in &kind_ids.filter_or_acc {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            let kind = kind_of(node);
            if kind != NodeKind::Test
                && kind != NodeKind::Negation
                && kind != NodeKind::Exists
                && kind != NodeKind::Accumulate
            {
                continue;
            }
            let new_tokens: Vec<Token> = match d_beta.get(node_id) {
                Some(ts) if !ts.is_empty() => ts.clone(),
                _ => continue,
            };
            let child_ids: &[i64] = arm
                .children_of
                .get(node_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            for child_id in child_ids {
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != NodeKind::HashJoin {
                    continue;
                }
                let alpha_id = feeding_alpha_of.get(child_id).copied().unwrap_or(-1);
                let elements = match wm.alpha.get(&alpha_id) {
                    Some(els) if !els.is_empty() => els.as_slice(),
                    _ => continue,
                };
                let joined = keyed_join_persistent(
                    &new_tokens,
                    elements,
                    alpha_id,
                    *child_id,
                    &mut FilterJoinIdx {
                        right_idx: &mut right_idx,
                        join_keys_cache: &mut join_keys_cache,
                        indexed_n: &mut right_idx_n,
                    },
                    &mut FireCtx {
                        compiled_conds,
                        scratch: &mut match_scratch,
                        pool: &mut wm.bind_pool,
                        match_pool: &mut wm.match_pool,
                        keys: &wm.bind_keys,
                        vals: &wm.bind_vals,
                        val_ids: &wm.bind_val_ids,
                        facts: &wm.facts,
                        derived: &wm.derived_facts,
                        n_input: wm.n_input,
                        i64_by_fact: &wm.i64_by_fact,
                        bind_only: &wm.bind_only,
                        cond_key_ids: &wm.cond_key_ids,
                    },
                )?;
                if joined.is_empty() {
                    continue;
                }
                if beta_readers.contains(child_id) {
                    beta_written(*child_id, joined.len() as u64);
                    wm.beta
                        .entry(*child_id)
                        .or_default()
                        .extend(joined.iter().cloned());
                }
                d_beta.entry(*child_id).or_default().extend(joined);
                after_join_frontier.push(*child_id);
            }
        }
        phase_end("join-after-filter", __pt36);

        // ── 3.7 Filter-after-join: Test/Neg/Exists whose parent just got tokens
        // in 3.6 (trailing `:where` after a mid-chain `:where` + join). A1 only
        // left-activated HashJoin children of a Test. The trailing Test is a
        // *child* of that HashJoin; the first filter pass already finished
        // before 3.6 wrote d_beta[join]. Spec's topo emit covers it; native
        // must too. Loop: a Test may itself parent another HashJoin.
        let __pt37 = phase_start();
        let mut frontier = after_join_frontier;
        while !frontier.is_empty() {
            let mut next_frontier: Vec<i64> = Vec::new();
            for hj_id in frontier {
                let hj_node = match get_node(&wm.network, hj_id) {
                    Some(n) => n,
                    None => continue,
                };
                let filter_kids = node_children(hj_node);
                let mut tests_dispatched = false;
                for filter_id in filter_kids.iter().copied() {
                    let filter_node = match get_node(&wm.network, filter_id) {
                        Some(n) => n,
                        None => continue,
                    };
                    let fkind = kind_of(filter_node);
                    if fkind != NodeKind::Test
                        && fkind != NodeKind::Negation
                        && fkind != NodeKind::Exists
                    {
                        continue;
                    }
                    let new_tokens: Vec<Token> = match d_beta.get(&hj_id) {
                        Some(ts) if !ts.is_empty() => ts.clone(),
                        _ => continue,
                    };
                    if fkind == NodeKind::Test {
                        if !tests_dispatched {
                            let empty: Vec<i64> = Vec::new();
                            let test_sibs = test_children.get(&hj_id).unwrap_or(&empty);
                            dispatch_where_tests(
                                test_sibs,
                                &new_tokens,
                                &mut WhereSink {
                                    where_tree,
                                    compiled_wheres,
                                    beta_readers,
                                    wm: &mut wm,
                                    d_beta: &mut d_beta,
                                    sym,
                                },
                            )?;
                            tests_dispatched = true;
                        }
                    } else {
                        let is_exists = fkind == NodeKind::Exists;
                        let Some(alpha_id) = node_ref_alpha_id(filter_node) else {
                            continue;
                        };
                        if new_tokens.is_empty() {
                            continue;
                        }
                        let driver = driver_of(compiled_drivers, alpha_id)?;
                        for tok in new_tokens {
                            let any_compat = token_exists_under(
                                driver,
                                &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                                &wm,
                                compiled_conds,
                                &mut match_scratch,
                                sym,
                                &mut gather_cache,
                            )?;
                            let pass = if is_exists { any_compat } else { !any_compat };
                            if pass {
                                if beta_readers.contains(&filter_id) {
                                    beta_written(filter_id, 1);
                                    wm.beta.entry(filter_id).or_default().push(tok);
                                }
                                d_beta.entry(filter_id).or_default().push(tok);
                            }
                        }
                    }
                    // Walk children of this filter: HashJoin (3.6's grandchild) AND
                    // Test/Neg/Exists (Test→Test after join-after-filter — spoken
                    // two-temps: filter, join, filter, filter).
                    let mut chain: Vec<i64> = vec![filter_id];
                    while let Some(fid) = chain.pop() {
                        let fnode = match get_node(&wm.network, fid) {
                            Some(n) => n,
                            None => continue,
                        };
                        let parent_toks: Vec<Token> = match d_beta.get(&fid) {
                            Some(ts) if !ts.is_empty() => ts.clone(),
                            _ => continue,
                        };
                        let kids = node_children(fnode);
                        let empty: Vec<i64> = Vec::new();
                        let test_sibs = test_children.get(&fid).unwrap_or(&empty);
                        if !test_sibs.is_empty() {
                            dispatch_where_tests(
                                test_sibs,
                                &parent_toks,
                                &mut WhereSink {
                                    where_tree,
                                    compiled_wheres,
                                    beta_readers,
                                    wm: &mut wm,
                                    d_beta: &mut d_beta,
                                    sym,
                                },
                            )?;
                            chain.extend(test_sibs);
                        }
                        for gc_id in kids {
                            let gc = match get_node(&wm.network, gc_id) {
                                Some(n) => n,
                                None => continue,
                            };
                            let gkind = kind_of(gc);
                            if gkind == NodeKind::Test {
                                continue;
                            }
                            if gkind == NodeKind::HashJoin {
                                let alpha_id = feeding_alpha_of.get(&gc_id).copied().unwrap_or(-1);
                                let elements = match wm.alpha.get(&alpha_id) {
                                    Some(els) if !els.is_empty() => els.as_slice(),
                                    _ => continue,
                                };
                                let joined = keyed_join_persistent(
                                    &parent_toks,
                                    elements,
                                    alpha_id,
                                    gc_id,
                                    &mut FilterJoinIdx {
                                        right_idx: &mut right_idx,
                                        join_keys_cache: &mut join_keys_cache,
                                        indexed_n: &mut right_idx_n,
                                    },
                                    &mut FireCtx {
                                        compiled_conds,
                                        scratch: &mut match_scratch,
                                        pool: &mut wm.bind_pool,
                                        match_pool: &mut wm.match_pool,
                                        keys: &wm.bind_keys,
                                        vals: &wm.bind_vals,
                                        val_ids: &wm.bind_val_ids,
                                        facts: &wm.facts,
                                        derived: &wm.derived_facts,
                                        n_input: wm.n_input,
                                        i64_by_fact: &wm.i64_by_fact,
                                        bind_only: &wm.bind_only,
                                        cond_key_ids: &wm.cond_key_ids,
                                    },
                                )?;
                                if joined.is_empty() {
                                    continue;
                                }
                                if beta_readers.contains(&gc_id) {
                                    beta_written(gc_id, joined.len() as u64);
                                    wm.beta
                                        .entry(gc_id)
                                        .or_default()
                                        .extend(joined.iter().cloned());
                                }
                                d_beta.entry(gc_id).or_default().extend(joined);
                                next_frontier.push(gc_id);
                                continue;
                            }
                            if gkind != NodeKind::Negation && gkind != NodeKind::Exists {
                                continue;
                            }
                            let is_exists = gkind == NodeKind::Exists;
                            let Some(alpha_id) = node_ref_alpha_id(gc) else {
                                continue;
                            };
                            let driver = driver_of(compiled_drivers, alpha_id)?;
                            for tok in &parent_toks {
                                let any_compat = token_exists_under(
                                    driver,
                                    &bind_view(
                                        &wm.bind_keys,
                                        &wm.bind_vals,
                                        &wm.bind_pool,
                                        tok.binds,
                                    ),
                                    &wm,
                                    compiled_conds,
                                    &mut match_scratch,
                                    sym,
                                    &mut gather_cache,
                                )?;
                                let pass = if is_exists { any_compat } else { !any_compat };
                                if pass {
                                    if beta_readers.contains(&gc_id) {
                                        beta_written(gc_id, 1);
                                        wm.beta.entry(gc_id).or_default().push(*tok);
                                    }
                                    d_beta.entry(gc_id).or_default().push(*tok);
                                }
                            }
                            chain.push(gc_id);
                        }
                    }
                }
            }
            frontier = next_frontier;
        }
        phase_end("filter-after-join", __pt37);

        // ── 4. Production delta: fire production nodes on NEW tokens only. ────────
        let __pt5 = phase_start();
        let mut next_delta: Vec<u32> = Vec::new();
        for node_id in &kind_ids.prod {
            // Skip get_node unless a parent has tokens this round
            // (`DESIGN-STONE-dirty-production`).
            let Some(pids) = parents_of.get(node_id) else {
                continue;
            };
            if !pids
                .iter()
                .any(|pid| d_beta.get(pid).is_some_and(|ts| !ts.is_empty()))
            {
                continue;
            }
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(node) != NodeKind::Production {
                continue;
            }
            let Some(rule_name) = node_named_string(node, "rule-name") else {
                continue;
            };
            // Production gate: rule name must be in this arm's compiled :then
            // (stratified slices pass a rules subset — a ProductionNode whose
            // rule is absent is inert).
            let compiled_rhs_forms = match compiled_rhs_cache.get(rule_name) {
                Some(forms) => forms,
                None => continue,
            };

            // Fire on NEW tokens at EVERY parent (condition `:or` has N).
            // Walk d_beta in place — production only reads bindings
            // (`DESIGN-STONE-prod-no-token-clone`).
            for pid in pids {
                let Some(ts) = d_beta.get(pid) else {
                    continue;
                };
                if ts.is_empty() {
                    continue;
                }
                // `seen` grows by one entry per NEW derived fact, and hashbrown stores only 7-bit
                // control tags — it RE-HASHES every element on every resize. Reserve the exact
                // upper bound for this parent's tokens × RHS forms.
                seen_ids.reserve(ts.len().saturating_mul(compiled_rhs_forms.len()));

                let first = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, ts[0].binds);
                let slot_tables: crate::rete::compiled_rhs::RhsSlotTables = compiled_rhs_forms
                    .iter()
                    .map(|c| crate::rete::compiled_rhs::rhs_bind_slots(c, &first))
                    .collect();
                for tok in ts {
                    for (compiled, slots) in compiled_rhs_forms.iter().zip(&slot_tables) {
                        let __prhs = phase_start();
                        let derived = {
                            let pairs =
                                bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
                            crate::rete::compiled_rhs::exec_compiled_rhs_at(
                                compiled, pairs, slots, sym,
                            )?
                        };
                        phase_end("  ├ prod:compiled-rhs", __prhs);
                        // Arc 278 — the LAST split probe. build_insert_fact's own four parts summed to
                        // ~18ms instrumented while `production` read ~51ms, so ~30ms lives OUTSIDE the
                        // function. This mark brackets the dedup-and-store block. One pair per
                        // derivation, same tax as the four inside — so these five are comparable to
                        // each other and to nothing else.
                        //
                        // It used to cost two full-aggregate hashes per derivation (`contains`, then
                        // `insert`) on top of the resize ladder; both are gone — `insert` alone reports
                        // newness, and the reserve above sizes the set once. Measured on the
                        // 40,000-pair fanout cell, 3 runs each: 610 -> 489 (kill the second hash)
                        // -> 244 (reserve) ns per derivation, ranges disjoint at every step.
                        // ~120-165 ns of what remains is this mark pair itself, so the block is at
                        // the instrument's resolution — measure something else before cutting here.
                        let __pd = phase_start();
                        census_count("prod:derivations");
                        // Dedup + termination guard: only propagate truly new facts.
                        if seen_insert(&mut seen_ids, &mut seen_rest, &derived) {
                            // P12a: record the support index (first-producer-wins; or_insert_with).
                            if let Some(ref mut idx) = support {
                                idx.entry(derived.clone()).or_insert_with(|| {
                                    (
                                        rule_name.to_string(),
                                        native_token_to_value(*tok, &encode_view(&wm)),
                                    )
                                });
                            }
                            wm.production
                                .entry(*node_id)
                                .or_default()
                                .push(derived.clone());
                            let idx = wm.n_input + wm.derived_facts.len() as u32;
                            wm.derived_facts.push(derived);
                            next_delta.push(idx);
                        }
                        phase_end("  ├ prod:dedup-store", __pd);
                    }
                }
            }
        }

        // ── A8 instrument: census this round BEFORE the terminate check. ─────────
        // Placed here so the row reflects the round's cumulative totals after the round body,
        // and so the LAST round is recorded too (the break below would otherwise skip it).
        // `delta_facts` still holds this round's INPUT — it is not reassigned until after the
        // terminate check, so `.len()` here is what entered, not what leaves.
        #[cfg(test)]
        FIRE_CENSUS.with(|c| {
            let mut slot = c.borrow_mut();
            let rounds = match slot.as_mut() {
                Some(r) => r,
                None => return, // not armed — every other test in the suite pays nothing
            };
            let mut beta_by_node: Vec<(i64, &'static str, usize)> = Vec::new();
            let mut beta_tokens: usize = 0;
            let mut beta_token_matches: usize = 0;
            for node_id in node_ids {
                let toks = match wm.beta.get(node_id) {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };
                let kind = match get_node(&wm.network, *node_id) {
                    Some(n) => census_kind(kind_of(n)),
                    None => "?",
                };
                beta_tokens += toks.len();
                beta_token_matches += toks.iter().map(|t| t.matches.len as usize).sum::<usize>();
                beta_by_node.push((*node_id, kind, toks.len()));
            }
            // Per-node DELTA, the same shape. Needed because the beta-readers guard
            // (DESIGN-STONE-beta-is-written-only-for-readers) stops materialising `wm.beta` for
            // nodes nothing reads — so a node whose beta is deliberately empty is now invisible
            // above, and any census reading of it would be an artifact of the guard rather than a
            // measurement of the join.
            //
            // This is the SAME quantity, not a weaker proxy: before the guard, every token was
            // pushed to `wm.beta[node]` and `d_beta[node]` by the same unconditional statement
            // pair, so `Σ over rounds |d_beta[node]| == |wm.beta[node]|` at end of fire, exactly.
            // `d_beta` is also the more honest instrument for "did this join re-run per rule?" —
            // it is what the node PRODUCED, where beta was a cumulative copy of the same tokens.
            let mut d_beta_by_node: Vec<(i64, &'static str, usize)> = Vec::new();
            for node_id in node_ids {
                let toks = match d_beta.get(node_id) {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };
                let kind = match get_node(&wm.network, *node_id) {
                    Some(n) => census_kind(kind_of(n)),
                    None => "?",
                };
                d_beta_by_node.push((*node_id, kind, toks.len()));
            }
            rounds.push(RoundCensus {
                round: round_no,
                delta_facts_in: this_round_in,
                alpha_nodes: wm.alpha.values().filter(|v| !v.is_empty()).count(),
                alpha_elements: wm.alpha.values().map(|v| v.len()).sum(),
                beta_nodes: beta_by_node.len(),
                beta_tokens,
                beta_token_matches,
                d_beta_nodes: d_beta.values().filter(|v| !v.is_empty()).count(),
                d_beta_tokens: d_beta.values().map(Vec::len).sum(),
                left_idx_tokens: left_idx
                    .values()
                    .flat_map(|m| m.values())
                    .map(Vec::len)
                    .sum(),
                right_idx_elements: right_idx
                    .values()
                    .flat_map(|m| m.values())
                    .map(Vec::len)
                    .sum(),
                production_facts: wm.production.values().map(Vec::len).sum(),
                seen_facts: seen_ids.len() + seen_rest.len(),
                network_edges: node_ids
                    .iter()
                    .filter_map(|id| get_node(&wm.network, *id))
                    .map(|n| node_children(n).len())
                    .sum(),
                beta_by_node,
                d_beta_by_node,
            });
        });
        #[cfg(test)]
        {
            round_no += 1;
        }

        phase_end("production", __pt5);

        // ── 5. Terminate or loop. ─────────────────────────────────────────────────
        let __ep = phase_start();
        let __done = next_delta.is_empty();
        if !__done {
            owned_delta = next_delta;
        }
        phase_end("  └ round:epilogue", __ep);
        if __done || matches!(kind, FireKind::Once) {
            break;
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

    // Same stratify-or-delta door as fire-rules; support records on both arms.
    let mut idx: HashMap<Value, (String, Value)> = HashMap::new();
    let session_out = fire_rules_on_session(&session, sym, Some(&mut idx))?;

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

    Ok(explained)
}

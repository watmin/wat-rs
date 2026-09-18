//! Pass 3 of the fire round — hash-join delta.
//!
//! The largest pass, moved last and deliberately: 397 lines carrying the
//! catch-up take/restore invariant (`DESIGN-STONE-partire-fire-loop`
//! § sequencing). Settled method — the `arm` aliases are re-declared here as
//! the fire prologue declares them, so the body needs no re-spelling.
//!
//! ⚠ THE TAKE/RESTORE INVARIANT IS GONE — the situation was removed, not
//! guarded. `DESIGN-STONE-catchup-take-left` had this pass `wm.beta.remove` the
//! parent, walk it, and re-insert it at two sites — one of them an error path
//! nested twelve levels in — because a HashMap split-borrow was believed to
//! need the parent OUT while the catch-up ran. Taking was a real improvement
//! over the `.cloned()` it replaced.
//!
//! It is no longer needed. Every mutable touch of the session inside that
//! window is `wm.bind_pool` or `wm.match_pool`, and those are DISJOINT FIELDS
//! from `wm.beta`: a shared borrow of the parent coexists with them, so the
//! parent can simply be READ. The emit that the take was protecting now happens
//! after the window in any case. The workaround outlived its cause.
//!
//! rune:lint(cited-name-absent) restore_parent — never a function: it names the two `wm.beta` re-insert sites that
//! the borrow-not-take stone removed, so nothing bears the name today.
//! So there is no invariant to hold: no take, no `restore_parent`, no two
//! restore sites, and no way for a future `?` in this window to drop a beta
//! memory — because nothing is removed from the map to begin with.
//! (`DESIGN-STONE-catchup-borrow-not-take`.)
//!
//! `dirty_parents` is pass-local: seeded, tested and inserted into entirely
//! within this body.

use super::super::*;
use super::record_tokens;
use super::RoundScratch;
use crate::rete::compiled_cond::CompiledCond;

/// Join this round's new tokens and elements, ascending node id (topological).
// 8 args since fix-list F: `sym` joined so a computed inline operand can run through the one
// expression core (`Op::Eval`). The alternative — a context struct — would have to be built at
// every call site in the per-fact hot path purely to satisfy a lint, and the parameters here are
// already the fire pass's working set rather than an accidental pile.
#[allow(clippy::too_many_arguments)]
pub(crate) fn hash_join_delta(
    sym: &SymbolTable,
    wm: &mut FireSession,
    arm: &InternedNetwork,
    scratch: &mut RoundScratch<'_>,
    d_beta: &mut BetaMemory,
    left_idx: &mut JoinLeftIndex,
    right_idx: &mut JoinRightIndex,
) -> Result<(), EvalBreak> {
    let kind_ids = &arm.kind_ids;
    let compiled_conds = &arm.compiled_conds;
    let beta_readers = &arm.beta_readers;
    let feeding_alpha_of = &arm.feeding_alpha_of;
    let parents_of = &arm.parents_of;
    let RoundScratch {
        d_alpha,
        packed_full,
        match_scratch,
        ..
    } = scratch;

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
    d_beta,
    d_alpha,
    packed_full,
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

        // Step 1: first_keying iff the left index has not been keyed. Membership
        // of a sibling key-cache is no longer the latch (`JoinLeftIndex`).
        // Compute keys from a sample token at P and a sample element at A.
        let first_keying = !left_idx.is_keyed(*child_id);
        let jk_owned: Vec<Value> = if first_keying {
            let sample_tok = wm.beta.get(node_id).and_then(|v| v.first());
            // READ #1 of 2: one sample token, to derive this join's keys.
            if sample_tok.is_some() {
                beta_read(*node_id, 1);
            }
            let sample_el = wm.alpha.get(&alpha_id).and_then(|v| v.first());
            match (sample_tok, sample_el) {
                (Some(tok), Some(el)) => gather_join_keys(
                    &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                    std::slice::from_ref(el),
                    GatherIntern::from_wm(wm, alpha_id),
                ),
                _ => {
                    // Neither side has data yet — skip this node for this round.
                    continue;
                }
            }
        } else {
            left_idx
                .keys(*child_id)
                .expect("keyed join has keys")
                .to_vec()
        };
        let jk: &[Value] = &jk_owned;

        // CATCH-UP (first keying only): J was skipped every prior round while one side
        // was empty, so right_idx[J] was never populated from those rounds' facts.
        // Index the right tail `wm.alpha[alpha_id][already..]`, cross-join fully against
        // cumulative wm.beta[parent], and build the left index. Safe: J produced ZERO
        // tokens before first keying so there is nothing to double-count. On subsequent
        // rounds the incremental semi-naive path (steps 2–5 below) handles new arrivals.
        //
        // Note: at this point in the round, steps 1 (alpha delta) and 2 (root-join delta)
        // have ALREADY run, so wm.alpha and wm.beta contain ALL cumulative data including
        // this round's new elements — the catch-up covers historical AND current-round facts.
        if first_keying {
            // Occupancy is already Arc-shared. Bump the Arc; do not memcpy
            // the Vec (`DESIGN-STONE-catchup-arc-occupancy`). Parent beta
            // is taken, walked, put back — not cloned
            // (`DESIGN-STONE-catchup-take-left`).
            let all_right = wm.alpha.get(&alpha_id).cloned();
            let n_all = all_right.as_ref().map(|v| v.len()).unwrap_or(0);
            // ★ D1 (arc 278): the mark is a prefix length. Index `right[already..]`,
            // the same slice `keyed_join_persistent` uses. Walking the whole memory
            // here was safe only while first_keying implied `already == 0` (call-order
            // coupling with the left latch). Every writer now respects the mark.
            let already = right_idx.already(*child_id);
            // BORROWED, not taken. The removal this replaces existed to dodge a
            // borrow conflict that the compiler does not actually have: every
            // mutable touch inside this window is `wm.bind_pool` or
            // `wm.match_pool`, and those are DISJOINT FIELDS from `wm.beta`, so
            // a shared borrow of the parent coexists with them.
            let all_left: &[Token] = wm
                .beta.get(node_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            // READ #2 of 2: the parent's cumulative tokens, for the catch-up cross-join.
            beta_read(*node_id, all_left.len() as u64);
            // Key from packed occupancy (empty binds), then write BindSpan
            // onto the indexed copy (`DESIGN-STONE-join-index-span`).
            // Keying after materialize used the binds-path JoinKey, which
            // missed token probes (7b/7exists/8b native=0).
            let __cri = phase_start();
            {
                let mut ridx = right_idx.writer(*child_id);
                if let Some(right) = all_right.as_deref() {
                    let tail = right.get(already..).unwrap_or(&[]);
                    for &el in tail {
                        let k = key_of_el(&el, jk, &GatherIntern::from_wm(wm, alpha_id));
                        let el = element_with_row_span(
                            el,
                            &mut wm.bind_pool,
                            alpha_id,
                            &wm.i64_by_fact,
                            &wm.bind_only,
                            &wm.cond_key_ids,
                        );
                        // ★ D2 (arc 278): this walk USED to append straight into the bucket map
                        // and leave `indexed_n[J]` absent, so the maintainer's next visit read
                        // `already = 0` and re-pushed every element already here. There is no
                        // longer a form of this statement that can skip the mark.
                        ridx.push(k, el);
                    }
                }
            }
            phase_end("  ├ hj:catchup:right-idx", __cri);
            // ★ D2 census (test-only statement — no release code, see `census.rs`): this block
            // appended `right[already..].len()` elements to `right_idx[J]`, and since the D2
            // cure it advanced the mark by the same count — the row stays because WHICH site
            // wrote an index is still the reading, and the probe's non-vacuity guard needs it.
            // The capacity heuristic below still uses `n_all` because the catch-up cross-join
            // is against the full index.
            #[cfg(test)]
            crate::rete::kernel::census::right_idx_appended(
                *child_id,
                crate::rete::kernel::census::RIGHT_IDX_SITE_CATCHUP,
                n_all.saturating_sub(already),
            );
            // Reserve the 40k appends. Isolated unreserved extend paid
            // G−E = 4.13 ms (`DESIGN-STONE-probe-gap-split`).
            let n_join = match right_idx.get(child_id) {
                Some(idx) if !idx.is_empty() && n_all > 0 => {
                    all_left.len().saturating_mul(n_all / idx.len())
                }
                _ => 0,
            };
            wm.bind_pool.reserve(n_join.saturating_mul(4));
            wm.match_pool.reserve(n_join.saturating_mul(2));
            // Full cross-join: every left token keyed against right_idx[J].
            let __cpr = phase_start();
            let mut new_tokens: Vec<Token> = Vec::with_capacity(n_join);
            if let Some(ridx) = right_idx.get(child_id) {
                for tok in all_left {
                    let k = key_of(
                        &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                        jk,
                        &wm.bind_val_ids,
                    );
                    if let Some(bucket) = ridx.get(&k) {
                        for el in bucket {
                            match join_extend(
                                tok,
                                el,
                                alpha_id,
                                &mut FireCtx {
                            sym,
                                    compiled_conds,
                                    scratch: match_scratch,
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
                            ) {
                                Ok(Some(new_tok)) => new_tokens.push(new_tok),
                                Ok(None) => {}
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
            }
            phase_end("  ├ hj:catchup:probe", __cpr);
            // Build left_idx[J] from ALL cumulative left tokens — same act as
            // recording the keys (`JoinLeftIndex::key_and_index`).
            let __cli = phase_start();
            left_idx.key_and_index(*child_id, jk_owned.clone(), all_left, |tok| {
                key_of(
                    &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                    jk,
                    &wm.bind_val_ids,
                )
            });
            phase_end("  ├ hj:catchup:left-idx", __cli);
            // Emit catch-up tokens into cumulative and delta memories.
            let __cem = phase_start();
            // `entry()` HOISTED out of the per-token loop: the key is constant, so the
            // old form paid two map lookups per token (80,000 on the fanout cell) where
            // two total will do. Correct regardless of the guard below.
            let n_emit = new_tokens.len();
            record_tokens(&mut wm.beta, d_beta, beta_readers, *child_id, &new_tokens);
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
        let dr = AlphaNews::of(d_alpha, &wm.alpha, alpha_id, packed_full);

        // Skip if nothing new on either side.
        if dl.is_empty() && dr.is_empty() {
            continue;
        }

        // Step 2: add Δright (dr) to right_idx[J] FIRST.
        // dr is indices into wm.alpha[A]; right_idx still owns Elements (P6).
        // Span once onto the indexed copy (`DESIGN-STONE-join-index-span`).
        let __s2 = phase_start();
        {
            let mut ridx = right_idx.writer(*child_id);
            let right_mem = wm.alpha.get(&alpha_id).map(|v| v.as_slice()).unwrap_or(&[]);
            for ei in dr.iter() {
                let el = right_mem[ei];
                let k = key_of_el(&el, jk, &GatherIntern::from_wm(wm, alpha_id));
                let el = element_with_row_span(
                    el,
                    &mut wm.bind_pool,
                    alpha_id,
                    &wm.i64_by_fact,
                    &wm.bind_only,
                    &wm.cond_key_ids,
                );
                // ★ D2 (arc 278): this Δright append USED to leave `indexed_n[J]` stale-low, so
                // pass 3.7's `keyed_join_persistent` re-pushed the very elements step 2 had just
                // placed. The mark now advances with the bucket, in one act.
                ridx.push(k, el);
            }
        }
        phase_end("  ├ hj:step2-right-idx", __s2);
        // ★ D2 census (test-only statement — no release code, see `census.rs`): step 2 appended
        // one element per `dr` slot to `right_idx[J]`, and since the cure it advanced the mark by
        // the same count. Recorded even when `dr` is empty, so "step 2 ran and had nothing to
        // add" stays distinguishable from "step 2 never ran" — the blind spot the first D2 probe
        // shipped.
        #[cfg(test)]
        crate::rete::kernel::census::right_idx_appended(
            *child_id,
            crate::rete::kernel::census::RIGHT_IDX_SITE_STEP2,
            dr.iter().count(),
        );

        let mut new_tokens = hj_step3_term1(
            sym,
            wm,
            compiled_conds,
            match_scratch,
            right_idx,
            dl,
            jk,
            child_id,
            alpha_id,
        )?;

        hj_step4_term2(
            sym,
            wm,
            compiled_conds,
            match_scratch,
            left_idx,
            &mut new_tokens,
            dr,
            jk,
            child_id,
            alpha_id,
        )?;

        // Step 5: add Δleft (dl) to left_idx[J] AFTER term2 (no-double-count invariant).
        // writer is None until key_and_index — cannot skip the door.
        let __s5 = phase_start();
        if let Some(mut w) = left_idx.writer(*child_id) {
            for tok in dl {
                let k = key_of(
                    &bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds),
                    jk,
                    &wm.bind_val_ids,
                );
                w.push(k, *tok);
            }
        }
        phase_end("  ├ hj:step5-left-idx", __s5);

        // Step 6: push new tokens to wm.beta[J] and d_beta[J].
        let __s6 = phase_start();
        // Same hoist + guard as the catch-up emit above.
        let n_emit = new_tokens.len();
        record_tokens(&mut wm.beta, d_beta, beta_readers, *child_id, &new_tokens);
        if n_emit > 0 {
            dirty_parents.insert(*child_id);
        }
        phase_end("  ├ hj:step6-emit", __s6);
    }
}

phase_end("hash-join", __pt2);
    Ok(())
}

/// Step 4 of the P6 ordering — `term2 = old_left ⋈ Δright`.
///
/// Lifted out of `hash_join_delta`, where it sat at nesting NINE, the deepest
/// point in the whole rete engine. It could not be lifted before: `dr` is an
/// `AlphaNews`, and `AlphaNews::of` used to tie its `alpha` parameter to the
/// struct's lifetime, so the compiler believed `dr` pinned `wm.alpha` and
/// refused the `&mut wm` this body needs. That claim was false — `alpha` is
/// read once for a length — and correcting it is what made this a move rather
/// than a thirteen-parameter explosion. See `AlphaNews::of`.
#[allow(clippy::too_many_arguments)]
fn hj_step4_term2(
    sym: &SymbolTable,
    wm: &mut FireSession,
    compiled_conds: &HashMap<i64, CompiledCond>,
    match_scratch: &mut SlotFrame,
    left_idx: &JoinLeftIndex,
    new_tokens: &mut Vec<Token>,
    dr: AlphaNews<'_>,
    jk: &[Value],
    child_id: &i64,
    alpha_id: i64,
) -> Result<(), EvalBreak> {
// Step 4: term2 = old_left ⋈ Δright (probe left_idx[J] — still OLD, Δleft not yet added).
// left_idx is a separate map from right_idx; no aliasing — safe immutable borrow.
let __s4 = phase_start();
if !dr.is_empty() {
    if let Some(lidx) = left_idx.get(child_id) {
        let right_mem = wm.alpha.get(&alpha_id).map(|v| v.as_slice()).unwrap_or(&[]);
        for ei in dr.iter() {
            let el = right_mem[ei];
            let k = key_of_el(&el, jk, &GatherIntern::from_wm(wm, alpha_id));
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
                            sym,
                            compiled_conds,
                            scratch: match_scratch,
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
    Ok(())
}

/// Step 3 of the P6 ordering — `term1 = Δleft ⋈ all_right`.
///
/// Twin of `hj_step4_term2`, and lifted for the same reason: it was the other
/// arm sitting at nesting nine. `dl` borrows `d_beta`, not `wm`, so this one
/// was never blocked by the `AlphaNews` lifetime — it simply had nowhere to go
/// while its twin was stuck. Builds and returns the round's `new_tokens`, which
/// step 4 then extends and step 6 drains.
#[allow(clippy::too_many_arguments)]
fn hj_step3_term1(
    sym: &SymbolTable,
    wm: &mut FireSession,
    compiled_conds: &HashMap<i64, CompiledCond>,
    match_scratch: &mut SlotFrame,
    right_idx: &JoinRightIndex,
    dl: &[Token],
    jk: &[Value],
    child_id: &i64,
    alpha_id: i64,
) -> Result<Vec<Token>, EvalBreak> {
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
                            sym,
                            compiled_conds,
                            scratch: match_scratch,
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
    Ok(new_tokens)
}

// ── Moved here from `kernel/arm.rs` (2026-08-24) ─────────────────────────────
//
// `solvere`: this is fire-ROUND logic — it reads `d_beta`, `d_alpha` and `packed_full`,
// none of which exist outside a round — and it lived in `arm.rs`, whose module doc scopes
// it to network interning and the compilation cache. Its only non-test caller is the pass
// below. A reader of `arm.rs` tripped over round logic unrelated to interning; a reader of
// this pass could not find it.
//
// A PURE MOVE — no clone removed, no name improved, no comment rewritten. Visibility
// tightened from `pub(crate)` to private, which the compiler proves is safe now that the
// definition sits beside its one caller.

/// Seed dirty join-parents: left `d_beta` or a HashJoin child whose
/// feeding alpha has right-delta. The hash-join pass grows this set as
/// it emits (middle joins: J1's tokens dirty J1 as parent of J2).
fn seed_dirty_join_parents(
    join_parent: &[i64],
    d_beta: &BetaMemory,
    d_alpha: &AlphaDelta,
    packed_full: &HashSet<i64>,
    joins_fed_by: &JoinsFedBy,
    parents_of: &ParentsOf,
) -> rustc_hash::FxHashSet<i64> {
    let mut dirty = rustc_hash::FxHashSet::default();
    for (pid, toks) in d_beta {
        if !toks.is_empty() && join_parent.binary_search(pid).is_ok() {
            dirty.insert(*pid);
        }
    }
    let mut dirty_from_alpha = |aid: i64| {
        let Some(joins) = joins_fed_by.get(&aid) else {
            return;
        };
        for j in joins {
            let Some(ps) = parents_of.get(j) else {
                continue;
            };
            for p in ps {
                if join_parent.binary_search(p).is_ok() {
                    dirty.insert(*p);
                }
            }
        }
    };
    for (aid, idxs) in d_alpha {
        if idxs.is_empty() {
            continue;
        }
        dirty_from_alpha(*aid);
    }
    for &aid in packed_full {
        dirty_from_alpha(aid);
    }
    dirty
}

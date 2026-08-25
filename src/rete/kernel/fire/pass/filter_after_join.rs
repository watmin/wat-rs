//! Pass 3.7 of the fire round — filter-after-join.
//!
//! Test / Negation / Exists whose parent only received tokens in pass 3.6.
//! Moved verbatim out of `fire_fixpoint_delta_armed`
//! (`DESIGN-STONE-partire-fire-loop`), using the settled method: the `arm`
//! aliases are RE-DECLARED here exactly as the fire prologue declares them, so
//! the body needs no re-spelling and struct-field shorthand cannot break.
//!
//! `after_join_frontier` arrives BY VALUE because the body consumes it —
//! `let mut frontier = after_join_frontier;` was the first line inline, and it
//! still is.

use super::super::*;
use super::RoundScratch;

/// Drain the frontier pass 3.6 produced, dispatching the trailing filters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn filter_after_join(
    wm: &mut FireSession,
    arm: &InternedNetwork,
    scratch: &mut RoundScratch<'_>,
    d_beta: &mut BetaMemory,
    right_idx: &mut JoinRightIndex,
    right_idx_n: &mut HashMap<i64, usize>,
    join_keys_cache: &mut JoinKeysCache,
    gather_cache: &mut GatherCache,
    after_join_frontier: Vec<i64>,
    sym: &SymbolTable,
) -> Result<(), EvalBreak> {
    let compiled_conds = &arm.compiled_conds;
    let compiled_drivers = &arm.compiled_drivers;
    let compiled_wheres = &arm.compiled_wheres;
    let where_tree = &arm.where_tree;
    let beta_readers = &arm.beta_readers;
    let feeding_alpha_of = &arm.feeding_alpha_of;
    let test_children = &arm.test_children;
    // `match_scratch` only. The pre-scan again reported `bind_only` and
    // `cond_key_ids`, and again wrongly: here they appear as struct FIELD NAMES
    // (`bind_only: &wm.bind_only`), which a receiver-aware name test still
    // admits. Exclude field-name position as well as a preceding dot.
    let RoundScratch { match_scratch, .. } = scratch;

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
            // A HashJoin child of a frontier HashJoin. THIS CASE WAS MISSING, and its
            // absence was a silent wrong answer: this walk descends through FILTER
            // children, so `:where → HashJoin(a) → HashJoin(b)` stalled at (a) — (b) was
            // not a filter, nothing left-activated it, production read an empty d_beta,
            // and the fixpoint exited having matched nothing. See `left_activate_join`.
            if fkind == NodeKind::HashJoin {
                let new_tokens: Vec<Token> = match d_beta.get(&hj_id) {
                    Some(ts) if !ts.is_empty() => ts.clone(),
                    _ => continue,
                };
                if super::left_activate_join(
                    wm,
                    arm,
                    d_beta,
                    &mut super::JoinIdx {
                        right_idx,
                        right_idx_n,
                        join_keys_cache,
                        match_scratch,
                    },
                    &new_tokens,
                    filter_id,
                )? {
                    next_frontier.push(filter_id);
                }
                continue;
            }
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
                            wm,
                            d_beta,
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
                        wm,
                        compiled_conds,
                        match_scratch,
                        sym,
                        gather_cache,
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
                            wm,
                            d_beta,
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
                                right_idx,
                                join_keys_cache,
                                indexed_n: right_idx_n,
                            },
                            &mut FireCtx {
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
                            wm,
                            compiled_conds,
                            match_scratch,
                            sym,
                            gather_cache,
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
    Ok(())
}

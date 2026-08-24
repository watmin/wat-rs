//! Pass 3.25 of the fire round — the accumulate pass.
//!
//! Dispatches AccumulateNode: gather the matched elements, group them, fold.
//! Moved verbatim out of `fire_fixpoint_delta_armed`
//! (`DESIGN-STONE-partire-fire-loop`), settled method — the `arm` aliases are
//! re-declared here exactly as the fire prologue declares them, so the body
//! needs no re-spelling.
//!
//! THIS PASS WAS MISSED WHEN THE STRIKE WAS FIRST DECLARED COMPLETE. Seven
//! passes plus the A8 census had been extracted and the census was miscounted
//! as the eighth pass, so `fire_fixpoint_delta_armed` was reported finished at
//! 657 lines with 290 of them still being this. Recorded here because a
//! refactor that mis-states what it did is worse than one that stops early.

use super::super::*;
use super::RoundScratch;

/// Dispatch the accumulate nodes over this round's delta.
pub(crate) fn accumulate_pass(
    wm: &mut FireSession,
    arm: &InternedNetwork,
    scratch: &mut RoundScratch<'_>,
    d_beta: &mut BetaMemory,
    gather_cache: &mut GatherCache,
    sym: &SymbolTable,
) -> Result<(), EvalBreak> {
    let kind_ids = &arm.kind_ids;
    let compiled_conds = &arm.compiled_conds;
    let compiled_acc_folds = &arm.compiled_acc_folds;
    let beta_readers = &arm.beta_readers;
    let parents_of = &arm.parents_of;
    let RoundScratch { match_scratch, .. } = scratch;

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
    let mut new_tokens: Vec<Token> = d_beta_from_parents(parents_of, d_beta, *node_id);
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
        gather_cache,
        wm,
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
                &acc_view(wm, col_keys, col_fields),
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
                    match_scratch,
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
                &acc_view(wm, col_keys, col_fields),
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
    Ok(())
}

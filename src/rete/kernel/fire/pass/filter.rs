//! Pass 3.5 of the fire round — the filter pass.
//!
//! Dispatches TestNode, NegationNode and ExistsNode. Moved verbatim out of
//! `fire_fixpoint_delta_armed` (`DESIGN-STONE-partire-fire-loop`).
//!
//! THE `arm` ALIASES ARE RE-DECLARED HERE, exactly as the fire prologue
//! declares them. Five of them (`beta_readers`, `compiled_conds`,
//! `compiled_wheres`, `where_tree`, `sym`) appear as struct-field SHORTHAND in
//! this body, so re-spelling them to `arm.<field>` would have broken the
//! literals — the failure pass 3.6 actually hit. Re-declaring instead keeps
//! every name in the body resolving to what it did inline, which is what makes
//! the diff a move rather than a rename sweep, and keeps the signature at six
//! parameters instead of fourteen.
//!
//! `tests_done` is pass-local: declared, read and extended entirely within this
//! body, so it does not escape and is not returned.

use super::super::*;
use super::RoundScratch;

/// Dispatch the filter nodes — Test, Negation, Exists — over this round's delta.
pub(crate) fn filter_pass(
    wm: &mut FireSession,
    arm: &InternedNetwork,
    scratch: &mut RoundScratch<'_>,
    d_beta: &mut BetaMemory,
    gather_cache: &mut GatherCache,
    leading_emitted: &mut LeadingEmitted,
    sym: &SymbolTable,
) -> Result<(), EvalBreak> {
    let kind_ids = &arm.kind_ids;
    let compiled_conds = &arm.compiled_conds;
    let compiled_drivers = &arm.compiled_drivers;
    let compiled_wheres = &arm.compiled_wheres;
    let where_tree = &arm.where_tree;
    let parents_of = &arm.parents_of;
    let beta_readers = &arm.beta_readers;
    let test_sibs_of = &arm.test_sibs;
    // Only `match_scratch` is this pass's: the scan that suggested `bind_only`
    // and `cond_key_ids` was matching `wm.bind_only` / `wm.cond_key_ids`,
    // i.e. the SESSION fields, not the round locals of the same name.
    let RoundScratch { match_scratch, d_alpha, .. } = scratch;

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
// Seeded with what pass 3.20 already dispatched — see `RoundScratch::pre_dispatched`.
let mut tests_done: HashSet<i64> = scratch.pre_dispatched.clone();
for node_id in &kind_ids.filter {
    let node = match get_node(&wm.network, *node_id) {
        Some(n) => n,
        None => continue,
    };
    let kind = kind_of(node);
    if kind != NodeKind::Test && kind != NodeKind::Negation && kind != NodeKind::Exists {
        continue;
    }
    // COPY, and it is the data flow rather than a borrow-checker dodge — checked
    // 2026-08-24 before trying to remove it. The parent's delta must survive
    // this call intact because SIBLING children read the same `d_beta[parent]`,
    // so it cannot be taken; and reading `d_beta[parent]` while `entry(node_id)`
    // writes is a genuine same-map conflict, not a disjoint-field one the
    // compiler could split (contrast `DESIGN-STONE-catchup-borrow-not-take`,
    // where it could). `Token` is 16 B and `Copy`; measured volume is ~6
    // allocating calls and ~187 KB per fire (`dbeta_gather_volume`,
    // `dbeta_copy_size`). Every alternative is worse to READ: taking the parent
    // reintroduces the restore invariant just deleted from hash-join, and
    // buffering the output needs `new_tokens` to become Cow-shaped because the
    // leading-negation arm below REPLACES it with a synthetic token.
    // A Test/:not/:exists after condition `:or` has N parents.
    let pids = parents_of.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
    let mut new_tokens: Vec<Token> = d_beta_from_parents(parents_of, d_beta, *node_id);
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
        // ROUND GATE. The extension set is a pure function of this leaf's
        // CUMULATIVE alpha, so if that alpha gained nothing this round it can only
        // re-derive bindings `leading_emitted` already holds. `contains_key` is the
        // "have we run at all" test — no round counter needed, and none available
        // outside `cfg(test)`. This is not only cheaper: on the non-Leaf arm below,
        // re-deriving would re-intern a whole extension set into the bind pool every
        // idle round.
        if leading_emitted.contains_key(node_id) && !d_alpha.contains_key(&alpha_id) {
            continue;
        }
        let driver = driver_of(compiled_drivers, alpha_id)?;
        if matches!(driver, CondDriver::Leaf(_)) {
            // An Arc bump, not a memcpy of the bag — the contract that landed for
            // catch-up (`DESIGN-STONE-catchup-arc-occupancy`), applied to its last
            // unconverted sibling. Holding the Arc is also what releases `wm.alpha`,
            // and THAT is what lets the walk below take `&mut wm.bind_pool` per
            // element: the intermediate `candidates` Vec this replaced existed only
            // to buy that same freedom, and bought it with an allocation per element.
            let els = wm.alpha.get(&alpha_id).cloned().unwrap_or_default();
            for el in els.iter() {
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
                // ONE token per distinct inner binding, for the WHOLE fire (Clara
                // test-simple-exists: two Winds at MCI -> one {?loc MCI}). The set is
                // fire-scoped on purpose; as a round-local it re-emitted every round.
                if !leading_emitted
                    .entry(*node_id)
                    .or_default()
                    .insert(pool_slice(&wm.bind_pool, binds).to_vec())
                {
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
            wm,
            &empty,
            compiled_conds,
            match_scratch,
            sym,
            gather_cache,
        )?;
        for ext in exts {
            let binds = span_from_pairs(
                &mut BindIntern {
                    keys: &mut wm.bind_keys,
                    vals: &mut wm.bind_vals,
                    ids: &mut wm.bind_val_ids,
                    pool: &mut wm.bind_pool,
                },
                ext.iter().map(|(k, v)| (k.clone(), v.clone())),
            );
            // Keyed on the INTERNED pairs, same as the Leaf arm — one dedup rule for
            // both drivers rather than one on `ext` and one on the span.
            if !leading_emitted
                .entry(*node_id)
                .or_default()
                .insert(pool_slice(&wm.bind_pool, binds).to_vec())
            {
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
                wm,
                d_beta,
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
                wm,
                compiled_conds,
                match_scratch,
                sym,
                gather_cache,
            )?;
            // ExistsNode passes iff any-compat; NegationNode passes iff NOT any-compat.
            let pass = if is_exists { any_compat } else { !any_compat };
            if pass {
                // A LEADING filter (no parent) passes at most once per distinct
                // binding for the whole FIRE. A non-leading one needs no guard: its
                // tokens come from a parent's delta, which is already round-scoped.
                // Leading `:not` binds nothing, so its key is the empty vector — the
                // same mechanism, no special case. Without this it re-seeded and
                // re-passed one empty token on EVERY round into a cumulative beta.
                if pids.is_empty()
                    && !leading_emitted
                        .entry(*node_id)
                        .or_default()
                        .insert(pool_slice(&wm.bind_pool, tok.binds).to_vec())
                {
                    continue;
                }
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
    Ok(())
}

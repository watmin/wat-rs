//! Pass 4 of the fire round — production delta.
//!
//! Moved verbatim out of `fire_fixpoint_delta_armed`
//! (`DESIGN-STONE-partire-fire-loop`). Declared adaptations, all forced by the
//! move and none a logic change: dedent one level, and the prologue aliases
//! `kind_ids` / `parents_of` / `compiled_rhs_cache` re-spelled as the `arm`
//! fields they always were, and `&mut seen_ids` / `&mut seen_rest` written
//! unborrowed — inline they were owned locals, here they arrive already
//! `&mut`. That last one is the same adaptation root-join needed for
//! `&d_alpha`; it is a property of extraction, not of these passes.
//!
//! `next_delta` is BUILT here and consumed by the round epilogue, so it is
//! returned rather than written through an out-parameter — the caller binds it
//! exactly where it used to be declared.
//!
//! THE `production` PHASE MARK STAYS AT THE CALL SITE. `__pt5` opens before
//! this call and closes after the A8 census that follows it, so the mark's span
//! is unchanged by this move. That span is WIDER than this function (see
//! `round_census.rs`), and every `production` figure in the arc reflects the
//! wider span. Narrowing it is a census-tree change and out of scope here.

use super::super::*;

/// Fire production nodes on the tokens that are NEW this round, returning the
/// derived-fact indices that seed the next round.
#[allow(clippy::too_many_arguments)]
pub(crate) fn production_delta(
    wm: &mut FireSession,
    arm: &InternedNetwork,
    d_beta: &BetaMemory,
    seen_ids: &mut rustc_hash::FxHashSet<u64>,
    seen_rest: &mut rustc_hash::FxHashSet<Value>,
    support: &mut Option<&mut ExplainSupport>,
    sym: &SymbolTable,
) -> Result<Vec<u32>, EvalBreak> {
    let mut next_delta: Vec<u32> = Vec::new();
    for node_id in &arm.kind_ids.prod {
        // Skip get_node unless a parent has tokens this round
        // (`DESIGN-STONE-dirty-production`).
        let Some(pids) = arm.parents_of.get(node_id) else {
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
        let compiled_rhs_forms = match arm.compiled_rhs.get(rule_name) {
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
                    if seen_insert(seen_ids, seen_rest, &derived) {
                        // P12a: record the support index (first-producer-wins; or_insert_with).
                        if let Some(ref mut idx) = support {
                            idx.entry(derived.clone()).or_insert_with(|| {
                                (
                                    rule_name.to_string(),
                                    native_token_to_value(*tok, &encode_view(wm)),
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
    Ok(next_delta)
}

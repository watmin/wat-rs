//! Pass 1 of the fire round — alpha delta, in its two paths.
//!
//! Moved verbatim out of `fire_fixpoint_delta_armed`
//! (`DESIGN-STONE-partire-fire-loop`). The inline pass was one
//! `if seed_round { … } else { … }`; those two arms are genuinely different
//! work — 90 lines of first-round seeding against 18 of per-round delta — so
//! they come out as two functions and the branch stays at the call site with
//! the outer `alpha` mark.
//!
//! THE BODIES ARE UNCHANGED, and that is what `RoundScratch` is for. The round
//! carries nine mutable working structures; passing them individually gave the
//! seed path a FOURTEEN-parameter signature, which is not an improvement over
//! the thing this refactor is attacking. Grouping them into a struct of borrows
//! and DESTRUCTURING it on entry means every name in the body still resolves to
//! exactly what it did inline — no re-spelling, so the diff stays reviewable.
//!
//! `RoundScratch` owns nothing. It is borrows only, per the stone: a context
//! object that quietly cloned would trade a readability problem for a
//! performance one, and every test would stay green while it did.
//!
//! `seed_round = false` stays at the call site — it is the caller's loop state,
//! not this pass's.

use super::super::*;
use super::RoundScratch;
use crate::rete::alpha_tree::AlphaTree;
use crate::rete::compiled_cond::CompiledCond;

/// First round: seed alpha memories from the input facts.
pub(crate) fn alpha_seed(
    wm: &mut FireSession,
    scratch: &mut RoundScratch<'_>,
    input_facts: &crate::value::pvec::PVec,
    compiled_conds: &HashMap<i64, CompiledCond>,
    alpha_tree: &AlphaTree,
    scan_classes: &HashSet<&str>,
) -> Result<(), EvalBreak> {
    let RoundScratch {
        d_alpha,
        packed_full,
        bind_only,
        cond_key_ids,
        cand_scratch,
        match_scratch,
        seen_ids,
        seen_rest,
        leaf_aids,
        ..
    } = scratch;
    // Two pairs / fire, not per fact (`DESIGN-STONE-alpha-leftover-split`).
    let __seed = phase_start();
    let mut class_ids: HashMap<String, Vec<u32>> = HashMap::new();
    for class in leaf_aids.keys() {
        class_ids.insert(class.clone(), Vec::with_capacity(input_facts.len()));
    }
    for (i, fact) in input_facts.iter().enumerate() {
        seen_insert(seen_ids, seen_rest, fact);
        let (class, fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => {
                alpha_activate_fact(
                    fact,
                    i as u32,
                    &mut AlphaActivateCx {
                        wm,
                        d_alpha,
                        alpha_tree,
                        compiled_conds,
                        match_scratch,
                        cand_scratch,
                        cond_key_ids,
                        bind_only,
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
                wm,
                d_alpha,
                alpha_tree,
                compiled_conds,
                match_scratch,
                cand_scratch,
                cond_key_ids,
                bind_only,
            },
        )?;
    }
    for (class, aids) in leaf_aids.iter() {
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
        wm,
        alpha_tree,
        compiled_conds,
        bind_only,
        input_facts,
    );
    Ok(())
}

/// Later rounds: activate only the facts derived last round.
pub(crate) fn alpha_delta(
    wm: &mut FireSession,
    scratch: &mut RoundScratch<'_>,
    owned_delta: &[u32],
    compiled_conds: &HashMap<i64, CompiledCond>,
    alpha_tree: &AlphaTree,
) -> Result<(), EvalBreak> {
    let RoundScratch {
        d_alpha,
        bind_only,
        cond_key_ids,
        cand_scratch,
        match_scratch,
        ..
    } = scratch;
    let __delta = phase_start();
    for &idx in owned_delta {
        let fact = fact_at(&wm.facts, &wm.derived_facts, wm.n_input, idx).clone();
        alpha_activate_fact(
            &fact,
            idx,
            &mut AlphaActivateCx {
                wm,
                d_alpha,
                alpha_tree,
                compiled_conds,
                match_scratch,
                cand_scratch,
                cond_key_ids,
                bind_only,
            },
        )?;
    }
    phase_end("  └ alpha:delta", __delta);
    Ok(())
}

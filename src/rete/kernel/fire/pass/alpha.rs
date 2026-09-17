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
    sym: &SymbolTable,
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
    // ── CLASS-UNIFORM BATCHING — the D7 invariant ────────────────────────────────────────────
    //
    // ★ NO `aid` MAY RECEIVE BOTH A PUSH AND A REPLACE IN ONE SEED PASS.
    //
    // The two writers of `wm.alpha[aid]` in this pass are `alpha_activate_fact`
    // (`fire/delta.rs` — `entry(aid).or_default()` then PUSH) and the batch loop below
    // (`wm.alpha.insert(aid, els)` — a whole-entry REPLACE). `build_alpha_index` files each
    // alpha node under exactly ONE `pat.type_head` and `candidates_into` walks `root_for(class)`,
    // so every fact of class C reaches exactly C's aids and nothing else does. The two writers
    // therefore collide precisely when ONE class sends some facts down each path.
    //
    // ⛔ THAT IS REACHABLE, AND IT SILENTLY DROPPED A DERIVED FACT (D7, driven 2026-09-02:
    // `native=2 oracle=3`). `pack_i64_row` tests RUNTIME values, and a PARAMETRIC record erases
    // its type argument into ONE runtime class: `(defrecord :Box :- [T] [k <- i64  v <- :T])`
    // gives one class whose `Box[i64]` instances pack and whose `Box[String]` instances do not.
    // The unpacked ones pushed; the batch then replaced the whole `Arc<Vec<Element>>` and
    // discarded them. Worse than short: `d_alpha[aid]` still held the pushed SLOT INDICES, which
    // after the replace index DIFFERENT elements.
    //
    // THE CURE IS THE `bool` BELOW — "every fact of this class packed". A class batches only if
    // it is uniform; a mixed class takes the activate path for ALL of its facts (the deferred
    // loop after the batch), so exactly one writer ever touches an aid. The decision cannot be
    // made until every fact has been seen, which is why the mixed class's facts are DEFERRED
    // here rather than activated in place.
    //
    // Why not decide from the DECLARED schema (which is what `session.rs`'s `i64_by_fact` doc
    // says)? Two reasons, and the second is the stronger: `FireSession` holds no `TypeEnv`, so it
    // would need new state threaded down the fire path; and it is strictly MORE conservative than
    // this — `Box[T]`'s declared `v <- :T` is not `i64`, so a fire whose Boxes all happen to hold
    // i64 would lose the fast path that this keeps. Why not merge instead of replace? Because
    // `d_alpha` holds INDICES into this vector and the batch shares one `Arc` across every aid of
    // the class; appending re-orders elements away from fact order and forfeits the share.
    let mut class_ids: HashMap<String, (Vec<u32>, bool)> = HashMap::new();
    for class in leaf_aids.keys() {
        class_ids.insert(class.clone(), (Vec::with_capacity(input_facts.len()), true));
    }
    // Set when some leaf class turned out mixed. Gates the deferred-activate loop entirely, so a
    // corpus with no mixed class pays one bool test for the whole seed pass.
    let mut any_mixed = false;
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
                        sym,
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
        // A leaf class's facts are DEFERRED — none of them may activate here, because a later
        // fact of the same class can still take the batch away from all of them.
        //
        // ⚠ THE PACKED ARM IS FIRST ON PURPOSE, and the duplicated `get_mut` is the price. Written
        // as one lookup with the `packed` test inside, this taxes the batch fast path — the very
        // path the cure exists to preserve — with a probe on every fact that does NOT pack,
        // including facts of classes that were never batchable. Written this way, a PACKING fact
        // executes exactly the instruction sequence it did before the cure, and the extra probe
        // falls only on facts already committed to a full alpha-tree walk and a compiled exec,
        // where it is noise.
        if packed {
            if let Some((ids, _)) = class_ids.get_mut(class) {
                ids.push(i as u32);
                continue;
            }
        } else if let Some((_, uniform)) = class_ids.get_mut(class) {
            *uniform = false;
            any_mixed = true;
            continue;
        }
        alpha_activate_fact(
            fact,
            i as u32,
            &mut AlphaActivateCx {
                        sym,
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
        let Some((ids, uniform)) = class_ids.get(class) else {
            continue;
        };
        // THE D7 GUARD. A mixed class forfeits the batch — its facts are activated below
        // instead, so this `insert` cannot land on an aid that already took a push.
        //
        // The two counters are how a test can see WHICH WAY the decision went. Without them the
        // batch and the activate path are observationally identical (they derive the same facts,
        // which is the point), so a cure that quietly stopped batching everything would keep
        // every correctness gate green — the exact failure
        // `docs/arc/2026/06/278-rules-engine/strike-cure-alpha-double-write/EXPECTATIONS.md`
        // names as fatal. `tests/rete/probe_arc278_d7_parametric_erasure_differential.rs` cannot
        // see it; `seed_batches_uniform_classes_and_defers_mixed_ones` can.
        //
        // ⚠ `record_seed_leaf_vs_alpha` (the test-only `LeafOccDiff` instrument) does NOT model this
        // decision: it predicts occupancy from packability alone, so on a MIXED class it now
        // under-predicts and would report a spurious `missing`. That instrument's own defect is a
        // separate row (C16) and is deliberately not touched here; no test drives a mixed class
        // through it today.
        if !*uniform {
            census_count("seed:batch-class-mixed");
            continue;
        }
        if ids.is_empty() {
            continue;
        }
        census_count("seed:batch-class-uniform");
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
    // The mixed leaf classes, deferred out of the fact loop above.
    if any_mixed {
        activate_deferred_mixed_classes(
            sym,
            wm,
            d_alpha,
            alpha_tree,
            compiled_conds,
            match_scratch,
            cand_scratch,
            cond_key_ids,
            bind_only,
            input_facts,
            &class_ids,
        )?;
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

/// Activate every fact of a leaf class that FORFEITED the occupancy batch (D7).
///
/// Running HERE — after the batch loop, in ascending fact order — is what keeps element order
/// inside `wm.alpha[aid]` identical to what a plain activate pass would have produced, which
/// matters because `d_alpha` holds INDICES into that vector. These aids never enter
/// `packed_full`, so `AlphaNews::of` reads them from `d_alpha`'s slots like any other activated
/// alpha, and no aid ever sees both a push and a replace.
///
/// ⚠ `#[cold]` + `#[inline(never)]` is the point of the extraction, not decoration. Inline, this
/// loop grew `alpha_seed`'s body — the hot seed pass — with a second `AlphaActivateCx`
/// construction that a corpus with no mixed class never executes. A mixed class is by
/// construction the rare case (it needs one runtime class whose instances differ in packability),
/// so the branch is cold and the code belongs out of the hot function's body.
#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn activate_deferred_mixed_classes(
    sym: &SymbolTable,
    wm: &mut FireSession,
    d_alpha: &mut AlphaDelta,
    alpha_tree: &AlphaTree,
    compiled_conds: &HashMap<i64, CompiledCond>,
    match_scratch: &mut SlotFrame,
    cand_scratch: &mut Vec<i64>,
    cond_key_ids: &CondKeyIds,
    bind_only: &BindOnlyFields,
    input_facts: &crate::value::pvec::PVec,
    class_ids: &HashMap<String, (Vec<u32>, bool)>,
) -> Result<(), EvalBreak> {
    for (i, fact) in input_facts.iter().enumerate() {
        let Value::Aggregate(a) = fact else {
            continue;
        };
        if a.nature == Nature::Struct {
            continue;
        }
        if !class_ids
            .get(a.class.as_ref())
            .is_some_and(|(_, uniform)| !*uniform)
        {
            continue;
        }
        census_count("seed:mixed-class-activate");
        alpha_activate_fact(
            fact,
            i as u32,
            &mut AlphaActivateCx {
                sym,
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
    Ok(())
}

/// Later rounds: activate only the facts derived last round.
pub(crate) fn alpha_delta(
    sym: &SymbolTable,
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
                        sym,
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

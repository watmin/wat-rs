//! Pass 3.6 of the fire round — join-after-filter.
//!
//! Moved verbatim out of `fire_fixpoint_delta_armed`
//! (`DESIGN-STONE-partire-fire-loop`). Declared adaptations, forced by the move
//! and none a logic change: dedent one level, and the prologue aliases
//! `kind_ids` / `beta_readers` / `compiled_conds` / `feeding_alpha_of`
//! re-spelled as the `arm` fields they always were (`compiled_conds` sits in a
//! struct literal, so it becomes `compiled_conds: arm.compiled_conds` rather
//! than shorthand). Re-borrows of already-borrowed parameters are the
//! standing extraction adaptation — see `production.rs`.
//!
//! The `join-after-filter` phase mark moves WITH the pass: it bracketed exactly
//! this body inline, so the census tree is unchanged. (Contrast `production`,
//! whose mark spans wider than its pass and therefore had to stay behind.)
//!
//! `after_join_frontier` is built here and consumed by pass 3.7, so it is
//! returned; the caller binds it where it was declared.

use super::super::*;
use super::record_tokens;

/// Push tokens a filter just produced across the next hash join.
///
/// The main hash-join pass only left-activates from Root/HashJoin, but compile
/// will parent a HashJoin on a mid-chain `:where`. Returns the child ids that
/// received tokens — pass 3.7's frontier.
// 8 args since fix-list F: `sym` joined so a computed inline operand can run through the one
// expression core (`Op::Eval`). The alternative — a context struct — would have to be built at
// every call site in the per-fact hot path purely to satisfy a lint, and the parameters here are
// already the fire pass's working set rather than an accidental pile.
#[allow(clippy::too_many_arguments)]
pub(crate) fn join_after_filter(
    sym: &SymbolTable,
    wm: &mut FireSession,
    arm: &InternedNetwork,
    d_beta: &mut BetaMemory,
    right_idx: &mut JoinRightIndex,
    join_keys_cache: &mut JoinKeysCache,
    match_scratch: &mut SlotFrame,
) -> Result<Vec<i64>, EvalBreak> {
    // ── 3.6 Join-after-filter (A1): HashJoin children of Test/Neg/Exists/Accum. ─
    // The P6 loop above only left-activates from Root/HashJoin. Compile will parent
    // a HashJoin on a mid-chain :where (Clara does; Join → Test → Join). Filter just
    // filled d_beta[test]; push those tokens across the next join. keyed_join against
    // the full alpha is the catch-up: this child produced nothing in step 3, so there
    // is nothing to double-count.
    let __pt36 = phase_start();
    let mut after_join_frontier: Vec<i64> = Vec::new();
    for node_id in &arm.kind_ids.filter_or_acc {
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
            let alpha_id = arm.feeding_alpha_of.get(child_id).copied().unwrap_or(-1);
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
                    right_idx,
                    join_keys_cache,
                },
                &mut FireCtx {
                            sym,
                    compiled_conds: &arm.compiled_conds,
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
            record_tokens(&mut wm.beta, d_beta, &arm.beta_readers, *child_id, &joined);
            after_join_frontier.push(*child_id);
        }
    }
    phase_end("join-after-filter", __pt36);
    Ok(after_join_frontier)
}

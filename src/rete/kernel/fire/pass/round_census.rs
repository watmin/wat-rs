//! The A8 round census — the per-round instrument, `#[cfg(test)]` only.
//!
//! Moved verbatim out of `fire_fixpoint_delta_armed`
//! (`DESIGN-STONE-partire-fire-loop`). 85 lines of instrument were sitting in
//! the middle of the round body; nothing in a release build ever ran them, and
//! every reader of the fire loop had to step over them.
//!
//! The `production` mark used to close AFTER this census; it was narrowed on
//! 2026-08-24 so it brackets the pass and nothing else.
//!
//! ⚠ AND THE REASON GIVEN FOR NARROWING IT WAS WRONG. This module previously
//! claimed the census was "a third contributor to the inflated production
//! reading". It is not, and measuring the narrowing proved it: production read
//! 18.315 ms before and 18.662 after — no change. The body below early-returns
//! unless `FIRE_CENSUS` is armed (`None => return`), and the phase harnesses arm
//! only `PHASE_NANOS`, so in every measurement in this arc it cost one TLS
//! access. Narrowing the mark was still right — a mark should name what it
//! measures — but it bought no accuracy, and the claim that it would is
//! corrected here rather than left standing.

use super::super::*;

/// Record one round into the fire census. Reads only; the caller owns
/// `round_no` and advances it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_round_census(
    wm: &FireSession,
    node_ids: &[i64],
    d_beta: &BetaMemory,
    left_idx: &JoinLeftIndex,
    right_idx: &JoinRightIndex,
    seen_ids: &rustc_hash::FxHashSet<u64>,
    seen_rest: &rustc_hash::FxHashSet<Value>,
    round_no: usize,
    this_round_in: usize,
) {
    // ── A8 instrument: census this round BEFORE the terminate check. ─────────
    // Placed here so the row reflects the round's cumulative totals after the round body,
    // and so the LAST round is recorded too (the break below would otherwise skip it).
    // `owned_delta` still holds this round's INPUT — it is not reassigned until after the
    // terminate check, so `.len()` here is what entered, not what leaves.
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
            left_idx_tokens: left_idx.total_tokens(),
            right_idx_elements: right_idx.total_elements(),
            // ★ D2: the counter beside the population, per join. The union of both key sets — a
            // join present in ONE map only is exactly the case worth seeing, so neither map may
            // drive the iteration on its own. Both maps are fields of `JoinRightIndex` now, and
            // the union is `per_join_marks`.
            right_idx_by_join: right_idx.per_join_marks(),
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
}

//! Pass 2 of the fire round — root-join delta.
//!
//! Extracted verbatim from `fire_fixpoint_delta_armed`
//! (`DESIGN-STONE-partire-fire-loop`). This is a MOVE: the body below is the
//! same code that sat inline, with exactly three mechanical adaptations, each
//! forced by the move and none of them a logic change:
//!   1. dedented one level (it sat inside the round `loop {`);
//!   2. the prologue aliases `kind_ids` and `beta_readers` re-spelled as the
//!      `arm` fields they always were;
//!   3. `&d_alpha` / `&packed_full` written as `d_alpha` / `packed_full` —
//!      inline they were owned locals, here they arrive already borrowed, and
//!      `&&T` would be a `clippy::needless_borrow` (clippy is `-D warnings` here).
//!
//! No clone was added to make it compile — the pass reads
//! `wm.alpha` (through `AlphaNews`) while writing `wm.bind_pool`,
//! `wm.match_pool` and `wm.beta`, and those are disjoint field paths, so the
//! borrow checker splits them without help. That was STOP-1 for this strike.

use super::super::*;
use super::record_token;
use std::collections::HashSet;

/// Seed root-join children from the elements that are NEW this round.
///
/// `d_alpha` names the new slots; a packed seed is the whole `0..len` range
/// (`DESIGN-STONE-seed-d-alpha-range`), which is what `packed_full` selects.
pub(crate) fn root_join_delta(
    wm: &mut FireSession,
    arm: &InternedNetwork,
    d_alpha: &AlphaDelta,
    d_beta: &mut BetaMemory,
    packed_full: &HashSet<i64>,
) {
    // ── 2. Root-join delta: seed tokens from NEW elements (d_alpha) only. ───
    let __pt1 = phase_start();
    for node_id in &arm.kind_ids.alpha {
        // New this round: indices into wm.alpha[node_id]. Packed seed
        // is 0..len (`DESIGN-STONE-seed-d-alpha-range`).
        let news = AlphaNews::of(d_alpha, &wm.alpha, *node_id, packed_full);
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
                record_token(&mut wm.beta, d_beta, &arm.beta_readers, *child_id, tok);
            }
        }
    }

    phase_end("root-join", __pt1);
}

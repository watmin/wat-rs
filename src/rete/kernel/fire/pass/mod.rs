//! The fire round's passes, one module each.
//!
//! `fire_fixpoint_delta_armed` was 1774 lines and 12 levels deep, with nine
//! passes braided into one body (`DESIGN-STONE-partire-fire-loop`). The seams
//! were already drawn by its own `// ── N.` section comments; these modules are
//! those sections, moved out whole.
//!
//! Each extraction is behaviour-identical by contract: no clone removed, no
//! name improved, no comment rewritten in the same commit as a move, so the
//! diff can be reviewed by reading it. The gate is the oracle differential —
//! a second implementation of the same semantics — plus the floor.

use super::*;

/// Record that ONE token reached `node_id`.
///
/// ── THE RULE THIS NAMES ──────────────────────────────────────────────────────────────────────
///
/// A token reaching a node is ALWAYS recorded in the round's delta (`d_beta`) — that is what the
/// next pass reads. It is written to the DURABLE beta (`wm.beta`) only if something downstream
/// reads that node: `beta_readers` is the set of nodes whose beta is consumed, and a node nobody
/// reads has no reason to accumulate one. That gate is the whole reason the two memories differ.
///
/// ⛔ **AND THE CENSUS MOVES WITH THE DURABLE WRITE, NOT BESIDE IT.** `beta_written` is the
/// `#[cfg(test)]` traffic counter the round-census tests read. It was previously a SECOND statement
/// next to the push at each of thirteen sites — two things that must agree, written twice, in
/// seven files. Here they are one act, so a future site cannot push without counting or count
/// without pushing.
///
/// ── WHY IT TAKES `&mut BetaMemory` AND NOT `&mut FireSession` ────────────────────────────────
///
/// Deliberate, and it is the lesson this arc already paid for on `FireCtx`: a helper that took
/// `wm` whole would borrow the session mutably and lock every caller out of `wm.alpha`, which the
/// join passes read in the same scope. Taking the ONE field it writes keeps the field-level
/// disjointness the callers depend on. *"Verbosity that encodes field-level disjointness is data
/// flow, not repetition."*
#[inline]
pub(crate) fn record_token(
    beta: &mut BetaMemory,
    d_beta: &mut BetaMemory,
    beta_readers: &HashSet<i64>,
    node_id: i64,
    tok: Token,
) {
    if beta_readers.contains(&node_id) {
        beta_written(node_id, 1);
        beta.entry(node_id).or_default().push(tok);
    }
    d_beta.entry(node_id).or_default().push(tok);
}

/// Record that `toks` reached `node_id` — the batch form of [`record_token`].
///
/// Same rule, same census pairing. Both memories are `reserve`d before the append, which is what
/// the hand-written sites did: a join can emit a whole bucket at once, and growing a `Vec` one
/// token at a time inside the fire loop is the cost that reserve exists to avoid. `Token` is
/// `Copy`, so this is an `extend_from_slice` and not a per-element clone.
#[inline]
pub(crate) fn record_tokens(
    beta: &mut BetaMemory,
    d_beta: &mut BetaMemory,
    beta_readers: &HashSet<i64>,
    node_id: i64,
    toks: &[Token],
) {
    if beta_readers.contains(&node_id) {
        beta_written(node_id, toks.len() as u64);
        let b = beta.entry(node_id).or_default();
        b.reserve(toks.len());
        b.extend_from_slice(toks);
    }
    let d = d_beta.entry(node_id).or_default();
    d.reserve(toks.len());
    d.extend_from_slice(toks);
}

/// The join-index borrows a left-activation needs, grouped so the helper below
/// takes one parameter for them.
pub(crate) struct JoinIdx<'a> {
    pub(crate) right_idx: &'a mut JoinRightIndex,
    pub(crate) right_idx_n: &'a mut HashMap<i64, usize>,
    pub(crate) join_keys_cache: &'a mut JoinKeysCache,
    pub(crate) match_scratch: &'a mut SlotFrame,
}

/// LEFT-ACTIVATE one HashJoin from tokens its parent just produced, writing the
/// result into `d_beta` (and `wm.beta` if anything reads it). Returns true if it
/// emitted, so a caller walking a frontier knows whether to enqueue `join_id`.
///
/// ONE COPY. This body was written twice — `join_after_filter` (3.6, filter → join)
/// and `filter_after_join`'s grandchild branch (3.7, filter → join one level down) —
/// and a THIRD site needed it: a HashJoin that is the child of a HashJoin already on
/// the frontier. That case was simply missing, and it was a SILENT WRONG ANSWER:
/// pass 3.7 descends only through FILTER children, so in
///
///     Node → :where → HashJoin(a) → HashJoin(b) → production
///
/// 3.6 put `HashJoin(a)` on the frontier, 3.7 saw `HashJoin(b)` was not a filter and
/// skipped it, nothing left-activated it, production read an empty `d_beta`,
/// `next_delta` came back empty and the fixpoint exited. The rule matched NOTHING,
/// compiled clean, and exited 0 — the one outcome that cannot be right, because it is
/// the one that lies. Both references disagree with it: `$oracle` and Clara 0.24.0 each
/// match correctly. Characterised exactly as TWO OR MORE fact conditions after a
/// `where`; with one, the chain is short enough that 3.6 alone finishes it.
/// Gated by `tests/rete/probe_arc278_where_is_positionally_free`.
pub(crate) fn left_activate_join(
    sym: &SymbolTable,
    wm: &mut FireSession,
    arm: &InternedNetwork,
    d_beta: &mut BetaMemory,
    idx: &mut JoinIdx<'_>,
    parent_toks: &[Token],
    join_id: i64,
) -> Result<bool, EvalBreak> {
    let alpha_id = arm.feeding_alpha_of.get(&join_id).copied().unwrap_or(-1);
    let elements = match wm.alpha.get(&alpha_id) {
        Some(els) if !els.is_empty() => els.as_slice(),
        _ => return Ok(false),
    };
    let joined = keyed_join_persistent(
        parent_toks,
        elements,
        alpha_id,
        join_id,
        &mut FilterJoinIdx {
            right_idx: idx.right_idx,
            join_keys_cache: idx.join_keys_cache,
            indexed_n: idx.right_idx_n,
        },
        &mut FireCtx {
            sym,
            compiled_conds: &arm.compiled_conds,
            scratch: idx.match_scratch,
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
        return Ok(false);
    }
    if arm.beta_readers.contains(&join_id) {
        beta_written(join_id, joined.len() as u64);
        wm.beta
            .entry(join_id)
            .or_default()
            .extend(joined.iter().cloned());
    }
    d_beta.entry(join_id).or_default().extend(joined);
    Ok(true)
}

/// The round's mutable working set, grouped so a pass takes ONE parameter for
/// it instead of nine. Borrows only — it owns nothing, and must not: a context
/// that copied would turn a readability fix into a performance regression that
/// every test would pass (`DESIGN-STONE-partire-fire-loop` § trap doors).
///
/// Passes destructure it on entry, so their bodies use the same bare names they
/// used inline and the extraction diff stays a move.
pub(crate) struct RoundScratch<'a> {
    pub(crate) d_alpha: &'a mut AlphaDelta,
    pub(crate) packed_full: &'a mut std::collections::HashSet<i64>,
    pub(crate) bind_only: &'a BindOnlyFields,
    pub(crate) cond_key_ids: &'a CondKeyIds,
    pub(crate) cand_scratch: &'a mut Vec<i64>,
    pub(crate) match_scratch: &'a mut SlotFrame,
    pub(crate) seen_ids: &'a mut rustc_hash::FxHashSet<u64>,
    pub(crate) seen_rest: &'a mut rustc_hash::FxHashSet<Value>,
    pub(crate) leaf_aids: &'a LeafAidsByClass,
    /// Tests the ACCUMULATE pass (3.20) had to dispatch early to feed an accumulate whose parent
    /// is a Test. The FILTER pass (3.5) must skip them: a Test dispatched twice against one
    /// parent delta duplicates its tokens all the way into production. Written by 3.20, read by
    /// 3.5, cleared by 3.20 at the top of every round — per-round scratch, which is what this
    /// struct is for, and it keeps both passes at their argument budget.
    pub(crate) pre_dispatched: &'a mut std::collections::HashSet<i64>,
}

mod accumulate;
pub(crate) use accumulate::*;
mod alpha;
pub(crate) use alpha::*;
mod filter;
pub(crate) use filter::*;
mod filter_after_join;
pub(crate) use filter_after_join::*;
mod hash_join;
pub(crate) use hash_join::*;
mod join_after_filter;
pub(crate) use join_after_filter::*;
mod production;
pub(crate) use production::*;
mod root_join;
pub(crate) use root_join::*;

#[cfg(test)]
mod round_census;
#[cfg(test)]
pub(crate) use round_census::*;

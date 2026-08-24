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
    pub(crate) bind_only: &'a mut std::collections::HashMap<i64, Vec<u8>>,
    pub(crate) cond_key_ids: &'a mut CondKeyIds,
    pub(crate) cand_scratch: &'a mut Vec<i64>,
    pub(crate) match_scratch: &'a mut SlotFrame,
    pub(crate) seen_ids: &'a mut rustc_hash::FxHashSet<u64>,
    pub(crate) seen_rest: &'a mut rustc_hash::FxHashSet<Value>,
    pub(crate) leaf_aids: &'a std::collections::HashMap<String, Vec<i64>>,
}

mod alpha;
pub(crate) use alpha::*;
mod filter;
pub(crate) use filter::*;
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

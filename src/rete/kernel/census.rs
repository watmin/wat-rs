//! Fire-loop census / phase marks. Test-only counters plus always-inline no-ops
//! on the production path so the round loop can call them unconditionally.

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use crate::ast::WatAST;
#[cfg(test)]
use crate::runtime::Value;
#[cfg(test)]
use super::{pmap_from_span, Token};

// ── Arc 278 A8 instrument: per-round census of the native fire memories ──────
//
// WHY THIS EXISTS. Grid axis A8 (node-share) is the one cell where Clara wins, and by 2026-07-30
// the compiler was proven INNOCENT: `probe-node-share-dedup.wat` counts the compiled network at
// `4 + 2N` nodes (Alpha flat at 2, HashJoin flat at 1) across N = 1..32 — textbook optimal
// sharing. So the blow-up (>4 GiB to join 500 facts against 20 rules) is in the FIRE path.
//
// It cannot be measured from wat: `wm.beta.clear()` runs before freeze (see the end of
// `fire_fixpoint_delta`), so a frozen Session carries an EMPTY beta-memory and a wat-side probe
// reading `Session/beta-memory` would report all zeros — a number that looks like a finding and
// is an artifact. The census is therefore taken HERE, inside the real loop, before the clear —
// the same reasoning that relocated the 3a/3b join assertions into this module (see the P11
// relocation note in `mod tests`).
//
// It measures the REAL path. There is no second implementation to drift from and no re-derived
// oracle to compare against itself: `fire_fixpoint_delta` records into the thread-local below,
// and production is untouched because every line of it is `#[cfg(test)]`.

/// One round's census of every native structure the fire loop grows.
///
/// Recorded at the END of each round, after all five passes and before the terminate check, so
/// the counts are that round's cumulative totals. Fields are deliberately exhaustive: the point
/// is to let the growth term name ITSELF rather than confirm a guess about which one it is.
#[cfg(test)]
#[derive(Debug, Clone, Default)]
pub(crate) struct RoundCensus {
    /// 0-based round index within this fire.
    pub(crate) round: usize,
    /// Facts entering this round (the previous round's derivations; round 0 = the input facts).
    pub(crate) delta_facts_in: usize,
    /// Distinct node-ids holding alpha elements, and the total element count across them.
    pub(crate) alpha_nodes: usize,
    pub(crate) alpha_elements: usize,
    /// Distinct node-ids holding beta tokens, and the total token count across them.
    pub(crate) beta_nodes: usize,
    pub(crate) beta_tokens: usize,
    /// Σ over every beta token of `matches.len()` — the per-token support-chain edges. This is the
    /// real memory driver (a Token owns its `Vec<(Value, i64)>`), so it separates "N× more tokens"
    /// from "same tokens carrying N× longer chains".
    pub(crate) beta_token_matches: usize,
    /// The per-round delta (new-this-round tokens), same two measures.
    pub(crate) d_beta_nodes: usize,
    pub(crate) d_beta_tokens: usize,
    /// The P6 persistent join indexes, summed across every HashJoinNode.
    pub(crate) left_idx_tokens: usize,
    pub(crate) right_idx_elements: usize,
    /// Derived facts retained in production-memory, and the size of the `seen` dedup set.
    pub(crate) production_facts: usize,
    pub(crate) seen_facts: usize,
    /// Σ over every node of `children.len()` — the compiled network's EDGE count.
    ///
    /// Counted here because nothing else ever counted it: the compile-time census
    /// (`probe-node-share-dedup.wat`) counts NODES, and a shared node reached by N duplicate
    /// edges is indistinguishable from a shared node reached once if nodes are all you count.
    pub(crate) network_edges: usize,
    /// Per-node beta occupancy as `(node-id, kind, tokens)`, ascending by id — the breakdown that
    /// distinguishes "one shared join holds M tokens" from "N tails each hold their own copy".
    pub(crate) beta_by_node: Vec<(i64, &'static str, usize)>,
    /// The same, for the per-round DELTA. Load-bearing since the beta-readers guard: a node whose
    /// `wm.beta` is deliberately not materialised is invisible in `beta_by_node`, but every token
    /// it produced still passes through `d_beta`. Summed across rounds this equals what
    /// `beta_by_node` reported before the guard, by construction (both were pushed by the same
    /// unconditional statement pair).
    pub(crate) d_beta_by_node: Vec<(i64, &'static str, usize)>,
}

#[cfg(test)]
// rune:sequi(performance-counter) — test-only fire census; off unless with_fire_census.
thread_local! {
    /// Enabled by `with_fire_census`; `None` means "do not record" (the default for every other
    /// test in the suite, so the instrument costs nothing it is not asked for).
    pub(crate) static FIRE_CENSUS: std::cell::RefCell<Option<Vec<RoundCensus>>> =
        const { std::cell::RefCell::new(None) };
}

// ─── DESIGN-STONE-compiled-where Step 0: capture the filter loop's real inputs ────────────────
//
// The decomposition benchmark must time the EXACT values production hands the filter pass, not a
// hand-fabricated stand-in — a probe that does not walk the substrate path production uses proves
// nothing (`[[feedback_feasibility_probe_must_exercise_the_exact_mechanism]]`). So the loop hands
// its first (predicate, parent-delta-tokens) pair to this slot, once, under `#[cfg(test)]`.

/// What the filter loop hands Step 0: the TestNode's predicate, and the parent's new-this-round
/// tokens (the vector `:2701` clones once per TestNode).
#[cfg(test)]
pub(crate) type WhereSample = (WatAST, Vec<crate::value::pmap::PMap>);

#[cfg(test)]
// rune:sequi(performance-counter) — test-only where-sample slot; off unless armed.
thread_local! {
    /// Armed by [`with_where_sample`]; the OUTER `None` means "do not record" (the default
    /// everywhere else), the inner one means "armed, nothing caught yet".
    pub(crate) static WHERE_SAMPLE: std::cell::RefCell<Option<Option<WhereSample>>> =
        const { std::cell::RefCell::new(None) };
}

/// Record the filter loop's inputs, FIRST occurrence only (later TestNodes in the same round see
/// the same parent delta by construction on a shared-prefix axis, and overwriting would make the
/// captured sample depend on node iteration order).
#[cfg(test)]
pub(crate) fn capture_where_sample(
    expr: &WatAST,
    tokens: &[Token],
    keys: &[Value],
    vals: &[Value],
    pool: &[(u32, u32)],
) {
    WHERE_SAMPLE.with(|c| {
        if let Some(slot @ None) = c.borrow_mut().as_mut() {
            *slot = Some((
                expr.clone(),
                tokens
                    .iter()
                    .map(|t| pmap_from_span(t.binds, keys, vals, pool))
                    .collect(),
            ));
        }
    });
}

/// Run `f` with the filter-input capture armed, and return what it caught.
#[cfg(test)]
pub(crate) fn with_where_sample<R>(f: impl FnOnce() -> R) -> (R, Option<WhereSample>) {
    let prior = WHERE_SAMPLE.with(|c| c.borrow_mut().replace(None));
    let out = f();
    let caught = WHERE_SAMPLE.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    (out, caught.flatten())
}

/// Run `f` with the per-round census enabled, and return what it recorded.
///
/// Any previously-armed census is restored afterwards, so nesting cannot silently swallow an
/// outer measurement.
#[cfg(test)]
pub(crate) fn with_fire_census<R>(f: impl FnOnce() -> R) -> (R, Vec<RoundCensus>) {
    let prior = FIRE_CENSUS.with(|c| c.borrow_mut().replace(Vec::new()));
    let out = f();
    let recorded = FIRE_CENSUS.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    (out, recorded.unwrap_or_default())
}

/// Map a node kind label onto a `&'static str` so a census row can be printed without holding a
/// borrow of the network. Any kind the compiler can emit that is not listed reads as `"?"` — an
/// unrecognised kind must be visible in the output, never silently folded into a neighbour.
#[cfg(test)]
pub(crate) fn census_kind(kind: &str) -> &'static str {
    match kind {
        "AlphaNode" => "Alpha",
        "RootJoinNode" => "RootJoin",
        "HashJoinNode" => "HashJoin",
        "TestNode" => "Test",
        "NegationNode" => "Negation",
        "ExistsNode" => "Exists",
        "AccumulateNode" => "Accumulate",
        "ProductionNode" => "Production",
        "QueryNode" => "Query",
        _ => "?",
    }
}

// Test-only instrument: one element EXAMINED by an Accumulate / Negation / Exists gather.
//
// The gathers share `gather_cache` with the keyed joins' shape: each token probes its
// join-key bucket. Acc, Negation, and Exists Leaf all miss through `ensure_gather`.
//
// Counting the EXAMINATIONS — rather than the wall-clock — is what makes the keyed-gather gate
// honest. A timing wall can pass for reasons that have nothing to do with the mechanism (a wall
// drawn over a cheap container passed before its fix existed, 2026-07-30), and it is flaky under
// load. A visit count cannot be faked by a scan: if the gather still scans, the count still scales
// with the token count, whatever the machine was doing at the time.
#[cfg(test)]
// rune:sequi(performance-counter) — test-only gather visit count; honesty of the keyed-gather gate.
thread_local! {
    /// Elements examined by an Accumulate/Negation/Exists gather since the counter was armed.
    pub(crate) static GATHER_VISITS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
#[inline]
pub(crate) fn census_gather_visit() {
    GATHER_VISITS.with(|c| c.set(c.get() + 1));
}

/// In every non-test build this is nothing at all — the instrument costs the production fire path
/// zero instructions, exactly as `FIRE_CENSUS` records nothing unless armed.
#[cfg(not(test))]
#[inline(always)]
pub(crate) fn census_gather_visit() {}

/// Run `f` with the gather-visit counter zeroed, and return what it counted.
///
/// Any outer count is restored afterwards, so nesting cannot silently swallow a measurement.
#[cfg(test)]
pub(crate) fn with_gather_census<R>(f: impl FnOnce() -> R) -> (R, u64) {
    let prior = GATHER_VISITS.with(|c| c.replace(0));
    let out = f();
    let counted = GATHER_VISITS.with(|c| c.replace(prior));
    (out, counted)
}

// ── Per-phase wall-clock inside the fire loop ────────────────────────────────
//
// `RoundCensus` counts STRUCTURES (how many tokens, how many elements); this counts NANOSECONDS,
// summed across every round, per step of `fire_fixpoint_delta`. The two answer different questions
// and neither substitutes for the other: the census says the shape is linear, this says where the
// linear cost is spent.
//
// Why it exists: the `accum` axis is ~1.5x behind a WARMED Clara, and the keyed gather is under 10%
// of our fire — so the remaining cost is somewhere else and nothing on this box can profile it (no
// `perf`). Rather than narrate a plausible root — four perf hypotheses died this week by exactly
// that move — the loop is made to say where its own time goes.
//
// Deliberately start/end marks rather than an RAII guard: the steps are sequential blocks that
// mutate `wm`/`d_beta` in place, and wrapping them in scopes to host a guard would re-indent the
// hot path for the benefit of a test-only instrument. In a non-test build every call here is a
// no-op on a `()` and the phase map does not exist.

#[cfg(test)]
type PhaseMark = std::time::Instant;
/// A zero-sized stand-in in non-test builds. Deliberately NOT `()`: `let __pt = phase_start();`
/// against a unit value trips `clippy::let_unit_value` at nine call sites, and nine `#[allow]`s
/// would be suppressing a lint rather than not earning it. A ZST compiles to nothing and the lint
/// simply does not apply.
#[cfg(not(test))]
#[derive(Clone, Copy)]
pub(crate) struct PhaseMark;

#[cfg(test)]
// rune:sequi(performance-counter) — test-only phase clock; ~75ns/pair, subtracted via pair count.
thread_local! {
    /// phase name → (nanoseconds, MARK PAIRS FIRED), summed over every round. `None` = not recording.
    ///
    /// ★ The pair COUNT is not bookkeeping — it is what makes the timing readable. A mark pair
    /// costs ~75-80ns, and the `alpha:*` marks fire PER FACT: at 40,200 facts that is ~3.2ms of
    /// pure clock-reading per row. Measured 2026-08-01 against a no-sub-marks control: the fire
    /// read 78.5ms instrumented vs 58.2ms bare — 26% of the "measurement" was the instrument, and
    /// THREE of alpha's five children (candidates/element/fieldnames) were individually SMALLER
    /// than their own instrument, i.e. their rows measured nothing but themselves. Without the
    /// count there is no way to say that from the table; with it, the table subtracts.
    pub(crate) static PHASE_NANOS: std::cell::RefCell<Option<HashMap<&'static str, (u64, u64)>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
#[inline]
pub(crate) fn phase_start() -> PhaseMark {
    std::time::Instant::now()
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn phase_start() -> PhaseMark {
    PhaseMark
}

#[cfg(test)]
#[inline]
pub(crate) fn phase_end(name: &'static str, t: PhaseMark) {
    let ns = t.elapsed().as_nanos() as u64;
    PHASE_NANOS.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            let e = m.entry(name).or_insert((0, 0));
            e.0 += ns;
            e.1 += 1; // pairs fired — the divisor for the instrument subtraction
        }
    });
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn phase_end(_name: &'static str, _t: PhaseMark) {}

// ── Operation counters (for granularity where a TIMER would measure mostly itself) ──────────
//
// One level below `alpha`, the sub-operations cost ~100-300ns each while a phase mark pair costs
// ~52ns (calibrated). Timing there would tax each operation 20-50% and — worse — tax them UNEVENLY,
// making a cheap operation look expensive purely because it was called often. So this level counts
// instead: a `Cell` increment is ~1-2ns. Combined with the phase timer's un-taxed total for the
// enclosing phase, counts give ns-per-operation without distorting the thing being measured.

#[cfg(test)]
// rune:sequi(performance-counter) — test-only op counts; Cell increment ~1-2ns vs timer tax.
thread_local! {
    /// counter name → occurrences. `None` = not recording.
    pub(crate) static CENSUS_COUNTS: std::cell::RefCell<Option<HashMap<&'static str, u64>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
#[inline]
pub(crate) fn census_count_n(name: &'static str, n: u64) {
    CENSUS_COUNTS.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            *m.entry(name).or_insert(0) += n;
        }
    });
}

#[cfg(test)]
#[inline]
pub(crate) fn census_count(name: &'static str) {
    census_count_n(name, 1);
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn census_count(_name: &'static str) {}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn census_count_n(_name: &'static str, _n: u64) {}

/// Run `f` with operation counting enabled, and return what it counted (descending).
#[cfg(test)]
pub(crate) fn with_count_census<R>(f: impl FnOnce() -> R) -> (R, Vec<(&'static str, u64)>) {
    let prior = CENSUS_COUNTS.with(|c| c.borrow_mut().replace(HashMap::new()));
    let out = f();
    let recorded = CENSUS_COUNTS.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    let mut rows: Vec<(&'static str, u64)> = recorded.unwrap_or_default().into_iter().collect();
    rows.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    (out, rows)
}

// ── BETA TRAFFIC — is a beta memory ever READ by the fire that writes it? ────────────────
//
// `wm.beta` is written once per join result (a Token CLONE) and then `wm.beta.clear()`ed before
// freeze, so nothing downstream of the fire can observe it. Inside the fire it is read at exactly
// two places, both in the hash-join's first-keying path and both against the PARENT node:
// `.first()` for one sample token (to derive join keys) and `all_left` for the catch-up cross-join.
//
// That makes a WRITE-BUT-NEVER-READ hypothesis available for terminal joins — and a hypothesis is
// all it is. The identical shape ("surely this store is redundant") was proposed for
// production-memory's freeze one session ago and died on the disk: derived facts live ONLY there,
// so the freeze IS the output. This instrument exists so the beta question is answered by
// measurement instead of by the same reasoning that was wrong last time.
//
// Per node: tokens written in, tokens read back out. A node with writes and zero reads is a
// candidate; a node with reads is not. No timing here — this is a counting question.
#[cfg(test)]
// rune:sequi(performance-counter) — test-only beta write/read traffic; counting, not domain.
thread_local! {
    /// node_id → (tokens written into `wm.beta`, tokens read back out). `None` = not recording.
    pub(crate) static BETA_TRAFFIC: std::cell::RefCell<Option<HashMap<i64, (u64, u64)>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
#[inline]
pub(crate) fn beta_written(node_id: i64, n: u64) {
    BETA_TRAFFIC.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            m.entry(node_id).or_insert((0, 0)).0 += n;
        }
    });
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn beta_written(_node_id: i64, _n: u64) {}

#[cfg(test)]
#[inline]
pub(crate) fn beta_read(node_id: i64, n: u64) {
    BETA_TRAFFIC.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            m.entry(node_id).or_insert((0, 0)).1 += n;
        }
    });
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn beta_read(_node_id: i64, _n: u64) {}

/// Run `f` with beta write/read traffic recorded, returning it as (node_id, written, read).
#[cfg(test)]
pub(crate) fn with_beta_traffic<R>(f: impl FnOnce() -> R) -> (R, Vec<(i64, u64, u64)>) {
    let prior = BETA_TRAFFIC.with(|c| c.borrow_mut().replace(HashMap::new()));
    let out = f();
    let recorded = BETA_TRAFFIC.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    let mut rows: Vec<(i64, u64, u64)> = recorded
        .unwrap_or_default()
        .into_iter()
        .map(|(id, (w, r))| (id, w, r))
        .collect();
    rows.sort_by_key(|&(id, _, _)| id);
    (out, rows)
}

/// Run `f` with per-phase timing enabled, and return what it recorded (descending by nanoseconds).
///
/// Any previously-armed map is restored afterwards, so nesting cannot swallow an outer measurement.
#[cfg(test)]
pub(crate) fn with_phase_census<R>(f: impl FnOnce() -> R) -> (R, Vec<(&'static str, u64)>) {
    let (out, rows) = with_phase_census_counted(f);
    (out, rows.into_iter().map(|(n, ns, _)| (n, ns)).collect())
}

/// As [`with_phase_census`], but each row also carries **how many mark pairs fired**.
///
/// ONE implementation, two views: the count only matters to a caller that intends to subtract the
/// instrument from the reading, and most callers just want the split. A mark pair is ~75-80ns and
/// the `alpha:*` marks fire PER FACT, so at 40,200 facts a single row carries ~3.2ms of clock
/// reads — enough that three of alpha's five children measured nothing but themselves. A caller
/// that reports raw nanoseconds on a per-fact-marked phase is reporting its own instrument.
#[cfg(test)]
pub(crate) fn with_phase_census_counted<R>(
    f: impl FnOnce() -> R,
) -> (R, Vec<(&'static str, u64, u64)>) {
    let prior = PHASE_NANOS.with(|c| c.borrow_mut().replace(HashMap::new()));
    let out = f();
    let recorded = PHASE_NANOS.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    let mut rows: Vec<(&'static str, u64, u64)> = recorded
        .unwrap_or_default()
        .into_iter()
        .map(|(n, (ns, k))| (n, ns, k))
        .collect();
    rows.sort_by_key(|&(_, ns, _)| std::cmp::Reverse(ns));
    (out, rows)
}

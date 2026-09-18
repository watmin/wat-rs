# DESIGN — Stone P6: persistent keyed join memories (beat Clara at width)

The Clara head-to-head (P5b) diagnosed the one regime where Clara wins: **width-heavy** (30×10: Clara 14 ms vs
our 36 ms). Cause, grounded in our own code: the P4b delta is round-based semi-naive AND **`keyed_join` rebuilds
the join index from scratch on every call** (`src/rete/kernel/fire/mod.rs::keyed_join` computes `join_keys` + a
`HashMap<key,Vec<usize>>` every round, every join node). At width W that re-index is O(W) per round per node →
O(D²·W)-ish. Clara keeps **persistent per-node indexes** and probes O(matches). P6 closes that gap: maintain the
join indexes **across rounds**, update them incrementally, and **probe** them in the delta join — never rebuild.

**Behavior-preserving** (the P3/P4b pattern): `fire-rules'` stays observationally identical to `fire-rules-spec`
(same `query` counts). NO RED probe — the differential gates are the net and must STAY green; the win shows on
the Clara bench. The round structure is KEPT; only the indexing becomes persistent (the cost was the rebuild,
not the rounds).

## The nets (green at HEAD; must stay green)
- `tests/probe_arc278_deep_cascade.rs` — native `fire-rules'` == `fire-rules-spec` == full closure, depth 10 + 20.
- `…P4a_native_fire_rules`, `…P4c_native_retraction`, `…P2_native_fire_once`, the acceptance probes (2b–5a,
  north star — they run native via the public `fire-rules` wrapper), lib floor, oracle.
- The win (measured, not gated): `wat-scripts/perf/deep-cascade.wat` + the Clara head-to-head
  (`wat-scripts/perf/clara/`) — native-ns at 20×10 / 30×10 must drop **below Clara** (currently 12.2/36.2 ms vs
  Clara 12.1/14.1). Re-run both each iteration.

## What changes (Rust — `src/rete/kernel/fire/delta.rs::fire_fixpoint_delta` + helpers only)
Each `HashJoinNode` J has exactly one left parent P (the node whose `children` include J) and one feeding alpha
A (`alpha_feeding(J)`). The join key set for J — `join_keys[J]` = the sorted shared binding-var names — is fixed;
compute it ONCE (lazily, the first round J has ≥1 token and ≥1 element) and cache it.

Maintain, **persistent across rounds** (local to `fire_fixpoint_delta`):
- `left_idx:  HashMap<i64 /*J*/, HashMap<Vec<Value> /*key*/, Vec<Value> /*Token*/>>`
- `right_idx: HashMap<i64 /*J*/, HashMap<Vec<Value> /*key*/, Vec<Value> /*Element*/>>`
- `join_keys: HashMap<i64 /*J*/, Vec<Value>>` (cached).

The hash-join delta step (replaces the two `keyed_join` rebuild calls), per (P, J):
1. Ensure `join_keys[J]` (compute from a sample token of `wm.beta[P]` ∩ a sample element of `wm.alpha[A]` if
   not cached; if either side still empty, skip — nothing to join yet).
2. **Add this round's Δright into `right_idx[J]` FIRST** — for each element in `Δalpha[A]` (= the elements
   newly added to A this round), `right_idx[J][key(el)].push(el)`. Now `right_idx[J]` holds *all* right
   elements including this round's.
3. **term1 = Δleft ⋈ all_right:** for each token in `Δbeta[P]` (this round's new left tokens), look up
   `right_idx[J][key(tok)]` → for each element, `extend_token` → a new J token.
4. **term2 = old_left ⋈ Δright:** for each element in `Δalpha[A]`, look up `left_idx[J][key(el)]` → for each
   token (these are the OLD left tokens — `left_idx[J]` does NOT yet contain this round's Δleft) →
   `extend_token` → a new J token.
5. **Then add this round's Δleft into `left_idx[J]`** — for each token in `Δbeta[P]`,
   `left_idx[J][key(tok)].push(tok)`. (After term2, so old_left excluded it — the no-double-count invariant.)
6. The new J tokens (term1 ∪ term2) → append to `wm.beta[J]` AND `Δbeta[J]` (drive J's own children this round,
   ascending-id as today).

`key(x)` = `[ x.bindings.get(k) for k in join_keys[J] ]` (the same tuple `keyed_join` computes today). Reuse
`element_fact_bindings` / `token_matches_bindings` / `extend_token`. The empty-`join_keys` (cartesian) case →
one bucket `vec![]`, same as today.

**Correctness = the same semi-naive invariant, re-expressed with persistent indexes:** term1 (Δleft × all_right,
incl Δleft×Δright) ∪ term2 (old_left × Δright). (Δleft×Δright) is in term1 only (right_idx had Δright added in
step 2); old_left×Δright is in term2 only (left_idx lacked Δleft in step 4). No double-count, no miss — identical
to P4b's `(Δleft ⋈ all_right) ∪ (old_left ⋈ Δright)`, just probed against persistent indexes instead of rebuilt
ones. The deep-cascade differential (depth 20) is the proof.

## Out of scope / sequencing
- `keyed_join` (the rebuild helper) stays for the BATCH `hash_join_pass` (the P3 `fire-once'` path) — only the
  delta path (`fire_fixpoint_delta`) switches to persistent indexes. Do not touch `fire-once'`/the batch passes.
- If persistent indexes alone do not pass Clara at 20×10/30×10, the residual is round overhead → a follow-on
  **per-element** (non-round) propagation pass; named here, not built in P6 unless the bench still trails.
- No oracle change, no `WorkingMemory` *stored shape* change (the indexes are local to the fire call, rebuilt
  from the staged session each fire — value-semantics preserved), no public-surface change, no retract change.

## Files
- `src/rete/kernel/fire/delta.rs` — rework the hash-join delta inside `fire_fixpoint_delta` to maintain + probe persistent
  `left_idx`/`right_idx`/`join_keys`. Add a small `key_of(bindings, join_keys)` helper. Nothing else.

## Verify
- deep-cascade differential 2/2 (depth 10 + 20), P4a 4/4, P4c 3/3, P2 4/4, acceptance probes green, lib 935/36.
- bench: `wat-scripts/perf/deep-cascade.wat` native-ns drops at 20×10/30×10; re-run the Clara head-to-head →
  native ≤ Clara across the board (the close condition for 278's bar: Clara-parity-or-superior).

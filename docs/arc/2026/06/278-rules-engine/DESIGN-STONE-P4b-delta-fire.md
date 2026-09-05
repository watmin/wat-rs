# DESIGN — Stone P4b: delta-incremental `fire-rules'` (the smart activation)

The headline of the perf close. P4a's native `fire-rules'` is a **re-run-from-scratch** fixpoint: each round
`fire_once_session` clears all memories and recomputes the entire match over the whole growing fact set —
O(rounds × full-match) = O(depth²) on the deep cascade. P4b converts that fixpoint **in place** to
**semi-naive delta propagation** (Clara's "smart activation"): memories **persist and accumulate**; each round
propagates only the facts derived in the previous round (the delta), joining the new facts against the
accumulated memories. O(depth²) → linear.

**Behavior-preserving, like P3.** `fire-rules'` stays observationally identical to wat `fire-rules` (same
derived facts, same `query` counts). There is NO RED probe — the differential gates already exist and are GREEN
under P4a; they must STAY green. The win shows on the wat perf bench (`wat-scripts/perf/deep-cascade.wat`):
re-run O(depth²) → delta linear.

## The nets (already green at HEAD under P4a — must stay green)
- `tests/probe_arc278_P4a_native_fire_rules.rs` — 4/4 (single + 2-deep cascade, native == wat).
- `tests/probe_arc278_deep_cascade.rs` — 2/2 (native `fire-rules'` == wat `fire-rules` == full closure at
  depth 10 **and depth 20** — the depth where re-run-vs-delta bites; THE net for this stone).
- `tests/probe_arc278_P2_native_fire_once.rs`, `…P3` (keyed join still correct), the lib floor, the oracle.
- The bend (measured, not gated): `wat-scripts/perf/deep-cascade.wat` — us/round should go ~flat in depth
  (linear total) vs P4a's climbing re-run.

## What changes (Rust — `src/rete/kernel/` only)
`fire-rules'` (`eval_fire_rules_native`) stops calling the re-run `fire_fixpoint` and calls a new
**`fire_fixpoint_delta`**. `fire_once_session` and the four batch passes stay UNTOUCHED (they remain
`fire-once'`'s impl and the P4a re-run path — keep `fire_fixpoint` too, unused-but-kept is fine, or
`#[allow(dead_code)]`; do not delete the re-run path, it documents the spec). The keyed-join machinery (P3 —
`join_keys` + index) is REUSED inside the delta join.

## The algorithm — semi-naive round-based delta (pinned)

State held across the whole fire (NOT cleared between rounds):
- `wm.alpha / wm.beta / wm.production` — the accumulated memories (persist).
- `seen: HashSet`-equivalent of facts (use a `Vec<Value>` + `contains`, or hash a stable key) — every fact
  ever in the working set, for dedup + termination. Seed with all input facts.

Per-round delta sets (recomputed each round, local to the loop):
- `delta_alpha: HashMap<i64, Vec<Value>>` — elements created THIS round, per AlphaNode id.
- `delta_beta:  HashMap<i64, Vec<Value>>` — tokens created THIS round, per Root/HashJoinNode id.
- `delta_facts: Vec<Value>` — facts derived last round (round 0 = all input facts).

**Round r** (`delta_facts` = round r-1's new facts; round 0 = input facts):

1. **Alpha delta.** For each `AlphaNode`, for each fact in `delta_facts`: `alpha_match_inner` → on match, make
   an `Element`; append to `wm.alpha[node]` AND to `delta_alpha[node]`. (Same matching as `alpha_pass`, but
   only over `delta_facts`, not all facts.)

2. **Root-join delta.** For each `AlphaNode` with non-empty `delta_alpha`: for each `RootJoinNode` child: seed
   one `Token` per NEW element (from `delta_alpha[alpha]`); append to `wm.beta[child]` AND `delta_beta[child]`.
   (Same seed shape as `root_join_pass`, but only over the new elements.)

3. **Hash-join delta** (ascending node-id = topological, so a node's `delta_beta` is built before its child
   consumes it). For each `RootJoinNode`/`HashJoinNode` P that has a `HashJoinNode` child J (feeding alpha A):
   the new tokens at J are the **semi-naive delta join** —
   ```
   Δbeta[J]  =  ( Δbeta[P]              ⋈  all wm.alpha[A] )
             ∪  ( (wm.beta[P] \ Δbeta[P]) ⋈  Δalpha[A]      )
   ```
   i.e. **(new-left ⋈ all-right) ∪ (old-left ⋈ new-right)**, where old-left = the tokens at P that existed
   *before* this round (`wm.beta[P]` minus the ones just added in `delta_beta[P]`). This is the standard
   semi-naive form: it produces exactly the token pairs that use ≥1 new element/token, with **no
   double-count** of (new-left ⋈ new-right) and **no miss**. Use the P3 keyed index for BOTH joins (index the
   right side by `join_keys` value-tuple, probe with the left). Append results to `wm.beta[J]` AND
   `delta_beta[J]`.
   - **old-left** = the prefix of `wm.beta[P]` before this round's appends. Track it: snapshot
     `len(wm.beta[P])` at round start, or build `delta_beta` first then `old = &wm.beta[P][..old_len]`. The
     cleanest: capture each node's beta length at the START of the hash-join step, before appending, so
     `old_left = wm.beta[P][0..start_len_P]` and `Δbeta[P]` = what root-join/earlier-hash-join added this round.

4. **Production delta.** For each `ProductionNode`, for each NEW token in `delta_beta[parent]` (parent =
   `node_parent(prod)`): for each rhs form → `build_insert_fact` → derived fact. For each derived fact: **if not
   in `seen`**, add to `seen`, append to `wm.production[prod]`, and push to `next_delta_facts`. (The `seen`
   guard is the dedup + termination invariant — exactly what `merge_facts` was in the re-run path.)

5. **Terminate** when `next_delta_facts` is empty. Else `delta_facts = next_delta_facts`, clear the per-round
   `delta_alpha`/`delta_beta`, loop.

Return: `to_persistent(wm)` with `facts = input` (the `fire-rules` contract — derived live in
production-memory; same restore as P4a's `eval_fire_rules_native`).

## Why this is observationally identical to re-run (the correctness argument)
- Re-run's FINAL state = the last round's full firing over the whole closure = every join token created once,
  every production fired once. `query` counts those.
- Delta creates **the same multiset of tokens** (each token is produced in exactly the round its last-arriving
  support appears — semi-naive guarantees each qualifying tuple is enumerated exactly once across all rounds),
  and fires each production once. → identical `wm.production` multiset → identical `query` counts.
- The `seen` dedup mirrors `merge_facts`'s `contains?` guard → same fact set, same termination (monotone-finite,
  no round cap).
So `query(fire-rules' s, T) == query(fire-rules s, T)` for every T — the differential nets prove it at depth.

## The one contract decision (pinned)
Behavior is preserved (observable `query` results unchanged); only the algorithm's complexity changes
(O(depth²) re-run → linear delta). The differential nets are the proof of identity; the wat bench is the proof
of the bend. The wat oracle is the reference; `fire-rules'` conforms to it.

## Files touched
- `src/rete/kernel/fire/delta.rs` — add `fire_fixpoint_delta` (+ small helpers: a delta-alpha, delta-root-join, delta-
  hash-join using the existing `join_keys`/index + `extend_token`, delta-production using `build_insert_fact`);
  point `eval_fire_rules_native` at it. NO change to the four batch passes, `fire_once_session`, `fire-once'`,
  the keyed-join helpers, `WorkingMemory`'s shape, the dispatch arm, the TypeScheme, or the oracle.
- (the differential gate `tests/probe_arc278_deep_cascade.rs` already exists, green.)

## Out of scope = REJECTED
- **Retract / TM delta cascade** — P4c (this stone is insert-delta only; `fire-rules'` still fires a staged
  session from empty memories — the delta is the within-fire fixpoint, closing 4b's re-run).
- **Public `fire` / Clara bench** — P5.
- **Per-element (non-round) incremental / cross-fire persistent memories** — not needed for the bend; the
  round-based semi-naive delta is the win. A streaming insert-and-fire that reuses memories across separate
  `fire` calls is a later optimization, not this stone.
- No change to the batch passes, `fire-once'`, the oracle, or `WorkingMemory`'s shape.

# BRIEF — Stone P4b: delta-incremental `fire-rules'` (semi-naive)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A RUST
stone, ONE function family added to `src/rete/kernel.rs`. Build, run the named tests, report verbatim. Another
agent weighs. This is the hardest stone of the arc — read the DESIGN twice; implement the pinned algorithm, do
not invent a variant.

## The work
Convert the native `fire-rules'` fixpoint from **re-run-from-scratch** (P4a — clears + recomputes every round,
O(depth²)) to **semi-naive delta propagation** (memories persist + accumulate; each round propagates only the
new facts, joined against accumulated memory; linear). **Behavior-preserving**: `fire-rules'` must stay
observationally identical to wat `fire-rules` (same `query` counts). The differential gates are already GREEN
under P4a — they must STAY green; the win shows on the perf bench.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P4b-delta-fire.md` — **THE algorithm, pinned.** The
   semi-naive round loop, the delta sets, and especially the delta-join formula `Δbeta[J] = (Δbeta[P] ⋈
   all_alpha[A]) ∪ (old_left[P] ⋈ Δalpha[A])` with old_left = beta[P] before this round. Implement exactly this.
2. `src/rete/kernel.rs` — the current pieces you reuse, do NOT change:
   - `fire_once_session` + the four batch passes (`alpha_pass`, `root_join_pass`, `hash_join_pass`,
     `production_pass`) — the re-run path; KEEP them (they are `fire-once'` + the P4a re-run reference).
   - `eval_fire_rules_native` (`:925`) + `fire_fixpoint` (`:885`) — you repoint the entry from `fire_fixpoint`
     to your new `fire_fixpoint_delta`. Keep `fire_fixpoint` (the re-run path; `#[allow(dead_code)]` if needed).
   - The keyed-join internals INSIDE `hash_join_pass` (`:575-633`): the `join_keys` computation (sorted shared
     binding-key names) + the `HashMap<Vec<Value>,Vec<usize>>` index + probe. **Factor the keyed join into a
     reusable helper** `fn keyed_join(left_tokens: &[Value], right_elements: &[Value], alpha_id: i64) ->
     Vec<Value>` (returns the new extended tokens) so BOTH `hash_join_pass` (batch) and your delta join call
     it. This refactor must leave `hash_join_pass` behaviorally identical (P3 differential stays green).
   - Helpers: `alpha_match_inner` call shape (see `alpha_pass`), `make_element`/`make_token`,
     `element_fact_bindings`/`token_matches_bindings`, `extend_token`, `node_children`, `alpha_feeding`,
     `node_parent`, `build_insert_fact`, `collect_derived` (for the initial vs reuse), `session_facts`,
     `session_with_facts`, `to_transient`/`to_persistent`, `sorted_node_ids`, `kind_of`.
3. `wat/rete.wat` — the oracle (`fire-rules`/`fire-fixpoint`) is the REFERENCE; do NOT change it.
4. The gates (already live + green; do NOT modify): `tests/probe_arc278_deep_cascade.rs` (depth 10 + 20
   differential — THE net), `…P4a_native_fire_rules.rs`, `…P2_native_fire_once.rs`.

## Implementation sketch (fill it; the shape is fixed by the DESIGN)
```
fn fire_fixpoint_delta(session: &Value, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let mut wm = to_transient(session)?;          // staged session: memories start empty
    wm.alpha.clear(); wm.beta.clear(); wm.production.clear();   // start from empty (staged may carry stale)
    let mut seen: Vec<Value> = facts_vec(&wm.facts);            // all input facts
    let mut delta_facts: Vec<Value> = seen.clone();            // round 0 delta = input
    let node_ids = sorted_node_ids(&wm.network);
    loop {
        let mut d_alpha: HashMap<i64,Vec<Value>> = HashMap::new();
        let mut d_beta:  HashMap<i64,Vec<Value>> = HashMap::new();
        // 1. alpha delta: match ONLY delta_facts → new elements → wm.alpha + d_alpha
        // 2. root-join delta: per AlphaNode with d_alpha, seed tokens from NEW elements → wm.beta + d_beta
        // 3. hash-join delta (ascending id): capture old_len[P]=len(wm.beta[P]) BEFORE this step's appends;
        //    Δbeta[P] = d_beta[P]; old_left[P] = wm.beta[P][..old_len]; new tokens =
        //    keyed_join(Δbeta[P], all wm.alpha[A]) ++ keyed_join(old_left[P], Δalpha[A]); → wm.beta[J] + d_beta[J]
        // 4. production delta: per ProductionNode, for each NEW token in d_beta[parent], build_insert_fact;
        //    if !seen.contains(fact): seen.push, wm.production[prod].push, next_delta.push
        if next_delta.is_empty() { break; }
        delta_facts = next_delta;
    }
    Ok(session_with_facts(&to_persistent(wm), input_facts))  // facts = input (fire-rules contract)
}
```
The within-round order is alpha → root-join → hash-join (ascending id) → production, exactly as the batch
fire, but each step restricted to the delta. The hash-join's old_left/Δ split is the ONLY subtle part — get it
from the DESIGN's formula. `keyed_join` is the shared P3 helper.

## Builder directive: build missing deps, never hack around
Everything exists. **If a primitive is genuinely missing → STOP + name it.** Do NOT change the oracle, the
batch passes, `fire-once'`, or `WorkingMemory`'s shape.

## STOP triggers
1. A needed primitive is missing → STOP, name it.
2. The differential goes red and the only fix you see touches the wat ORACLE or the batch passes / `fire-once'`
   → STOP (the oracle is the reference; the delta engine conforms). The batch passes are P4a's reference path
   and must stay behaviorally identical (P2/P3 are their canary).
3. You reach for: retract / TM / support-store / public `fire` / cross-`fire` persistent memories / a Clara
   bench → P4c/P5; STOP.
4. The `keyed_join` extraction changes `hash_join_pass` behavior (P3 differential red) → STOP; the extraction
   must be behavior-preserving.
5. The delta join double-counts or misses (deep-cascade differential red: deepest ≠ width, or native ≠ wat) →
   STOP, re-read the DESIGN's `(Δleft ⋈ all_right) ∪ (old_left ⋈ Δright)` formula; do not patch with a dedup
   band-aid over a wrong join.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_deep_cascade                                   # 2/2 GREEN (depth 10 + 20: native delta == wat == full closure) — THE net
cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored      # 4/4 (single + cascade)
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored         # 4/4 (fire-once' unchanged)
cargo test --release -p wat --test probe_arc278_P2_native_fire_once 2>&1 | grep result           # (P3 keyed join lives here too — keyed_join extraction safe)
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored           # 4/4
cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored                # 4/4 (oracle TM intact)
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result       # 1/1
cargo test --release -p wat --lib rete 2>&1 | grep "test result"                                 # kernel units green
cargo test --release -p wat --lib 2>&1 | grep "test result"                                      # 935/36 (36 pre-existing UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                       # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                              # Finished; no NEW warnings
```
Report: `fire_fixpoint_delta` + the `keyed_join` extraction + the repoint of `eval_fire_rules_native`; all test
outputs verbatim; any STOP hit. No git.

## Blast radius
`src/rete/kernel.rs` ONLY: add `fire_fixpoint_delta` + helpers, extract `keyed_join`, repoint
`eval_fire_rules_native`. NO change to the batch passes' behavior, `fire-once'`, the oracle, `WorkingMemory`'s
shape, the dispatch arm, or the TypeScheme. No git.

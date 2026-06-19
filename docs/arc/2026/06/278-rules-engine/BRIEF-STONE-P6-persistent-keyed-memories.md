# BRIEF — Stone P6: persistent keyed join memories (beat Clara at width)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A RUST
stone, ONE function reworked: the hash-join delta inside `fire_fixpoint_delta` in `src/rete/kernel.rs`. Build,
run the named tests, report verbatim. Another agent weighs. Behavior-preserving — read the DESIGN; implement
the pinned index algorithm exactly (the update ORDERING is the correctness crux).

## The work
The native delta engine (`fire-rules'`) currently rebuilds the join index every round (`keyed_join` recomputes
a `HashMap` per call) — that re-index is why Clara wins width-heavy. Replace the two `keyed_join` calls in the
hash-join delta step of `fire_fixpoint_delta` with **persistent, incrementally-maintained, probed** indexes
(per join node, kept across rounds). **Same observable result** (the differential gates stay green); the win
shows on the Clara bench. Keep the round structure; keep `keyed_join` for the BATCH `hash_join_pass`
(`fire-once'`) — touch only the delta path.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P6-persistent-keyed-memories.md` — THE algorithm: per-J
   `left_idx`/`right_idx`/`join_keys` (persistent across rounds), and the 6-step update with the **exact
   ordering** (add Δright to right_idx → term1 probe right_idx with Δleft → term2 probe left_idx [old] with
   Δright → add Δleft to left_idx). That ordering IS the no-double-count/no-miss invariant.
2. `src/rete/kernel.rs` — `fire_fixpoint_delta` (the hash-join delta block you rework: the `for node_id`
   loop that computes `old_len_p`, `delta_left`, `all_right`, `delta_right` and calls `keyed_join` twice).
   Reuse: `element_fact_bindings`, `token_matches_bindings`, `extend_token`, `alpha_feeding`, `node_children`,
   `kind_of`, `sorted_node_ids`. Look at `keyed_join` (`:549`) for the `join_keys` computation + the key-tuple
   shape — factor a `key_of(bindings: &HashTrieMapSync<Value,Value>, join_keys: &[Value]) -> Vec<Value>` helper
   and use it for both the index build and the probe (and optionally inside `keyed_join` too).
3. `wat/rete.wat` — the oracle (`fire-rules-spec`) is the REFERENCE; do NOT change it.
4. The gates (green; do NOT modify): `tests/probe_arc278_deep_cascade.rs` (depth 10 + 20 — THE net),
   `…P4a_native_fire_rules`, `…P4c_native_retraction`, `…P2_native_fire_once`.

## Implementation sketch (the shape is fixed by the DESIGN)
In `fire_fixpoint_delta`, BEFORE the round loop, declare persistent maps:
```
let mut left_idx:  HashMap<i64, HashMap<Vec<Value>, Vec<Value>>> = HashMap::new();
let mut right_idx: HashMap<i64, HashMap<Vec<Value>, Vec<Value>>> = HashMap::new();
let mut join_keys: HashMap<i64, Vec<Value>> = HashMap::new();
```
Replace the hash-join delta block. For each parent P (Root/HashJoinNode, ascending id) with HashJoinNode child
J (feeding alpha A = `alpha_feeding(J)`), with `dl = Δbeta[P]` (this round's new tokens at P) and
`dr = Δalpha[A]` (this round's new elements at A):
```
// 1. join_keys[J] (cache): if absent, compute from a sample of wm.beta[P] ∩ wm.alpha[A] (sorted shared
//    var names, exactly as keyed_join does). If either side empty AND no dl/dr, skip.
// 2. add Δright: for el in dr { right_idx[J][key_of(el.bindings)].push(el) }
// 3. term1 (Δleft ⋈ all_right): for tok in dl { for el in right_idx[J].get(key_of(tok.bindings)) {
//        new = extend_token(tok.matches, tok.bindings, el.fact, el.bindings, A); push to wm.beta[J] + Δbeta[J] } }
// 4. term2 (old_left ⋈ Δright): for el in dr { for tok in left_idx[J].get(key_of(el.bindings)) {
//        new = extend_token(...); push to wm.beta[J] + Δbeta[J] } }   // left_idx still = OLD left here
// 5. add Δleft: for tok in dl { left_idx[J][key_of(tok.bindings)].push(tok) }
```
`key_of(bindings) = join_keys[J].iter().map(|k| bindings.get(k).cloned().expect(...)).collect()`. Empty
join_keys → `vec![]` key → one bucket (cartesian), correct.

Δbeta/Δalpha are the per-round delta sets you already build in steps 1-2 of the existing loop (alpha-delta,
root-join-delta). Keep those. Only the hash-join delta computation changes (persistent probe instead of
`keyed_join` rebuild). Production-delta step unchanged.

## STOP triggers
1. A needed primitive is missing → STOP, name it.
2. The deep-cascade differential goes red (deepest ≠ width, or native ≠ wat) → STOP, re-read the DESIGN's
   step ordering. Double-count (native > wat) = you added Δleft to left_idx BEFORE term2, or Δright not in
   right_idx for term1. Miss (native < wat) = you dropped term1 or term2. Fix the ORDERING, not with a dedup.
3. You change `keyed_join` semantics / the batch `hash_join_pass` / `fire-once'` (P2/P3 red) → STOP.
4. You change the oracle, the `WorkingMemory` stored shape, the public surface, or retract → STOP (out of scope).

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_deep_cascade                                   # 2/2 (depth 10 + 20: native delta == wat == closure) — THE net
cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored      # 4/4
cargo test --release -p wat --test probe_arc278_P4c_native_retraction                           # 3/3
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored         # 4/4 (keyed_join / fire-once' untouched)
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored           # 4/4
cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored                # 4/4
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result       # 1/1
cargo test --release -p wat --lib rete 2>&1 | grep "test result"                                 # green
cargo test --release -p wat --lib 2>&1 | grep "test result"                                      # 935/36 (36 pre-existing UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                       # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                              # Finished; no NEW warnings
```
Report: the reworked hash-join delta block (full code) + the `key_of` helper + the persistent index decls; all
test outputs verbatim; any STOP hit. No git. (The orchestrator runs the deep-cascade + Clara benches.)

## Blast radius
`src/rete/kernel.rs` — `fire_fixpoint_delta`'s hash-join delta block + a `key_of` helper. NO change to the batch
passes, `keyed_join`'s use by `hash_join_pass`, `fire-once'`, the oracle, `WorkingMemory`'s stored shape, the
dispatch arm, or the TypeScheme. No git.

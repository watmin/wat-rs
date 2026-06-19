# EXPECTATIONS — Stone P6: persistent keyed join memories

Independent scorecard, fixed BEFORE the strike. Behavior-preserving perf stone — every gate stays green; the
win is on the bench (native ≤ Clara at width). Weigh the index-update ORDERING (no double-count / no miss) +
the keyed_join/fire-once' untouched + the actual bend on the Clara bench hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | deep differential STAYS green (THE net) | `cargo test --release -p wat --test probe_arc278_deep_cascade` | **2/2** (depth 10 → 3, depth 20 → 2: native delta == wat == full closure) |
| 2 | P4a + P4c (single/cascade/retraction) | `…P4a_native_fire_rules -- --include-ignored` · `…P4c_native_retraction` | 4/4 · 3/3 |
| 3 | keyed_join / fire-once' untouched | `…P2_native_fire_once -- --include-ignored` | 4/4 |
| 4 | acceptance (native via wrapper) | `…4a_production_fire` · `…4c_retraction` · `…northstar` | 4/4 · 4/4 · 1/1 |
| 5 | rete units + lib floor | `…--lib rete` · `…--lib` | green · 935/36 (36 pre-existing UNCHANGED) |
| 6 | deftest / build | `…--test test` · `cargo build --release` | 264/1 · Finished, no NEW warnings |
| 7 | the win (perf, orchestrator runs, NOT gated) | deep-cascade.wat + Clara head-to-head | native-ns at 20×10 / 30×10 **drops below Clara** (was 12.2/36.2 vs Clara 12.1/14.1) |

## Trap-doors named — weigh hardest

- **The ordering is the whole correctness story.** The DESIGN's 6 steps must run in order: (2) add Δright to
  `right_idx` → (3) term1 probes `right_idx` (now full) with Δleft → (4) term2 probes `left_idx` (still OLD,
  Δleft not yet in) with Δright → (5) add Δleft to `left_idx`. Get it wrong:
  - Δleft added to `left_idx` BEFORE term2 → term2 double-counts Δleft×Δright (also in term1) → deepest > width,
    native > wat → row 1 red.
  - Δright NOT in `right_idx` before term1 → Δleft misses this round's new right → deepest < width → row 1 red.
  Depth 20 (row 1) is the canary — a shallow cascade can mask a subtly-wrong order. Read the diff against the
  DESIGN steps; do NOT accept a `seen`/dedup band-aid over a mis-ordered index (STOP-2).
- **Persistent means persistent.** `left_idx`/`right_idx`/`join_keys` are declared OUTSIDE the round loop and
  must NOT be cleared between rounds — that's the entire point (no rebuild). If they're rebuilt/cleared each
  round, row 1 may still pass but row 7 shows NO bend (native still ~36ms at 30×10). Row 7 is how a
  "correct-but-didn't-actually-persist" slips through — the orchestrator runs it.
- **keyed_join + fire-once' are untouched.** Only the delta path changes. `hash_join_pass` (batch) still calls
  `keyed_join`; P2/P3 (row 3) is the canary. Read the diff: `keyed_join`'s body + `hash_join_pass`'s call are
  byte-identical.
- **The oracle is NEVER touched.** `git diff wat/rete.wat` EMPTY. `git diff --stat` → only `src/rete/kernel.rs`.
- **No stored-shape change.** The indexes are LOCAL to `fire_fixpoint_delta` (rebuilt from the staged session
  each fire — value-semantics preserved). `WorkingMemory`'s `HashMap<i64,Vec<Value>>` fields unchanged. No new
  wat surface, no retract change.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-6 myself; ALL green/baseline. Row 1 at depth 10 AND 20 is the load-bearing correctness proof.
2. `git diff wat/rete.wat` → EMPTY. `git diff --stat` → only `src/rete/kernel.rs`.
3. Read the reworked block against the DESIGN's 6 steps — confirm the persistent decls are outside the loop,
   the add-Δright-before-term1 and term2-before-add-Δleft ordering, `key_of` matches `keyed_join`'s tuple.
4. **Run BOTH benches myself (row 7):** `deep-cascade.wat` across sizes (native-ns must drop at 20×10/30×10 vs
   the P4b numbers 12.2/36.2) AND re-run the Clara head-to-head (`wat-scripts/perf/clara/`) — the close
   condition is native ≤ Clara across 5×5 … 30×10. Read it as evidence.
5. If native now beats (or ties) Clara across the board → **arc 278 CLOSES at Clara-parity-or-superior**: write
   the close (REALIZATIONS R4 + the final head-to-head), consonare it, then arc 280. If a residual width gap
   remains → name the per-element (non-round) follow-on; do not over-claim.
6. Commit SCOPED on green; push.

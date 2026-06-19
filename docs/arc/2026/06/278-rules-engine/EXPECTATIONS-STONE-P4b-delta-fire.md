# EXPECTATIONS — Stone P4b: delta-incremental `fire-rules'` (semi-naive)

Independent scorecard, fixed BEFORE the strike. The hardest stone of the arc. Weigh the deep-cascade
differential (native delta == wat == full closure at depth) + the semi-naive correctness (no double-count, no
miss) + the keyed_join extraction safety hardest. Behavior-preserving — every gate that was green STAYS green;
the win is on the bench, not the gates.

| # | what | command | expected |
|---|---|---|---|
| 1 | deep differential STAYS green (THE net) | `cargo test --release -p wat --test probe_arc278_deep_cascade` | **2/2 GREEN** (depth 10 → 3, depth 20 → 2: native `fire-rules'` delta == wat == full closure) |
| 2 | P4a differential (single + cascade) | `…P4a_native_fire_rules -- --include-ignored` | 4/4 |
| 3 | fire-once' + keyed join unchanged | `…P2_native_fire_once -- --include-ignored` | 4/4 (the keyed_join extraction is behavior-preserving) |
| 4 | production + oracle TM + north star | `…4a_production_fire` · `…4c_retraction` · `…northstar` | 4/4 · 4/4 · 1/1 |
| 5 | rete units | `cargo test --release -p wat --lib rete 2>&1 \| grep "test result"` | green |
| 6 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 935/36 (the 36 pre-existing UNCHANGED) |
| 7 | deftest / build | `…--test test` · `cargo build --release` | 264/1 · Finished, no NEW warnings |
| 8 | the bend (perf, NOT gated, orchestrator runs) | `echo '[5 5]'… '[30 10]' \| wat wat-scripts/perf/deep-cascade.wat` | native-ns ~linear in depth (vs P4a's O(depth²) re-run); deepest==width every size |

## Trap-doors named — weigh hardest

- **Semi-naive double-count is THE failure mode.** The delta join `(Δleft ⋈ all_right) ∪ (old_left ⋈ Δright)`
  must use `old_left = beta[P] BEFORE this round`, not all of `beta[P]`. If the sonnet wrote `(Δleft ⋈
  all_right) ∪ (all_left ⋈ Δright)`, the (Δleft ⋈ Δright) pairs are counted twice → production counts inflate →
  the deep-cascade differential goes red (native > wat). Read the diff: confirm the old_left/Δ split via a
  beta-length snapshot at round start. A `seen`-style dedup band-aid hiding a double-counting join is a STOP-5
  violation, not a fix — check the join is correct, not just that the count happens to match on this fixture.
- **Miss is the other failure.** If the delta only does `(Δleft ⋈ all_right)` (forgets `old_left ⋈ Δright`),
  derivations that need a NEW right element against an OLD left token are lost → deepest < width → differential
  red (native < wat). Both terms must be present.
- **The deep-cascade gate is the canary, not the 2-deep P4a.** Depth 20 is where a subtly-wrong delta diverges
  from re-run (a shallow cascade can pass a broken delta by luck). Row 1 at depth 20 is the load-bearing
  assertion. If row 1 is green at BOTH depths and native==wat, the semi-naive is correct.
- **`keyed_join` extraction is behavior-preserving.** Pulling the P3 keyed index+probe out of `hash_join_pass`
  into a shared `keyed_join` must leave the batch `hash_join_pass` identical — P2/P3 (row 3) is the canary. Read
  the diff: `hash_join_pass`'s loop now calls `keyed_join(tokens, elements, alpha_id)`; the body of
  `keyed_join` is the old inline code verbatim.
- **Memories persist; do NOT clear between rounds.** The whole point: `wm.alpha/beta/production` accumulate
  across rounds. If the sonnet cleared them (copy-paste from `fire_once_session`), it's still re-run → the bench
  shows no bend (row 8 still O(depth²)) even though the differential is green. Row 8 (the bench) is how you
  catch a "correct but didn't actually go delta" — if native-ns still climbs O(depth²), the delta didn't take.
- **`facts = input` on return.** Same `fire-rules` contract as P4a — the returned Session's facts hold only the
  input, derived live in production-memory. (4c is the oracle canary; also read the diff.)
- **The oracle + batch passes are NEVER touched.** `git diff wat/rete.wat` EMPTY. The four batch passes +
  `fire_once_session` behaviorally identical (P2/P3/P4a are their canaries). `git diff --stat` → only
  `src/rete/kernel.rs`.
- **No scope creep.** No retract/TM, no public `fire`, no cross-`fire` persistence, no bench edits.

## Weigh (orchestrator — extra rigorous, this is the boss)
1. Re-run rows 1-7 myself; ALL stay green/baseline. Row 1 (deep differential) is the load-bearing proof —
   native delta == wat at depth 10 AND 20.
2. `git diff wat/rete.wat` → EMPTY. `git diff --stat` → only `src/rete/kernel.rs`.
3. Read `fire_fixpoint_delta` line by line against the DESIGN: persistent memories (no per-round clear); alpha
   delta over `delta_facts` only; the `(Δleft ⋈ all_right) ∪ (old_left ⋈ Δright)` join with the correct old_left
   snapshot; `seen` dedup on production; terminate on empty next-delta; `facts = input` restore.
4. Confirm `keyed_join` extraction left `hash_join_pass` behaviorally identical (read the diff).
5. **Run the bench myself (row 8)** — native-ns must flatten to ~linear in depth vs P4a's climbing re-run
   (compare against the P4a numbers in the deep-cascade harness commit: depth 5/10/20/30 native 1.36 / 7.2 /
   29 / 303 ms — under delta the growth should be markedly sub-quadratic). This is the proof the delta took;
   read it as evidence, not a gate.
6. Commit SCOPED on green; push. (Then P4c: delta retract + TM cascade; then P5: wire public `fire` + bench vs
   Clara → 278 CLOSES.)

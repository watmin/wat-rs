# EXPECTATIONS — Stone P3: keyed hash-joins

Independent scorecard, fixed BEFORE the strike. This is a **perf stone**: behavior is preserved, so the
differential is the correctness gate (asserted) and the bench is the proof of the bend (measured, not gated).
Weigh the differential-stays-green + the behavior-preserving discipline hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | differential STAYS green (the net) | `cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored` | **4/4 GREEN** (Oslo match=1, Bergen=0, 2×2 == exactly 2 same-loc joins / no cross-loc leakage, native==wat each) |
| 2 | oracle join untouched | `…3b_hash_join -- --include-ignored` | 4/4 (wat oracle join unchanged) |
| 3 | production + north star intact | `…4a_production_fire -- --include-ignored` · `…northstar_cold_and_windy` | 4/4 · 1/1 |
| 4 | rete unit tests | `cargo test --release -p wat --lib rete 2>&1 \| grep "test result"` | green (kernel round-trip + matcher) |
| 5 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 935/36 (the 36 pre-existing UNCHANGED) |
| 6 | deftest / load order | `…--test test` · `test_stdlib_load_order` | 264/1 · 1/0 |
| 7 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings |
| 8 | the bend (perf, NOT gated) | `…perf_arc278_fire_baseline native_fire_once_join_scaling -- --ignored --nocapture` | us/fact ~flat across N (vs the climbing P2 baseline 19.3→62.3) — measured, eyeballed, no timing assertion |

## Trap-doors named — weigh hardest

- **The 2×2 is THE canary.** Row 1's `native_no_cross_loc_leakage` asserts exactly 2 same-loc joins, not 4
  (cross-product leakage) and not 0 (broken keying). If keying on the shared var drops a real match → it goes
  to 0 or 1; if the bucket leaks → it goes to 4. Either way row 1 fails. A keyed join that passes the 2×2 is
  the proof the bucket == the compatible set.
- **Behavior-preserving means the differential cannot move.** P3 changes NO observable behavior. If the
  differential needed ANY edit to stay green, the rewrite changed behavior — that is a mis-port, not a pass.
  Read the diff: only `hash_join_pass` changed; the probe file is byte-identical.
- **Empty shared keys must still cross.** A join with no shared var → every tuple is `[]` → one bucket → full
  cross (correct). Confirm the rewrite handles `join_keys == []` naturally (no panic, no skip). The current
  north-star/2×2 worlds all DO share `?loc`, so this path isn't directly asserted — verify it by READING the
  code (an empty `join_keys` yields `vec![]` tuples on both sides → same bucket → cross). If the sonnet
  special-cased it with a branch, check that branch is equivalent to the cross.
- **The oracle is NEVER touched.** `git diff wat/rete.wat` must be EMPTY. The native keyed join conforms to the
  oracle's cross; the oracle does not move to meet it.
- **No `WorkingMemory` shape change.** The index is a transient `HashMap<Vec<Value>, Vec<usize>>` (or
  `Vec<&Value>`) built inside the pass per (node,child). The stored `wm.alpha/beta` stay `HashMap<i64,
  Vec<Value>>`. If the sonnet changed the stored memory shape → that's P4 scope creep; STOP-worthy.
- **No scope creep.** Only `hash_join_pass`. No other pass, no fixpoint/delta/retract, no public `fire`, no
  bench changes (the bench already exists; running it is fine, editing it is not).
- **The bend is measured, not asserted.** Row 8 has NO hard threshold — machine-relative timings race in CI.
  The orchestrator eyeballs us/fact-goes-flat by re-running the bench and reading the keyed diff. A flat-ish
  curve + a correct keyed diff = the bend earned.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-7 myself; ALL stay at baseline — only the *internals* of `hash_join_pass` change, nothing in
   the observable surface. If ANY of rows 1-7 moved from baseline, the rewrite isn't behavior-preserving.
2. `git diff wat/rete.wat` → EMPTY (oracle untouched). `git diff --stat` → only `src/rete/kernel.rs` (+ the
   two new docs already committed STRIKE-READY). The probe file UNCHANGED.
3. Read the rewritten `hash_join_pass` line by line against the oracle's `cross-join-node` +
   `token-element-compatible?`: the join key = sorted shared-var names; the index buckets right elements by
   key-tuple; the probe hits exactly the matching bucket; `extend_token` called per bucket element with the
   same (matches conj + bindings merge) semantics as before. Confirm the empty-`join_keys` path == the cross.
4. Run the bench myself (row 8); confirm us/fact flattens vs the captured climbing baseline. Read it as
   evidence, not a gate.
5. Commit SCOPED on green; push. (Then P4: incremental delta = Clara's smart activation — key the STORED
   memory; native fire-rules fixpoint via delta == wat fire-rules; TM/retract delta cascade.)

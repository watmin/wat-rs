# EXPECTATIONS — Stone P4a: native `fire-rules'` (re-run fixpoint cascade)

Independent scorecard, fixed BEFORE the strike. Weigh the differential (native == wat derived facts, including
the CASCADE) + the behavior-preserving `fire-once'` extraction + no-oracle-change hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | differential: native fire-rules' == wat | `cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored` | **4/4 GREEN** (single match=1, no-match=0, cascade ColdAndWindy=1, cascade WeatherAlert=1 — each native==wat) |
| 2 | fire-once' unchanged by the extraction | `…P2_native_fire_once -- --include-ignored` | 4/4 |
| 3 | production + oracle TM + north star | `…4a_production_fire` · `…4c_retraction` · `…northstar_cold_and_windy` | 4/4 · 4/4 · 1/1 |
| 4 | rete unit tests | `cargo test --release -p wat --lib rete 2>&1 \| grep "test result"` | green |
| 5 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 935/36 (the 36 pre-existing UNCHANGED) |
| 6 | deftest / load order | `…--test test` · `test_stdlib_load_order` | 264/1 · 1/0 |
| 7 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings |

## Trap-doors named — weigh hardest

- **The cascade is THE canary.** `native_matches_wat_cascade_second_rule` asserts WeatherAlert=1 — derivable
  ONLY if the round-1 ColdAndWindy re-entered the network and triggered ruleB. If the fixpoint doesn't actually
  loop (e.g. it returns after one pass), native goes to 0 while wat is 1 → row 1 fails. A passing cascade is
  the proof the fixpoint propagates derived→higher-rule across rounds.
- **The termination guard is the dedup.** `merge_facts` must conj ONLY non-present facts (structural `==`), and
  the loop terminates on `len(new_facts) == len(cur.facts)`. If dedup is wrong (re-adds a present fact), facts
  grows every round → infinite loop (test hangs) — NOT a wrong answer, a HANG. If the loop terminates too early
  (returns before fixpoint), the cascade under-derives. Read the loop against `fire-fixpoint` line by line.
- **`facts = input` on return.** The returned Session's `facts` must hold ONLY the input/asserted facts, NOT
  the derived closure (derived live in production-memory). This is the `fire-rules` contract (4c TM depends on
  it). If the sonnet returns `facts = closure`, the 4c retraction probe (row 3) is the canary — but it tests
  the ORACLE's facts, not fire-rules'; so ALSO read the diff: `eval_fire_rules_native` must restore
  `facts = input` exactly like `fire-rules` (`wat/rete.wat:1006-1018`).
- **The `fire_once_session` extraction is behavior-preserving.** Pulling the pure pass out of
  `eval_fire_once_native` must leave `fire-once'` identical — row 2 (P2, 4/4) is the canary. Read the diff: the
  entry evals the arg then calls `fire_once_session`; the inner is the old body verbatim (to_transient → clear
  → 4 passes → to_persistent).
- **The oracle is NEVER touched.** `git diff wat/rete.wat` must be EMPTY. If matching the differential needed
  an oracle change, the native impl is wrong.
- **No delta / no scope creep.** P4a loops `fire_once_session` (re-run-from-scratch). NO persistent-across-round
  memories, NO keyed STORED memory, NO retract/TM, NO public `fire`, NO bench. `grep` the diff for unexpected
  `:wat::` additions beyond the single `:wat::rete::fire-rules'` dispatch arm + TypeScheme.
- **Internal mutation sealed.** `WorkingMemory` never returns to wat; only a frozen `Session` via
  `to_persistent`. No new wat-callable mutation/transient op.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-7 myself; 2-7 EXACTLY baseline (+ no new lib tests unless the sonnet added kernel units); only
   row 1 flips RED→GREEN.
2. `git diff wat/rete.wat` → EMPTY. `git diff --stat` → only `src/rete/kernel.rs`, `src/runtime.rs`,
   `src/check.rs`, the probe.
3. Read the fixpoint line by line against `fire-fixpoint`: round = `fire_once_session`; `collect_derived`;
   `merge_facts` (dedup); terminate on no-new-fact; recurse with `facts = new_facts`. Confirm
   `eval_fire_rules_native` restores `facts = input`. Confirm `fire_once_session` extraction left `fire-once'`
   byte-equivalent in behavior.
4. Confirm the cascade actually exercises ≥2 rounds (the WeatherAlert=1 result proves it).
5. Commit SCOPED on green; push. (Then P4b: convert `fire-rules'` to delta-incremental — persistent memories +
   delta propagation; the P4a differential stays green, the bench shows the re-run→delta bend.)

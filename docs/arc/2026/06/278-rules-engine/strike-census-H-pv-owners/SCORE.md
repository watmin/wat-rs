# SCORE — the tripwire trips, and the sum is named as a sum

`assert_eq!(total, 0)` is armed, with the cliff text as the failure message. `merge:pv-owners` is `merge:pv-owners-sum`. Both facts now sit at the increment. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ HEAD measured first | **HOLD.** Before any edit: `calls 6   summed owners 0   mean 0.00   arm Tree (rpds VectorSync)`. `total == 0`, `calls == STRATA`. Not STOP-1. |
| 2 ★ the tripwire is armed | **HOLD.** `assert_eq!(total, 0, "{out}")` — `{out}` is the existing LATENT CLIFF paragraph, pointing at `strat_merge_cow_parts`. |
| 3 ★ non-vacuity written down | **HOLD.** At the assertion: both bumps live in ONE `#[cfg(test)]` block in `fire/rules.rs`, so `calls == STRATA` proves the block ran. |
| 4 ★ that non-vacuity is proved live | **HOLD, with the arm named.** Deleted `census_count_n("merge:pv-calls", 1)` on the live test. RED at `strat_cost.rs:512` — `assert!(calls > 0)`, *before* `assert_eq!(calls, STRATA)`. Quoted below. Did not reorder to steal the STRATA arm. Restored. |
| 5 ★ rename + site facts | **HOLD.** `merge:pv-owners-sum`. Increment comment: sum across calls; `0` means Tree, not zero owners. `grep -rn '"merge:pv-owners"' src/ tests/` → no hits. |
| 6 no split | **HOLD.** No Array/Tree counters added. |
| 7 floor | **HOLD.** `5465 passed, 21 skipped` — same count. No new tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is what stops row 2 being a tautology**, and row 4 is what stops row 3 being a claim.

## Row 1 — HEAD, before anything

```
merge_facts receiver — strat-neg [6 2000]
calls 6   summed owners 0   mean 0.00   arm Tree (rpds VectorSync)
```

## Row 4 — live mutation

Deleted `census_count_n("merge:pv-calls", 1)`. Drove `strat_merge_pv_owner_count` (the live test):

```
merge_facts receiver — strat-neg [6 2000]
calls 0   summed owners 0   mean 0.00   arm Tree (rpds VectorSync)

thread '…strat_merge_pv_owner_count' panicked at src/rete/kernel/tests/strat_cost.rs:512:5:
merge_facts never ran:
```

`calls > 0` is the first of the two call-count gates; `calls == STRATA` is the next line and did not run (`0 != 6` would also fail). The pair is live: deleting the calls bump makes `total == 0` the tautology the comment warns about, and the earlier gate catches it. Restored.

After restore, same numbers as HEAD: `calls 6   summed owners 0   mean 0.00   arm Tree`.

## Final floor

`.floor/2026-09-06T23-29-08Z/`: `Summary [ 460.321s] 5465 tests run: 5465 passed (2 slow), 21 skipped`. Count unchanged. No new tests.

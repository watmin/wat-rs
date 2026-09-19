# SCORE — `seed:mixed-class-activate` is `seed:mixed-fact-activate`

Latent. Benign but real. No number was wrong; the consumer already spelled the units. One key renamed so the family no longer implies the wrong unit. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ rename complete | **HOLD.** `grep -rn 'seed:mixed-class-activate' src/ tests/` → no hits. |
| 2 ★ the mutation REDs | **HOLD.** Deleted the bump. Live test `seed_batches_uniform_classes_and_defers_mixed_ones`. Arm named below. Restored. |
| 3 ★ units stated at the sites | **HOLD.** Two lines at the fact bump: `batch-class-*` per CLASS; this one per FACT of a mixed class. |
| 4 siblings untouched | **HOLD.** `seed:batch-class-uniform` / `seed:batch-class-mixed` literals unchanged. |
| 5 severity reported honestly | **HOLD.** Latent. The audit's word is "benign but real." A reader's second glance was at risk, not a live number. |
| 6 floor | **HOLD.** `5465 passed, 21 skipped` — same count. No new tests. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is required because it is available.**

## Row 2 — live mutation, the arm that fired

Deleted `census_count("seed:mixed-fact-activate")`. Drove `seed_batches_uniform_classes_and_defers_mixed_ones`.

**Direction 1** (`all-uniform`, `activated == 0` at `:728`) **passed** — unwrap_or(0) is what that arm already expects.

**Direction 2** (`box-mixed`) caught it, **`pass_semantics.rs:747`**:

```
assertion `left == right` failed: box-mixed world: ALL THREE Box facts must take the activate path — batching the two that packed is the double-write that dropped a derived fact; 0 were activated
  left: 0
 right: 3
```

`assert_eq!(activated, 3)`. `0 != 3`. Restored. Test green.

## Final floor

`.floor/2026-09-07T00-04-32Z/`: `Summary [ 460.196s] 5465 tests run: 5465 passed (3 slow), 21 skipped`. Count unchanged. No new tests.

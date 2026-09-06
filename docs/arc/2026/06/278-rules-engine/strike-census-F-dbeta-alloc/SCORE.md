# SCORE — `dbeta:alloc` is `dbeta:nonempty` now

0 or 1 per call, not an allocation count. Same expression, new name. The printed column no longer says `allocating`. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ rename complete | **HOLD.** `grep -rn '"dbeta:alloc"' src/ tests/` → no hits. Leftover `dbeta:alloc` in `fire/mod.rs:1068` is a comment *without* double quotes, naming the rejected prefix. |
| 2 ★ the mutation REDs | **HOLD.** Deleted `census_count_n("dbeta:nonempty", …)` on the live engine. `node_share_where_cost_decomposition` failed at `:314`. Quoted below. Restored. |
| 3 ★ the human-facing label is true | **HOLD.** `dbeta_gather_volume` column is `nonempty`. Quoted below. |
| 4 ★ quantity unchanged | **HOLD.** Still `u64::from(!out.is_empty())`. Identity: `gathers 50 x 200 tokens` (50 × 200 = 10,000). Accum world: `dbeta:nonempty = 5`. |
| 5 ★ the rejection is recorded | **HOLD.** Bump-site comment: counting real allocations considered and rejected — would exceed `dbeta:calls`, break the `node_share_cost.rs` identity, silently mis-scale arm L. |
| 6 siblings untouched | **HOLD.** `dbeta:calls` / `dbeta:tokens` / `dbeta:multi` literals unchanged. |
| 7 NAMES list still sorted | **HOLD.** `accum_cost.rs`: `dbeta:calls`, `dbeta:multi`, `dbeta:nonempty`, `dbeta:tokens`. |
| 8 floor | **HOLD.** `5465 passed, 21 skipped` — same count. No new tests. |
| 9 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is the one this strike can actually produce** — D and E could not.

## Row 2 — live mutation

Deleted `census_count_n("dbeta:nonempty", u64::from(!out.is_empty()))`. Drove `node_share_where_cost_decomposition` (the live test, not a copy):

```
thread '…node_share_where_cost_decomposition' panicked at src/rete/kernel/tests/node_share_cost.rs:314:5:
`d_beta_from_parents` reported 0 non-empty gathers carrying 10000 tokens; arm L replays 0 clones of a 200-token vector and that only reconstructs the gather if the width divides out
```

`fire_gathers > 0` failed (0). `dbeta:tokens` still 10,000 — the sibling was not the thing that died. Restored.

## Row 3 — printed table

```
d_beta_from_parents volume

strat-neg [6 2000]   calls     12   nonempty      6   MULTI-parent      0   tokens    12000
accum [200 200]      calls     10   nonempty      5   MULTI-parent      0   tokens     1000
```

No `allocating`. Field and column both `nonempty`.

## Row 4 — identity after restore

```
  fire counters, same axis: reuse 200  evals 0  gathers 50 x 200 tokens
```

50 × 200 = 10,000. Same numbers the mutation left on the token side.

## Final floor

`.floor/2026-09-06T22-31-13Z/`: `Summary [ 462.898s] 5465 tests run: 5465 passed (3 slow), 21 skipped`. Count unchanged. No new tests.

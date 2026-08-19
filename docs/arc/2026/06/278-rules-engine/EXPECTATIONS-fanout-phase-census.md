# EXPECTATIONS — fanout-phase-census (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | table | `cargo nextest run --release -E 'test(fanout_fire_phase_census)' --no-capture` | three rungs; hash-join row; ROUND LOOP > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Copy the accum helper.

**Predicted:** hash-join + production own the 141 ms. `hj:step6-emit`
/ `right_idx` clone likely lead. Rank from the table.

## Trap doors

1. Sizes are `(keys, fanout)`, not `(items, fanout)`.
2. Do not assert filter-dominates (stale node-share copy).

## Will not accept

- Engine rewrite in this diff.
- FIRE wall gate.

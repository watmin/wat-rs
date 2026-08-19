# EXPECTATIONS — drop-memories-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(drop_memories_cost_split)' --no-capture` | A / B / M / T / D printed; D > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Census only.

**Predicted:** A (fact Arc Drop) leads ~2 ms. B second. T ~0.
Next intern is fact-as-index, not this stone.

## Trap doors

1. Do not retry 2e (fatter Element ate the drop win).
2. Do not arena-and-forget (2f).
3. Rebuild the vec each run — timing Drop, not construction.

## Will not accept

- Engine rewrite in this diff.
- Persist gather. 2e retry. Intern `names`.

# EXPECTATIONS — cell-rank-after-grid (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rank | `cargo nextest run --release -E 'test(cell_rank_after_grid)' --no-capture` | three FIRE rows; each > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Census only.

**Predicted:** fanout `[100 20]` leads native FIRE
(~50–60 ms). Deep-cascade `[50 100]` ~35 ms. Accum
`[200 200]` ~26 ms. Next intern cell is fanout.

## Trap doors

1. FIRE is the four TOP phases, not a parent-plus-children sum.
2. Do not rank off the T23-57-10Z wat-ns (141 ms fanout is dead).
3. Do not intern `setup:seen` off this rank.

## Will not accept

- Engine rewrite in this diff.
- Clara grid as this stone's gate.
- Per-fact timers. 297. Insertion.

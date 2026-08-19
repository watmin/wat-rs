# EXPECTATIONS — cell-rank-after-fanout (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rank | `cargo nextest run --release -E 'test(cell_rank_after_fanout)' --no-capture` | three FIRE rows; each > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Census only.

**Predicted:** accum `[200 200]` leads native FIRE. Node-share
is polish. Fanout is dry. Next cell is accum.

## Trap doors

1. FIRE is the four TOP phases, not a parent-plus-children sum.
2. Do not rank off the T23-57-10Z wat-ns (141 ms fanout is dead).
3. Do not start persist-gather because accum "might" be multi-round.

## Will not accept

- Engine rewrite in this diff.
- Clara grid as this stone's gate.
- Fanout intern.

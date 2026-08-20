# EXPECTATIONS — honest-rank-after-arm (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rank | `cargo nextest run --release -E 'test(honest_cell_rank_after_arm)' --no-capture` | three FIRE > 0; fanout honest < raw |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 10–20 minutes.** Census only.

**Predicted:** accum leads honest FIRE (~21 ms). Fanout
honest ~10. Cascade honest ~17. Production is not internable.

## Trap doors

1. Same production kids as 2p. Do not restore 80k marks
   on other phases.
2. Do not intern off raw top-row.
3. FIRE is the four TOP phases.

## Will not accept

- Engine rewrite. Production intern. 297. Insertion.

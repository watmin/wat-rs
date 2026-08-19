# EXPECTATIONS — honest-fire-rank (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rank | `cargo nextest run --release -E 'test(fanout_honest_fire_rank)' --no-capture` | honest_FIRE / probe / compiled-rhs_net printed; rhs pairs = 40 000 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Census only.

**Predicted:** honest_FIRE ≈ 27. Probe 7.08 leads engine
leftover and is 2o-dead on B. compiled-rhs_net ~5 is a pile.
No drawable intern. Absolute ms will wash vs 39.48.

## Trap doors

1. Subtract **remainder + tax**, not child nets, from FIRE.
2. Do not treat production raw as engine work (2p).
3. Phase names carry the box-drawing prefix.

## Will not accept

- Engine rewrite in this diff.
- FIRE wall gate.
- Intern `names`. 2o retry.

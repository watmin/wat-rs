# EXPECTATIONS — prod-leftover-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(fanout_production_leftover_split)' --no-capture` | remainder_raw / tax_in_parent / compiled-rhs_net printed; rhs pairs = 40 000 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Census only.

**Predicted rank:** tax_in_parent owns most of the naive ~12 ms
unmarked. remainder_raw is small. compiled-rhs_net is the real
wall and is still 2l's pile (bind-get / stamp / wrap / Arc / Vec).
Absolute ms will wash vs 19.34 — the ratio names 2p.

## Trap doors

1. Do not add 40k nested marks (rows go below instrument).
2. Subtract **raws** for remainder. Subtracting nets double-counts
   the children's clock reads as leftover work.
3. Phase names carry the box-drawing prefix
   (`  ├ prod:compiled-rhs`).

## Will not accept

- Engine rewrite in this diff.
- FIRE wall gate.
- Intern `names`. Skip stamp. Retry 2o.

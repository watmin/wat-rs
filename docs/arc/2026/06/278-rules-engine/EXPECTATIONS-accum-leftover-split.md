# EXPECTATIONS — accum-leftover-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_leftover_split)' --no-capture` | remainder_alpha / tax_in_alpha / honest_FIRE printed; FIRE > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Census only.

**Predicted:** remainder+tax own most of alpha 41.65.
honest_alpha is a few ms. Leftover engine is `setup:seen`
and/or drop. Fold stays small. Absolute ms will wash vs 61.72.

## Trap doors

1. Subtract **raws** for remainder. Nets double-count tax.
2. Do not intern alpha off 41.65.
3. `alpha:candidates` may sit below its own instrument.

## Will not accept

- Engine rewrite in this diff.
- Persist gather. 297. Alpha intern off the raw.

# EXPECTATIONS — compiled-match-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_compiled_match_split)' --no-capture` | O > 0; table printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** `O−T` owns most of 7.65. `Mc−O` < 1 ms
unless `fact_bind` clones Records. FIRE not wall-gated.
No intern.

## Trap doors

1. Do not restore 281,800 child timers.
2. Cold intern on Mc (first-fire). Mw keeps pools.
3. Do not intern this stone.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

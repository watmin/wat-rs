# EXPECTATIONS — fact-as-index (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 2 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 3 | isolated A | `cargo nextest run --release -E 'test(drop_memories_cost_split)' --no-capture` | A collapses toward T (~0) |
| 4 | in-fire | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `[200 200]` drop + FIRE printed; fold < 25; snapshot < 1 |

## Independent prediction

**Runtime: 25–40 minutes.** Type change + weigh.

**Predicted:** isolated A **~0**. in-fire `drop-memories`
3.63 → **~2.6**. FIRE 57.92 → **~56–57**. Push stays thin.

## Trap doors

1. Do not retry 2e (fatter Element).
2. Do not arena-and-forget (2f).
3. Do not `fact_pool.clone()` then clear (same Drop).
4. `VectorSync::get` on a hot reader can eat the win — print
   it; do not cache `Value` back onto Element.

## Will not accept

- FIRE wall-gate. Persist gather. 2e retry. Intern `names`.
- Facts in `bind_pool`. `match_pool` rewrite in this diff.

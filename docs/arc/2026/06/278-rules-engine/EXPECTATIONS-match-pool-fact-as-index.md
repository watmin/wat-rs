# EXPECTATIONS — match-pool fact-as-index (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 2 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 3 | isolated M | `cargo nextest run --release -E 'test(drop_memories_cost_split)' --no-capture` | M collapses toward T (~0) |
| 4 | in-fire | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `[200 200]` drop + FIRE printed; fold < 25; snapshot < 1 |

## Independent prediction

**Runtime: 20–35 minutes.** Type change + weigh.

**Predicted:** isolated M **~0**. D **~0.8** (B remains).
in-fire drop 1.14 → **~0.8**. FIRE 51.56 → **~49–50**.

## Trap doors

1. Do not retry 2e (fatter Element) or 2o (two-span get).
2. Do not skip matches on `fire-rules` (2n).
3. Do not `fact_pool.clone()` then clear.
4. Encode/explain must still show the fact Value.

## Will not accept

- FIRE wall-gate. Persist gather. Intern `names`.
- Facts in `bind_pool`. Token grows a third field.

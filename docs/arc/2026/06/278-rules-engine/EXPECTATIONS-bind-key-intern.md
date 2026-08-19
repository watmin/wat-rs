# EXPECTATIONS — bind-key intern (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 2 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 3 | isolated B | `cargo nextest run --release -E 'test(drop_memories_cost_split)' --no-capture` | B collapses toward T (~0) |
| 4 | in-fire | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `[200 200]` drop + FIRE printed; fold < 25; snapshot < 1 |

## Independent prediction

**Runtime: 25–40 minutes.** Type change + weigh.

**Predicted:** isolated B **~0**. D **~0**. in-fire drop 1.18
→ **~0.4**. FIRE 53.83 → **~52–53**.

## Trap doors

1. Do not intern record `names`.
2. Do not put facts in `bind_pool`.
3. Do not revert on alpha-instrument wash (2ac).
4. Encode/query still surface `?var` keys as Values.

## Will not accept

- FIRE wall-gate. Persist gather. Process-lifetime `?var` intern.
- Token grows a third field. Facts in the pair pool.

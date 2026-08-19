# EXPECTATIONS — bind-value intern (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 2 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 3 | isolated B | `cargo nextest run --release -E 'test(drop_memories_cost_split)' --no-capture` | B collapses toward T (~0) |
| 4 | in-fire | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `[200 200]` drop + FIRE printed; fold < 25; snapshot < 1 |

## Independent prediction

**Runtime: 25–40 minutes.** Type change + weigh.

**Predicted:** isolated B **~0**. D **~0**. in-fire drop 0.46
→ **~0.1**. FIRE 48.52 → **~48–49**.

## Trap doors

1. Do not intern record `names`.
2. Do not put facts in `bind_pool`.
3. Do not linear-scan intern values.
4. Do not revert on alpha-instrument wash (2ac).
5. Encode/query still surface filler Values.

## Will not accept

- FIRE wall-gate. Persist gather. Process-lifetime intern.
- Token grows a third field. Facts in the pair pool.
- Fact-insertion strike (parked).

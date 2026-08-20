# EXPECTATIONS — materialize-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_materialize_split)' --no-capture` | V > 0; table printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** `V−K` (`intern_val` get) owns most of
6.18. Clone / key / push small. FIRE not wall-gated.
No intern.

## Trap doors

1. Do not restore 281,800 child timers.
2. Cold intern each C/K/V/P/M run.
3. Do not intern this stone.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

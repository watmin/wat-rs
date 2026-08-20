# EXPECTATIONS — cond-key-ids (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | leftover | `cargo nextest run --release -E 'test(accum_leftover_split)' --no-capture` | FIRE > 0 |
| 2 | split | `cargo nextest run --release -E 'test(accum_materialize_split)' --no-capture` | K−C printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** FIRE −1.2 ms (20.93 → ~19.7). Isolated
K−C stays ~1.2. Scratch untouched.

## Trap doors

1. Isolated K still calls `intern_key` per fact.
2. `bind_keys` still cleared each fire.
3. Do not intern record `names`.

## Will not accept

- Process-lifetime `?var` intern. 297. Insertion. 2e / 2o.

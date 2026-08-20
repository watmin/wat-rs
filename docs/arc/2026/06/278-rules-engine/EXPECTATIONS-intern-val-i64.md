# EXPECTATIONS — intern-val-i64 (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_intern_val_i64_split)' --no-capture` | V > 0; unique/range printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** I cuts ~1.5–2 ms. Intern i64 map.
FIRE not wall-gated.

## Trap doors

1. Do not intern `seen`.
2. Non-i64 still uses `FxHashMap<Value>`.
3. Do not put i64 in the pool pair.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

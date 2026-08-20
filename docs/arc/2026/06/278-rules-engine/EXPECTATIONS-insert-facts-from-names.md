# EXPECTATIONS — insert-facts-from-names (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | probe | `echo '[20000]' \| ./target/release/wat wat-scripts/scratch-pad/probe-insert-cost-split.wat` | insert-len = n; insert − conj printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–20 minutes.**

**Predicted:** insert − conj 2037 → ~900 ns/fact.

## Trap doors

1. By name, never slot 5.
2. `available` only on miss.
3. insert-all' shares the helper.

## Will not accept

- Session-`Vec`. 297. Fire-path. 2-ary through insert-all.

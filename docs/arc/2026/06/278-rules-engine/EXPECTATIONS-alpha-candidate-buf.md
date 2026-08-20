# EXPECTATIONS — alpha-candidate-buf (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | leftover | `cargo nextest run --release -E 'test(accum_alpha_leftover_split)' --no-capture` | A > 0; seed > 0; T−I still named on walk split |
| 2 | walk | `cargo nextest run --release -E 'test(accum_alpha_tree_walk_split)' --no-capture` | T−I printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** isolated A −2.6 ms. Seed 12.83 → ~10.2.
FIRE 21.08 → ~18.5. `setup:seen` unchanged.

## Trap doors

1. Isolated T still calls `candidates()` (new Vec).
2. Over-approx unchanged.
3. Do not intern `setup:seen`.

## Will not accept

- 297. Insertion. Retry 2e / 2o. Range edges.

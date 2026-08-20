# EXPECTATIONS — alpha-tree-fxhash (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | walk | `cargo nextest run --release -E 'test(accum_alpha_tree_walk_split)' --no-capture` | I−G printed |
| 2 | leftover | `cargo nextest run --release -E 'test(accum_leftover_split)' --no-capture` | FIRE > 0 |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** I−G 1.03 → ~0.4. FIRE −0.5. Scratch STOP.

## Trap doors

1. Over-approx unchanged.
2. Do not intern scratch.
3. Isolated T still allocates a new Vec.

## Will not accept

- Range edges. 297. Insertion. Intern `names`.

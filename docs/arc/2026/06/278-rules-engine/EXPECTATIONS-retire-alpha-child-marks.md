# EXPECTATIONS — retire-alpha-child-marks (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | leftover split | `cargo nextest run --release -E 'test(accum_leftover_split)' --no-capture` | alpha child pairs 0; honest_FIRE > 0; setup:seen printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 4 | node-share | `cargo nextest run --release -E 'test(node_share_filter_eval_census)' --no-capture` | `[50 200]` reuse 200, evals 0 |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** FIRE 48.65 → **~18–22**. honest_alpha 2–8.
setup:seen ~4 stays. Not wall-gated.

## Trap doors

1. Do not under-approx (skip a true alpha match).
2. Outer `alpha` still fires (2×/round).
3. Do not fold `setup:seen` this stone.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

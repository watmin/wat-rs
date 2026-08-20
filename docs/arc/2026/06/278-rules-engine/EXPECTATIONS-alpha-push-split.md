# EXPECTATIONS — alpha-push-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_alpha_push_split)' --no-capture` | D > 0; table printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** `H−M` ≥ 1 ms. Intern FxHashMap for
`wm.alpha` + `d_alpha`. FIRE not wall-gated.

## Trap doors

1. Do not intern `setup:seen`.
2. Do not change `beta` / `production` this stone.
3. Do not put facts in `bind_pool`.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

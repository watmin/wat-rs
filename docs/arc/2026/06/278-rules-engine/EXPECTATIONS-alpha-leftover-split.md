# EXPECTATIONS — alpha-leftover-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_alpha_leftover_split)' --no-capture` | seed > 0; A > 0; table printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** seed ~17 ms, delta ~1 ms. `M−T` largest
isolated piece. FIRE not wall-gated. No intern.

## Trap doors

1. Do not restore 281,800 child timers.
2. Cold intern each isolated run (first-fire, not warm).
3. Do not intern a Session `Vec`.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

# EXPECTATIONS — exec-ops-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_exec_ops_split)' --no-capture` | O > 0; table printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** R−T ≥ 1 ms. Intern fill(None). FIRE not
wall-gated. `seen` untouched.

## Trap doors

1. Do not intern `seen`.
2. Do not skip clearing leftover scratch slots.
3. Do not add a second scratch representation.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

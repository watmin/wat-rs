# EXPECTATIONS — alpha-tree-walk-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_alpha_tree_walk_split)' --no-capture` | I > 0; table printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** `T−I` (alloc) 1.5–2.5 ms → intern reused
buffer. Walk owns the rest. FIRE not wall-gated.

## Trap doors

1. Do not under-approx (drop a true alpha).
2. Do not populate alpha-tree `range_children`.
3. Do not intern HashMap keys this stone.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

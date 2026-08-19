# EXPECTATIONS — where-range-edges (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | unit | `cargo nextest run --release --lib rete::where_tree` | 0 failed; range prune/prove |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 4 | node-share | `cargo nextest run --release -E 'test(node_share_filter_eval_census)' --no-capture` | `[50 200]` reuse 200, evals 0 |

## Independent prediction

**Runtime: 20–35 minutes.**

**Predicted:** node-share wash. Accum wash. Unit is the
gate. FIRE not wall-gated.

## Trap doors

1. Do not under-approx (skip a true `where`).
2. Dim raise still maybe.
3. Two constraints on one dim → wildcard.
4. Do not touch alpha-tree `range_children`.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.

# EXPECTATIONS — arm-kind-lists (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | lists | `cargo nextest run --release -E 'test(cascade_kind_list_split)' --no-capture` | sizes printed; disjoint; ids ∈ node_ids |
| 2 | A0 extra | `cargo nextest run --release -E 'test(a0_depth_cost_split_at_equal_work)' --no-capture` | ROUND extra printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** A0 ROUND extra 7.04 → ~4. Empty-kind
passes → ~0. Hash-join leftover remains. Accum FIRE wash.

## Trap doors

1. Topo order: each list is a subsequence of `node_ids`.
2. Join-after-filter must merge filter+acc, not chain out of order.
3. Census loops stay on `node_ids`.

## Will not accept

- Dirty agenda. Scratch. 297. Insertion. Intern `names`.

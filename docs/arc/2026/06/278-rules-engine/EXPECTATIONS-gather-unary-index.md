# EXPECTATIONS — gather-unary-index (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(gather_unary_index_split)' --no-capture` | K / V / U / B / S printed; B > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

If the stone implements, also print `accum_leftover_split`
`accum:index`. No FIRE wall.

## Independent prediction

**Runtime: 20–35 minutes.** Probe first; intern only if licensed.

**Predicted:** B − S ≈ 2 ms/build, ~4 ms on the cell.
`accum:index` 5.16 → ~2. Token stays thin.

## Trap doors

1. Persist gather does not move a cold fire.
2. Unary only when `join_keys.len() == 1`. N-ary stays a Vec.
3. Bucket order is insertion order (foldl).

## Will not accept

- Engine rewrite when B − S < 0.5 ms.
- Persist gather. Second hasher. Intern `names`.

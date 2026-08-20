# EXPECTATIONS — insert-all-empty-identity (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | protocol insert | accum `[200 200]` wat Result `:insert-ns` | printed; facts length 40200 |
| 2 | insert-all diffs | `cargo nextest run --release --test rete -E 'test(equivalence_batch) + test(oracle_matches_native) + test(non_vacuity)'` | 0 failed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–20 minutes.**

**Predicted:** protocol insert 9.42 ms → ~0.2 ms.

## Trap doors

1. `empty ++ x` equals `x` (order, length).
2. Non-empty left still appends (chained insert-all).
3. Fire path unchanged.

## Will not accept

- Session-`Vec`. Query harvest. 297.

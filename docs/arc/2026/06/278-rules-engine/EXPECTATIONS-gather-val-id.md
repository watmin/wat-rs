# EXPECTATIONS — gather-val-id (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(gather_val_id_split)' --no-capture` | U / I / B printed; I > 0 |
| 2 | leftover | `cargo nextest run --release -E 'test(accum_leftover_split)' --no-capture` | FIRE > 0; index printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** U−I ≥ 0.5/build. Intern UnaryId. index
1.97 → ~0.7. FIRE −1.3.

## Trap doors

1. Probe intern_vals an already-interned filler.
2. N-ary unchanged.
3. Do not persist gather.

## Will not accept

- i64-only third variant. 297. Insertion. Intern `names`.

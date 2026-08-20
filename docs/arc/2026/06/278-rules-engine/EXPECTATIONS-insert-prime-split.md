# EXPECTATIONS — insert-prime-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | probe | `echo '[20000]' \| ./target/release/wat wat-scripts/scratch-pad/probe-insert-cost-split.wat` | three lens = n; insert-prime-ns printed |
| 2 | rete lib (if intern) | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | insert diffs (if intern) | `cargo nextest run --release --test probe_arc278_insert_all_differential --test probe_arc278_native_insert_differential` | 0 failed |
| 4 | clippy (if intern) | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–20 minutes.**

**Predicted:** `I − P` (defclause) ≥ 0.5 µs/fact. `P − C`
under the bar. Unique-owner dead on the foldl path.

## Trap doors

1. 2-ary insert does not allocate a one-element PV.
2. Witnesses held or the number is void.
3. `make_mut` with rc ≥ 2 is not an intern.

## Will not accept

- Session-`Vec`. 297. Fire-path. Two interns.

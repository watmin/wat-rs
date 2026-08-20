# EXPECTATIONS — cascade-setup-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(cascade_setup_leftover_split)' --no-capture` | SETUP > 0; arm / remainder / ARM_BUILDS printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 10–20 minutes.**

**Predicted:** `setup:arm` ≥ 10 ms of SETUP 12.70.
ARM_BUILDS moves once per census world. FIRE not
wall-gated. No intern this stone.

## Trap doors

1. One extra pair, not per node.
2. Remainder = SETUP − seen − arm (raw).
3. Do not intern accum `setup:seen`.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.
- Engine rewrite in this diff.

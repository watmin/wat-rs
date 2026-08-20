# EXPECTATIONS — arm-at-compile (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(cascade_setup_leftover_split)' --no-capture` | setup:arm net < 1 ms; ARM_BUILDS 1.00/run |
| 2 | reuse | `cargo nextest run --release -E 'test(fire_rules_reuses_arm_across_fire_and_insert_overlay)'` | green |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** `setup:arm` 12.51 → **~0**. SETUP → **~0.01**.
Cascade FIRE 30.12 → **~17.6**. ARM_BUILDS stays 1/run
(compile intern). FIRE not wall-gated.

## Trap doors

1. If `network_identity` is None, intern cannot HIT — STOP.
2. Do not intern on insert to dodge a miss.
3. Do not drop ARM_BUILDS to 0 (that would mean no build).

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.
- Session field. Second intern table.

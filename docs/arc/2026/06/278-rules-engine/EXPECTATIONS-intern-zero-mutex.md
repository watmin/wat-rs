# EXPECTATIONS — intern-zero-mutex (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | Mutex gone | `rg Mutex src/rete` | empty |
| 2 | overlay HIT | `cargo nextest run --release -E 'test(fire_rules_reuses_arm_across_fire_and_insert_overlay)'` | green; ARM_BUILDS 1 then 0 |
| 3 | N threads | named test in `kernel/tests.rs` | N workers, per-thread delta 1, no deadlock |
| 4 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 5 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Door change, not FIRE.

**Predicted:** overlay numbers unchanged. N-thread test
does not share HIT across workers (instance id). No FIRE
wall movement.

## Trap doors

1. Do not intern record `names`.
2. Do not put facts in `bind_pool`.
3. Do not hold `RefCell` across `build_rete_arm`.
4. Do not spawn a keeper. Do not add a crate.

## Will not accept

- FIRE wall-gate. Persist gather. 297. Service-ify.
- `RwLock` as "not a Mutex." Stamp `vigilatum`.

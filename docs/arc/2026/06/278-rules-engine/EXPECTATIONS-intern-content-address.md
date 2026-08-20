# EXPECTATIONS — intern-content-address (written BEFORE the strike)

> **RULED 2026-08-20 — REJECTED.** Do not run this
> scorecard. Connections stay discrete.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | same-thread Athena | named test | two compile-all, equal rules, ARM_BUILDS += 1 |
| 2 | cross-thread Athena | named test | two workers, ARM_BUILDS process += 1 |
| 3 | unequal rules | named test | MISS, ARM_BUILDS += 1 |
| 4 | eviction still | named test | both release → rebuild |
| 5 | overlay | `fire_rules_reuses_arm_across_fire_and_insert_overlay` | green |
| 6 | Mutex still gone | `rg Mutex src/rete` | empty |
| 7 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 8 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 30–45 minutes.** Key change + keeper.

**Predicted:** overlay HIT unchanged. FIRE wall unchanged.
Second compile-all of equal rules is intern HIT.

## Trap doors

1. Do not intern `names`. Do not hash facts.
2. Do not put a Mutex back. Do not `AtomicPtr`.
3. Do not intern query encode or scratch.
4. Do not ship TLS-only and call Athena done.

## Will not accept

- FIRE wall-gate. Persist gather. 297. Service-ify.
- Stamp `vigilatum`. Recast vigilia before this lands.

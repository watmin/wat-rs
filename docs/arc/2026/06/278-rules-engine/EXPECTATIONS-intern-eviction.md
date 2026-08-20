# EXPECTATIONS — intern-eviction (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | drop+rebuild | named test | release then next fire ARM_BUILDS += 1 |
| 2 | two Sessions | named test | release A, B still HIT |
| 3 | overlay | `fire_rules_reuses_arm_across_fire_and_insert_overlay` | green (no release) |
| 4 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 5 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 20–30 minutes.** Lease count + one native op.

**Predicted:** overlay test unchanged. No FIRE wall
movement. Session still 8 fields.

## Trap doors

1. Do not Weak the table.
2. Do not intern `names`. Do not add a Session field.
3. Do not release inside fire.
4. Overlay is not a second lease.

## Will not accept

- FIRE wall-gate. Persist gather. 297. Service-ify.
- Drop intern on fire return. Stamp `vigilatum`.

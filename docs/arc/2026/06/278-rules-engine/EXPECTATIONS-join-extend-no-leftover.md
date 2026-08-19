# EXPECTATIONS — join-extend-no-leftover (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | gate | read the diff | rematch only if `has_seed_cmp()` |
| 2 | fanout census | `cargo nextest run --release -E 'test(fanout_fire_phase_census)' --no-capture` | hash-join > 0; probe printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed (`where-join-left`) |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 20–30 minutes.** One `if`.

**Predicted:** probe 30.87 → ~8–15 ms. FIRE 96 → ~75–85.
If probe stays ~31, leftover is `extend_token` — stop.

## Trap doors

1. `where-join-left` must still rematch.
2. `extend_token` still runs.

## Will not accept

- Persist. `right_idx` rewrite. Skipping leftover rematch.

# EXPECTATIONS — match-pool (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | type | read the diff | `Token.matches: BindSpan`; `match_pool`; Token `Copy` |
| 2 | fanout census | `cargo nextest run --release -E 'test(fanout_fire_phase_census)' --no-capture` | probe printed; hash-join > 0 |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 30–50 minutes.** Same shape as token-bind-pool.

**Predicted:** probe 10.99 → ~6–9. FIRE 47.48 → ~42–45.

## Trap doors

1. Explain / encode after `wm` drops — encode **while the
   pool lives** (already the bind-pool door).
2. `extend_token` must not hold a `match_pool` slice across
   `push`.
3. Tests that walk `tok.matches` need the pool.

## Will not accept

- SmallVec. Skip matches. FIRE wall. 297.

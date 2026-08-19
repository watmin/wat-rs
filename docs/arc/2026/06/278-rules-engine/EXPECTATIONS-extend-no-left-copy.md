# EXPECTATIONS — extend-no-left-copy (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | type | read the diff | `Token.extra`; extend does not copy `tok.binds` |
| 2 | fanout census | `cargo nextest run --release -E 'test(fanout_fire_phase_census)' --no-capture` | probe printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 25–40 minutes.** Extra span + reader sweep.

**Predicted:** probe 11.39 → ~7–10. FIRE 44.23 → ~40–43.

## Trap doors

1. Production / `key_of` / harvest still read only `binds`
   — `?r` lives on `extra`.
2. Second join must keep `tok.extra`, not drop it.
3. Do not hold a pool slice across `push`.

## Will not accept

- SmallVec. FIRE wall. 297.

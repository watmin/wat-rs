# EXPECTATIONS — token-bind-pool (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | type | read the diff | `Token.binds: BindSpan`; root seed shares; extend appends |
| 2 | fanout census | `cargo nextest run --release -E 'test(fanout_fire_phase_census)' --no-capture` | probe printed; hash-join > 0 |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 40–70 minutes.** Token type change + encode/explain.

**Predicted:** probe 14.35 → ~8–11. FIRE 60.90 → ~54–58.
Matches Vec is the leftover.

## Trap doors

1. Explain / `WhereSample` after `wm` drops — encode or
   materialize **while the pool lives**.
2. `join_extend` must not hold a pool slice across `pool.push`.
3. Accum `assoc` is a new span, not a PMap write-back forgotten.

## Will not accept

- `unsafe`. Matches rewrite. FIRE wall. 297.

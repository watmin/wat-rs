# EXPECTATIONS — bind-pool (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | type | read the diff | `Element` has `BindSpan`; populate writes `bind_pool`; no `unsafe` |
| 2 | census | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | fold < 25; snapshot < 1; drop printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 40–60 minutes.** Span + pool threading.

**Predicted:** drop 10.49 → ~5–6 ms. FIRE 76.85 → ~71–73.
If FIRE stays ~77, leftover is fact `Value` Drop — stop.

## Trap doors

1. Clear alpha **before** the pool (spans must not dangle).
2. `attach_fact` still prepends `?p` into the pool range.
3. HashJoin `el.clone()` copies the span, not the pairs.

## Will not accept

- Leak. `unsafe`. Inline retry. Persist bundled in.

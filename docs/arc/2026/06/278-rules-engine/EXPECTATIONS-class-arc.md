# EXPECTATIONS — class-arc (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | type | read the diff | `class: Arc<str>` on Aggregate + CompiledRhs |
| 2 | fanout census | `cargo nextest run --release -E 'test(fanout_fire_phase_census)' --no-capture` | production / compiled-rhs printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | Debug golden | `test(probe_6_debug_contains_class)` | pass |
| 6 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 25–40 minutes.** Type change + compile fixes.

**Predicted:** compiled-rhs 8.05 → ~5–7. FIRE 61 → ~58–60.
If it stays ~8, leftover is fields/`seen` — stop.

## Trap doors

1. `type_name` / error `record_class` still `String`.
2. `a.class == "wat::rete::Session"` still compiles.

## Will not accept

- Global intern map. Persist. `names` intern.

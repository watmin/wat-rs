# EXPECTATIONS — setup-fxhash (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | hasher on the three maps | read the diff | `seen` is `FxHashSet<Value>`; `GatherIndex` / `GatherCache` are `FxHashMap` |
| 2 | census | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | fold < 25 ms; snapshot < 1 ms; `setup:seen` printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 20–30 minutes.** Crate + three aliases.

**Predicted:** `setup:seen` 13.26 → ~5–8 ms. FIRE 86 → ~78–82.
If `setup:seen` stays ~13, leftover is the Hash walk + insert,
not SipHash mix — next stone is not another hasher.

## Trap doors

1. Inputs still in `seen` before derived production.
2. First-round order stays PV order.
3. `i64`-keyed WM maps stay std.

## Will not accept

- Pointer-hash.
- Persist bundled in.
- `impl Hash for Value` rewritten here.

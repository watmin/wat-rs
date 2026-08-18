# EXPECTATIONS — elem-bindings-inline (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | type | read the diff | `Element.bindings` is `ElemBindings`; `exec_compiled` returns it |
| 2 | census | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | fold < 25; snapshot < 1; drop printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 25–40 minutes.** Type + materialize + helper signatures.

**Predicted:** drop 10.49 → ~6–8 ms. FIRE 76.85 → ~72–74.
If drop stays ~10, leftover is `Value` Drop — next is not an arena.

## Trap doors

1. `attach_fact` still prepends `?p`.
2. `slot_i64` indexes the slice (`N1` is `from_ref`).
3. HashJoin `el.clone()` copies pairs — leave it.

## Will not accept

- Leak. Crate. Persist bundled in.

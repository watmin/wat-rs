# BRIEF — fanout phase census at the grid ladder

## The work

Print the same instrument-subtracted phase table accum has, at
fanout's grid ladder. Do not change the engine.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2h is this census.
2. `DESIGN-STONE-fanout-phase-census.md`.
3. `kernel.rs` `render_phase_table`, `FANOUT_CENSUS_WORLD`,
   `fanout_per_call_alpha_census`.

## Sketch

```rust
fn fanout_phase_census(keys, fanout) -> rows
// fire-rules (fan::seed (compile …) keys fanout)

fn fanout_fire_phase_census() {
    render_phase_table(..., &[(25,20),(50,20),(100,20)], ...)
}
```

## STOP

1. **STOP-1** — rewrite `right_idx` / HashJoin in this diff.
2. **STOP-2** — gate FIRE on a wall.
3. **STOP-3** — require accumulate/filter (this axis never hits them).

## Done

- Table printed. ROUND LOOP > 0. hash-join present.

Leave dirty.

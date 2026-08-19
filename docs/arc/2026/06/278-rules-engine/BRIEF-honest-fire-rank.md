# BRIEF — honest FIRE rank after the reserve

## The work

Re-print leftover production at today's fire. Subtract the
80k test marks. Rank engine rows. Do not change the engine.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2t is this census.
2. `DESIGN-STONE-prod-leftover-split.md` weigh (tax + remainder).
3. `DESIGN-STONE-probe-gap-split.md` weigh (FIRE 39.48).
4. `DESIGN-STONE-honest-fire-rank.md`.
5. `kernel.rs` `fanout_production_leftover_split`.

## Sketch

```rust
fn fanout_honest_fire_rank() {
    // cal, 3× fanout_phase_census(100, 20)
    // honest_FIRE = FIRE − remainder_raw − tax_in_parent
    // print; rhs pairs == 40_000
}
```

## STOP

1. **STOP-1** — nest `phase_start` in a token loop.
2. **STOP-2** — intern `names` / skip stamp / `Token.extra`.
3. **STOP-3** — gate FIRE on a wall.

## Done

- Table printed. 40k compiled-rhs pairs. Largest drawable
  engine row named (or none).

Leave dirty.

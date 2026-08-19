# BRIEF — leftover production is remainder vs tax-in-parent

## The work

Print whether production **19.34** is construct or children's
instrument left in the parent. Do not change the engine.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2p is this census.
2. `DESIGN-STONE-match-pool.md` weigh (production 19.34).
3. `DESIGN-STONE-rhs-construct-split.md` weigh (pile, not a row).
4. `DESIGN-STONE-prod-leftover-split.md`.
5. `kernel.rs` `render_phase_table` net law; production loop marks.

## Sketch

```rust
fn fanout_production_leftover_split() {
    // cal, 3× fanout_phase_census(100, 20)
    // remainder_raw = prod_raw - rhs_raw - dedup_raw
    // tax_in_parent = (rhs_pairs + dedup_pairs) * cal
    // print; pairs(compiled-rhs) == 40_000
}
```

## STOP

1. **STOP-1** — nest `phase_start` in the production token loop.
2. **STOP-2** — intern `names` / skip stamp / rewrite `seen`.
3. **STOP-3** — gate FIRE on a wall. Retry 2o.

## Done

- Table printed. 40k compiled-rhs pairs. Larger of
  remainder_raw / compiled-rhs_net named.

Leave dirty.

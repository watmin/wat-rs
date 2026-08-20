# BRIEF — rank the closest cells after intern 7–14

## The work

Print current native FIRE at fanout `[100 20]`,
deep-cascade `[50 100]`, accum `[200 200]`. Name the
next intern cell. Do not change the engine.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 14, leftovers
   ranked on accum; 08-20 grid moved the closest cell.
2. `DESIGN-STONE-cell-rank-after-fanout.md` weigh (2v).
3. `GRID-native-vs-clara-2026-08-20T00-17-00Z.txt`.
4. `DESIGN-STONE-cell-rank-after-grid.md`.
5. `kernel.rs` `fanout_phase_census` / `accum_phase_census`
   / `depth_split_phases`.

## Sketch

```rust
fn cascade_phase_census(depth, width) -> counted rows
fn cell_rank_after_grid() {
    // 3× each top rung; print FIRE + largest named child
}
```

## STOP

1. **STOP-1** — full Clara grid / persist gather / 297.
2. **STOP-2** — intern this stone / intern `seen` / 2e / 2o.
3. **STOP-3** — gate a ratio. Per-fact timers.

## Done

- Table printed. Three FIRE > 0. Next cell named.

Leave dirty.

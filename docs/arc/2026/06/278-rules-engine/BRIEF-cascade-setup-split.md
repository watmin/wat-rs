# BRIEF — split cascade SETUP without per-node timers

## The work

Rank `setup:arm` vs remainder of SETUP at deep-cascade
`[50 100]`. Print ARM_BUILDS. Do not intern.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 15, cascade SETUP 12.7.
2. `DESIGN-STONE-cell-rank-after-grid.md` weigh.
3. `kernel.rs` `fire_fixpoint_delta` SETUP / `rete_arm_get_or_build`.
4. `DESIGN-STONE-cascade-setup-split.md`.

## Sketch

```
setup:arm  rete_arm_get_or_build
remainder  SETUP − seen − arm
print ARM_BUILDS
```

## STOP

1. **STOP-1** — intern off an unranked lump. Per-node timers.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. Fold accum `seen`.

## Done

- Table printed. SETUP > 0.
- rete lib green. clippy `-D warnings` silent.

Leave dirty.

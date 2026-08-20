# BRIEF — fire-path passes iterate kind lists on the arm

## The work

`ReteArm` carries kind-partitioned id lists. Six
`fire_fixpoint_delta` passes iterate those lists. Weigh
A0 ROUND extra. Do not intern dirty agenda. Do not intern
scratch.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 24, cascade ROUND extra +7.04.
2. `a0_depth_cost_split_at_equal_work` comment (idle scans × D).
3. `kernel.rs` `ReteArm` / `build_rete_arm` / `fire_fixpoint_delta` passes.
4. `DESIGN-STONE-arm-kind-lists.md`.

## Sketch

```
kind_ids.alpha / join_parent / acc / filter / prod / filter_or_acc
for node_id in &arm.kind_ids.acc { … }   // empty on cascade
```

## STOP

1. **STOP-1** — dirty agenda / skip idle same-kind.
2. **STOP-2** — intern scratch / names / 2e / 2o.
3. **STOP-3** — 297 / insertion / `fire_once`.

## Done

- lists interned. six passes. rete + clippy silent.

Leave dirty.

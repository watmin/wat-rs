# BRIEF — hash-join iterates dirty join-parents only

## The work

`joins_fed_by` on the arm. Each round the hash-join pass
iterates dirty join-parents (left `d_beta` ∪ right
`d_alpha` via invert). Weigh A0 hash-join extra. Do not
dirty-agenda root-join. Do not intern scratch.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 25, hash-join extra +1.43.
2. `DESIGN-STONE-arm-kind-lists.md` weigh (empty-kind dead).
3. `kernel.rs` hash-join pass (`for node_id in &kind_ids.join_parent`).
4. `DESIGN-STONE-dirty-join-parents.md`.

## Sketch

```
joins_fed_by: alpha → [HashJoin]
dirty = d_beta∩join_parent ∪ parents(joins_fed_by[d_alpha])
for node_id in dirty { … same body … }
```

## STOP

1. **STOP-1** — dirty-agenda root-join / production / filter.
2. **STOP-2** — intern scratch / names / 2e / 2o.
3. **STOP-3** — 297 / insertion / rewrite join-after-filter.

## Done

- dirty join-parents. rete + clippy silent.

Leave dirty.

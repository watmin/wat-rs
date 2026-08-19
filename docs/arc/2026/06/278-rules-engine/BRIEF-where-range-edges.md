# BRIEF — populate `range_children`

## The work

`< > <= >=` become guard edges on `ShadowNode`. Over-approx
only. Proven pure-cmp residues skip `exec_where`. Alpha-tree
ranges are not this stone.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 5 is this populate.
2. `where_tree.rs` `range_children: Vec<()>`.
3. `DESIGN-STONE-where-dim-reuse.md` (proven vs maybe).
4. `DESIGN-STONE-where-range-edges.md`.

## Sketch

```
range_children: Vec<(RangeEdge { op, threshold }, Arc<ShadowNode>)>
walk: if range_holds(v, op, thr) { walk proven } else { skip }
```

## STOP

1. **STOP-1** — populate alpha-tree ranges. Treat `not=` as a range.
2. **STOP-2** — two constraints on one dim as a conjunction in the walk.
3. **STOP-3** — intern `names` / facts in `bind_pool` / 2e / 2o.
4. **STOP-4** — FIRE wall. 297. Fact insertion.

## Done

- Unit: `> ?k 10` prunes 5, proves 15.
- rete lib green. clippy `-D warnings` silent.
- Node-share evals still 0.

Leave dirty.

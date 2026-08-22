# BRIEF — occupancy leaf-set column

## The work

Undiscriminated class (tree root = leaves only):
one fact-id column, fill **those leaves**. Not
`kind_ids ∩ class`. 3-stratum is the gate. Then
shared occupant vec if green.

## Read in order

1. `DESIGN-STONE-bind-only-class-fill.md` (union
   reverted).
2. `DESIGN-STONE-occupancy-leaf-column.md`.

## STOP

1. **STOP-1** — fill `kind_ids.alpha` of a class.
   Skip `candidates_into` on a class with
   equality children.
2. **STOP-2** — insert empty column (wipe).
3. **STOP-3** — shared `Vec<u32>` this stone
   unless 7strat is already green.

## Done when

- Pack-all + leaf fill. 7strat 3-stratum green.
- FIRE **13.7** (was 17.8), seed **10.5**.
- Arc shared occupant list. Clippy `--lib`
  silent. Diagnostic `n3_leaf_set_vs_occupancy`
  stays.

Leave dirty unless asked to commit.

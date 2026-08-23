# BRIEF — split deep-cascade harvest from SETUP and the 50-round loop

## The work

Grid deep-cascade `[50 100]` **10.9 ms**. Two class-scan queries
of types that are in input. Print without / with. Rank SETUP /
ROUND / harvest:query.

## Read in order

1. `DESIGN-STONE-cascade-harvest-split.md`.
2. `strat_neg_query_harvest_split` — the same instrument.
3. `cascade_phase_census` compiles without queries.

## STOP

1. **STOP-1** — intern off stale SETUP 12.70 without the split.
2. **STOP-2** — Session-Vec / skip freeze / intern `names`.
3. **STOP-3** — skip-input on Node/Tag (level 0 is those classes).

# BRIEF — split strat-neg harvest from the six-stratum engine

## The work

Grid strat-neg `[6 2000]` **14.2 ms**. Ten class-scan
queries. Print without / with. Intern harvest only if
harvest:query ≥ 1 ms.

## Read in order

1. `DESIGN-STONE-strat-neg-harvest-split.md`.
2. `accum_query_harvest_split` — the same instrument.
3. `harvest_stratified_queries` — second fire-once on
   `acc_facts` as `facts`.

## STOP

1. **STOP-1** — intern off grid 14.2 without the split.
2. **STOP-2** — Session-Vec / skip freeze / intern `names`.

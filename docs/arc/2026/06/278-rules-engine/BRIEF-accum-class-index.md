# BRIEF — index the closed bag by class once

## The work

Accum `[200 200]` grid `compile-all`s five queries.
Class-scan walked the 40k-fact bag **per query**.
Index once when N>1. Fanout stays one filter.

## Read in order

1. `accum_query_harvest_split` — harvest **13.11 → 6.17**.
2. `DESIGN-STONE-accum-class-index.md`.

## STOP

1. **STOP-1** — index on fanout's single class (measured slower).
2. **STOP-2** — Session-Vec / skip freeze / intern `names`.

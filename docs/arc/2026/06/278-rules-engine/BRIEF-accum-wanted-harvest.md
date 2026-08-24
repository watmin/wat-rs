# BRIEF — index only the classes the scans name

## The work

Accum harvest:query **6.23** still indexes 40k Readings
no scan asks for. Print I / W / D / M. Intern the
≥ 1 ms row. Fanout stays the filter path.

## Read in order

1. `accum_query_harvest_split` — harvest **6.23**, maps 1,000.
2. `DESIGN-STONE-accum-wanted-harvest.md`.

## Sketch

```
I  both bags, every class          // current
W  both bags, scan.class only
D  derived only, scan.class only
M  wrap 1,000 maps
```

## STOP

1. **STOP-1** — guess production types.
2. **STOP-2** — index on fanout's single class.
3. **STOP-3** — Session-Vec / skip freeze / intern `names`.

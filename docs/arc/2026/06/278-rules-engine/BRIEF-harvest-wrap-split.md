# BRIEF — split harvest:query (scan vs wrap)

## The work

Print whether harvest:query **7.69** is the class
scan or `from_pairs` × 40k. Do not change the
engine this stone.

## Read in order

1. `fanout_three_leftover_split` `[100 20]` —
   harvest:query **7.69**, out:query **0**.
2. `DESIGN-STONE-harvest-wrap-split.md`.

## Sketch

```
S  scan (filter PVec by class)
W  wrap (from_pairs × 40k)
H  harvest (S then W)
```

Tight loop, 40k facts, mean of 3.

## STOP

1. **STOP-1** — Session-Vec / skip freeze.
2. **STOP-2** — a third PMap arm (Array1).
3. **STOP-3** — intern `names`. 297.

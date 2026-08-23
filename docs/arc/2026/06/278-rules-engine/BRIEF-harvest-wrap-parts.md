# BRIEF — split harvest wrap (clones vs Arc vs intern-id)

## The work

Print which part of wrap is clones, Arc alloc, or
intern-id. `PMap::from_one` on the existing Array arm
LANDED (harvest:query 6.37 → 5.97). Do not add
`PMap::Array1`.

## Read in order

1. `harvest_wrap_split` — W **8.78**, harvest:query **6.06**.
2. `DESIGN-STONE-harvest-wrap-parts.md`.

## Sketch

```
C  clone (var, fact) × 40k
R  Arc::from([pair]) × 40k
I  fetch_add × 40k
W  from_pairs × 40k
```

## STOP

1. **STOP-1** — Session-Vec / skip freeze.
2. **STOP-2** — PMap::Array1 sibling of Array|Trie.
3. **STOP-3** — intern `names`. 297.

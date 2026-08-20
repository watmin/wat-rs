# BRIEF — intern `intern_val` for i64

## The work

Measure `FxHashMap<Value>` vs `FxHashMap<i64>` vs a
small-int table on 80,200 interned fillers. Intern the
winner if the cut is ≥ 1 ms. Do not intern `seen`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 12, intern_val.
2. `DESIGN-STONE-materialize-split.md` weigh (V−K 2.77).
3. `compiled_cond.rs` `intern_val`.
4. `DESIGN-STONE-intern-val-i64.md`.

## Sketch

```
V Value-map   I i64-map   A slot table
if V − min(I,A) ≥ 1: intern_val i64 fast path
```

## STOP

1. **STOP-1** — intern `seen`. Tagged i64 in the pool.
   Facts in `bind_pool`. Intern `names`.
2. **STOP-2** — 2e / 2o. 297. Fact insertion.
3. **STOP-3** — per-fact timers.

## Done

- Table printed. V > 0.
- If intern: V−K falls. rete lib. clippy silent.

Leave dirty.

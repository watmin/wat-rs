# BRIEF — fill `seen` on the seed walk

## The work

`seen_insert` each input fact inside `alpha:seed`. Delete
the standalone insert loop. Alloc stays at SETUP.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 20, fold-into-seed.
2. `DESIGN-STONE-seen-fire-context.md` weigh.
3. `kernel.rs` `setup:seen` / seed loop.
4. `DESIGN-STONE-fold-seen-into-seed.md`.

## Sketch

```
// SETUP: alloc HashSets
for fact in input_facts {
    seen_insert(fact)
    alpha_activate_fact(fact)
}
```

## STOP

1. **STOP-1** — skip seen inputs / Session-`Vec` / 2e.
2. **STOP-2** — intern `names` / 297 / insertion.
3. **STOP-3** — per-fact seen timers.

## Done

- `setup:seen` < 0.5 ms at `[200 200]`.
- rete lib green. clippy silent.

Leave dirty.

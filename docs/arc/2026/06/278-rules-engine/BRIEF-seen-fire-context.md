# BRIEF — split in-fire `setup:seen` (alloc vs insert)

## The work

Rank HashSet alloc vs `seen_insert` in-fire, and isolated
on real seeded facts. Do not intern Session-`Vec`. Do not
skip filling `seen`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 19, seen ~4.0.
2. `DESIGN-STONE-seen-pv-walk.md` weigh (isolated P 1.67).
3. `kernel.rs` `setup:seen` / `seen_insert`.
4. `DESIGN-STONE-seen-fire-context.md`.

## Sketch

```
setup:seen:alloc
setup:seen:insert
A / X / S  isolated on seeded Session facts
```

## STOP

1. **STOP-1** — Session-`Vec` / skip seen inputs / fold-into-seed.
2. **STOP-2** — intern `names` / 2e / 297 / insertion.
3. **STOP-3** — intern off an unranked lump.

## Done

- Table printed. Insert > 0.
- rete lib green. clippy silent.

Leave dirty.

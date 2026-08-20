# BRIEF — unary gather hashes interned filler ids

## The work

Rank `HashMap<Value>` clone vs `HashMap<u32>` vid. Intern
`UnaryId` if U−I ≥ 0.5 ms per build.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 22, index 1.97.
2. `DESIGN-STONE-gather-unary-index.md` weigh.
3. `kernel.rs` `build_gather_index` Unary arm.
4. `DESIGN-STONE-gather-val-id.md`.

## Sketch

```
I  HashMap<u32> from bind-pool vid
if U−I ≥ 0.5: GatherIndex::UnaryId
```

## STOP

1. **STOP-1** — persist gather / intern `names` / 2e.
2. **STOP-2** — 297 / insertion / i64-only third variant.
3. **STOP-3** — intern if U−I < 0.5.

## Done

- Table printed. If intern: UnaryId. rete + clippy silent.

Leave dirty.

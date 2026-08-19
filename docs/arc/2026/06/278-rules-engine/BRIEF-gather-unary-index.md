# BRIEF — gather index is unary when the join key is one

## The work

Print `build_gather_index` vs unary `Value` keys on 40,200
Readings. If B − S ≥ 0.5 ms, `GatherIndex` is unary at
`join_keys.len() == 1`. Do not persist gather.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2y is this stone.
2. `DESIGN-STONE-seen-identity-set.md` weigh (index 5.16).
3. `DESIGN-STONE-gather-no-snapshot.md` (index, no clone).
4. `DESIGN-STONE-gather-unary-index.md`.
5. `kernel.rs` `build_gather_index` / `ensure_gather`.

## Sketch

```rust
fn gather_unary_index_split() { /* K V U B S */ }

// only if B−S ≥ 0.5 ms:
// GatherIndex::Unary(FxHashMap<Value, Vec<usize>>)
```

## STOP

1. **STOP-1** — B − S < 0.5 ms: do not touch the index.
2. **STOP-2** — persist gather / second hasher / 297.
3. **STOP-3** — gate FIRE on a wall.

## Done

- Table printed. If implemented: Token still Copy.
  `accum:index` printed. rete + clippy.

Leave dirty.

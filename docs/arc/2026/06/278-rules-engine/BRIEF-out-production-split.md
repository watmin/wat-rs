# BRIEF — split `out:production` without a Session rewrite

## The work

Apportion OUT **3.26**. Tight loops on 40k Pair records.
If a drop-in vector build beats `push_back_mut` by ≥ 1 ms,
take it. Do not change the Session shape.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2u is this stone.
2. `DESIGN-STONE-honest-fire-rank.md` weigh (OUT 3.26).
3. `DESIGN-STONE-out-production-split.md`.
4. `kernel.rs` `hashmap_to_pm` / `to_persistent`.

## Sketch

```rust
fn out_production_cost_split() {
    // C clone  V push_back_mut  H hashmap_to_pm  I from_iter
}
```

## STOP

1. **STOP-1** — largest drop-in < 1 ms: do not touch the engine.
2. **STOP-2** — native Vec in Session / skip freeze / intern `names`.
3. **STOP-3** — gate FIRE on a wall.

## Done

- Table printed. H > 0. If implemented: Token still Copy.
  OUT printed.

Leave dirty.

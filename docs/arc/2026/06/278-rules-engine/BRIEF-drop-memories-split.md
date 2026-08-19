# BRIEF — split leftover `drop-memories` without fattening Element

## The work

Apportion `drop-memories` **3.63**. Tight loops on the four
clears. Do not change the engine. Do not retry 2e.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2aa is this census.
2. `DESIGN-STONE-bind-pool.md` weigh (leftover 3.63 is fact Drop).
3. `DESIGN-STONE-elem-bindings-inline.md` weigh (2e reverted).
4. `DESIGN-STONE-drop-memories-split.md`.
5. `kernel.rs` `drop-memories` four clears.

## Sketch

```rust
fn drop_memories_cost_split() { /* A B M T D */ }
```

## STOP

1. **STOP-1** — inline-enum / SmallVec / arena-and-forget.
2. **STOP-2** — put facts in `bind_pool` / skip pool Drop.
3. **STOP-3** — gate FIRE on a wall. Persist gather.

## Done

- Table printed. D > 0. Largest of A / B / M named.

Leave dirty.

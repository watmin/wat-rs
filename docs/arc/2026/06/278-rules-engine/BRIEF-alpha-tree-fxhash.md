# BRIEF — alpha-tree children are FxHashMap

## The work

`Node.children` is `FxHashMap`. Weigh I−G. Do not intern
scratch. Do not populate range edges.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 23, scratch STOP.
2. `DESIGN-STONE-alpha-tree-walk-split.md` weigh (walk 0.08).
3. `alpha_tree.rs` `walk` / `Node.children`.
4. `DESIGN-STONE-alpha-tree-fxhash.md`.

## Sketch

```
children: FxHashMap<Value, Arc<Node>>
```

## STOP

1. **STOP-1** — intern scratch / names / range edges / 2e.
2. **STOP-2** — 297 / insertion.
3. **STOP-3** — i64-only children as a third map.

## Done

- children is FxHashMap. rete + clippy silent.

Leave dirty.

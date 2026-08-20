# BRIEF — intern alpha-tree class lookup

## The work

Measure std HashMap vs FxHash vs linear on 40,200 class
strings. Intern the winner if the cut is ≥ 1 ms.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 10, class lookup.
2. `DESIGN-STONE-alpha-tree-walk-split.md` weigh (G−E 3.26).
3. `alpha_tree.rs` `roots`.
4. `DESIGN-STONE-alpha-class-lookup.md`.

## Sketch

```
S HashMap  F FxHashMap  L Vec<(String, Node)>
if S − min(F,L) ≥ 1 ms: intern roots to the winner
```

## STOP

1. **STOP-1** — pointer-hash `Arc<str>`. Intern `children`.
   Populate `range_children`.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. Per-fact timers.

## Done

- Table printed. S > 0.
- If intern: G−E falls. rete lib green. clippy silent.

Leave dirty.

# BRIEF — split the 4.46 ms alpha-tree walk

## The work

Rank class-HashMap vs walk vs `Vec` alloc. Intern a
reused candidate buffer **only if** alloc ≥ 1 ms.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 9, tree first.
2. `DESIGN-STONE-alpha-leftover-split.md` weigh (T 4.46).
3. `alpha_tree.rs` `candidates` / `walk`.
4. `DESIGN-STONE-alpha-tree-walk-split.md`.

## Sketch

```
E extract  G class get  I walk into reused Vec  T new Vec
if T−I ≥ 1 ms: candidates_into in alpha_activate_fact
```

## STOP

1. **STOP-1** — populate `range_children`. Intern HashMap.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. Per-fact timers.

## Done

- Table printed. I > 0.
- If intern: activate uses `candidates_into`.
- rete lib green. clippy `-D warnings` silent.

Leave dirty.

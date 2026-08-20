# BRIEF — split `exec_ops` (scratch vs Bind)

## The work

Rank scratch reset vs `exec_ops` body. Intern
`fill(None)` only if scratch ≥ 1 ms. Do not intern `seen`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 13, ops 1.90.
2. `DESIGN-STONE-compiled-match-split.md` weigh (O−T 1.90).
3. `compiled_cond.rs` `exec_compiled` / `exec_ops`.
4. `DESIGN-STONE-exec-ops-split.md`.

## Sketch

```
R  scratch clear/resize
O  + exec_ops
if R−T ≥ 1: fill(None) instead of clear+resize
```

## STOP

1. **STOP-1** — intern `seen`. Second scratch type.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. Per-fact timers.

## Done

- Table printed. O > 0.
- If intern: fill(None). rete lib. clippy silent.

Leave dirty.

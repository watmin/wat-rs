# BRIEF — split `exec_compiled` (ops vs intern)

## The work

Rank the 7.65 ms `M−T` lump. Isolated T / O / Mc / Mw.
No intern. No 80k timers.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 7, `M−T` first.
2. `DESIGN-STONE-alpha-leftover-split.md` weigh.
3. `compiled_cond.rs` `exec_compiled` / `exec_ops` /
   `materialize_into`.
4. `DESIGN-STONE-compiled-match-split.md`.

## Sketch

```
O  scratch + exec_ops
Mc exec_compiled, pools reset
Mw exec_compiled, pools kept
```

## STOP

1. **STOP-1** — restore per-fact alpha timers. Intern
   off an unranked lump.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. Fold seen. Tree/push.

## Done

- Table printed. O > 0.
- rete lib green. clippy `-D warnings` silent.

Leave dirty.

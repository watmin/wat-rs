# BRIEF — split `materialize_into`

## The work

Rank the 6.18 ms success path. Isolated O / C / K / V /
P / M. No intern. No 80k timers.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 8, materialize.
2. `DESIGN-STONE-compiled-match-split.md` weigh.
3. `compiled_cond.rs` `materialize_into`.
4. `DESIGN-STONE-materialize-split.md`.

## Sketch

```
C clone   K intern_key   V intern_val   P pool.push
```

## STOP

1. **STOP-1** — restore per-fact timers. Intern off an
   unranked lump. Tagged-i64 ids this stone.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. Tree / seen.

## Done

- Table printed. V > 0.
- rete lib green. clippy `-D warnings` silent.

Leave dirty.

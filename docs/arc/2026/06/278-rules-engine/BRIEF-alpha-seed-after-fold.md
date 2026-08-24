# BRIEF — split `alpha:seed` after seen folded in

## The work

Rank the 16.6 ms in-fire seed. Isolated stacked P / S /
X / K / E / N / A on the facts PV, `candidates_into`,
`seen_insert`. No intern. No 80k timers.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — leftovers after 26.
2. `DESIGN-STONE-fold-seen-into-seed.md` weigh.
3. `DESIGN-STONE-alpha-leftover-split.md` (isolated A
   without seen; T used `candidates()`).
4. `DESIGN-STONE-alpha-seed-after-fold.md`.

## Sketch

```
P  PV iter
S  + seen_insert
X  + class extract
K  + candidates_into
E  + exec_compiled_with_key_ids
N  + push
A  seen + alpha_activate_fact
```

## STOP

1. **STOP-1** — restore per-fact alpha timers. Intern
   off an unranked lump.
2. **STOP-2** — intern `names` / facts in `bind_pool` /
   scratch-as-new-repr / Session-`Vec` / 2e.
3. **STOP-3** — 297. Packed rows this stone.

## Done

- Table printed. Seed > 0, A > 0.
- rete lib green. clippy `-D warnings` silent.

Leave dirty.

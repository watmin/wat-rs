# BRIEF — split Element push / `d_alpha`

## The work

Rank HashMap `entry` vs Vec push vs `d_alpha`. Intern
the largest lump if ≥ 1 ms. Do not intern `seen`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 11, push 3.45.
2. `DESIGN-STONE-alpha-leftover-split.md` weigh (A−M 3.45).
3. `kernel.rs` `alpha_activate_fact` success arm.
4. `DESIGN-STONE-alpha-push-split.md`.

## Sketch

```
H  alpha.entry    V  Vec::push    D  d_alpha.entry
if H−M ≥ 1: FxHashMap for wm.alpha + d_alpha
```

## STOP

1. **STOP-1** — intern `seen` / Session-`Vec` / fold seen
   into seed. Change `beta` hasher.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. Per-fact timers.

## Done

- Table printed. D > 0.
- If intern: largest lump interned. rete lib. clippy silent.

Leave dirty.

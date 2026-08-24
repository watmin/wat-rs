# BRIEF — seed occupancy: reserve(n) + straight fill

## The work

Seed skip-span matches: collect `fact_idx` per
alpha, then `reserve(n)` and sequential Element
fill. Same candidate set. Weigh FIRE `[200 200]`.
Revert if FIRE does not fall ≥ 1 ms vs 17.8.

## Read in order

1. `DESIGN-STONE-bind-only-class-fill.md` (class-
   union reverted; this is not that).
2. `DESIGN-STONE-seed-occupancy-fill.md` (this
   file's stone).

## STOP

1. **STOP-1** — skip `candidates_into`. Fill
   every bind-only alpha of a class (7strat red).
2. **STOP-2** — Session-`Vec`. 297. SIMD. Rayon.
3. **STOP-3** — pending on the Cmp path (needs
   BindSpan). Delta-round pending (small n).

## Done when

- Interned. 7strat green. FIRE **17.48** vs
  17.8 — under 1 ms, noise. Reverted.
- Realloc was not the leftover. 2/3 occupancy
  (leaf-set / shared `Vec<u32>`) still the
  representation hunt if we continue.

Leave dirty unless asked to commit.

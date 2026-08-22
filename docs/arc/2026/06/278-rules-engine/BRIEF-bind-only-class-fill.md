# BRIEF — bind-only class fill (pass 2)

## The work

Bind-only packed facts of a class: one column of
fact ids, then reserved `Element` fills into every
alpha of that class. No per-fact candidate walk.
Weigh FIRE on accum `[200 200]`. Revert if FIRE
rises vs 17.8.

## Read in order

1. `DESIGN-STONE-column-gather-fold.md` weigh
   (skip BindSpan held FIRE; leftover is the 80k
   visits).
2. `DESIGN-STONE-bind-only-class-fill.md` (this
   file's stone).

## STOP

1. **STOP-1** — batch a class that has Cmp /
   BindCheck / `fact_bind`. Tree under-approx.
2. **STOP-2** — rayon inside one fire. Session-
   `Vec`. 297. SIMD.
3. **STOP-3** — SETUP intern_val walk. Skip Token
   BindSpan.

## Done when

- Interned. 7strat 3-stratum **red**
  (Safe 2 vs 1). Reverted.
- Stone records the miss. Pass 2 is still
  the intern; this fill is not
  class-match-enough on `:not` + strata.
- Do not re-land without 3-stratum green.

Leave dirty unless asked to commit.

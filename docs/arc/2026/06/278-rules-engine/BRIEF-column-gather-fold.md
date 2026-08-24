# BRIEF — column gather/fold, then skip BindSpan

## The work

Point gather/fold/join/exists at packed i64 rows.
Then bind-only populate writes an empty BindSpan.
Token still gets binds at join from the row.
Weigh FIRE on accum `[200 200]`. Revert if FIRE
rises (scout 2 was 19→70).

## Read in order

1. `DESIGN-STONE-fire-i64-columns.md` weigh
   (pack-on-activate LANDED; SETUP walk inverted).
2. `DESIGN-STONE-packed-fire-rows.md` scout 2.
3. `DESIGN-STONE-column-gather-fold.md` (this
   file's stone).

## STOP

1. **STOP-1** — skip BindSpan before gather/fold
   read columns. Re-land scout 2.
2. **STOP-2** — Session-`Vec`. Facts in
   `bind_pool`. 297. SIMD. Invert Cmp.
3. **STOP-3** — skip Token BindSpan. SETUP PV
   walk. Insert-time SoA.

## Done when

- Engine interned. 7strat green.
- Clippy `--lib -D warnings` silent.
- Census `[200 200]`: FIRE **17.8** (held).
  Seed 15.1 → 14.7. Predicted 11–14 missed:
  skip BindSpan is not the leftover of seed.
  Keep. Do not skip Token spans.

Leave dirty unless asked to commit.

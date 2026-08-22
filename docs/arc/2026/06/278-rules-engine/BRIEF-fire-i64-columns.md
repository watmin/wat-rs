# BRIEF — fire-scoped i64 columns (kill E−K)

## The work

Pack i64 rows on first activate from fields
already in hand. Invert bind-only populate
(no `exec_ops`). BindSpan stays. Do not pack
at SETUP (weighed: extra PV walk FIRE 19→25).

## Read in order

1. `DESIGN-STONE-packed-fire-rows.md` (three
   scouts reverted; do not re-land).
2. `DESIGN-STONE-fire-i64-columns.md` (this
   file's stone).

## STOP

1. **STOP-1** — re-land scout 1 / 2 / 3.
2. **STOP-2** — Session-`Vec`. Facts in
   `bind_pool`. 297. SIMD. Invert Cmp.
3. **STOP-3** — skip BindSpan or SoA insert
   stamp this stone. Gather/fold already
   have pool slots; do not delete them.

## Done when

- Engine interned. 7strat green.
- Clippy `--lib -D warnings` silent.
- Census `[200 200]`: FIRE **17.8** (was 19.3),
  seed **15.1** (was 16.6). SETUP walk reverted
  inside the stone. Keep.

Leave dirty unless asked to commit.

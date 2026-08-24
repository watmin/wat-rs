# BRIEF — BindSpan once at join-index, not per product

## The work

Occupancy stays empty. When a right occupant
first enters `right_idx`, write its BindSpan
onto **that copy** from the packed row.
`join_extend` then shares two words.
Weigh FIRE `[200 200]` (must not rise) and
fanout `[100 20]` probe (must drop).

## Read in order

1. `DESIGN-STONE-occupancy-leaf-column.md` (LANDED).
2. `DESIGN-STONE-column-gather-fold.md` skip BindSpan.
3. `DESIGN-STONE-join-index-span.md` (this file's stone).

## STOP

1. **STOP-1** — skip Token BindSpan.
2. **STOP-2** — write spans onto the shared occupancy Arc.
3. **STOP-3** — rewrite `right_idx`. Session-Vec. 297.

## Done when

- Engine interned. 7strat green.
- Clippy `--lib -D warnings` silent.
- FIRE `[200 200]` does not rise vs 13.7.
- Fanout `[100 20]` `hj:catchup:probe` drops.

## Weigh (2026-08-22) — LANDED

FIRE **13.48** (held). Probe **3.76 → 1.62**.
7strat 3/3. Clippy `--lib` silent.

Leave dirty unless asked to commit.

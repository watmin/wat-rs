# BRIEF — class-scan query harvest, skip the query chain

## The work

Fanout `[40000]` pays 39 ms to park `:fan::q-Pair`.
Query-only Alpha→RootJoin is not a production chain.
Skip those alphas. Harvest `{?fact: fact}` from
derived + input by class. Weigh with-query wall.

## Read in order

1. `DESIGN-STONE-fanout-three-leftover.md` weigh.
2. `DESIGN-STONE-query-class-scan-harvest.md`.

## STOP

1. **STOP-1** — skip an alpha that still feeds a
   Production / HashJoin / Test / `:not`.
2. **STOP-2** — change query-memory shape.
   Session-Vec. intern `names`. 297.
3. **STOP-3** — gate FIRE on a wall. Revert if
   7strat red or with-query maps ≠ 40k.

## Done when

- with-query wall drops ≥ 1 ms vs 65.9.
- 40k maps still. 7strat green. Clippy `--lib`.

## Weigh (2026-08-22) — LANDED

with-query **65.89 → 49.59**. Grid `[40000]`
**58.1 → 42.8**. Harvest 16.91 remains.

Leave dirty unless asked to commit.

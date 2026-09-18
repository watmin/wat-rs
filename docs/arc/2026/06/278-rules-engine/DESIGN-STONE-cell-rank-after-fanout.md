# DESIGN-STONE — rank the cells now that fanout is dry

> **Origin (2026-08-18).** 2u: OUT is rpds node-per-fact; no freeze
> intern. Fanout has no drawable intern ≥ 1 ms that is not 2o-dead
> / names / stamp / Session rewrite. Grid `T23-57-10Z` closest
> cell was fanout `[40000]` ratio **1.42** (wat 141 ms). Native
> FIRE there is now **~40 ms**. That ratio is stale. Weigh
> before drawing a different cell.

## The measurement we do not have

Post-2g stones (2i–2s) apply to every join / production /
pool, not only fanout. Accum `[200 200]` was **63.10** at
2g. Node-share `[50 200]` was polish. Neither has a current
FIRE on disk. Ranking the next cell off the August-18 grid
is the R61 error: we would walk a ratio that includes a
141 ms fire we no longer pay.

## The algorithm

Reuse the existing census helpers. Mean of 3. Top rung only.

```
fanout     [100 20]    fanout_phase_census
accum      [200 200]   accum_phase_census
node-share [50 200]    node_share_phase_census
```

FIRE = IN + SETUP + ROUND + OUT. Print FIRE and the largest
named child (not a TOP row). Rank by FIRE. The next cell is
the largest FIRE that still has a drawable leftover (not
2o-dead / names / stamp / Session rewrite).

No fire-path change. No Clara grid (stale ratios; this stone
ranks native FIRE). No persist-gather.

## ★ THE ONE CONTRACT DECISION

**This stone prints the rank. It does not change the engine.**
The next strike is drawn on the named cell, not on fanout.

## The gate

1. `cell_rank_after_fanout` prints three FIRE rows. Each
   FIRE > 0. Do not wall-gate a ratio.
2. rete lib.
3. clippy `-D warnings`.

## Predicted win

A ranking. Independent guess (written first): **accum
`[200 200]` still leads native FIRE** (fold is gone; leftover
is `setup:seen` insert + drop + production). Node-share stays
polish. Fanout is dry. Next cell is accum.

## Blast radius

`src/rete/kernel/tests/` only. No `.wat`. No engine change.

## Out of scope = REJECTED

- Full Clara grid. Persist gather (#3). 297. Fanout intern
  (2o / names / stamp / Session).
- Nested 40k marks.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`cell_rank_after_fanout` (mean of 3). Skipped `WHOLE EVAL`
(compile+seed, not fire).

| cell | FIRE | top-row |
|---|---:|---|
| **accum `[200 200]`** | **61.72** | alpha 41.65 |
| fanout `[100 20]` | 40.22 | production 19.39 (2p: ~12 ms test instrument) |
| node-share `[50 200]` | 1.71 | filter 0.50 |

Prediction held. Accum leads. Alpha 41.65 is the candidates
trap until a leftover split names remainder vs tax (same
class as 2p). Do not intern alpha off the raw. Fanout is
dry. Node-share is polish.

Next cell is **accum `[200 200]`**. Weigh leftover before
drawing. Do not persist gather. Do not start 297.

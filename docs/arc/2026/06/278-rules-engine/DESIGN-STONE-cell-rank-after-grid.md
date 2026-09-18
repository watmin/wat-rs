# DESIGN-STONE — rank the closest cells after intern 7–14

> **Origin (2026-08-20).** Grid `T00-17-00Z`: 30/30 `:match`,
> 30/30 `:us`. Closest *ratio* is **deep-cascade `[50 100]`
> 3.34** (wat 36.3 ms). Closest large fire is **fanout
> `[40000]` 3.48** (wat 58.5 ms). Accum `[200 200]` is
> **6.29** (27.5 ms). Intern 7–14 + leftover 1–14 ranked
> **accum**. The 08-18 cell-rank (2v) ranked off fanout-dry
> at `[100 20]` FIRE 40.22 and put us on accum. That rank
> is stale. Dominance is the closest Clara cells.

## The measurement we do not have

Native FIRE + largest named child at the three closest
08-20 rungs, on this tip (intern 7–14 dirty). Grid
`:wat-ns` is fire-only protocol time; census FIRE is
IN+SETUP+ROUND+OUT. Neither tells us which *named
phase* still owns fanout or cascade. Ranking the next
intern off accum `setup:seen` ~3.9 while fanout is 58 ms
is the R61 error again.

## The algorithm

Reuse existing census helpers. Mean of 3. Top rung only.

```
fanout        [100 20]     fanout_phase_census     // grid [40000]
deep-cascade  [50 100]     cascade_phase_census    // grid [50 100]
accum         [200 200]    accum_phase_census
```

FIRE = IN + SETUP + ROUND + OUT. Print FIRE and the
largest named child (not a TOP row). Rank by FIRE.
The next intern cell is the largest FIRE that still
has a drawable leftover (not 2o-dead / names / stamp /
Session-`Vec` / persist-gather / scratch-repr).

No fire-path change. No Clara grid (this stone ranks
native FIRE). Node-share is polish (ratio 11.43) — omit.

## ★ THE ONE CONTRACT DECISION

**This stone prints the rank. It does not change the
engine.** The next strike is drawn on the named cell.

## The gate

1. `cell_rank_after_grid` prints three FIRE rows. Each
   FIRE > 0. Do not wall-gate a ratio.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

A ranking. Independent guess (written first): **fanout
`[100 20]` leads native FIRE** (~50–60 ms, tracking
grid 58.5). Deep-cascade ~35 ms. Accum ~26 ms (alpha).
Next intern cell is fanout unless its top-row is
2o-dead / names / stamp.

## Blast radius

`src/rete/kernel/tests/` only. One counted helper
for cascade (depth-split already fires this world).
No `.wat`. No engine change.

## Out of scope = REJECTED

- Full Clara grid. Persist gather. 297. Insertion.
- Intern this stone. Intern `setup:seen` without a
  new split. Scratch second representation.
- Nested per-fact marks. Node-share in the table.

## Sequencing

1. Helper + test. Print. Rank. Stop.

## Weigh (2026-08-20) — LANDED, no intern

`cell_rank_after_grid` (mean of 3). Gate: rete lib 94, clippy
`-D warnings` silent.

| cell | FIRE | top-row |
|---|---:|---|
| **deep-cascade `[50 100]`** | **30.12** | production 4.96 |
| fanout `[100 20]` | 26.91 | production 17.91 |
| accum `[200 200]` | 21.71 | alpha 13.11 |

Prediction missed: fanout does **not** lead. Cascade does.
Production is 16% of cascade FIRE — the other 25 ms is
elsewhere. `a0_depth_cost_split_at_equal_work` at the same
cell: **SETUP 12.70** (seen 0.004) vs depth-10 SETUP 0.77
(+11.93). ROUND LOOP 17.36. Census FIRE matches.

Next intern cell is **deep-cascade `[50 100]`**. Next strike
is a SETUP leftover split, not accum `setup:seen` 3.9, not
fanout production (2p already ranked that pile). Do not
persist gather. Do not start 297.

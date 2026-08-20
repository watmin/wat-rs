# DESIGN-STONE — honest FIRE rank after the arm intern

> **Origin (2026-08-20).** 17 interned the arm. `cell_rank_after_grid`
> then named **fanout production 17.39** as the leftover. 2p
> already taught that production raw on this cell is 80k
> `#[cfg(test)]` marks. 2t printed honest_FIRE. We do not intern
> off 17.39 without the honest rank at this tip.

## The measurement we do not have

`fanout_honest_fire_rank` last weighed at 2t: honest_FIRE
**28.42**, compiled-rhs_net 4.66, OUT 3.26. Intern 2u–17
moved FIRE. Today's probe (this session): honest_FIRE
**9.76**, production raw 17.99, tax_in_parent **12.67**,
honest_prod **0.44**. The 17 ms is instrument. Cell-rank
raw does not know that.

Accum and cascade were ranked on raw FIRE too. We do not
know which cell leads *honest* FIRE now.

## The algorithm

Reuse the three census helpers. Mean of 3. Calibrate.
Print raw FIRE, top-row, and

```
honest_FIRE = FIRE − remainder_raw − tax_in_parent
remainder_raw = prod_raw − rhs_raw − dedup_raw
tax_in_parent = (rhs_pairs + dedup_pairs) × cal
```

Same kids as 2p (`prod:compiled-rhs`, `prod:dedup-store`).
If those pairs are 0, honest_FIRE = FIRE.

Rank intern from **honest_FIRE**, not raw. A row is drawable
only if ≥ 1 ms and not 2o-dead / names / stamp / Session-`Vec`.

No fire-path change. No new 40k marks.

## ★ THE ONE CONTRACT DECISION

**This stone prints the honest rank. It does not change the
engine.** Do not intern production off cell-rank raw.

## The gate

1. `honest_cell_rank_after_arm` prints raw FIRE and
   honest_FIRE for three cells. Each FIRE > 0. Fanout
   honest_FIRE **<** fanout raw. Do not wall-gate a ratio.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

A ranking. Independent guess (written first): **accum
leads honest FIRE** (~21). Fanout honest ~10. Cascade
honest ~17. Next intern cell is accum. Fanout production
is not internable.

## Blast radius

`kernel.rs` tests only. No `.wat`. No engine change.

## Out of scope = REJECTED

- Intern production. Intern `names`. Skip stamp. 2e / 2o.
- 297. Insertion. Session-`Vec`. Per-fact timers.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-20) — LANDED, no intern

`honest_cell_rank_after_arm` (mean of 3). Gate: rete lib 96,
clippy `-D warnings` silent.

| cell | FIRE | honest_FIRE | top-row |
|---|---:|---:|---|
| fanout `[100 20]` | 27.31 | **12.67** | production 18.10 |
| deep-cascade `[50 100]` | 17.96 | 12.90 | production 5.14 |
| **accum `[200 200]`** | 21.11 | **20.73** | alpha 12.74 |

Prediction held. Fanout production 18 ms is 2p instrument
(tax + remainder). Honest intern cell is **accum**. Alpha
12.74 is honest (child timers retired in 6). Next strike
is an accum leftover split (`setup:seen` ~3.9 still needs
a new split; scratch 1.71 STOP; clone 1.02). Do not intern
production. Do not intern `names`. Do not start 297.

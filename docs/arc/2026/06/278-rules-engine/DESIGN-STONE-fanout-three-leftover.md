# DESIGN-STONE — rank the three fanout leftovers

> **Origin (2026-08-22).** Occupancy + join-index-span LANDED.
> Vigilia recast 18 then 19 consecutive 0+0. Grid
> `T20-37-11Z` 30/30 `:match` `:us`. Fanout `[40000]`
> wat-ns **58.1** (held vs 59.3). Census FIRE **28.8**.
> Production raw 19.6 is 2p instrument. Three named
> leftovers. This stone prints the rank.

## The enemy

Grid fire-ns is **58.1**. Kernel phases sum to **28.8**.
The census world (`FANOUT_CENSUS_WORLD`) has **no
Query**. The grid `compile-all`s `:fan::q-Pair` and
harvests 40k binding maps inside `fire-rules`.
`harvest_query_memory` has no phase mark.
`query_memory_to_pm` sits in `to_persistent` **after**
`out:production`. Comparing 58 to 28 is comparing a
query-bearing fire to a query-less one.

The other two leftovers are already split and bounded:

- **B** compiled-rhs_net **~5.1** (40k). 2l pile.
  Bind-slot interned. Do not intern `names`. Do not
  skip the stamp.
- **C** out:production **~3.5**. 2u: no drop-in ≥ 1 ms.
  Session stays a PersistentVector.

## The algorithm

Two fires at `[100 20]`, seed outside, mean of 3.

```
without  compile (collect-rules :fan)
with     compile-all rules (:fan::q-Pair)

wall     Instant around fire-rules only
FIRE     IN + SETUP + ROUND + OUT
A        harvest:query + out:query     // 1 mark each
B        prod:compiled-rhs net
C        out:production
delta    with.wall − without.wall
```

Treat **delta** as the query-bearing leftover (A).
Rank A / B / C. A row is drawable if ≥ 1 ms and not
2o-dead / names / stamp / Session-Vec / 2p instrument.

One mark per fire for harvest and out:query — not 40k.
No fire-path change.

## ★ THE ONE CONTRACT DECISION

**This stone prints the rank. It does not change the
engine.** The next strike is the largest drawable of
A / B / C. Query-memory stays a PersistentMap of
name → vector of binding maps. Dual-impl WHAT is
derived facts/query.

## The gate

1. `fanout_three_leftover_split` prints wall / FIRE /
   A / B / C. `prod:compiled-rhs` pairs = 40,000 on
   both fires. With-query harvest pairs = 1. Do not
   wall-gate FIRE.
2. rete lib.
3. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): **A owns the 58−29
gap.** Census never paid QueryNode. B is ~5 ms 2l.
C is 2u-dead. Next intern is harvest of 40k query
maps (and the QueryNode walk that fills parent beta)
if delta ≥ 1 ms.

## Blast radius

`src/rete/kernel` tests + two `#[cfg(test)]` phase
marks (no-op on the production path). No `.wat`.
No engine change.

## Out of scope = REJECTED

- Intern production off the raw 19.6.
- Intern `names`. Skip stamp. Rewrite `seen`.
- Session-Vec. Skip freeze. 297. Persist gather.
- Nested 40k marks.

## Sequencing

1. Marks. Test. Print. Rank. Stop.
2. Next stone is the intern the rank names.

## Weigh (2026-08-22) — LANDED (rank); intern is class-scan

`fanout_three_leftover_split` `[100 20]`, mean of 3.
Instrument **109.7** ns/pair.

| lump | ms |
|---|---:|
| without-query wall / FIRE | 26.67 / 26.62 |
| with-query wall / FIRE | 65.89 / 65.85 |
| **delta (A)** | **39.22** |
| harvest:query | 14.73 |
| out:query | 3.02 |
| A sum | 17.75 |
| B compiled-rhs net | 3.07 |
| C out:production | 3.61 |

Prediction held. Census world had no query. A owns
the 58−29 gap. Next intern:
`DESIGN-STONE-query-class-scan-harvest`.

# DESIGN-STONE — split deep-cascade harvest from SETUP and the 50-round loop

> **Origin (2026-08-23).** Strat-neg harvest Once LANDED (~0.5–0.7 ms).
> Grid `T04-55-59Z` deep-cascade `[50 100]` wat-ns **10.9**. Cell-rank
> (2026-08-20) named SETUP **12.70** at this cell vs **0.77** at
> depth 10, same 10k derived. That number is stale (fxhash / seen-once
> / PVec). The grid `compile-all`s `q-Node` and `q-Tag`. Census
> `cascade_phase_census` compiles **without** queries.

## The enemy

Two class-scan queries of types that **are** in input (level 0 Node
and Tag). Skip-input cannot fire. Closed bag:

```
200 input (100 Node + 100 Tag)
+ 10,000 derived (50 levels × 100 ids × 2 types)
= 10,200 one-entry maps
```

Unstratified fire. Harvest is in-place, not a second Once.

```
without  compile (build-rules 50)
with     compile-all q-Node q-Tag
wall / FIRE / SETUP / ROUND / harvest:query / query-maps
```

Rank SETUP vs ROUND vs harvest:query. Intern the ≥ 0.5 ms honest
Instant that removes theater. Do not intern wrap of 10k maps as a
new representation. Dual-impl WHAT unchanged.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the engine.**

## The gate

1. `cascade_query_harvest_split` prints without / with.
   with query-maps = 10,200. Do not wall-gate FIRE.
2. If intern: honest Instant drop ≥ 0.5 ms vs the print (this
   tier — 1 ms was the seconds-scale / 40k-mark floor).
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): **ROUND owns the 10.9.** SETUP
is no longer 12.7 (fxhash). harvest:query is wrap of 10,200 maps
(~1.5 ms; 40k wrap was 6). If harvest ≥ 0.5 ms it is wrap physics
on types that must include input — stop. If SETUP still ≥ 0.5 ms
theater, intern that.

## Blast radius

`kernel/tests.rs` print. No `.wat`. No engine change unless a
row earns intern.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. 297.
- Skip-input on Node/Tag (level 0 is those classes).
- Dirty-node agenda in this stone.

## Sequencing

1. Print without/with. Rank SETUP / ROUND / harvest.
2. Intern only a ≥ 0.5 ms theater row. Weigh. Stop.

## Weigh (2026-08-23) — LANDED print; no intern

`cascade_query_harvest_split` `[50 100]`, mean of 3.

| | wall | FIRE | SETUP | ROUND | harvest:query | maps |
|---|---:|---:|---:|---:|---:|---:|
| without queries | 9.59 | 8.95 | **0.06** | **8.63** | 0 | 0 |
| with q-Node q-Tag | 12.04 | 11.37 | 0.06 | 11.10 | **1.97** | 10,200 |
| delta | 2.45 | | | | | |

SETUP 12.70 is dead (fxhash / seen-once). harvest:query is wrap of 10,200 maps (40k wrap was 6 ms). Node/Tag are input at level 0 — skip-input cannot fire. ROUND **8.63** is the 50-round engine (idle scan at equal work). Do not intern wrap. Next intern on this cell is the round loop, a different stone. Clippy `--lib` not re-run this print.

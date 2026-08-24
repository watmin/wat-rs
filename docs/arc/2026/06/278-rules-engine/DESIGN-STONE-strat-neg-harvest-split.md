# DESIGN-STONE — split strat-neg harvest from the six-stratum engine

> **Origin (2026-08-23).** Accum harvest intern LANDED. Grid
> `T04-55-59Z` 30/30 `:match` `:us`. Accum `[200 200]`
> **18.4 → 14.2**. Fanout wrap is physics. Ranked leftover
> is fanout 30.5, then **strat-neg `[6 2000]` 14.2**.
> The grid `compile-all`s ten `(?fact <- :S0)` … `:S9`.
> Census of this cell has never split with vs without those
> queries — the same instrument error accum had.

## The enemy

Ten class-scan queries of derived-only types (`Item` is
input; `S0`–`S9` are produced). Skip-input is interned on
the unstratified harvest path. Stratified fire does **not**
use that path. After the six stratum slices it calls
`harvest_stratified_queries`: a second `fire-once` whose
`facts` is the **closed bag** (`acc_facts` = input ∪
derived). `S0`–`S5` now sit in `wm.facts`. Seed sets
`input_has_scan_class`. Skip-input does not skip.

```
without  compile (build-rules 6)
with     compile-all ten q-S*
wall / FIRE / harvest:query / query-maps
delta    with.wall − without.wall
```

Do not intern until harvest:query ≥ 1 ms. Dual-impl WHAT
is still query-memory maps of the closed bag.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the
engine.** The next intern is harvest:query if it is ≥ 1 ms
and not wrap physics of ~6k maps.

## The gate

1. `strat_neg_query_harvest_split` prints without / with.
   with query-maps = 6,000 (6 strata × 1,000). Do not
   wall-gate FIRE.
2. If intern: harvest:query drops ≥ 1 ms vs the print.
   maps still 6,000. 7strat 3/3 including three-stratum.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): harvest:query owns
the delta (≥ 1 ms) because the harvest re-fire seeds the
closed bag as input. Wrap of 6,000 maps is ~1 ms (40k
was 6). If harvest:query is already wrap-only, the 14.2
is the six-stratum engine and this stone stops.

## Blast radius

`kernel/tests.rs` print. No `.wat`. No engine change
unless harvest:query ≥ 1 ms.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. 297.
- Guess production types. Fanout onto the index path.
- Occupancy skip-activate (3-stratum already gated).

## Sequencing

1. Print without/with. Rank harvest:query.
2. If ≥ 1 ms, intern. Weigh. Stop.

## Weigh (2026-08-23) — LANDED print; no intern

`strat_neg_query_harvest_split` `[6 2000]`, mean of 3.

| | wall | FIRE | harvest:query | maps |
|---|---:|---:|---:|---:|
| without queries | 13.11 | 8.15 | 0 | 0 |
| with ten q-S* | **14.84** | 9.84 | **0.96** | 6,000 |
| delta | **1.74** | | | |

harvest:query is wrap of 6,000 maps (40k wrap was 6 ms). Under the ≥ 1 ms intern gate. Grid 14.2 is the six-stratum engine (without 13.11) plus ~1.7 ms query tax. Skip-input cannot fire: `harvest_stratified_queries` re-seeds `acc_facts` as `wm.facts`. That re-fire's unmarked SETUP is the rest of the 1.74. Do not intern wrap. A later stone may skip occupancy seed on the harvest Once. Clippy `--lib -D warnings` not re-run this print.

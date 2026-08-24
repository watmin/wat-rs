# DESIGN-STONE — query-only alphas never occupy

> **Origin (2026-08-23).** `accum_query_harvest_split` `[200 200]`:
> harvest **0.17**, wall tax **2.45**, ROUND-harvest **2.24**.
> Alpha **9.90 → 11.86 (+1.96)**. Skip-activate is a `continue`
> after `candidates_into` already named the query-only aid.
> Derived CountF (1,000) still pack and walk for occupancy
> harvest does not read.

## The enemy

`DESIGN-STONE-query-class-scan-harvest` skip-activates
query-only alphas. They still sit in `AlphaTree`. A derived
fact of a query-only class:

```
candidates_into  →  [q-CountF aid]
pack i64 row
skip-activate continue
```

Empty-candidates would have returned before pack. The
HashSet check is the stem patch. Occupancy of a class-scan
query is harvest's closed bag, not `wm.alpha`.

```
occupancy_tree = alpha_tree.restrict(kind_ids.alpha − q_only)
activate uses occupancy_tree
skip-activate gone
```

Shared alphas (production + query) stay. Constrained
queries stay on the chain. Dual-impl WHAT unchanged.

## ★ THE ONE CONTRACT DECISION

**The occupancy tree does not contain query-only alphas.**
`skip-activate` has nothing to skip.

## The gate

1. `accum_query_harvest_split` still 1,000 maps. Honest Instant
   wall tax besides harvest drops ≥ **0.5 ms** vs this print
   (ROUND-harvest **2.24**). Do not wall-gate FIRE.
2. 7strat 3/3 including three-stratum.
3. `fanout_three_leftover_split` maps 40,000. harvest:query
   not worse by ≥ 1 ms.
4. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): alpha extra **+1.96 → ~0**.
Wall tax **2.45 → ~0.3** (harvest wrap of 1,000). Grid accum
`[200 200]` **13.9 → ~12**.

## Blast radius

`fire/delta.rs` occupancy tree + `AlphaActivateCx`.
`alpha_tree.restrict` already exists (stratum slice).
No `.wat`. No Session field.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. 297.
- Wrap of query maps. Occupancy of production Readings.
- Dirty-agenda root-join / filter.

## Sequencing

1. Restrict occupancy tree. Drop skip-activate.
2. Weigh split. Stop. Revert if < 0.5 ms.

## Weigh (2026-08-23) — LANDED

`accum_query_harvest_split` `[200 200]`, mean of 3.

| | wall tax | harvest | ROUND-harvest | alpha extra |
|---|---:|---:|---:|---:|
| skip-activate | 2.45 | 0.17 | 2.24 | +1.96 |
| occupancy tree restrict | **1.08** | 0.16 | **0.94** | **+0.66** |

ROUND-harvest **−1.30 ms** (≥ 0.5). Maps 1,000. Fanout harvest 7.85 → 6.66 (not worse). 7strat 3/3 including three-stratum. Clippy `--lib -D warnings` silent. Skip-activate gone. Remaining ROUND-harvest 0.94 is not this intern.

# DESIGN-STONE — index the closed bag by class once

> **Origin (2026-08-23).** Wrap leftover is physics (40k maps
> is WHAT). Accum `[200 200]` wat-ns **26.1** vs occupancy
> FIRE **~13.5**. The grid `compile-all`s five
> `(?fact <- :Type)` queries. Class-scan harvest walks
> input ∪ derived **once per query**. Five scans of 40k
> Readings looking for CountF is the tax.

## The enemy

`harvest_class_scan` filters the whole bag by
`a.class == scan.class`. Fanout has one query class
that IS the bag. Accum has five query classes that
are **derived-only**, and 40,200 input facts that
never match.

```
5 queries × (200 Group + 40,000 Reading + 1,000 derived)
```

Census `accum_phase_census` compiles **without**
queries — FIRE ~13. Grid wat-ns is fire-rules with
the five queries. Comparing 26 to 13 without that
split is the instrument error.

## The algorithm

If any class-scan query exists this fire, index the
closed bag **once**:

```
class → [&Value]
for fact in input ∪ derived:
    Aggregate → push under a.class
harvest: maps from index[scan.class], else []
```

One pass. Five lookups. Fanout stays one pass of
40k Pairs. Dual-impl WHAT unchanged.

Tight print first:

```
without  compile (collect-rules :apx)
with     compile-all five q-*
FIRE / harvest:query / query-maps
delta    with.FIRE − without.FIRE
```

Then intern the index if harvest:query ≥ 1 ms.

## ★ THE ONE CONTRACT DECISION

**Index once. Do not scan the bag per query.**
Query-memory stays name → vector of binding maps.
Do not skip freeze. Do not Session-Vec.

## The gate

1. `accum_query_harvest_split` prints without / with.
   with query-maps = 1,000 (200 × 5). Do not wall-gate FIRE.
2. If intern: harvest:query drops ≥ 1 ms vs the print.
   `fanout_three_leftover_split` still 40k maps.
   harvest:query not worse by ≥ 1 ms vs 6.06.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): harvest:query owns
most of the 26−13 gap (~10 ms of five bag walks).
Index → harvest:query **~2 ms** (1,000 maps). Grid
accum `[200 200]` **26 → ~16**.

## Blast radius

`fire/mod.rs` harvest. `kernel/tests.rs` print.
No `.wat`. No Session field.

## Out of scope = REJECTED

- Skip derived-type queries by guessing production
  types (the index is the general form).
- Session-Vec. Skip freeze. intern `names`. 297.
- Revert class-scan (fanout harvest dies with it).

## Sequencing

1. Print without/with queries. Rank harvest:query.
2. Index the bag once. Weigh. Stop.

## Weigh (2026-08-23) — LANDED

`accum_query_harvest_split` `[200 200]`, mean of 3.

Print (before intern):

| | FIRE | harvest:query | maps |
|---|---:|---:|---:|
| without queries | 12.90 | 0 | 0 |
| with five q-* | 26.97 | **13.11** | 1,000 |
| delta | 14.09 | | |

Index when `scans.len() > 1`. Fanout is one class — HashMap+Vec of 40k refs was **slower** (harvest 6.06 → 9.25); keep the filter walk.

After intern:

| | FIRE | harvest:query |
|---|---:|---:|
| accum with five q-* | **20.35** | **6.17** |
| fanout with q-Pair | 30.64 | **5.88** (was 6.06) |

harvest:query −6.94 ms. Fanout not worse. Query-maps 1,000 / 40,000. Clippy `--lib -D warnings` silent.

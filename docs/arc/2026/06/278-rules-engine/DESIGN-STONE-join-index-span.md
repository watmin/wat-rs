# DESIGN-STONE — BindSpan once at join-index

> **Origin (2026-08-22).** Occupancy leaf-fill LANDED:
> empty `Element.binds`, packed row, shared Arc.
> Accum FIRE **13.7**. Full grid 30/30 `:match` `:us`.
> Fanout `[40000]` wat-ns **55.5 → 61.7**: install
> got cheaper, join-read paid `span_from_row` × 40k
> products. This stone keeps occupancy and writes
> the span **once per occupant** onto the join-index
> copy.

## The enemy

`join_extend` sees empty binds and rebuilds the
span from `bind_only` + `i64_by_fact` on every
product. Fanout is 4k occupants and 40k products.
Census `[100 20]`: seed **0.58 ms (2%)**,
`hj:catchup:probe` **3.76 ms**. Occupancy attacked
seed. The tax is probe.

## The algorithm

```
occupancy Arc stays Element { fact, binds: empty }

right_idx insert (catch-up, step 2, filter-join):
    copy = occupancy element          // binds empty
    k = key_of_el(copy)               // packed path
    copy.binds = span_from_row(...)   // once, AFTER the key
    bucket[k].push(copy)

join_extend:
    el.binds.len > 0 → share two words   // already the path
    else span_from_row                   // leftover / exists
```

Token still has BindSpan (root-join already
writes it). Gather/fold still read columns.
Shared occupant list is not mutated.

## ★ THE ONE CONTRACT DECISION

**Materialize on the indexed copy, not on
occupancy, not per product.** Skip-Token-spans
stays forbidden. `right_idx` stays
`key → Vec<Element>`.

## The gate

1. 7strat including `differential_three_stratum_negation`.
2. rete lib. clippy `--lib -D warnings`.
3. `accum_fire_phase_census` `[200 200]`. **Revert if
   FIRE rises vs 13.7.**
4. `fanout_fire_phase_census` `[100 20]`. **Revert if
   `hj:catchup:probe` does not drop.**

## Predicted win

Independent guess (written first): fanout probe
**3.76 → 1–2 ms**. FIRE `[200 200]` **holds ~13.7**
(accum does not index these occupants as a 40k
join). Deep-cascade is discriminated (already
has binds) — not this stone.

## Sequencing

1. `element_with_row_span` next to `span_from_row`.
2. Catch-up right_idx, step 2, step 4 Δright, filter-join index.
3. 7strat. Census. Stop.

## Weigh (2026-08-22) — LANDED

7strat 3/3 including three-stratum. Clippy `--lib`
silent. Occupancy Arc still empty.

| cell | before | after |
|---|---:|---:|
| accum `[200 200]` FIRE | 13.7 | **13.48** (held) |
| accum seed | 10.5 | 10.39 |
| fanout `[100 20]` `hj:catchup:probe` | 3.76 | **1.62** |
| fanout hash-join | 4.14 | 2.02 |

Probe is the intern. Fanout FIRE still **~29.6**
because production is 19.6 / 66% (40k RHS) — that
leftover was ranked 2026-08-18, not this stone.

# DESIGN-STONE — class-scan harvest after stratified fire does not re-seed occupancy

> **Origin (2026-08-23).** `strat_neg_query_harvest_split` `[6 2000]`:
> without queries wall **13.11**, with ten q-S* **14.84**, harvest:query
> **0.96** (6,000 maps). harvest:query is wrap, under 1 ms. Wall delta
> **1.74** is the harvest Once: `harvest_stratified_queries` re-fires
> `acc_facts` as `wm.facts`, so skip-input never fires and seed walks
> Item ∪ S0–S5 for occupancy the query-only alphas then skip.

## The enemy

Unstratified harvest class-scans `facts` ∪ `derived_facts` in place.
Stratified harvest is a second `FireKind::Once` whose facts field is
the closed bag. `S0` is input on that fire. Occupancy seed of 8,000
facts is paid so `harvest_query_memory` can walk them.

When every QueryNode is a class-scan, the Once is occupancy theater.
The closed bag is already `input` ∪ `acc_derived`.

```
if class-scans cover every query:
    wm.facts = input
    wm.derived_facts = acc_derived
    input_has_scan_class = wanted ∩ input classes
    harvest_query_memory
else:
    existing Once on acc_facts
```

Dual-impl WHAT unchanged. Constrained / join queries keep the Once.

## ★ THE ONE CONTRACT DECISION

**Do not seed occupancy to harvest a class-scan after stratified
fire.** Query-memory stays name → vector of binding maps.

## The gate

1. `strat_neg_query_harvest_split` still 6,000 maps. Honest Instant
   wall (same instrument both arms) drops vs **14.84**. At this
   tier a ≥ 0.5 ms drop that *removes occupancy theater* is the
   intern — the 1 ms floor was “bigger than 80k mark pairs” when
   FIRE was seconds. Do not wall-gate FIRE.
2. 7strat 3/3 including three-stratum.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): wall **14.84 → ~13.2**. harvest:query
stays wrap of 6,000 (~1 ms). Grid strat-neg `[6 2000]` **14.2 → ~13**.

## Blast radius

`fire/rules.rs` harvest_stratified_queries. `fire/mod.rs` pub(crate)
the existing harvest helpers. No `.wat`. No Session field.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. 297.
- Skip Once for constrained queries.
- Occupancy skip-activate on production strata.

## Sequencing

1. Class-scan cover → harvest bags, no Once.
2. Weigh split. Stop.

## Weigh (2026-08-23) — LANDED

`strat_neg_query_harvest_split` `[6 2000]`, mean of 3.

| | wall | harvest:query | maps |
|---|---:|---:|---:|
| with, Once | 14.84 | 0.96 | 6,000 |
| with, skip Once | **14.16** | 0.71 | 6,000 |

Wall **−0.68 ms** first weigh (~5% of the 14 ms cell); confirmation
14.84-path vs 14.12 (delta 0.52, harvest 0.70, maps 6,000). Honest Instant,
not mark tax. The 1 ms floor was the seconds-scale / 40k-mark instrument.
This intern removes occupancy seed query-only alphas already skip.
Remaining wrap of 6,000 maps is physics. 7strat 3/3 including three-stratum.
Clippy `--lib -D warnings` silent.

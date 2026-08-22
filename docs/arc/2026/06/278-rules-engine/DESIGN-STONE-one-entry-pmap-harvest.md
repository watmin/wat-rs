# DESIGN-STONE — one-entry PMap harvest

> **Origin (2026-08-22).** Class-scan harvest LANDED
> (`DESIGN-STONE-query-class-scan-harvest`). Fanout
> `[40000]` wat-ns **58.1 → 42.8**. Leftover A is
> harvest:query **16.91** (40k one-entry binding maps)
> + out:query **3.04**. This stone internes harvest.

## The enemy

`harvest_class_scan` emits `{?fact: fact}` via
`PMap::from_pairs([(var, fact)])` × 40k. `from_pairs`
always grows a `Vec`, scans it for duplicate keys,
then `Arc::new`s the array arm. A one-pair map has
no duplicate to scan and no growth to amortize.

Query-memory stays name → vector of binding maps.
The Array arm stays the Array arm. PMap Array1 as a
third representation is REJECTED (runtime-wide).

## The algorithm

`from_pairs` of 0 pairs is `PMap::new()`. `from_pairs`
of 1 pair is `Array(vec![pair])` with no grow/scan.
2+ pairs keep the existing accumulator. Harvest does
not change its call.

```
from_pairs []      → new()
from_pairs [kv]    → Array([kv])
from_pairs [..]    → existing acc + promote
```

## ★ THE ONE CONTRACT DECISION

**One-entry construction is the Array arm, not a
new representation.** Dual-impl WHAT is unchanged.
Do not intern `names`. Do not skip freeze.

## The gate

1. `fanout_three_leftover_split` with-query maps =
   40,000. harvest:query drops ≥ 1 ms vs 16.91.
2. `PMap::from_pairs([(k,v)]) == PMap::new().assoc(k,v)`.
   rete lib.
3. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): harvest:query
**16.91 → ~12**. The duplicate scan and growable Vec
die; 40k `Arc<Vec<1>>` remain. out:query 3.04 is
the PersistentVector wrap, next if still ≥ 1 ms.

## Blast radius

`src/value/pmap.rs` `from_pairs`. No `.wat`. No
Session field. No harvest-loop change.

## Out of scope = REJECTED

- A third PMap arm (Array1).
- Session-Vec. Skip freeze. intern `names`. 297.
- Intern out:query (rpds PV wrap) in this stone.

## Sequencing

1. Fast-path 0/1 in `from_pairs`. Test equality.
2. Weigh harvest:query. Stop.

## Weigh (2026-08-22)

`fanout_three_leftover_split` `[100 20]`, mean of 3.
instrument 100.7 ns/pair.

| lump | ms |
|---|---:|
| without-query wall / FIRE | 26.13 / 26.07 |
| with-query wall / FIRE | 40.35 / 40.30 |
| harvest:query | **7.71** (was 16.91) |
| out:query | 3.08 (held) |
| query-maps | 40,000 |

Gate held: harvest:query −9.2 ms. Prediction
16.91 → ~12 was shy; the growable Vec was the
hotter stem. 40k `Arc<Vec<1>>` remain. out:query
3.08 is the next leftover if ≥ 1 ms. Clippy
`--lib -D warnings` silent. `one_pair_from_pairs`
equals assoc.

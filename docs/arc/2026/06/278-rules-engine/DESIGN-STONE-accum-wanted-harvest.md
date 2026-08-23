# DESIGN-STONE — index only the classes the scans name

> **Origin (2026-08-23).** Class-index LANDED. Index when
> `scans.len() > 1`. Fanout filter path held. Accum
> `[200 200]` wat-ns **18.4**. `accum_query_harvest_split`
> harvest:query **6.23** (1,000 maps). The index still
> walks 40,200 Group+Reading and builds a 40k-ref `Vec`
> under `Reading` that no scan looks up.

## The enemy

Five queries: `(?fact <- :CountF|:SumF|:MinF|:MaxF|:ExistsF)`.
All derived-only. `closed_bag_by_class` keys **every**
aggregate in input ∪ derived:

```
200 Group + 40,000 Reading + 1,000 derived
```

The Reading vec is the sibling of the fanout index
regression (HashMap+Vec of 40k refs, harvest 6.06 → 9.25).
Fanout stayed on the filter path. Accum pays it.

Wrap of 1,000 maps is not the 6.23 — 40k wrap was 6 ms.

Do not guess production types. The scan set is known.
Input classes are known at seed. Dual-impl WHAT stays
input ∪ derived.

## The algorithm

Tight print first. Isolated bags, mean of 3, unscaled.
No 40k fire marks.

```
input    200 Group + 40,000 Reading
derived  200 × {CountF, SumF, MinF, MaxF, ExistsF}
wanted   those five class names

I  current index (both bags, every class)
W  wanted-only (both bags; push iff class ∈ wanted)
D  derived-only wanted (skip wm.facts)
M  wrap 1,000 maps
```

Treat **I** as today's harvest tax. Treat **W** as
dropping the Reading vec. Treat **D** as dropping the
input walk. Treat **M** as wrap physics of 1,000.

Then intern the ≥ 1 ms row:

1. **W drops ≥ 1 ms vs I**: index only `scan.class`.
   Still walk both bags. Completeness: a wanted class
   in input still lands.
2. **D drops ≥ 1 ms vs W**: skip `wm.facts` when no
   wanted class was packed as input. Fire-scoped flag
   from the seed walk (already visits every input fact).
   Not a Session field. If someone inserts a `CountF`,
   the flag is true and facts are walked.
3. **M ≥ 1 ms**: stop — that is wrap of WHAT, a
   different stone. Do not Array1.
4. Largest part **< 1 ms**: stop.

Fanout stays `scans.len() == 1` filter. Do not put
fanout on the index path.

## ★ THE ONE CONTRACT DECISION

**Index only the classes the scans name. Do not walk
input when none of those classes exist there.**
Query-memory stays name → vector of binding maps.
Do not skip freeze. Do not Session-Vec.

## The gate

1. `accum_harvest_index_parts` prints I / W / D / M.
   M wraps 1,000. I > 0. Do not wall-gate FIRE.
2. If intern: `accum_query_harvest_split` harvest:query
   drops ≥ 1 ms vs **6.23**. maps = 1,000.
   `fanout_three_leftover_split` maps = 40,000.
   harvest:query not worse by ≥ 1 ms vs **5.59**.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): **I owns ~5–6 ms**
(the Reading vec). **W** is the intern. **D** is this
cell's completeness (no scan class in input) and lands
if W still ≥ 1 ms. **M** is noise (~0.15 ms).
harvest:query **6.23 → ≤ 1**. Grid accum `[200 200]`
**18.4 → ~13–14** (query-less engine floor). Fanout
28.5 does not move.

## Blast radius

`fire/mod.rs` harvest. Seed flag only if D is interned.
`kernel/tests.rs` print. No `.wat`. No Session field.

## Out of scope = REJECTED

- Guess production types (the scan set is the general
  form; class-index already rejected the guess).
- Session-Vec. Skip freeze. intern `names`. 297.
- Fanout onto the index path. Array1. Fuse harvest
  into freeze to move the mark.

## Sequencing

1. Print I / W / D / M. Rank.
2. Intern the ≥ 1 ms row. Weigh harvest:query. Stop.

## Weigh (2026-08-23) — LANDED

`accum_harvest_index_parts` `[200 200]`, mean of 3 (Array bags):

| lump | ms |
|---|---:|
| I both bags, every class | 2.07 |
| W both bags, wanted only | 1.65 |
| **D derived only, wanted** | **0.05** |
| M wrap 1,000 maps | 0.12 |
| I−W Reading vec | 0.42 |
| **W−D input walk** | **1.60** |

W vs I is not ≥ 1 ms. **D vs W is.** Isolated I is 2 ms because `from_vec` is Array; in-fire `wm.facts` is a Tree (seed conj past 8). The 6.23 is that Tree walk. Skip-input is the intern. Wanted-only rides along (same loop). Fire-scoped `input_has_scan_class` from seed. Not a Session field. Fanout stays the filter path.

`class_scan_harvest_includes_input`: two T inserted, q-T=2, q-U=0.

After intern (`accum_query_harvest_split`, mean of 3):

| | FIRE | harvest:query | maps |
|---|---:|---:|---:|
| without queries | 13.49 | 0 | 0 |
| with five q-* | **15.23** | **0.15** | 1,000 |

harvest:query **6.23 → 0.15** (−6.08). Fanout harvest **5.20** (was 5.59, not worse). maps 40,000. 7strat 3/3 including three-stratum. Clippy `--lib -D warnings` silent.

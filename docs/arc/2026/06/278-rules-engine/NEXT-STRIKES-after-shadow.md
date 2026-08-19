# NEXT STRIKES — after ShadowNode (2026-08-18)

> Live breadcrumb: `CURRENT-STATE-annihilate-interpretation.md`.
> This page is the **queue**. One strike at a time. Measure, then kill.
> Do not start 297. Do not service-ify. Do not persist gather to
> dodge the fold.

## What we just proved

`(b)` ShadowNode landed. Node-share `[50 200]`: 10,000 evals → 200
(1.00/token). Grid 2026-08-18T20-55-00Z: 30/30 `:match`, 29 `:us`,
one unresolved — `accum [200 200]` (ratio 1.12, min 1.05).

The lab tricks we already took: tree, wildcard, `any_constrains`,
keyed gather, arm intern. The lab trick we have **not** taken is
"the map that lives." That is **not** the unresolved cell.

## The measurement that ranks the queue

`accum_fire_phase_census` (release, 3 runs, this session) at
`G=200 W=200` (40,200 facts), FIRE **160.70 ms**:

| phase | ms | share | note |
|---|---:|---:|---|
| **`accum:fold`** | **68.49** | **42.6%** | 4 nodes (count/sum/min/max) |
| alpha | 49.28 | 30.7% | instrument-heavy; net match ~2.7 ms |
| SETUP | 14.42 | 9.0% | |
| `accum:index` | 12.04 | 7.5% | 4 marks; cache already 2 builds |
| `accum:snapshot` | 5.10 | 3.2% | clone of `:from` |
| filter | 0.26 | 0.2% | `(b)` + keyed gather killed this |

`fold_cost_with_and_without_the_binding_lookup` at the same size,
one rule, 40,000 gathered elements:

| fold | `accum:fold` |
|---|---:|
| `count` (no per-element `acc_var_i64`) | **9.59 ms** |
| `sum` (one lookup / element) | **18.50 ms** |
| delta = the lookup | 8.91 ms · **223 ns/element** |

Four nodes ≈ 9.6 + 3×18.5 ≈ **65 ms**. That is the 68 ms row.
The fold mark **includes** the rematch walk that builds
`Vec<&Element>` — `count`'s 9.59 ms is not `gathered.len()`.

**Persist-across-rounds gather cannot move this cell.** A cold
`fire-rules` of a seeded session is one (maybe two) rounds. The
index is built once. The 68 ms is walking the bucket.

## After 2c — SETUP did not fall

`DESIGN-STONE-setup-seen-once` **landed**. First worklist is the facts
PV; `seen` is filled once; `alpha_activate_fact` is shared. Gate:
rete lib 66, `binary_id(wat::rete)` 299, clippy `-D warnings` silent.
Census `[200 200]` FIRE **86.26 ms**; SETUP **13.40 ms** of which
**`setup:seen` 13.26 ms**. The extra Vec clone was not the 14 ms.
Leftover is one SipHash + HashSet insert of 40k input Aggregates.
The stone named this: *if SETUP barely moves, next is a hasher,
not another copy.*

## After 2d — hasher landed; leftover is the walk + drop

`DESIGN-STONE-setup-fxhash` **landed**. `seen` is `FxHashSet<Value>`;
gather maps are `FxHashMap`. Gate: rete lib 66, `binary_id(wat::rete)`
299, clippy `-D warnings` silent. Census `[200 200]`:

| mark | before 2d | after 2d |
|---|---:|---:|
| FIRE | 86.26 | **76.85** |
| SETUP | 13.40 | **8.30** |
| `setup:seen` | 13.26 | **8.17** |
| `accum:index` | ~8 | **5.77** |
| `round:drop-memories` | ~10 | **10.49** |

SipHash mix was real (~5 ms, matching 40k × 121 ns). Remaining
`setup:seen` 8.17 is the Hash *walk* + HashSet insert. **Do not
add a second hasher.** Drop is now the largest named leftover
on this cell. Persist is still ~0 on a cold fire.

## The queue (fixed order)

| # | strike | predicted cut at `[200 200]` | status |
|---|---|---|---|
| **1** | **`DESIGN-STONE-accum-fold-the-wall.md`** — skip rematch when the cond has no `SeedCmp`; fold `Element.bindings[slot]`, not `Bindings::get`. Count is `bucket.len()`. | 68.49 → **2.32 ms**. FIRE 160.70 → **101.65 ms**. Count 9.59 → 0.15; sum 18.50 → 0.71 (14 ns/el). | **LANDED** |
| 2 | `DESIGN-STONE-gather-no-snapshot.md` — buckets index `wm.alpha[id]`, no clone. | snapshot 5.56 → **0.00 ms**. FIRE 101.65 → **92.54 ms**. Index 13.65 → 7.78 (clone was inside that mark too). | **LANDED** |
| 2b | `DESIGN-STONE-delta-alpha-indices.md` — `d_alpha` is `Vec<usize>` into `wm.alpha`. | Quiet FIRE 92.54 → **83.99 ms**. Loaded run 97.55 — do not gate the wall. One Element, not two. | **LANDED** |
| 2c | `DESIGN-STONE-setup-seen-once.md` — first worklist is the facts PV; `seen` once. | Predicted SETUP 14 → 7–10. **Measured SETUP 13.40, `setup:seen` 13.26.** Clone was not the row. | **LANDED** |
| 2d | `DESIGN-STONE-setup-fxhash.md` — `rustc-hash` on `seen` + gather maps that hash `Value`. | `setup:seen` 13.26 → **8.17**. FIRE 86.26 → **76.85**. Index 8 → 5.77. | **LANDED** |
| 2e | `DESIGN-STONE-elem-bindings-inline.md` — width 0–2 in the enum, not `Arc`. | drop 10.49 → **5.45**. FIRE 76.85 → **78.38**. Push ate the win (fatter Element). **Reverted.** | **TRIED — not a FIRE win** |
| 2f | `DESIGN-STONE-bind-pool.md` — pairs in `wm.bind_pool`; `Element` is `(off, len)`. Indices, not pointers. | drop 10.49 → **3.63**. FIRE 76.85 → **67.33**. Push stayed thin. | **LANDED** |
| 2g | `DESIGN-STONE-aggregate-identity.md` — shallow Aggregates stamp `identity` at birth. Hash writes the u64. | `setup:seen` ~8.8 → **6.89**. FIRE 67.33 → **63.10**. Leftover is HashSet insert. | **LANDED** |
| **2h** | **Fanout census** — grid closest cell is fanout `[40000]` (ratio 1.42). HashJoin / token emit, not accum leftovers. Weigh before drawing. | — | **NEXT** |
| 3 | `DESIGN-STONE-persist-gather-across-rounds.md` — P6 for gathers; append `d_alpha`. | ~0 on a cold fire (index is first-round hash). | after a census names a multi-round cell |
| 4 | `DESIGN-STONE-where-dim-reuse.md` — `(b)`: do not `exec_where` a proven `(= dim lit)`. | node-share polish (already 3.18 ms) | after the accum wall |
| 5 | `DESIGN-STONE-where-range-edges.md` — populate `range_children`. | chaos-engine / `where-numeric`; not this grid | after 4 |

Already on disk, **not** this queue:

- `DESIGN-STONE-cross-call-incrementality.md` — no-op re-fire costs ~100% of cold. Correctness is TM / negation. Do not fold into #3.
- 297, service-ify, persist gather *instead of* #1.

## Law

One stone. One gate. One grid cell named. Weigh, then the next.
Slow is smooth.

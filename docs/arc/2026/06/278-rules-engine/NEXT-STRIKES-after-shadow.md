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
| 2h | `DESIGN-STONE-fanout-phase-census.md` — phase table at the grid ladder. | `[100 20]` FIRE 96.66. production 40.0 (41%); `hj:catchup:probe` 30.9 (32%). `right_idx` clone 0.23. | **LANDED** |
| 2i | `DESIGN-STONE-join-extend-no-leftover.md` — rematch iff `has_seed_cmp()`. | probe 30.87 → **13.54**. FIRE 96.66 → **72.43**. | **LANDED** |
| 2j | `DESIGN-STONE-prod-no-token-clone.md` — production walks `d_beta`, no Token clone-collect. | production 36.84 → **26.30**. FIRE 72.43 → **61.35**. | **LANDED** |
| 2k | `DESIGN-STONE-class-arc.md` — `class: Arc<str>`; `record_arc` on the RHS. | compiled-rhs 8.05 → **8.78**. FIRE 61.35 → **60.90**. Type intern; no FIRE cut. | **LANDED** |
| 2l | `DESIGN-STONE-rhs-construct-split.md` — apportion compiled-rhs without per-fact marks. | bind-get 30% of D, stamp 28%. Pile, not one row. | **LANDED** |
| 2m | `DESIGN-STONE-token-bind-pool.md` — Token bindings are `BindSpan`; extend appends. | probe 14.35 → **10.99**. FIRE 60.90 → **47.48**. | **LANDED** |
| 2n | `DESIGN-STONE-match-pool.md` — Token matches are `BindSpan`; Token is `Copy`. | probe 10.99 → **11.39** (wash). FIRE 47.48 → **44.23**. | **LANDED** |
| 2o | `DESIGN-STONE-extend-no-left-copy.md` — extra span, no left-bind copy. | probe 11.39 → **12.54**. FIRE 44.23 → **46.95**. Two-span get ate the win. **Reverted.** | **TRIED — not a FIRE win** |
| **2p** | `DESIGN-STONE-prod-leftover-split.md` — remainder_raw vs tax-in-parent. | Naive unmarked 12.43 = tax 6.95 + remainder 5.48 (68 ns × 80k bookkeeping). compiled-rhs_net **6.68** is the engine row (2l pile). | **LANDED** |
| **2q** | `DESIGN-STONE-rhs-bind-slot.md` — slot from first token; Token stays thin. | slice−slot **1.24 ms**. compiled-rhs_net 6.68 → **4.14**. production 21.60 → **19.26**. FIRE 44.23 → **45.20** (probe wash). | **LANDED** |
| **2r** | `DESIGN-STONE-probe-extend-split.md` — B / M / E / K / H. | E **7.08** = B **5.38** + M 1.64. K+H **0.14**. B is 2o-dead. In-fire gap 12.30−7.08 ≈ **5.2**. | **LANDED** |
| **2s** | `DESIGN-STONE-probe-gap-split.md` — wrapper vs growth; reserve catch-up pools. | G−E **4.13**. J−E 0.99 (no hoist). probe 12.30 → **7.08**. FIRE 45.20 → **39.48**. | **LANDED** |
| **2t** | `DESIGN-STONE-honest-fire-rank.md` — strip 80k test marks; rank engine rows. | honest_FIRE **28.42**. Probe **8.11** (2o-dead B). compiled-rhs_net **4.66** pile. Drawable: **OUT 3.26**. | **LANDED** |
| **2u** | `DESIGN-STONE-out-production-split.md` — V / H / I on 40k Pairs. | V−C **2.75** (rpds node). from_iter **slower**. No intern. Session stays a PersistentVector. | **LANDED** |
| **2v** | `DESIGN-STONE-cell-rank-after-fanout.md` — native FIRE at three cells. | accum **61.72** (alpha 41.65). fanout 40.22 dry. node-share 1.71 polish. | **LANDED** |
| **2w** | `DESIGN-STONE-accum-leftover-split.md` — alpha remainder vs tax on `[200 200]`. | remainder+tax **42.58** of alpha 43.51. honest_FIRE **20.97**. setup:seen **7.43**. index 5.23. | **LANDED** |
| **2x** | `DESIGN-STONE-seen-identity-set.md` — `seen` of stamped facts is the fingerprint. | S−I **2.28**. setup:seen 7.43 → **4.31**. FIRE 63.83 → **61.00**. | **LANDED** |
| **2y** | `DESIGN-STONE-gather-unary-index.md` — unary gather when `join_keys.len()==1`. | B−S **1.03**/build. index 5.16 → **3.61**. FIRE 61.00 → **57.92**. | **LANDED** |
| **2z** | `DESIGN-STONE-seen-pv-walk.md` — PV walk vs Vec on leftover `setup:seen`. | P−V **0.38**. D+V worse. No intern. Isolated P 1.67 vs in-fire 4.30 is fire context. | **LANDED** |
| **2aa** | `DESIGN-STONE-drop-memories-split.md` — four clears. | A **1.06** (fact Arc). M 0.95. B 0.78. T 0.00. D 2.75. | **LANDED** |
| **2ab** | `DESIGN-STONE-fact-as-index.md` — Element.fact is a `u32`. | Isolated A 1.06 → **0.00**. drop 3.63 → **1.14**. FIRE 57.92 → **51.56**. | **LANDED** |
| **2ac** | match_pool does not own a fact clone (M ≥ 1). Index, like Element. Do not fatten Token. | — | **NEXT to draw** |
| 3 | `DESIGN-STONE-persist-gather-across-rounds.md` — P6 for gathers; append `d_alpha`. | ~0 on a cold fire (index is first-round hash). | after a census names a multi-round cell |
| 4 | `DESIGN-STONE-where-dim-reuse.md` — `(b)`: do not `exec_where` a proven `(= dim lit)`. | node-share polish (already 3.18 ms) | after the accum wall |
| 5 | `DESIGN-STONE-where-range-edges.md` — populate `range_children`. | chaos-engine / `where-numeric`; not this grid | after 4 |

Already on disk, **not** this queue:

- `DESIGN-STONE-cross-call-incrementality.md` — no-op re-fire costs ~100% of cold. Correctness is TM / negation. Do not fold into #3.
- 297, service-ify, persist gather *instead of* #1.

## Law

One stone. One gate. One grid cell named. Weigh, then the next.
Slow is smooth.

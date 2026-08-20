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
| **2ac** | `DESIGN-STONE-match-pool-fact-as-index.md` — match_pool is `(u32, i64)`. | Isolated M 1.08 → **0.00**. D 1.84 → **0.77**. in-fire drop 1.14 → 1.18 (wash). FIRE 51.56 → **53.83** (alpha instrument). root-join 0.14 → **0.03**. | **LANDED** |
| **2ad** | `DESIGN-STONE-bind-key-intern.md` — bind keys are a fire-scoped `u32`. | Isolated B 0.78 → **0.32** (value Drop leftover). drop 1.18 → **0.46**. FIRE 53.83 → **48.52**. | **LANDED** |
| **2ae** | `DESIGN-STONE-bind-value-intern.md` — fillers are a fire-scoped `u32`. | Isolated B 0.32 → **0.00**. drop 0.46 → **0.01**. FIRE 48.52 → **49.13** (alpha wash). Isolated drop exhausted. | **LANDED** |
| **3** | `DESIGN-STONE-persist-gather-across-rounds.md` — P6 for gathers; append `d_alpha`. | Accum 2 rounds, aEls **80200 both**. builds **2**. FIRE 49.13 → **51.68** (wash). Not a FIRE cut on this cell. | **LANDED** |
| **4** | `DESIGN-STONE-where-dim-reuse.md` — skip `exec_where` on proven `(= dim lit)`. | `[50 200]` evals **0**, reuse **200**, passes **200**. FIRE 1.71 → **1.62**. filter 0.50 → **0.43** (walk leftover). | **LANDED** |
| **5** | `DESIGN-STONE-where-range-edges.md` — populate `range_children`. | Unit: `(> ?k 10)` prune 5 / prove 15. `[50 200]` evals **0** / reuse **200**. FIRE 1.62 → **1.65** wash. Accum **48.32** wash. Mechanism, not a FIRE cut. Alpha-tree ranges untouched. | **LANDED** |
| **6** | `DESIGN-STONE-retire-alpha-child-marks.md` — per-fact alpha timers off. | Child pairs **0**. FIRE 48.65 → **26.53**. honest_FIRE −2.21 → **26.25**. honest_alpha **18.16** (was hidden in remainder). setup:seen **3.92**. Node-share evals 0 / reuse 200. | **LANDED** |
| **7** | `DESIGN-STONE-alpha-leftover-split.md` — rank 18 ms without per-fact timers. | Seed **17.97**, delta **0.04**. Isolated: `M−T` **7.65**, tree **4.46**, push **3.45**. A 16.54 ≈ seed 17.97. **No intern.** | **LANDED** |
| **8** | `DESIGN-STONE-compiled-match-split.md` — ops vs intern inside `M−T`. | Ops **1.90**. intern/materialize **6.18**. Cold-insert tax **0.22**. `fact_bind` 0. 80,200/80,200 ops-true. **No intern.** | **LANDED** |
| **9** | `DESIGN-STONE-materialize-split.md` — clone vs key vs val vs push. | intern_val **2.77**. clone **1.02**. intern_key 0.86. pool.push 0.28. Pile, not one intern. **No intern.** | **LANDED** |
| **10** | `DESIGN-STONE-alpha-tree-walk-split.md` — class HashMap vs walk vs alloc. | Class HashMap **3.26**. Walk **0.08**. Vec alloc **0.82** (< 1 — no reused buffer). **No intern.** | **LANDED** |
| **11** | `DESIGN-STONE-alpha-class-lookup.md` — intern class lookup. | 2 types. L **0.26**, F 0.71, S 1.81. Cut S−L **1.55**. `roots` is linear. G−E **3.26 → 0.65**. | **LANDED** |
| **12** | `DESIGN-STONE-alpha-push-split.md` — HashMap entry vs Vec vs d_alpha. | H−M **1.38**. Interned `wm.alpha`/`d_alpha` FxHashMap. A−M **3.81 → 2.30**. `seen` not interned. | **LANDED** |
| **13** | `DESIGN-STONE-intern-val-i64.md` — i64 intern table. | 120k i64, 0–999. A **0.29**, I 0.44, V 1.81. Cut **1.52**. V−K **2.77 → 0.44**. | **LANDED** |
| **14** | `DESIGN-STONE-exec-ops-split.md` — scratch vs Bind. | Scratch **1.71**. exec_ops body **0.12**. fill(None) **1.78** (not faster — intern reverted). | **LANDED, no intern** |
| **15** | `DESIGN-STONE-cell-rank-after-grid.md` — FIRE at the three closest 08-20 cells. | cascade **30.12** (prod 4.96). fanout 26.91 (prod 17.91). accum 21.71 (alpha 13.11). SETUP at cascade **12.70**. | **LANDED, no intern** |
| **16** | `DESIGN-STONE-cascade-setup-split.md` — arm vs remainder of SETUP. | SETUP **12.52**. arm **12.51**. remainder 0.01. ARM_BUILDS **1.00/run**. WAT compile does not intern. | **LANDED, no intern** |
| **17** | `DESIGN-STONE-arm-at-compile.md` — intern `ReteArm` at `compile-all`. | setup:arm **12.51 → 0.00**. SETUP 12.52 → **0.01**. cascade FIRE 30.12 → **17.62**. ARM_BUILDS 1.00/run (compile). | **LANDED** |
| **18** | `DESIGN-STONE-honest-rank-after-arm.md` — honest FIRE after 17. | fanout honest **12.67** (raw 27.31). cascade 12.90. **accum 20.73**. Production 18 is 2p tax. | **LANDED, no intern** |
| **19** | `DESIGN-STONE-alpha-candidate-buf.md` — reuse `candidates_into`. | seed 12.83 → **11.68**. A 11.39 → **10.15**. A−M 1.77 → **0.67**. FIRE 21.15 → **19.78**. | **LANDED** |
| **20** | `DESIGN-STONE-seen-fire-context.md` — in-fire alloc vs insert. | seen **3.99**. alloc 0.01. insert **3.98**. Isolated S 2.95 (X identity **1.73**). Fire context **1.03**. | **LANDED, no intern** |
| **21** | `DESIGN-STONE-fold-seen-into-seed.md` — `seen_insert` on the seed walk. | seen 3.99 → **0.01**. seed 11.68 → **16.01**. FIRE 19.78 → **19.57** (wash). | **LANDED** |
| **22** | `DESIGN-STONE-cond-key-ids.md` — intern cond slot keys once per fire. | FIRE 20.93 → **19.95**. Isolated K−C still ~1 (scans). | **LANDED** |
| **23** | `DESIGN-STONE-gather-val-id.md` — unary gather hashes filler ids. | U−I **1.40**/build. index 1.97 → **0.61**. FIRE 19.95 → **19.08**. | **LANDED** |
| **24** | `DESIGN-STONE-alpha-tree-fxhash.md` — `Node.children` FxHashMap. | I−G 1.03 → **0.98** wash (predicted 0.4 missed). FIRE 19.04 → **18.75**. Kept (hasher family). | **LANDED, wash** |
| **25** | `DESIGN-STONE-arm-kind-lists.md` — fire-path passes iterate kind lists on the arm. | A0 ROUND extra **+7.04 → +2.06**. Cascade FIRE 18.06 → **11.76**. Honest 13.25 → **7.32**. Hash-join extra **1.43** remains. | **LANDED** |
| **26** | `DESIGN-STONE-dirty-join-parents.md` — hash-join skips idle same-kind parents. | Hash-join extra **1.43 → 0.08**. ROUND extra +2.06 → **+0.95**. Cascade FIRE 11.76 → **10.35**. Honest 7.32 → **6.72**. | **LANDED** |

## After 26 — leftovers (ranked)

Idle same-kind hash-join is dead. A0 ROUND extra is a
pile of pieces under 1 (root-join 0.45, production 0.39).

1. **scratch reset ~1.75** — STOP new representation.
2. accum honest **~19** (alpha seed). Isolated pile < 1
   except scratch STOP.
3. fanout OUT 4.53 — 2u rpds, refused.
4. cascade ROUND extra +0.95 — pieces under 1. Do not
   dirty-agenda root-join (0.45).
5. clone 0.91 under.

Grid `T05-41-25Z` (post insert-prime-split, `GRID_SKIP_ORACLE=1`,
`GRID_RUNS=3`): 30/30 `:match`, 30/30 `:us`. Closest Clara
cell is still **fanout `[40000]` ratio 3.71** (wat 52.9 ms;
T03-32-37Z was 3.59 / 52.8). Cascade `[50 100]` 6.69
(wat 17.5). Accum `[200 200]` **8.09** (wat 18.4; was 6.07
/ 24.9). Dominance leftover is fanout fire (OUT 4.53
refused; production 2p tax).

Parked: fact insertion (section below). Refused: intern
`names`, facts in `bind_pool`, retry 2e/2o, persist gather
to dodge the fold, 297, service-ify, alpha-tree range
edges, `not=` as a range, two constraints on one dim as a
conjunction, sample marks, retire `prod:compiled-rhs`,
restore per-fact alpha timers.

## Insertion — unparked (2026-08-20)

FIRE internable ≥ 1 is scratch STOP / OUT refused.
`DESIGN-STONE-insert-facts-from-names` **LANDED, under
bar:** insert − conj 2037 → **1650 ns/fact** (cut 387).
`DESIGN-STONE-insert-all-empty-identity` **LANDED:**
protocol insert 9.42 ms → **0.013 ms**. Concat rebuilt
from empty; empty ++ x is identity.
`DESIGN-STONE-insert-prime-split` **LANDED:** I − P
**1474 ns** was `eval_tail` defclause TCO / apply_function.
insert − conj **1933 → 310 ns**. P − C 459 under bar.
`make_mut` STOP (foldl rc ≥ 2). Host query encode is
Claude's compiled-wat horizon, not rete. Do not
Session-`Vec`. Do not route 2-ary through insert-all.

`probe-insert-cost-split.wat` on **c800d7d5** (release, one
run per n; witnesses held). N chained `insert`, not
`insert-all`. `insert − conj` was **flat ~2000 ns/fact**:

| n | baseline | conj | insert | insert − conj |
|---:|---:|---:|---:|---:|
| 5 000 | 2991 | 2286 | 4305 | **2018** |
| 10 000 | 3191 | 2625 | 4722 | **2097** |
| 20 000 | 3060 | 2564 | 4645 | **2081** |
| 40 000 | 3100 | 2822 | 4802 | **1981** |

Against the stones: native-insert **+11.8 µs** (pre-`insert'`);
insert-all.md **+1027 ns**; this tip **+2.0 µs**. Still ~6×
the interpreted path. ~2× the post-native snapshot. All
three arms moved (baseline 2.2 → 3.0 µs) — foldl+construct
is heavier too.

Not seqable (`range` is still `Vec`; `foldl` still walks it).
Not foldr. Not the aggregate-identity O(n²) leak (Session
`identity = 0`; `value_is_shallow` bails on `network`).

`available` interned (this stone). Remaining suspects:
Session rebuild, PV conj, `insert` defclause in front of
`insert'`. Unique-owner `make_mut` if a split names ≥ 0.5 µs.

Already on disk, **not** this queue:

- `DESIGN-STONE-cross-call-incrementality.md` — no-op re-fire costs ~100% of cold. Correctness is TM / negation. Do not fold into #3.
- 297, service-ify, persist gather *instead of* #1.

## Law

One stone. One gate. One grid cell named. Weigh, then the next.
Slow is smooth.

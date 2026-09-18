# DESIGN-STONE — cache the gather index per (alpha, join-keys) within a round

> **Origin (2026-07-31):** the builder redirected a mis-aimed hunt. I had reported
> `alpha_match_inner` as "the single biggest line item" and gone to review the matcher — but
> **alpha runs on every axis and we win six of seven**, so alpha cannot be what makes us lose. The
> thing that *distinguishes* the one axis Clara takes is the **accumulate pass**. Biggest and
> differentiating are different questions; the measurement was right and the inference was wrong.

## The redundancy, counted

`gather_index` is a **pure function of (alpha memory, join keys)** — and it is rebuilt **per node,
per round**. At `G=200 W=200` (40,200 facts):

```
accum:index-builds          4          ← one per AccumulateNode
accum:index-elements  160,000          ← 40,000 × 4
alpha_nodes                 3   alpha_elements  80,200
```

The alpha shape settles what those four builds actually are — 200 Groups + **two** Reading alphas
of 40,000 each (`80,200 = 200 + 40,000 + 40,000`):

| alpha | binds | read by | index builds today |
|---|---|---|---|
| Group | `?g` | the root join | — |
| Reading-A | `?g` | `count` (accumulate) + `exists` (filter) | 2 |
| Reading-B | `?g`,`?v` | `sum`, `min`, `max` (accumulate) | 3 |

**Five index builds over two distinct `(alpha, join_keys)` pairs.** Three are pure repetition, and
each drags a full `from_elements` clone of its 40,000-element memory with it.

⚠ **This corrects `8bd2a88e`**, which said "~75% is repetition … ~14% of fire." Within the
accumulate pass alone it is **2 of 4 builds — 50%, not 75%.** The counts (`4` / `160,000`) are
equally consistent with one shared alpha or two, and I asserted the more flattering reading without
checking. `alpha_nodes 3` is what settled it.

## The algorithm

Hold a per-round cache keyed by `(alpha_id, join_keys)`, shared by the accumulate pass and the
Negation/Exists filter pass. First reader of a pair builds and stores; the rest borrow.

```rust
// cleared at the top of every round, after step 1 has finished growing wm.alpha
let mut gather_cache: HashMap<(i64, Vec<Value>), (Vec<Value>, HashMap<Vec<Value>, Vec<usize>>)>;
```

Builds go **5 → 2**. Full-memory clones go **5 → 2**.

## ★ THE ONE CONTRACT DECISION

**The cache key is `(alpha_id, join_keys)` — the alpha id ALONE is not sufficient.**

`join_keys` is the intersection of the *sample token's* binding keys with the *element's*. Two nodes
reading the same alpha can have parents that bind different variable sets, and would then need
different key tuples. Keying on `alpha_id` alone would silently hand one node an index bucketed by
the wrong tuple — every probe misses, every gather comes back empty, and `count`/`sum` quietly emit
their identity for groups that have elements. **A wrong answer, not a crash**, and the accum axis
would not catch it (there, all readers happen to key on `?g`).

Two supporting constraints:

- **Round-scoped, never longer.** `wm.alpha` grows during step 1 of each round. A cache that
  outlived a round would serve a stale index missing that round's new elements — again a wrong
  answer, silently. Clear it at the top of each round, after step 1.
  > **⛔ SUPERSEDED 2026-08-19 by `DESIGN-STONE-persist-gather-across-rounds.md`.** The cache does
  > outlive the round now; the staleness named here is closed by appending `d_alpha` rather than by
  > clearing (`src/rete/kernel/fire/delta.rs`). Annotated, not rewritten — the reasoning was sound
  > for this stone.
- **The cached snapshot is the one the index indexes.** Buckets hold *indices into `from_elements`*,
  so the cached `Vec<Value>` and its index must travel together and be borrowed together. Storing
  one and re-deriving the other re-introduces the clone this stone exists to remove.

## The RED gate — the census, not a wall

Same instrument as the keyed-gather stone, same reason: a wall-clock gate is flaky and can pass for
reasons unrelated to the mechanism.

> With the count census armed at `G=200 W=200`, assert `accum:index-builds` equals the number of
> **distinct `(alpha_id, join_keys)` pairs** rather than the number of nodes — i.e. **2, not 4** —
> and that `accum:index-elements` falls from 160,000 to 80,000.

Red today at 4 / 160,000. A per-node rebuild cannot fake it.

Extend the count to the filter pass too (it builds its own index today and is not yet counted), so
the gate covers all five builds and lands at 2.

## Expected win — stated as a range, to be measured

At `[200 200]`, `accum:snapshot` 3.61ms + `accum:index` 19.58ms = **23.19ms**, of which the cache
removes half (2 of 4 builds). The filter pass's own index sits inside its 5.94ms and is not
separately timed yet; sharing the Reading-A index with `count` removes one more build.

**Expect ~10–13ms of a ~120ms fire — roughly 8–11%.** That moves the warmed-Clara ratio from 0.665
toward ~0.73. **Real, not decisive** — the fold (~20%) and alpha (~50%) are genuine work, and this
stone does not touch them.

## Blast radius

`src/rete/kernel/fire/` only — the round loop (cache lifetime), the accumulate pass, the
Negation/Exists filter pass, and the counter. No `.wat` changes, no corpus migration; the wat oracle
does not move.

## Out of scope = REJECTED (affirmative cuts)

- **A cross-round or cross-fire cache.** Alpha memories grow every round; the invalidation is the
  hard part and the win is one build per round. Round-scoped is correct by construction.
  > **⛔ THIS REJECTION WAS OVERTURNED 2026-08-19** by `DESIGN-STONE-persist-gather-across-rounds.md`,
  > which solved the invalidation ("the hard part") by appending `d_alpha`. Cross-ROUND shipped;
  > cross-FIRE did not.
- **Deduplicating the alpha nodes themselves** (Reading-A vs Reading-B differ only by binding `?v`).
  That is a *compiler* question — whether two conditions over one fact type should share an alpha
  and project bindings per consumer — and it is a bigger, separate stone with its own correctness
  surface.
- **The fold and alpha.** The two largest remaining costs, untouched here, each their own
  investigation.
- **Chasing the last of the ~1.5×.** This stone narrows it; closing it is a constant-factor grind
  against a JIT that is good at exactly this shape, on one axis of seven. Whether that is worth
  doing is a priority call, not a technical one.

## Sequencing

1. Count the filter pass's index builds too (the gate must see all five).
2. Land the RED gate at 4-going-on-5 builds.
3. Thread the round-scoped cache through both passes.
4. Weigh: the census gate, the full `--release` floor, clippy — by my own re-run.
5. Re-run the phase census and the `accum` grid ladder; record before/after.

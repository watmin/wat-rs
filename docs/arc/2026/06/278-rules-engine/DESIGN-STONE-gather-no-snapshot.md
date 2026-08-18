# DESIGN-STONE — gather buckets index `wm.alpha`, no clone

> **Origin (2026-08-18).** `NEXT-STRIKES-after-shadow.md` #2.
> Fold-the-wall landed: `accum:fold` 68.49 → 2.32 ms. What remains
> of accumulate at `[200 200]` is `accum:index` 13.65 ms +
> **`accum:snapshot` 5.56 ms**. This stone is the snapshot.

## The measurement

`GatherCache` stores `(Vec<Element>, GatherIndex)`. The `Vec` is
`wm.alpha[id].clone()` on the miss path (`ensure_gather` and the
accumulate loop, twice per distinct pair). At `[200 200]` that is
two 40,000-element clones, marked `accum:snapshot` **5.56 ms**.

The index already holds **indices**. They were indices into the
clone so the clone and the index "travel together"
(`DESIGN-STONE-gather-index-cache.md`). That clause was correct
when the cache might have outlived a mutation of `wm.alpha`. It
does not: the cache is created inside the round, **after** we
know step 1 will grow alpha, and it is populated **after** step 1.
`wm.alpha.entry().push` has two sites; the fixpoint's is step 1
only. After that, the vec is append-idle until the next round
drops the cache.

So the clone is a frozen copy of a vec that will not move.

## The algorithm

`GatherCache` stores only `GatherIndex`. Buckets are indices into
**`wm.alpha[alpha_id]`**, the same vec `build_gather_index` walked.

```
ensure_gather:
    els = wm.alpha[id]          // borrow, no clone
    join_keys = intersection(sample, els[0])
    on miss: build_gather_index(els, join_keys)   // counted
probe:
    i in bucket → wm.alpha[id][i]
```

The accumulate loop stops inlining a second miss path. It calls
`ensure_gather` like Exists/Not. One builder.

`accum:snapshot` stays as a mark around the `wm.alpha.get` (no
clone) so the census still sees the row. It must collapse.

## ★ THE ONE CONTRACT DECISION

**Indices name positions in this round's `wm.alpha[id]`, not in a
private copy.** Order is still insertion order (step 1 appends;
we index after that). Empty alpha is still an empty gather.

Do **not** let the cache outlive the round. That is still
`DESIGN-STONE-gather-index-cache.md`'s silent-wrong-answer
clause, and this stone does not lift it. Persist-across-rounds
is #3.

## The gate

1. `accum_fire_phase_census` `[200 200]`: `accum:snapshot`
   mean **< 1.0 ms** (today 5.56). Non-zero mark still fires
   (row present).
2. `gather_index_is_built_once_per_alpha_and_keyset` still
   2 builds / ≤ 80,000 elements — we did not grow builds.
3. `keyed_gather_visits_do_not_scale_with_group_count` still
   ≤ 2.0.
4. rete differentials, clippy.

## Predicted win

~5 ms off a 101 ms fire. Real, not decisive. Alpha + index
remain. Do not bundle #3.

## Blast radius

`src/rete/kernel.rs` only: `GatherCache` type, `ensure_gather`,
the accumulate miss path, `any_seeded_keyed` /
`seeded_bindings_keyed`. No `.wat`.

## Out of scope = REJECTED

- Persisting the index across rounds (#3).
- Deduplicating Reading-A / Reading-B alphas.
- Touching `accum:fold` again.
- Deleting the snapshot mark to fake a zero.

## Sequencing

1. Change the type. One builder.
2. Acc + Exists/Not probe `wm.alpha[id][i]`.
3. Re-point snapshot < 1 ms. Weigh.

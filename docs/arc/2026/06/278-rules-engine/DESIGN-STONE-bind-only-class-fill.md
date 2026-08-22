# DESIGN-STONE — bind-only class fill (pass 2)

> **Origin (2026-08-22).** Column-gather-fold interned; FIRE
> held **17.8**, seed **14.7**. Skip BindSpan is not the
> leftover. Seed is still 80k (fact, alpha) visits:
> HashMap lookup + `Element` push. Bind-only + packed
> i64 is class-match-enough: every Reading matches both
> Reading alphas, and both alphas write the **same**
> `{ fact, empty span }`. This stone deletes the inner
> loop. One class column, then reserved fills.

## The enemy

```
for fact in 40,200:
    seen, pack, tree
    for aid in candidates:          // 80k
        alpha[aid].push(Element)
        d_alpha[aid].push(slot)
```

Two Reading alphas, identical Elements. 80k copies
of 40k fact ids. `entry().or_default()` twice per
fact. Sequential stores into two HashMap buckets,
not a contiguous fill.

## The algorithm

A class is **batchable** iff every alpha of that
class is bind-only and has no `fact_bind` (the
skip-span contract). Acc Group/Reading are.

```
SETUP:
    batch_class[class] = those alpha ids
    // Cmp / fact_bind / mixed class → not batched

SEED pass 1 — one PV walk:
    seen_insert
    if class is batchable:
        pack row (fields already in hand)
        if packed: class_ids[class].push(fact_idx)
        else: old activate
    else:
        old activate
    // no candidates_into, no per-alpha push

SEED pass 2 — one reserved fill per batched alpha:
    els = class_ids.map(|i| Element { i, empty })
    for aid in batch_class[class]:
        alpha[aid] = els.clone()     // memcpy
        d_alpha[aid] = 0..n
    census compiled:calls += n × |aids|

DELTA / Cmp / unpacked: old activate.
```

Pass 2 is the large write: grow the vec once,
fill it, memcpy to the sibling alpha. Not rayon.
Not async I/O. One session, one thread.

## ★ THE ONE CONTRACT DECISION

**Bind-only packed facts of a class enter every
alpha of that class without a per-fact candidate
walk.** The tree already over-approximates; Bind
always holds. We do not batch a class that has a
Cmp / BindCheck / `fact_bind` alpha. Token still
has BindSpan (root-join seeds from the filled
Elements). Session stays 8 Values.

## The gate

1. Batched `Element`s are `{ fact, empty span }`.
   Unbatched path unchanged.
2. rete lib. `probe_arc278_7strat_native_differential`.
3. clippy `-D warnings` (`--lib`).
4. `accum_fire_phase_census` `[200 200]`. **Revert
   if FIRE regresses vs 17.8.** Do not wall-gate.

## Predicted win

Independent guess (written first): drop tree
(~0.9) and 80k HashMap push (~1.3+) and the
skip_span inner loop. Seed **14.7 → 9–12**.
FIRE **17.8 → 12–15**. Seen (~3.4) and pack
`intern_val` stay in pass 1. If FIRE does not
fall ≥ 1 ms, leftover is seen + intern_val —
say so; do not parallelize the fill.

## Weigh (2026-08-22) — interned, REVERTED

7strat `differential_three_stratum_negation`
native **[1, 2, 2]** vs oracle **[1, 2, 1]**
(extra Safe). 2-stratum still matched. Skipping
`candidates_into` and filling every bind-only
alpha of a class from the fact-id column is
**not class-match-enough** once a `:not` inner
sits on a derived class in a later stratum.
Empty-column `insert` wipe was tried; still
red. Reverted. Pass 2 stays the intern; this
cut did not land. Do not re-land without a
3-stratum-green fill.

## Blast radius

`fire/delta.rs` seed_round only. SETUP classifies
batchable classes from `kind_ids.alpha` +
`bind_only` + `class_for_alpha`. No `.wat`. No
crate. No `unsafe`. No rayon. Freeze still Values.

## Out of scope = REJECTED this stone

- Intra-fire rayon / shared mut `alpha`.
- SETUP PV walk for intern_val (weighed 19→25).
- Skip Token BindSpan. SIMD. Invert Cmp.
- Session-`Vec`. Facts in `bind_pool`. 297.
- Share one `Vec<Element>` by pointer (clone is
  the memcpy; `Arc` is a later cut).

## Sequencing

1. Write this stone. Predicted win first.
2. Batchable classes. Pass 1 column. Pass 2 fill.
3. Weigh FIRE / seed. Revert on FIRE regression.
   Stop.

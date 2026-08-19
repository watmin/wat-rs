# DESIGN-STONE — persist gather indexes across rounds

> **Origin (2026-08-19).** 2xx intern queue is exhausted. NEXT
> named **3**: P6 for gathers; append `d_alpha`. Keyed-gather
> **rejected** persist as a second-order win. A census must name
> a multi-round cell. Do not persist gather to dodge the fold.
> Do not start 297.

## The measurement

`gather_cache` is declared **inside** the round loop, same
lifetime as `d_alpha`. First reader of `(alpha_id, join_keys)`
builds; the rest borrow. Next round the table is new and the
index is hashed again over the **full** `wm.alpha`.

P6 already persists `left_idx` / `right_idx` **outside** the
loop and **appends** Δ. Gathers did not get that. Acc / Not /
Exists share `ensure_gather`. A cold one-round fire pays the
hash once; a second round that still gathers Readings pays it
again even when that alpha did not grow.

Predicted on accum `[200 200]` if it is one (maybe two) rounds
and Reading alpha settles in round 0: **~0**. The win is a
cell whose alpha **grows** across rounds (cascade / strat).
The census prints that. Do not claim a FIRE cut on accum
without it.

## The algorithm

```
// fire-scoped, like P6 left_idx — NOT a Session field
gather_cache: GatherCache   // lives outside the round loop

after alpha (d_alpha filled):
  for ((aid, keys), idx) in gather_cache:
    append d_alpha[aid] into idx buckets   // insertion order

ensure_gather:
  miss → build full wm.alpha[aid] (first reader)
  hit  → already appended
```

`GatherIndex::append` pushes new `usize` ids onto existing
buckets. New ids are `>=` old len (alpha only appends). Foldl
order holds.

Drop of the cache is end of `fire_fixpoint_delta`, same as
P6. `drop-memories` still clears alpha. Next `fire-rules` is
a new cache.

## ★ THE ONE CONTRACT DECISION

**Gather indexes are P6: fire-scoped, persist across rounds,
append `d_alpha`. Not stored on the Session.**

Round-scoped rebuild is the stale-index wall this stone
removes. A cache that outlived a fire would miss
`drop-memories`. A cache keyed on `alpha_id` alone is still
forbidden (`DESIGN-STONE-gather-index-cache.md`).

## The gate

1. Census prints rounds + `alpha_elements` per round on
   accum `[200 200]`. `accum:index-builds` still ≤ 2 there.
2. rete lib.
3. clippy `-D warnings` (`--lib`).
4. Census `[200 200]`: fold < 25, snapshot < 1. FIRE
   printed, **not** wall-gated.

## Predicted win

Accum cold: FIRE **~49** (wash) if R=1 or Reading alpha is
stable after R0. Isolated rebuild of a grown alpha dies on
the cell the census names. If no cell grows alpha, say so
and stop — do not invent a cascade harvest to justify the
edit.

## Blast radius

`kernel.rs` (`gather_cache` lifetime, `GatherIndex::append`,
`ensure_gather`, one census print). No `.wat`. No crate.
No `unsafe`. Token stays two spans.

## Out of scope = REJECTED

- Persist gather to dodge the fold.
- Session-stored indexes. Process-lifetime intern.
- Key on `alpha_id` alone. Rebuild instead of append.
- Intern `names`. Facts in `bind_pool`. Retry 2e / 2o.
- 297. Service-ify. Fact insertion (parked).

## Sequencing

1. Print the round census. Name the cell or name the miss.
2. Move the cache outside the loop. Append `d_alpha`.
3. Weigh FIRE + `accum:index-builds`. Stop.

## Weigh (2026-08-19) — LANDED, ~0 on accum

`accum_alpha_memory_shape` G=200 W=200:

```
rounds 2
  r0  dIn 40200  aNodes 3  aEls 80200  prod 1000  seen 41200
  r1  dIn 1000   aNodes 3  aEls 80200  prod 1000  seen 41200
```

Alpha **does not grow**. `accum:index-builds` **2** / 80,000
(unchanged — round 1 does not re-gather Readings). Census
`[200 200]` FIRE **51.68** (2ae was 49.13; wash). fold 0.64.
snapshot 0.00.

The mechanism is P6-shaped and in. This cell is not a FIRE
cut. Do not invent a cascade harvest to justify it. Next is
**4** (where-dim-reuse), not a second persist.

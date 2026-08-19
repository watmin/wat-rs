# DESIGN-STONE — gather index is unary when the join key is one

> **Origin (2026-08-18).** 2x: `setup:seen` 7.43 → 4.31. Largest
> honest engine row: **`accum:index` 5.16**. Cold fire: persist
> gather is still ~0 (first-round hash). The mark is
> `ensure_gather` × 4 AccumulateNodes. Cache already 2 builds
> of 40k elements. `join_keys` on this axis is one `?g`.
> `build_gather_index` still `key_of` → `Vec<Value>` per element.

## The measurement we do not have

```
for el in 40_000 {
    key = key_of(el.binds, [?g])   // Vec of one
    index.entry(key).push(i)       // FxHashMap<Vec<Value>, _>
}
```

2r: `key_of` is 27 ns. × 40k = **1.08 ms**. HashMap of
`Vec<Value>` vs `Value` is unranked. Persist does not move a
cold fire.

## The algorithm

1. Tight loop. 40,200 Reading-shaped Elements (`?g`, `?v`).
   `join_keys = [?g]`. Mean of 3.

```
K  40k key_of
V  40k FxHashMap<Vec<Value>> insert     // engine key
U  40k FxHashMap<Value> insert          // unary
B  build_gather_index                   // authority
S  Bindings::get + unary insert         // no Vec
```

`(B − S)` is the predicted cut per build. Two builds on this
cell.

2. **STOP** if `B − S` < **0.5 ms** (two builds < 1 ms). Do not
   touch the index.

3. Else `GatherIndex` is unary when `join_keys.len() == 1`.
   N-ary stays `Vec<Value>`. Token stays two BindSpans. Cache
   key unchanged. Do not persist.

## ★ THE ONE CONTRACT DECISION

**A one-key gather hashes the filler, not a one-element Vec.**
Buckets are still insertion-order indices into `wm.alpha`. Eq
of `Value` is unchanged.

## The gate

1. `gather_unary_index_split` prints K / V / U / B / S. B > 0.
2. If the stone implements: `accum_leftover_split` `[200 200]`
   prints `accum:index`. Do not wall-gate FIRE. Token still
   two `BindSpan`s.
3. rete lib.
4. clippy `-D warnings`.

## Predicted win

Independent guess (written first): B − S ≈ **1.5–2.5 ms** per
build, **~3–5 ms** on the cell. `accum:index` 5.16 → **~2**.
FIRE 61 → **~56–58**. If B − S is small, leftover is the
HashMap of 200 buckets — say so.

## Blast radius

`kernel.rs` `GatherIndex` + `build_gather_index` + bucket
readers. Tests. No `.wat`. No persist.

## Out of scope = REJECTED

- Persist gather. Second hasher. Skip `seen`. 297.
- Intern `names`. 2o. Session rewrite.

## Sequencing

1. Print. Rank.
2. B − S < 0.5 ms → stop.
3. Else unary index. Weigh `accum:index`. Stop.

## Weigh (2026-08-18) — LANDED

`gather_unary_index_split` (40,200 Readings, mean of 3):

| lump | ms |
|---|---:|
| K key_of | 1.42 |
| V `HashMap<Vec>` | 2.16 |
| U `HashMap<Value>` | 1.22 |
| B build_gather_index | 2.17 |
| S get + unary | 1.14 |
| **B−S** | **1.03**/build |
| ×2 builds | **2.05** |

Cut ≥ 0.5. Intern licensed. `GatherIndex::Unary` when
`join_keys.len() == 1`. Token stayed two BindSpans.

`accum_leftover_split` `[200 200]`:

| mark | before | after |
|---|---:|---:|
| `accum:index` | 5.16 | **3.61** |
| accumulate | 7.06 | **5.44** |
| FIRE | 61.00 | **57.92** |
| honest_FIRE | 14.50 | **9.42** |

index **5.16 → 3.61** (−1.55). Do not persist gather.
Largest honest engine row is now **setup:seen 4.30**.

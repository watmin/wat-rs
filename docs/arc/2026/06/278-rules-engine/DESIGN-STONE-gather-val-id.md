# DESIGN-STONE — unary gather hashes interned filler ids

> **Origin (2026-08-20).** After 22: `accum:index` **1.97**.
> 2y interned `Vec<Value>` → `Value`. Isolated U **1.26**/build,
> B−S **0.09**. The remaining row is `v.clone()` into
> `FxHashMap<Value>`. Bind pool already stores filler `u32`
> ids. Hash those.

## The measurement we have

Two unary builds of 40,200 `?g` i64s. `entry(v.clone())` per
element. `intern_val` already interned those i64s at populate.

## The algorithm

1. Tight loop, same fixture as 2y. Mean of 3.

```
U  HashMap<Value> + clone     // engine
I  HashMap<u32> from bind-pool vid
B  build_gather_index
```

`U − I` per build. ×2 on the cell.

2. **STOP intern** if `U − I` < **0.5 ms**.
3. Else `GatherIndex::UnaryId(FxHashMap<u32, Vec<usize>>)`.
   Probe intern_vals the join filler (once per token).
   N-ary unchanged. Token stays two spans. Do not persist.

## ★ THE ONE CONTRACT DECISION

**A one-key gather hashes the interned filler id, not a
cloned `Value`.** Buckets are still insertion-order indices
into `wm.alpha`. Eq of fillers is intern-id eq.

## The gate

1. `gather_val_id_split` prints U / I / B. I > 0.
2. If intern: `accum_leftover_split` prints index / FIRE.
   Do not wall-gate FIRE.
3. rete lib.
4. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **U − I ≈ 0.6–0.8**/build.
×2 **~1.3**. index 1.97 → **~0.7**. FIRE 19.95 → **~18.7**.

## Blast radius

`kernel.rs` `GatherIndex` + `build_gather_index` + `bucket`
+ append. One test. No `.wat`. No Session field.

## Out of scope = REJECTED

- Persist gather. i64-only map as a third variant.
- Intern `names`. 2e / 2o. 297. Insertion. Scratch repr.

## Sequencing

1. Print U/I/B. Rank.
2. U−I < 0.5 → stop.
3. Else UnaryId. Weigh index. Stop.

## Weigh (2026-08-20) — LANDED

`gather_val_id_split`: U **2.02**, I **0.62**, U−I **1.40**/build.
Intern licensed. Gate: rete lib 98, clippy `-D warnings` silent.

`accum_leftover_split` `[200 200]`:

| lump | before | after |
|---|---:|---:|
| accum:index | 1.97 | **0.61** |
| accumulate | 2.58 | **1.25** |
| FIRE | 19.95 | **19.08** |
| honest_FIRE | 19.60 | **18.75** |
| B authority | 1.40 | **0.54** |

Predicted FIRE −1.3; measured **−0.87**. Index −1.36.
B tracks I. Builds still 2. Scratch STOP. Clone 0.79 under.

Next leftover: scratch 1.57 STOP / isolated M−T pile /
cascade·fanout honest ~13. Do not intern names. Do not
start 297.
